use colored::Colorize;
use console::Term;
use indicatif::{ProgressBar, ProgressStyle};
use parking_lot::Mutex;
use ratatui::style::Color;
use std::fmt::Write;
use std::time::Duration;

// ═══════════════════════════════════════════════════════════════════════════════
// SilkCircuit Neon — Electric meets elegant
// ═══════════════════════════════════════════════════════════════════════════════

/// Electric Purple #e135ff — Keywords, control flow, importance
pub const ELECTRIC_PURPLE: Color = Color::Rgb(225, 53, 255);
/// Pure Pink #ff00ff — Tags, booleans, maximum emphasis
pub const PURE_PINK: Color = Color::Rgb(255, 0, 255);
/// Soft Pink #ff99ff — Strings, secondary emphasis
pub const SOFT_PINK: Color = Color::Rgb(255, 153, 255);
/// Neon Cyan #80ffea — Functions, methods, interactions
pub const NEON_CYAN: Color = Color::Rgb(128, 255, 234);
/// Bright Cyan #00ffcc — High-energy interaction
pub const BRIGHT_CYAN: Color = Color::Rgb(0, 255, 204);
/// Coral #ff6ac1 — Numbers, constants
pub const CORAL: Color = Color::Rgb(255, 106, 193);
/// Electric Yellow #f1fa8c — Classes, types, warnings
pub const ELECTRIC_YELLOW: Color = Color::Rgb(241, 250, 140);
/// Success Green #50fa7b — Success states, confirmations
pub const SUCCESS_GREEN: Color = Color::Rgb(80, 250, 123);
/// Error Red #ff6363 — Errors, danger, removals
pub const ERROR_RED: Color = Color::Rgb(255, 99, 99);
/// Soft White #f8f8f2 — Primary text
pub const SOFT_WHITE: Color = Color::Rgb(248, 248, 242);
/// Purple Muted #6272a4 — Comments, secondary text
pub const PURPLE_MUTED: Color = Color::Rgb(98, 114, 164);
/// Dim Gray — Alias for purple muted
pub const DIM_GRAY: Color = PURPLE_MUTED;
/// Deep Purple #bd93f9 — Accents, borders
pub const DEEP_PURPLE: Color = Color::Rgb(189, 147, 249);
/// Void #282a36 — Background hints, surfaces
pub const VOID: Color = Color::Rgb(40, 42, 54);
/// Dark Base #12101a — Deep background
pub const DARK_BASE: Color = Color::Rgb(18, 16, 26);
/// Highlight #1a162a — Elevated surfaces
pub const HIGHLIGHT: Color = Color::Rgb(26, 22, 42);

// ═══════════════════════════════════════════════════════════════════════════════
// Legacy aliases (for backwards compatibility during transition)
// ═══════════════════════════════════════════════════════════════════════════════

pub const STARLIGHT: Color = SOFT_WHITE;
pub const NEBULA_PURPLE: Color = DEEP_PURPLE;
pub const CELESTIAL_BLUE: Color = NEON_CYAN;
pub const SOLAR_YELLOW: Color = ELECTRIC_YELLOW;
pub const AURORA_GREEN: Color = SUCCESS_GREEN;
pub const PLASMA_CYAN: Color = NEON_CYAN;
pub const METEOR_RED: Color = ERROR_RED;
pub const GALAXY_PINK: Color = CORAL;
pub const COMET_ORANGE: Color = ELECTRIC_YELLOW;
pub const BLACK_HOLE: Color = VOID;

/// Track quiet mode state
static QUIET_MODE: std::sync::LazyLock<Mutex<bool>> =
    std::sync::LazyLock::new(|| Mutex::new(false));

/// Enable or disable quiet mode
pub fn set_quiet_mode(enabled: bool) {
    let mut quiet_mode = QUIET_MODE.lock();
    *quiet_mode = enabled;
}

/// Check if quiet mode is enabled
pub fn is_quiet_mode() -> bool {
    *QUIET_MODE.lock()
}

