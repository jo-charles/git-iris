use crate::messages::ColoredMessage;
use crate::ui::{
    AURORA_GREEN, CELESTIAL_BLUE, GALAXY_PINK, METEOR_RED, NEBULA_PURPLE, PLASMA_CYAN,
    SOLAR_YELLOW, STARLIGHT,
};
use ratatui::style::Color;
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

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

/// Status tracker for Iris agent operations
#[derive(Debug, Clone)]
pub struct IrisStatus {
    pub phase: IrisPhase,
    pub message: String,
    pub color: Color,
    pub started_at: Instant,
    pub tools_executed: Vec<String>,
    pub current_step: usize,
    pub total_steps: Option<usize>,
}

impl IrisStatus {
    pub fn new() -> Self {
        Self {
            phase: IrisPhase::Initializing,
            message: "🤖 Iris is awakening...".to_string(),
            color: CELESTIAL_BLUE,
            started_at: Instant::now(),
            tools_executed: Vec::new(),
            current_step: 0,
            total_steps: None,
        }
    }

    pub fn planning() -> Self {
        Self {
            phase: IrisPhase::Planning,
            message: "🧠 Iris is planning her approach...".to_string(),
            color: NEBULA_PURPLE,
            started_at: Instant::now(),
            tools_executed: Vec::new(),
            current_step: 1,
            total_steps: Some(5),
        }
    }

    pub fn tool_execution(tool_name: &str, reason: &str) -> Self {
        let message = match tool_name {
            "Git Operations" => format!(
                "🔧 Iris is analyzing Git changes... ({})",
                reason.chars().take(40).collect::<String>()
            ),
            "File Analyzer" => format!(
                "📄 Iris is examining file contents... ({})",
                reason.chars().take(35).collect::<String>()
            ),
            "Code Search" => format!(
                "🔍 Iris is searching the codebase... ({})",
                reason.chars().take(35).collect::<String>()
            ),
            _ => format!(
                "🛠️ Iris is using {}... ({})",
                tool_name,
                reason.chars().take(40).collect::<String>()
            ),
        };

        Self {
            phase: IrisPhase::ToolExecution {
                tool_name: tool_name.to_string(),
                reason: reason.to_string(),
            },
            message,
            color: AURORA_GREEN,
            started_at: Instant::now(),
            tools_executed: Vec::new(),
            current_step: 2,
            total_steps: Some(5),
        }
    }

    pub fn plan_expansion() -> Self {
        Self {
            phase: IrisPhase::PlanExpansion,
            message: "🔄 Iris is adapting her plan based on discoveries...".to_string(),
            color: PLASMA_CYAN,
            started_at: Instant::now(),
            tools_executed: Vec::new(),
            current_step: 3,
            total_steps: Some(5),
        }
    }

    pub fn synthesis() -> Self {
        Self {
            phase: IrisPhase::Synthesis,
            message: "🧩 Iris is synthesizing insights from tools...".to_string(),
            color: GALAXY_PINK,
            started_at: Instant::now(),
            tools_executed: Vec::new(),
            current_step: 4,
            total_steps: Some(5),
        }
    }

    /// Create a dynamic status with custom message
    pub fn dynamic(phase: IrisPhase, message: String, step: usize, total: Option<usize>) -> Self {
        let color = match phase {
            IrisPhase::Initializing => CELESTIAL_BLUE,
            IrisPhase::Planning => NEBULA_PURPLE,
            IrisPhase::ToolExecution { .. } => AURORA_GREEN,
            IrisPhase::PlanExpansion => PLASMA_CYAN,
            IrisPhase::Synthesis => GALAXY_PINK,
            IrisPhase::Analysis => SOLAR_YELLOW,
            IrisPhase::Generation => STARLIGHT,
            IrisPhase::Completed => AURORA_GREEN,
            IrisPhase::Error(_) => METEOR_RED,
        };

        Self {
            phase,
            message,
            color,
            started_at: Instant::now(),
            tools_executed: Vec::new(),
            current_step: step,
            total_steps: total,
        }
    }

    pub fn analysis() -> Self {
        Self {
            phase: IrisPhase::Analysis,
            message: "🔬 Iris is performing deep analysis...".to_string(),
            color: SOLAR_YELLOW,
            started_at: Instant::now(),
            tools_executed: Vec::new(),
            current_step: 4,
            total_steps: Some(5),
        }
    }

    pub fn generation() -> Self {
        Self {
            phase: IrisPhase::Generation,
            message: "✨ Iris is crafting the perfect response...".to_string(),
            color: STARLIGHT,
            started_at: Instant::now(),
            tools_executed: Vec::new(),
            current_step: 5,
            total_steps: Some(5),
        }
    }

