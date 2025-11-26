use crate::messages::ColoredMessage;
use crate::ui::{
    AURORA_GREEN, CELESTIAL_BLUE, GALAXY_PINK, METEOR_RED, NEBULA_PURPLE, PLASMA_CYAN,
    SOLAR_YELLOW, STARLIGHT,
};
use ratatui::style::Color;
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

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

/// Status phases for the Iris agent
#[derive(Debug, Clone, PartialEq)]
pub enum IrisPhase {
    Initializing,
    Planning,
    ToolExecution { tool_name: String, reason: String },
    PlanExpansion,
    Synthesis,
    Analysis,
    Generation,
    Completed,
    Error(String),
}

/// Token counting information for live updates
#[derive(Debug, Clone, Default)]
pub struct TokenMetrics {
    pub input_tokens: u32,
    pub output_tokens: u32,
    pub total_tokens: u32,
    pub tokens_per_second: f32,
    pub estimated_remaining: Option<u32>,
}

/// Status tracker for Iris agent operations with dynamic messages and live token counting
#[derive(Debug, Clone)]
pub struct IrisStatus {
    pub phase: IrisPhase,
    pub message: String,
    pub color: Color,
    pub started_at: Instant,
    pub current_step: usize,
    pub total_steps: Option<usize>,
    pub tokens: TokenMetrics,
    pub is_streaming: bool,
}

impl IrisStatus {
    pub fn new() -> Self {
        Self {
            phase: IrisPhase::Initializing,
            message: "🤖 Initializing...".to_string(),
            color: CELESTIAL_BLUE,
            started_at: Instant::now(),
            current_step: 0,
            total_steps: None,
            tokens: TokenMetrics::default(),
            is_streaming: false,
        }
    }

    /// Create a dynamic status with LLM-generated message (constrained to 80 chars)
    pub fn dynamic(phase: IrisPhase, message: String, step: usize, total: Option<usize>) -> Self {
        let color = match phase {
            IrisPhase::Initializing => CELESTIAL_BLUE,
            IrisPhase::Planning => NEBULA_PURPLE,
            IrisPhase::ToolExecution { .. } | IrisPhase::Completed => AURORA_GREEN,
            IrisPhase::PlanExpansion => PLASMA_CYAN,
            IrisPhase::Synthesis => GALAXY_PINK,
            IrisPhase::Analysis => SOLAR_YELLOW,
            IrisPhase::Generation => STARLIGHT,
            IrisPhase::Error(_) => METEOR_RED,
        };

        // Constrain message to 80 characters as requested
        let constrained_message = if message.len() > 80 {
            format!("{}...", truncate_at_char_boundary(&message, 77))
        } else {
            message
        };

        Self {
            phase,
            message: constrained_message,
            color,
            started_at: Instant::now(),
            current_step: step,
            total_steps: total,
            tokens: TokenMetrics::default(),
            is_streaming: false,
        }
    }

    /// Create dynamic streaming status with live token counting
    pub fn streaming(
        message: String,
        tokens: TokenMetrics,
        step: usize,
        total: Option<usize>,
    ) -> Self {
        // Constrain message to 80 characters
        let constrained_message = if message.len() > 80 {
            format!("{}...", truncate_at_char_boundary(&message, 77))
        } else {
            message
        };

        Self {
            phase: IrisPhase::Generation,
            message: constrained_message,
            color: STARLIGHT,
            started_at: Instant::now(),
            current_step: step,
            total_steps: total,
            tokens,
            is_streaming: true,
        }
    }

    /// Update token metrics during streaming
    pub fn update_tokens(&mut self, tokens: TokenMetrics) {
        self.tokens = tokens;

        // Update tokens per second based on elapsed time
        let elapsed = self.started_at.elapsed().as_secs_f32();
        if elapsed > 0.0 {
            #[allow(clippy::cast_precision_loss, clippy::as_conversions)]
            {
                self.tokens.tokens_per_second = self.tokens.output_tokens as f32 / elapsed;
            }
        }
    }

    /// Create error status
    pub fn error(error: &str) -> Self {
        let constrained_message = if error.len() > 35 {
            format!("❌ {}...", truncate_at_char_boundary(error, 32))
        } else {
            format!("❌ {error}")
        };

        Self {
            phase: IrisPhase::Error(error.to_string()),
            message: constrained_message,
            color: METEOR_RED,
            started_at: Instant::now(),
            current_step: 0,
            total_steps: None,
            tokens: TokenMetrics::default(),
            is_streaming: false,
        }
    }

    /// Create completed status
    pub fn completed() -> Self {
        Self {
            phase: IrisPhase::Completed,
            message: "🎉 Done!".to_string(),
            color: AURORA_GREEN,
            started_at: Instant::now(),
            current_step: 0,
            total_steps: None,
            tokens: TokenMetrics::default(),
            is_streaming: false,
        }
    }

    pub fn duration(&self) -> Duration {
        self.started_at.elapsed()
    }

    #[allow(clippy::cast_precision_loss, clippy::as_conversions)]
    pub fn progress_percentage(&self) -> f32 {
        if let Some(total) = self.total_steps {
            (self.current_step as f32 / total as f32) * 100.0
        } else {
            0.0
        }
    }