pub fn create_spinner(message: &str) -> ProgressBar {
    // Don't create a spinner in quiet mode
    if is_quiet_mode() {
        return ProgressBar::hidden();
    }

    let pb = ProgressBar::new_spinner();

    // Use agent-aware spinner if agent mode is enabled
    if crate::agents::status::is_agent_mode_enabled() {
        pb.set_style(
            ProgressStyle::default_spinner()
                .tick_chars("⠋⠙⠹⠸⠼⠴⠦⠧⠇⠏")
                .template("{spinner:.bright_cyan.bold} {msg}")
                .expect("Could not set spinner style"),
        );

        // Start with Iris initialization message
        pb.set_message("◎ Iris initializing...");

        // Set up a custom callback to update the message from Iris status
        let pb_clone = pb.clone();
        tokio::spawn(async move {
            let mut interval = tokio::time::interval(tokio::time::Duration::from_millis(200));
            loop {
                interval.tick().await;
                let status_message = crate::agents::status::IRIS_STATUS.get_for_spinner();
                pb_clone.set_message(status_message.text);
            }
        });

        pb.enable_steady_tick(Duration::from_millis(100));
    } else {
        pb.set_style(
            ProgressStyle::default_spinner()
                .tick_chars("✦✧✶✷✸✹✺✻✼✽")
                .template("{spinner} {msg}")
                .expect("Could not set spinner style"),
        );
        pb.set_message(message.to_string());
        pb.enable_steady_tick(Duration::from_millis(100));
    }

    pb
}

pub fn print_info(message: &str) {
    if !is_quiet_mode() {
        println!("{}", message.cyan().bold());
    }
}

pub fn print_warning(message: &str) {
    if !is_quiet_mode() {
        println!("{}", message.yellow().bold());
    }
}

pub fn print_error(message: &str) {
    // Always print errors, even in quiet mode
    eprintln!("{}", message.red().bold());
}

pub fn print_success(message: &str) {
    if !is_quiet_mode() {
        println!("{}", message.green().bold());
    }
}

pub fn print_version(version: &str) {
    if !is_quiet_mode() {
        println!(
            "{} {} {}",
            "🔮 Git-Iris".magenta().bold(),
            "version".cyan(),
            version.green()
        );
    }
}

/// Print content with decorative borders
pub fn print_bordered_content(content: &str) {
    if !is_quiet_mode() {
        println!("{}", "━".repeat(50).bright_purple());
        println!("{content}");
        println!("{}", "━".repeat(50).bright_purple());
    }
}

/// Print a simple message (respects quiet mode)
pub fn print_message(message: &str) {
    if !is_quiet_mode() {
        println!("{message}");
    }
}

/// Print an empty line (respects quiet mode)
pub fn print_newline() {
    if !is_quiet_mode() {
        println!();
    }
}

/// Create gradient text with `SilkCircuit` Electric Purple -> Neon Cyan
pub fn create_gradient_text(text: &str) -> String {
    let gradient = vec![
        (225, 53, 255),  // Electric Purple
        (200, 100, 255), // Mid purple
        (180, 150, 250), // Light purple
        (150, 200, 245), // Purple-cyan
        (128, 255, 234), // Neon Cyan
    ];

    apply_gradient(text, &gradient)
}

/// Create secondary gradient with `SilkCircuit` Coral -> Electric Yellow
pub fn create_secondary_gradient_text(text: &str) -> String {
    let gradient = vec![
        (255, 106, 193), // Coral
        (255, 150, 180), // Light coral
        (255, 200, 160), // Coral-yellow
        (248, 230, 140), // Light yellow
        (241, 250, 140), // Electric Yellow
    ];

    apply_gradient(text, &gradient)
}

fn apply_gradient(text: &str, gradient: &[(u8, u8, u8)]) -> String {
    let chars: Vec<char> = text.chars().collect();
    let chars_len = chars.len();
    let gradient_len = gradient.len();

    let mut result = String::new();

    if chars_len == 0 || gradient_len == 0 {
        return result;
    }

    chars.iter().enumerate().fold(&mut result, |acc, (i, &c)| {
        let index = if chars_len == 1 {
            0
        } else {
            i * (gradient_len - 1) / (chars_len - 1)
        };
        let (r, g, b) = gradient[index];
        write!(acc, "{}", c.to_string().truecolor(r, g, b)).expect("writing to string cannot fail");
        acc
    });

    result
}

pub fn write_gradient_text(
    term: &Term,
    text: &str,
    gradient: &[(u8, u8, u8)],
) -> std::io::Result<()> {
    let gradient_text = apply_gradient(text, gradient);
    term.write_line(&gradient_text)
}

pub fn write_colored_text(term: &Term, text: &str, color: (u8, u8, u8)) -> std::io::Result<()> {
    let colored_text = text.truecolor(color.0, color.1, color.2);
    term.write_line(&colored_text)
}

pub fn write_bold_text(term: &Term, text: &str) -> std::io::Result<()> {
    term.write_line(&text.bold())
}
