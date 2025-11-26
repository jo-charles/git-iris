//! Tool registry for consistent tool attachment across agents
//!
//! This module provides macros and utilities for attaching the standard tool set
//! to agents in a consistent manner, preventing drift between main agents and subagents.

/// Attach the core analysis tools to an agent builder.
///
/// These are the standard tools available to all agents and subagents for
/// code analysis tasks. Does NOT include delegation tools (`Workspace`, `ParallelAnalyze`,
/// sub-agent) to prevent recursion.
///
/// # Usage
/// ```ignore
/// let agent = attach_core_tools!(client.agent(model).preamble("..."));
/// ```
#[macro_export]
macro_rules! attach_core_tools {
    ($builder:expr) => {{
        use $crate::agents::debug_tool::DebugTool;
        use $crate::agents::tools::{
            CodeSearch, FileRead, GitChangedFiles, GitDiff, GitLog, GitStatus, ProjectDocs,
        };

        $builder
            .tool(DebugTool::new(GitStatus))
            .tool(DebugTool::new(GitDiff))
            .tool(DebugTool::new(GitLog))
            .tool(DebugTool::new(GitChangedFiles))
            .tool(DebugTool::new(FileRead))
            .tool(DebugTool::new(CodeSearch))
            .tool(DebugTool::new(ProjectDocs))
    }};
}

/// Core tools list for reference - these are the tools attached by `attach_core_tools!`
pub const CORE_TOOLS: &[&str] = &[
    "git_status",
    "git_diff",
    "git_log",
    "git_changed_files",
    "file_read",
    "code_search",
    "project_docs",
];

// Re-export the macro at module level
pub use attach_core_tools;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn core_tools_count() {
        assert_eq!(CORE_TOOLS.len(), 7);
    }
}