    /// Format status for display - clean and minimal
    pub fn format_for_display(&self) -> String {
        // Just the message - clean and elegant
        self.message.clone()
    }
}

impl Default for IrisStatus {
    fn default() -> Self {
        Self::new()
    }
}

/// Global status tracker for Iris agent
pub struct IrisStatusTracker {
    status: Arc<Mutex<IrisStatus>>,
}

impl IrisStatusTracker {
    pub fn new() -> Self {
        Self {
            status: Arc::new(Mutex::new(IrisStatus::new())),
        }
    }

    /// Update status with dynamic message
    pub fn update(&self, status: IrisStatus) {
        crate::log_debug!(
            "📋 Status: Updating to phase: {:?}, message: '{}'",
            status.phase,
            status.message
        );
        if let Ok(mut current_status) = self.status.lock() {
            *current_status = status;
            crate::log_debug!("📋 Status: Update completed successfully");
        } else {
            crate::log_debug!("📋 Status: ⚠️ Failed to acquire status lock");
        }
    }

    /// Update with dynamic LLM-generated message
    pub fn update_dynamic(
        &self,
        phase: IrisPhase,
        message: String,
        step: usize,
        total: Option<usize>,
    ) {
        crate::log_debug!(
            "🎯 Status: Dynamic update - phase: {:?}, message: '{}', step: {}/{:?}",
            phase,
            message,
            step,
            total
        );
        self.update(IrisStatus::dynamic(phase, message, step, total));
    }

    /// Update streaming status with token metrics
    pub fn update_streaming(
        &self,
        message: String,
        tokens: TokenMetrics,
        step: usize,
        total: Option<usize>,
    ) {
        self.update(IrisStatus::streaming(message, tokens, step, total));
    }

    /// Update only token metrics for current status
    pub fn update_tokens(&self, tokens: TokenMetrics) {
        if let Ok(mut status) = self.status.lock() {
            status.update_tokens(tokens);
        }
    }

    pub fn get_current(&self) -> IrisStatus {
        self.status.lock().map_or_else(
            |_| IrisStatus::error("Status lock poisoned"),
            |guard| guard.clone(),
        )
    }

    pub fn get_for_spinner(&self) -> ColoredMessage {
        let status = self.get_current();
        ColoredMessage {
            text: status.format_for_display(),
            color: status.color,
        }
    }

    /// Set error status
    pub fn error(&self, error: &str) {
        self.update(IrisStatus::error(error));
    }

    /// Set completed status
    pub fn completed(&self) {
        self.update(IrisStatus::completed());
    }
}

impl Default for IrisStatusTracker {
    fn default() -> Self {
        Self::new()
    }
}

/// Global instance of the Iris status tracker
pub static IRIS_STATUS: std::sync::LazyLock<IrisStatusTracker> =
    std::sync::LazyLock::new(IrisStatusTracker::new);

/// Global flag to track if agent mode is enabled (enabled by default)
pub static AGENT_MODE_ENABLED: std::sync::LazyLock<std::sync::Arc<std::sync::atomic::AtomicBool>> =
    std::sync::LazyLock::new(|| std::sync::Arc::new(std::sync::atomic::AtomicBool::new(true)));

/// Enable agent mode globally
pub fn enable_agent_mode() {
    AGENT_MODE_ENABLED.store(true, std::sync::atomic::Ordering::Relaxed);
}

/// Check if agent mode is enabled
pub fn is_agent_mode_enabled() -> bool {
    AGENT_MODE_ENABLED.load(std::sync::atomic::Ordering::Relaxed)
}

/// Helper macros for dynamic status updates with LLM messages
#[macro_export]
macro_rules! iris_status_dynamic {
    ($phase:expr, $message:expr, $step:expr) => {
        $crate::agents::status::IRIS_STATUS.update_dynamic(
            $phase,
            $message.to_string(),
            $step,
            None,
        );
    };
    ($phase:expr, $message:expr, $step:expr, $total:expr) => {
        $crate::agents::status::IRIS_STATUS.update_dynamic(
            $phase,
            $message.to_string(),
            $step,
            Some($total),
        );
    };
}

#[macro_export]
macro_rules! iris_status_streaming {
    ($message:expr, $tokens:expr) => {
        $crate::agents::status::IRIS_STATUS.update_streaming(
            $message.to_string(),
            $tokens,
            0,
            None,
        );
    };
    ($message:expr, $tokens:expr, $step:expr, $total:expr) => {
        $crate::agents::status::IRIS_STATUS.update_streaming(
            $message.to_string(),
            $tokens,
            $step,
            Some($total),
        );
    };
}

#[macro_export]
macro_rules! iris_status_tokens {
    ($tokens:expr) => {
        $crate::agents::status::IRIS_STATUS.update_tokens($tokens);
    };
}

#[macro_export]
macro_rules! iris_status_error {
    ($error:expr) => {
        $crate::agents::status::IRIS_STATUS.error($error);
    };
}

#[macro_export]
macro_rules! iris_status_completed {
    () => {
        $crate::agents::status::IRIS_STATUS.completed();
    };
}
