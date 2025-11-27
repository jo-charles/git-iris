//! Debug observability module for Iris agent operations
//!
//! All debug output goes through tracing. Use `-l <file>` to log to file,
//! `--debug` to enable debug-level output.

use std::fs::{self, OpenOptions};
use std::io::{self, Write};
use std::path::PathBuf;
use std::sync::atomic::{AtomicBool, Ordering};
use std::time::{Duration, Instant};

#[cfg(unix)]
use std::os::unix::fs::{OpenOptionsExt, PermissionsExt};

const DEBUG_DIR_ENV: &str = "GIT_IRIS_DEBUG_DIR";

/// Global debug mode flag
static DEBUG_MODE: AtomicBool = AtomicBool::new(false);

/// Enable debug mode
pub fn enable_debug_mode() {
    DEBUG_MODE.store(true, Ordering::SeqCst);
}

/// Disable debug mode
pub fn disable_debug_mode() {
    DEBUG_MODE.store(false, Ordering::SeqCst);
}

/// Check if debug mode is enabled
pub fn is_debug_enabled() -> bool {
    DEBUG_MODE.load(Ordering::SeqCst)
}

/// Resolve the directory used for storing debug artifacts (LLM dumps, extracted JSON)
fn debug_artifacts_dir() -> io::Result<PathBuf> {
    let base = std::env::var_os(DEBUG_DIR_ENV)
        .map(PathBuf::from)
        .or_else(|| {
            dirs::cache_dir().map(|mut dir| {
                dir.push("git-iris");
                dir.push("debug-artifacts");
                dir
            })
        })
        .unwrap_or_else(|| {
            std::env::temp_dir()
                .join("git-iris")
                .join("debug-artifacts")
        });

    if !base.exists() {
        fs::create_dir_all(&base)?;
    }

    #[cfg(unix)]
    {
        let _ = fs::set_permissions(&base, fs::Permissions::from_mode(0o700));
    }

    Ok(base)
}

/// Write debug artifact with restrictive permissions and return the file path.
pub fn write_debug_artifact(filename: &str, contents: &str) -> io::Result<PathBuf> {
    let mut path = debug_artifacts_dir()?;
    path.push(filename);

    write_secure_file(&path, contents)?;
    Ok(path)
}

fn write_secure_file(path: &PathBuf, contents: &str) -> io::Result<()> {
    #[cfg(unix)]
    {
        let mut options = OpenOptions::new();
        options.write(true).create(true).truncate(true).mode(0o600);
        let mut file = options.open(path)?;
        file.write_all(contents.as_bytes())?;
        Ok(())
    }

    #[cfg(not(unix))]
    {
        let mut file = OpenOptions::new()
            .write(true)
            .create(true)
            .truncate(true)
            .open(path)?;
        file.write_all(contents.as_bytes())
    }
}

/// Format duration in a human-readable way
fn format_duration(duration: Duration) -> String {
    if duration.as_secs() > 0 {
        format!("{:.2}s", duration.as_secs_f64())
    } else if duration.as_millis() > 0 {
        format!("{}ms", duration.as_millis())
    } else {
        format!("{}μs", duration.as_micros())
    }
}

/// Safely truncate a string at a character boundary
fn truncate_at_char_boundary(s: &str, max_bytes: usize) -> &str {
    if s.len() <= max_bytes {
        return s;
    }
    let mut end = max_bytes;
    while end > 0 && !s.is_char_boundary(end) {
        end -= 1;
    }
    &s[..end]
}

/// Print a debug header
pub fn debug_header(title: &str) {
    if !is_debug_enabled() {
        return;
    }
    tracing::debug!(target: "iris", "══════════════════════════════════════════════════════════════════════════════");
    tracing::debug!(target: "iris", "◆ {} ◆", title);
    tracing::debug!(target: "iris", "══════════════════════════════════════════════════════════════════════════════");
}

/// Print a debug section
pub fn debug_section(title: &str) {
    if !is_debug_enabled() {
        return;
    }
    tracing::debug!(target: "iris", "▸ {}", title);
    tracing::debug!(target: "iris", "──────────────────────────────────────────────────────────────────────────────");
}

/// Print tool call information
pub fn debug_tool_call(tool_name: &str, args: &str) {
    if !is_debug_enabled() {
        return;
    }

    tracing::debug!(target: "iris", "🔧 Tool Call: {}", tool_name);

    if !args.is_empty() {
        let truncated = if args.len() > 200 {
            format!("{}...", truncate_at_char_boundary(args, 200))
        } else {
            args.to_string()
        };
        tracing::debug!(target: "iris", "   Args: {}", truncated);
    }
}

/// Print tool response information
pub fn debug_tool_response(tool_name: &str, response: &str, duration: Duration) {
    if !is_debug_enabled() {
        return;
    }

    let truncated = if response.len() > 500 {
        format!("{}...", truncate_at_char_boundary(response, 500))
    } else {
        response.to_string()
    };

    tracing::debug!(target: "iris", "✓ Tool Response: {} ({})", tool_name, format_duration(duration));
    tracing::debug!(target: "iris", "   {}", truncated);
}