    pub fn completed() -> Self {
        Self {
            phase: IrisPhase::Completed,
            message: "🎉 Iris has completed her work!".to_string(),
            color: METEOR_RED,
            started_at: Instant::now(),
            tools_executed: Vec::new(),
            current_step: 5,
            total_steps: Some(5),
        }
    }

    pub fn error(error: &str) -> Self {
        Self {
            phase: IrisPhase::Error(error.to_string()),
            message: format!(
                "❌ Iris encountered an issue: {}",
                error.chars().take(50).collect::<String>()
            ),
            color: METEOR_RED,
            started_at: Instant::now(),
            tools_executed: Vec::new(),
            current_step: 0,
            total_steps: Some(5),
        }
    }

    pub fn add_tool_executed(&mut self, tool_name: String) {
        self.tools_executed.push(tool_name);
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

    pub fn format_for_display(&self) -> String {
        let duration = self.duration();
        let seconds = duration.as_secs();

        let progress_bar = if let Some(total) = self.total_steps {
            let filled = (self.current_step * 10) / total;
            let empty = 10 - filled;
            format!(" [{}{}]", "█".repeat(filled), "░".repeat(empty))
        } else {
            String::new()
        };

        if seconds > 0 {
            format!("{}{} ({}s)", self.message, progress_bar, seconds)
        } else {
            format!("{}{}", self.message, progress_bar)
        }
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

    pub fn update(&self, status: IrisStatus) {
        if let Ok(mut current_status) = self.status.lock() {
            *current_status = status;
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

    pub fn planning(&self) {
        self.update(IrisStatus::planning());
    }

    pub fn tool_execution(&self, tool_name: &str, reason: &str) {
        self.update(IrisStatus::tool_execution(tool_name, reason));
    }

    pub fn plan_expansion(&self) {
        self.update(IrisStatus::plan_expansion());
    }

    pub fn synthesis(&self) {
        self.update(IrisStatus::synthesis());
    }

    pub fn analysis(&self) {
        self.update(IrisStatus::analysis());
    }

    pub fn generation(&self) {
        self.update(IrisStatus::generation());
    }

    pub fn completed(&self) {
        self.update(IrisStatus::completed());
    }

    pub fn error(&self, error: &str) {
        self.update(IrisStatus::error(error));
    }

    pub fn add_tool_executed(&self, tool_name: String) {
        if let Ok(mut status) = self.status.lock() {
            status.add_tool_executed(tool_name);
        }
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

/// Global flag to track if agent mode is enabled
pub static AGENT_MODE_ENABLED: std::sync::LazyLock<std::sync::Arc<std::sync::atomic::AtomicBool>> =
    std::sync::LazyLock::new(|| std::sync::Arc::new(std::sync::atomic::AtomicBool::new(false)));

/// Enable agent mode globally
pub fn enable_agent_mode() {
    AGENT_MODE_ENABLED.store(true, std::sync::atomic::Ordering::Relaxed);
}

/// Check if agent mode is enabled
pub fn is_agent_mode_enabled() -> bool {
    AGENT_MODE_ENABLED.load(std::sync::atomic::Ordering::Relaxed)
}

/// Helper macros for easy status updates
#[macro_export]
macro_rules! iris_status_planning {
    () => {
        $crate::agents::status::IRIS_STATUS.planning();
    };
}

#[macro_export]
macro_rules! iris_status_tool {
    ($tool:expr, $reason:expr) => {
        $crate::agents::status::IRIS_STATUS.tool_execution($tool, $reason);
    };
}

#[macro_export]
macro_rules! iris_status_expansion {
    () => {
        $crate::agents::status::IRIS_STATUS.plan_expansion();
    };
}

#[macro_export]
macro_rules! iris_status_synthesis {
    () => {
        $crate::agents::status::IRIS_STATUS.synthesis();
    };
}

#[macro_export]
macro_rules! iris_status_analysis {
    () => {
        $crate::agents::status::IRIS_STATUS.analysis();
    };
}

#[macro_export]
macro_rules! iris_status_generation {
    () => {
        $crate::agents::status::IRIS_STATUS.generation();
    };
}

#[macro_export]
macro_rules! iris_status_completed {
    () => {
        $crate::agents::status::IRIS_STATUS.completed();
    };
}

#[macro_export]
macro_rules! iris_status_error {
    ($error:expr) => {
        $crate::agents::status::IRIS_STATUS.error($error);
    };
}
