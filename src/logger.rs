use chrono::Local;
use log::{Level, LevelFilter, Metadata, Record};
use parking_lot::Mutex;
use std::fs::OpenOptions;
use std::io::{self, Write};
use tracing_subscriber::{
    EnvFilter, Registry,
    fmt::{self, format::FmtSpan},
    layer::SubscriberExt,
    util::SubscriberInitExt,
};

struct GitIrisLogger;

static LOGGER: GitIrisLogger = GitIrisLogger;
static LOGGING_ENABLED: std::sync::LazyLock<Mutex<bool>> =
    std::sync::LazyLock::new(|| Mutex::new(false));
static LOG_FILE: std::sync::LazyLock<Mutex<Option<std::fs::File>>> =
    std::sync::LazyLock::new(|| Mutex::new(None));
static LOG_TO_STDOUT: std::sync::LazyLock<Mutex<bool>> =
    std::sync::LazyLock::new(|| Mutex::new(false));
static VERBOSE_LOGGING: std::sync::LazyLock<Mutex<bool>> =
    std::sync::LazyLock::new(|| Mutex::new(false));

/// Custom writer that writes to both file and stdout/stderr
#[derive(Clone)]
struct UnifiedWriter;

impl Write for UnifiedWriter {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        // Write tracing logs only to file
        if let Some(file) = LOG_FILE.lock().as_mut() {
            let _ = file.write_all(buf);
            let _ = file.flush();
        }

        Ok(buf.len())
    }

    fn flush(&mut self) -> io::Result<()> {
        if let Some(file) = LOG_FILE.lock().as_mut() {
            let _ = file.flush();
        }
        Ok(())
    }
}

impl<'a> tracing_subscriber::fmt::MakeWriter<'a> for UnifiedWriter {
    type Writer = UnifiedWriter;

    fn make_writer(&'a self) -> Self::Writer {
        UnifiedWriter
    }
}

impl log::Log for GitIrisLogger {
    fn enabled(&self, metadata: &Metadata) -> bool {
        if !*LOGGING_ENABLED.lock() {
            return false;
        }

        // Always allow our own logs
        if metadata.target().starts_with("git_iris") {
            return metadata.level() <= Level::Debug;
        }

        // Allow rig logs - they provide valuable LLM operation insights
        if metadata.target().starts_with("rig") {
            return metadata.level() <= Level::Info;
        }

        // Filter external library logs unless verbose logging is enabled
        let verbose_enabled = *VERBOSE_LOGGING.lock();
        if !verbose_enabled {
            // Block common noisy external libraries
            let target = metadata.target();
            if target.starts_with("reqwest")
                || target.starts_with("hyper")
                || target.starts_with("h2")
                || target.starts_with("rustls")
                || target.starts_with("want")
                || target.starts_with("mio")
                || target.contains("anthropic")
                || target.contains("openai")
                || target.contains("completion")
                || target.contains("connection")
            {
                return false;
            }
        }

        metadata.level() <= Level::Debug
    }

    fn log(&self, record: &Record) {
        if self.enabled(record.metadata()) {
            let timestamp = Local::now().format("%Y-%m-%d %H:%M:%S%.3f");
            let target = if record.target().starts_with("rig") {
                "🦀 rig"
            } else {
                record.target()
            };
            let message = format!(
                "{} {} [{}] - {}\n",
                timestamp,
                record.level(),
                target,
                record.args()
            );

            if let Some(file) = LOG_FILE.lock().as_mut() {
                let _ = file.write_all(message.as_bytes());
                let _ = file.flush();
            }

            if *LOG_TO_STDOUT.lock() {
                print!("{message}");
            }
        }
    }

    fn flush(&self) {}
}