/// Print LLM request information
pub fn debug_llm_request(prompt: &str, max_tokens: Option<usize>) {
    if !is_debug_enabled() {
        return;
    }

    let char_count = prompt.chars().count();
    let word_count = prompt.split_whitespace().count();

    tracing::debug!(target: "iris", "🧠 LLM Request: {} chars, {} words {}",
        char_count,
        word_count,
        max_tokens.map(|t| format!("(max {} tokens)", t)).unwrap_or_default()
    );

    // Show first few lines of prompt
    for line in prompt.lines().take(5) {
        let truncated = if line.len() > 120 {
            format!("{}...", truncate_at_char_boundary(line, 120))
        } else {
            line.to_string()
        };
        tracing::debug!(target: "iris", "   {}", truncated);
    }
    if prompt.lines().count() > 5 {
        tracing::debug!(target: "iris", "   ... ({} more lines)", prompt.lines().count() - 5);
    }

    // Save full prompt to debug artifact
    if let Ok(path) = write_debug_artifact("iris_last_prompt.txt", prompt) {
        tracing::debug!(target: "iris", "   Full prompt saved to: {}", path.display());
    }
}

/// Print streaming chunk
pub fn debug_stream_chunk(_chunk: &str, chunk_number: usize) {
    if !is_debug_enabled() {
        return;
    }

    // Only print every 10th chunk to avoid overwhelming output
    if chunk_number.is_multiple_of(10) {
        tracing::debug!(target: "iris", "▹ chunk #{}", chunk_number);
    }
}

/// Print complete LLM response
pub fn debug_llm_response(response: &str, duration: Duration, tokens_used: Option<usize>) {
    if !is_debug_enabled() {
        return;
    }

    let char_count = response.chars().count();
    let word_count = response.split_whitespace().count();

    tracing::debug!(target: "iris", "✨ LLM Response: {} chars, {} words ({})",
        char_count,
        word_count,
        format_duration(duration)
    );

    if let Some(tokens) = tokens_used {
        tracing::debug!(target: "iris", "   Tokens: {}", tokens);
    }

    // Save full response to file for deep debugging
    if let Ok(path) = write_debug_artifact("iris_last_response.txt", response) {
        tracing::debug!(target: "iris", "   Full response saved to: {}", path.display());
    }

    // Show response (truncated if too long)
    let truncated = if response.len() > 1000 {
        format!(
            "{}...\n\n... ({} more characters)",
            truncate_at_char_boundary(response, 1000),
            response.len() - 1000
        )
    } else {
        response.to_string()
    };
    for line in truncated.lines() {
        tracing::debug!(target: "iris", "{}", line);
    }
}

/// Print JSON parsing attempt
pub fn debug_json_parse_attempt(json_str: &str) {
    if !is_debug_enabled() {
        return;
    }

    tracing::debug!(target: "iris", "📝 JSON Parse Attempt: {} chars", json_str.len());

    // Show first 500 chars
    let head = if json_str.len() > 500 {
        format!("{}...", truncate_at_char_boundary(json_str, 500))
    } else {
        json_str.to_string()
    };
    tracing::debug!(target: "iris", "{}", head);

    // Show last 200 chars to see where it got cut off
    if json_str.len() > 700 {
        tracing::debug!(target: "iris", "... truncated ...");
        let mut tail_start = json_str.len().saturating_sub(200);
        while tail_start < json_str.len() && !json_str.is_char_boundary(tail_start) {
            tail_start += 1;
        }
        tracing::debug!(target: "iris", "{}", &json_str[tail_start..]);
    }
}

/// Print JSON parse success
pub fn debug_json_parse_success(type_name: &str) {
    if !is_debug_enabled() {
        return;
    }
    tracing::debug!(target: "iris", "✓ JSON Parsed: {}", type_name);
}

/// Print JSON parse error
pub fn debug_json_parse_error(error: &str) {
    if !is_debug_enabled() {
        return;
    }
    tracing::warn!(target: "iris", "✗ JSON Parse Error: {}", error);
}

/// Print context management decision
pub fn debug_context_management(action: &str, details: &str) {
    if !is_debug_enabled() {
        return;
    }
    tracing::debug!(target: "iris", "🔍 {} {}", action, details);
}

/// Print an error
pub fn debug_error(error: &str) {
    if !is_debug_enabled() {
        return;
    }
    tracing::error!(target: "iris", "✗ Error: {}", error);
}

/// Print a warning
pub fn debug_warning(warning: &str) {
    if !is_debug_enabled() {
        return;
    }
    tracing::warn!(target: "iris", "⚠ {}", warning);
}

/// Print agent phase change
pub fn debug_phase_change(phase: &str) {
    if !is_debug_enabled() {
        return;
    }
    tracing::debug!(target: "iris", "◆ {}", phase);
    tracing::debug!(target: "iris", "──────────────────────────────────────────────────────────────────────────────");
}

/// Timer for measuring operation duration
pub struct DebugTimer {
    start: Instant,
    operation: String,
}

impl DebugTimer {
    pub fn start(operation: &str) -> Self {
        if is_debug_enabled() {
            tracing::debug!(target: "iris", "⏱ Started: {}", operation);
        }

        Self {
            start: Instant::now(),
            operation: operation.to_string(),
        }
    }

    pub fn finish(self) {
        if is_debug_enabled() {
            let duration = self.start.elapsed();
            tracing::debug!(target: "iris", "✓ Completed: {} ({})", self.operation, format_duration(duration));
        }
    }
}
