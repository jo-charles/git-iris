//! Debug observability module for Iris agent operations
//!
//! Provides beautiful, color-coded real-time visibility into agent execution including:
//! - Tool calls and responses
//! - LLM requests and streaming responses
//! - Context management decisions
//! - JSON parsing and validation
//! - Error states and recovery

use colored::*;
use std::sync::atomic::{AtomicBool, Ordering};
use std::time::{Duration, Instant};

// SilkCircuit Neon Color Palette
const ELECTRIC_PURPLE: (u8, u8, u8) = (225, 53, 255);
const NEON_CYAN: (u8, u8, u8) = (128, 255, 234);
const CORAL: (u8, u8, u8) = (255, 106, 193);
const ELECTRIC_YELLOW: (u8, u8, u8) = (241, 250, 140);
const SUCCESS_GREEN: (u8, u8, u8) = (80, 250, 123);
const ERROR_RED: (u8, u8, u8) = (255, 99, 99);

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

/// Create a section divider
fn divider(symbol: &str, color: (u8, u8, u8)) -> String {
    symbol.repeat(80).truecolor(color.0, color.1, color.2).to_string()
}

/// Create a timestamp string
fn timestamp() -> String {
    format!("[{}]", chrono::Local::now().format("%H:%M:%S%.3f"))
        .truecolor(ELECTRIC_YELLOW.0, ELECTRIC_YELLOW.1, ELECTRIC_YELLOW.2)
        .to_string()
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

/// Print a debug header
pub fn debug_header(title: &str) {
    if !is_debug_enabled() {
        return;
    }

    println!();
    println!("{}", divider("═", ELECTRIC_PURPLE));
    println!(
        "{} {} {}",
        "◆".truecolor(ELECTRIC_PURPLE.0, ELECTRIC_PURPLE.1, ELECTRIC_PURPLE.2),
        title.truecolor(ELECTRIC_PURPLE.0, ELECTRIC_PURPLE.1, ELECTRIC_PURPLE.2).bold(),
        "◆".truecolor(ELECTRIC_PURPLE.0, ELECTRIC_PURPLE.1, ELECTRIC_PURPLE.2)
    );
    println!("{}", divider("═", ELECTRIC_PURPLE));
    println!();
}

/// Print a debug section
pub fn debug_section(title: &str) {
    if !is_debug_enabled() {
        return;
    }

    println!();
    println!(
        "{} {} {}",
        "▸".truecolor(NEON_CYAN.0, NEON_CYAN.1, NEON_CYAN.2).bold(),
        title.truecolor(NEON_CYAN.0, NEON_CYAN.1, NEON_CYAN.2).bold(),
        timestamp()
    );
    println!("{}", divider("─", NEON_CYAN));
}

/// Print tool call information
pub fn debug_tool_call(tool_name: &str, args: &str) {
    if !is_debug_enabled() {
        return;
    }

    println!(
        "{} {} {} {}",
        timestamp(),
        "🔧".truecolor(NEON_CYAN.0, NEON_CYAN.1, NEON_CYAN.2),
        "Tool Call:".truecolor(NEON_CYAN.0, NEON_CYAN.1, NEON_CYAN.2).bold(),
        tool_name.truecolor(CORAL.0, CORAL.1, CORAL.2).bold()
    );

    if !args.is_empty() {
        let truncated = if args.len() > 200 {
            format!("{}...", &args[..200])
        } else {
            args.to_string()
        };
        println!(
            "  {} {}",
            "Args:".truecolor(ELECTRIC_YELLOW.0, ELECTRIC_YELLOW.1, ELECTRIC_YELLOW.2),
            truncated.truecolor(255, 255, 255)
        );
    }
}

/// Print tool response information
pub fn debug_tool_response(tool_name: &str, response: &str, duration: Duration) {
    if !is_debug_enabled() {
        return;
    }

    let truncated = if response.len() > 500 {
        format!("{}...", &response[..500])
    } else {
        response.to_string()
    };

    println!(
        "{} {} {} {} {}",
        timestamp(),
        "✓".truecolor(SUCCESS_GREEN.0, SUCCESS_GREEN.1, SUCCESS_GREEN.2).bold(),
        "Tool Response:".truecolor(SUCCESS_GREEN.0, SUCCESS_GREEN.1, SUCCESS_GREEN.2).bold(),
        tool_name.truecolor(CORAL.0, CORAL.1, CORAL.2).bold(),
        format!("({})", format_duration(duration)).truecolor(ELECTRIC_YELLOW.0, ELECTRIC_YELLOW.1, ELECTRIC_YELLOW.2)
    );
    println!(
        "  {}",
        truncated.truecolor(255, 255, 255)
    );
}

/// Print LLM request information
pub fn debug_llm_request(prompt: &str, max_tokens: Option<usize>) {
    if !is_debug_enabled() {
        return;
    }

    let char_count = prompt.chars().count();
    let word_count = prompt.split_whitespace().count();

    println!(
        "{} {} {} {} {}",
        timestamp(),
        "🧠".truecolor(ELECTRIC_PURPLE.0, ELECTRIC_PURPLE.1, ELECTRIC_PURPLE.2),
        "LLM Request:".truecolor(ELECTRIC_PURPLE.0, ELECTRIC_PURPLE.1, ELECTRIC_PURPLE.2).bold(),
        format!("{} chars, {} words", char_count, word_count).truecolor(ELECTRIC_YELLOW.0, ELECTRIC_YELLOW.1, ELECTRIC_YELLOW.2),
        max_tokens.map(|t| format!("(max {} tokens)", t)).unwrap_or_default().truecolor(ELECTRIC_YELLOW.0, ELECTRIC_YELLOW.1, ELECTRIC_YELLOW.2)
    );

    // Show first few lines of prompt
    let lines: Vec<&str> = prompt.lines().take(5).collect();
    for line in lines {
        let truncated = if line.len() > 120 {
            format!("{}...", &line[..120])
        } else {
            line.to_string()
        };
        println!("  {}", truncated.truecolor(200, 200, 200));
    }
    if prompt.lines().count() > 5 {
        println!(
            "  {}",
            format!("... ({} more lines)", prompt.lines().count() - 5)
                .truecolor(ELECTRIC_YELLOW.0, ELECTRIC_YELLOW.1, ELECTRIC_YELLOW.2).italic()
        );
    }
}

/// Print streaming chunk
pub fn debug_stream_chunk(_chunk: &str, chunk_number: usize) {
    if !is_debug_enabled() {
        return;
    }

    // Only print every 10th chunk to avoid overwhelming output
    if chunk_number % 10 == 0 {
        println!(
            "{} {} #{}",
            timestamp(),
            "▹".truecolor(NEON_CYAN.0, NEON_CYAN.1, NEON_CYAN.2),
            chunk_number.to_string().truecolor(CORAL.0, CORAL.1, CORAL.2)
        );
    }
}

/// Print complete LLM response
pub fn debug_llm_response(response: &str, duration: Duration, tokens_used: Option<usize>) {
    if !is_debug_enabled() {
        return;
    }

    let char_count = response.chars().count();
    let word_count = response.split_whitespace().count();

    println!();
    println!(
        "{} {} {} {} {}",
        timestamp(),
        "✨".truecolor(SUCCESS_GREEN.0, SUCCESS_GREEN.1, SUCCESS_GREEN.2),
        "LLM Response:".truecolor(SUCCESS_GREEN.0, SUCCESS_GREEN.1, SUCCESS_GREEN.2).bold(),
        format!("{} chars, {} words", char_count, word_count).truecolor(ELECTRIC_YELLOW.0, ELECTRIC_YELLOW.1, ELECTRIC_YELLOW.2),
        format!("({})", format_duration(duration)).truecolor(ELECTRIC_YELLOW.0, ELECTRIC_YELLOW.1, ELECTRIC_YELLOW.2)
    );

    if let Some(tokens) = tokens_used {
        println!(
            "  {} {}",
            "Tokens:".truecolor(ELECTRIC_YELLOW.0, ELECTRIC_YELLOW.1, ELECTRIC_YELLOW.2),
            tokens.to_string().truecolor(CORAL.0, CORAL.1, CORAL.2).bold()
        );
    }

    // Show response (truncated if too long)
    let truncated = if response.len() > 1000 {
        format!("{}...\n\n... ({} more characters)", &response[..1000], response.len() - 1000)
    } else {
        response.to_string()
    };
    println!("{}", truncated.truecolor(255, 255, 255));
}

/// Print JSON parsing attempt
pub fn debug_json_parse_attempt(json_str: &str) {
    if !is_debug_enabled() {
        return;
    }

    println!(
        "{} {} {} {} chars",
        timestamp(),
        "📝".truecolor(ELECTRIC_YELLOW.0, ELECTRIC_YELLOW.1, ELECTRIC_YELLOW.2),
        "JSON Parse Attempt:".truecolor(ELECTRIC_YELLOW.0, ELECTRIC_YELLOW.1, ELECTRIC_YELLOW.2).bold(),
        json_str.len().to_string().truecolor(CORAL.0, CORAL.1, CORAL.2).bold()
    );

    // Show first 500 chars
    let head = if json_str.len() > 500 {
        format!("{}...", &json_str[..500])
    } else {
        json_str.to_string()
    };
    println!("{}", head.truecolor(200, 200, 200));

    // Show last 200 chars to see where it got cut off
    if json_str.len() > 700 {
        println!("\n... truncated ...\n");
        let tail_start = json_str.len().saturating_sub(200);
        println!("{}", &json_str[tail_start..].truecolor(200, 200, 200));
    }
}

/// Print JSON parse success
pub fn debug_json_parse_success(type_name: &str) {
    if !is_debug_enabled() {
        return;
    }

    println!(
        "{} {} {} {}",
        timestamp(),
        "✓".truecolor(SUCCESS_GREEN.0, SUCCESS_GREEN.1, SUCCESS_GREEN.2).bold(),
        "JSON Parsed:".truecolor(SUCCESS_GREEN.0, SUCCESS_GREEN.1, SUCCESS_GREEN.2).bold(),
        type_name.truecolor(CORAL.0, CORAL.1, CORAL.2).bold()
    );
}

/// Print JSON parse error
pub fn debug_json_parse_error(error: &str) {
    if !is_debug_enabled() {
        return;
    }

    println!(
        "{} {} {}",
        timestamp(),
        "✗".truecolor(ERROR_RED.0, ERROR_RED.1, ERROR_RED.2).bold(),
        "JSON Parse Error:".truecolor(ERROR_RED.0, ERROR_RED.1, ERROR_RED.2).bold()
    );
    println!(
        "  {}",
        error.truecolor(ERROR_RED.0, ERROR_RED.1, ERROR_RED.2)
    );
}

/// Print context management decision
pub fn debug_context_management(action: &str, details: &str) {
    if !is_debug_enabled() {
        return;
    }

    println!(
        "{} {} {} {}",
        timestamp(),
        "🔍".truecolor(ELECTRIC_PURPLE.0, ELECTRIC_PURPLE.1, ELECTRIC_PURPLE.2),
        action.truecolor(ELECTRIC_PURPLE.0, ELECTRIC_PURPLE.1, ELECTRIC_PURPLE.2).bold(),
        details.truecolor(ELECTRIC_YELLOW.0, ELECTRIC_YELLOW.1, ELECTRIC_YELLOW.2)
    );
}

/// Print an error
pub fn debug_error(error: &str) {
    if !is_debug_enabled() {
        return;
    }

    println!();
    println!("{}", divider("─", ERROR_RED));
    println!(
        "{} {} {}",
        timestamp(),
        "✗".truecolor(ERROR_RED.0, ERROR_RED.1, ERROR_RED.2).bold(),
        "Error:".truecolor(ERROR_RED.0, ERROR_RED.1, ERROR_RED.2).bold()
    );
    println!(
        "  {}",
        error.truecolor(ERROR_RED.0, ERROR_RED.1, ERROR_RED.2)
    );
    println!("{}", divider("─", ERROR_RED));
}

/// Print a warning
pub fn debug_warning(warning: &str) {
    if !is_debug_enabled() {
        return;
    }

    println!(
        "{} {} {}",
        timestamp(),
        "⚠".truecolor(ELECTRIC_YELLOW.0, ELECTRIC_YELLOW.1, ELECTRIC_YELLOW.2).bold(),
        warning.truecolor(ELECTRIC_YELLOW.0, ELECTRIC_YELLOW.1, ELECTRIC_YELLOW.2)
    );
}

/// Print agent phase change
pub fn debug_phase_change(phase: &str) {
    if !is_debug_enabled() {
        return;
    }

    println!();
    println!(
        "{} {} {}",
        timestamp(),
        "◆".truecolor(ELECTRIC_PURPLE.0, ELECTRIC_PURPLE.1, ELECTRIC_PURPLE.2).bold(),
        phase.truecolor(ELECTRIC_PURPLE.0, ELECTRIC_PURPLE.1, ELECTRIC_PURPLE.2).bold()
    );
    println!("{}", divider("─", ELECTRIC_PURPLE));
}

/// Timer for measuring operation duration
pub struct DebugTimer {
    start: Instant,
    operation: String,
}

impl DebugTimer {
    pub fn start(operation: &str) -> Self {
        if is_debug_enabled() {
            println!(
                "{} {} {}",
                timestamp(),
                "⏱".truecolor(NEON_CYAN.0, NEON_CYAN.1, NEON_CYAN.2),
                format!("Started: {}", operation).truecolor(NEON_CYAN.0, NEON_CYAN.1, NEON_CYAN.2)
            );
        }

        Self {
            start: Instant::now(),
            operation: operation.to_string(),
        }
    }

    pub fn finish(self) {
        if is_debug_enabled() {
            let duration = self.start.elapsed();
            println!(
                "{} {} {} {}",
                timestamp(),
                "✓".truecolor(SUCCESS_GREEN.0, SUCCESS_GREEN.1, SUCCESS_GREEN.2).bold(),
                format!("Completed: {}", self.operation).truecolor(SUCCESS_GREEN.0, SUCCESS_GREEN.1, SUCCESS_GREEN.2),
                format!("({})", format_duration(duration)).truecolor(ELECTRIC_YELLOW.0, ELECTRIC_YELLOW.1, ELECTRIC_YELLOW.2)
            );
        }
    }
}