/// Initialize unified logging system supporting both log and tracing
pub fn init() -> Result<(), Box<dyn std::error::Error>> {
    use std::sync::{Once, OnceLock};
    static INIT: Once = Once::new();
    static INIT_RESULT: OnceLock<Result<(), String>> = OnceLock::new();

    INIT.call_once(|| {
        // Check if we should enable verbose logging from environment
        let verbose_from_env = std::env::var("GIT_IRIS_VERBOSE").is_ok()
            || std::env::var("RUST_LOG").is_ok_and(|v| v.contains("debug") || v.contains("trace"));

        if verbose_from_env {
            set_verbose_logging(true);
        }

        // Enable logging by default
        enable_logging();
        set_log_to_stdout(true);

        // Set up tracing subscriber with unified writer (for Rig logs)
        let env_filter = EnvFilter::try_from_default_env().unwrap_or_else(|_| {
            // Default filter: show git_iris debug, rig info, and warnings from others
            "git_iris=debug,rig=info,warn".into()
        });

        let fmt_layer = fmt::Layer::new()
            .with_target(true)
            .with_level(true)
            .with_timer(fmt::time::ChronoUtc::rfc_3339())
            .with_span_events(FmtSpan::CLOSE)
            .with_writer(UnifiedWriter);

        // Try to initialize tracing subscriber
        let tracing_result = Registry::default()
            .with(env_filter)
            .with(fmt_layer)
            .try_init();

        // Try to initialize the log system for backwards compatibility
        let log_result = log::set_logger(&LOGGER).map(|()| log::set_max_level(LevelFilter::Debug));

        let result = match (tracing_result, log_result) {
            (Ok(()), Ok(())) => Ok(()),
            (Ok(()), Err(_)) => {
                // Tracing succeeded, log failed - that's okay, tracing can handle everything
                eprintln!("Note: Using tracing-only logging (log crate setup skipped)");
                Ok(())
            }
            (Err(tracing_err), Ok(())) => {
                // Log succeeded, tracing failed - fallback to log only
                eprintln!("Note: Using log-only logging (tracing setup failed: {tracing_err})");
                Ok(())
            }
            (Err(tracing_err), Err(log_err)) => {
                // Both failed
                Err(format!(
                    "Failed to initialize logging: tracing={tracing_err}, log={log_err}"
                ))
            }
        };

        let _ = INIT_RESULT.set(result);
    });

    match INIT_RESULT.get() {
        Some(Ok(())) => Ok(()),
        Some(Err(e)) => Err(e.clone().into()),
        None => Err("Initialization failed unexpectedly".into()),
    }
}

pub fn enable_logging() {
    let mut logging_enabled = LOGGING_ENABLED.lock();
    *logging_enabled = true;
}

pub fn disable_logging() {
    let mut logging_enabled = LOGGING_ENABLED.lock();
    *logging_enabled = false;
}

pub fn set_verbose_logging(enabled: bool) {
    let mut verbose_logging = VERBOSE_LOGGING.lock();
    *verbose_logging = enabled;

    // Note: Verbose logging changes will take effect on next application restart
    // or can be controlled via RUST_LOG environment variable before startup
}

pub fn set_log_file(file_path: &str) -> std::io::Result<()> {
    let file = OpenOptions::new()
        .create(true)
        .append(true)
        .open(file_path)?;

    let mut log_file = LOG_FILE.lock();
    *log_file = Some(file);
    Ok(())
}

pub fn set_log_to_stdout(enabled: bool) {
    let mut log_to_stdout = LOG_TO_STDOUT.lock();
    *log_to_stdout = enabled;
}

// Macros for git-iris logging (maintains compatibility)
#[macro_export]
macro_rules! log_debug {
    ($($arg:tt)*) => {
        log::debug!($($arg)*)
    };
}

#[macro_export]
macro_rules! log_error {
    ($($arg:tt)*) => {
        log::error!($($arg)*)
    };
}

#[macro_export]
macro_rules! log_info {
    ($($arg:tt)*) => {
        log::info!($($arg)*)
    };
}

#[macro_export]
macro_rules! log_warn {
    ($($arg:tt)*) => {
        log::warn!($($arg)*)
    };
}

// New tracing macros for enhanced logging (following Rig patterns)
#[macro_export]
macro_rules! trace_debug {
    (target: $target:expr, $($arg:tt)*) => {
        tracing::debug!(target: $target, $($arg)*)
    };
    ($($arg:tt)*) => {
        tracing::debug!($($arg)*)
    };
}

#[macro_export]
macro_rules! trace_info {
    (target: $target:expr, $($arg:tt)*) => {
        tracing::info!(target: $target, $($arg)*)
    };
    ($($arg:tt)*) => {
        tracing::info!($($arg)*)
    };
}

#[macro_export]
macro_rules! trace_warn {
    (target: $target:expr, $($arg:tt)*) => {
        tracing::warn!(target: $target, $($arg)*)
    };
    ($($arg:tt)*) => {
        tracing::warn!($($arg)*)
    };
}

#[macro_export]
macro_rules! trace_error {
    (target: $target:expr, $($arg:tt)*) => {
        tracing::error!(target: $target, $($arg)*)
    };
    ($($arg:tt)*) => {
        tracing::error!($($arg)*)
    };
}
