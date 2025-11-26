//! Main application for Iris Studio
//!
//! Event loop and rendering coordination.

use anyhow::Result;
use crossterm::event::{
    self, DisableMouseCapture, EnableMouseCapture, Event, KeyEventKind, MouseButton, MouseEvent,
    MouseEventKind,
};
use crossterm::execute;
use crossterm::terminal::{
    EnterAlternateScreen, LeaveAlternateScreen, disable_raw_mode, enable_raw_mode,
};
use ratatui::Frame;
use ratatui::Terminal;
use ratatui::backend::CrosstermBackend;
use ratatui::layout::Rect;
use ratatui::style::{Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::Paragraph;
use std::io::{self, Stdout};
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::mpsc;

use crate::agents::IrisAgentService;
use crate::config::Config;
use crate::git::GitRepo;
use crate::services::GitCommitService;
use crate::types::GeneratedMessage;

use super::components::{DiffHunk, DiffLine, FileDiff, FileGitStatus, parse_diff};
use super::events::{Action, handle_key_event};
use super::layout::{LayoutAreas, calculate_layout, get_mode_layout};
use super::render::{
    render_changelog_panel, render_commit_panel, render_explore_panel, render_modal,
    render_pr_panel, render_review_panel,
};
use super::state::{GitStatus, IrisStatus, Mode, Notification, PanelId, StudioState};
use super::theme;

// ═══════════════════════════════════════════════════════════════════════════════
// Async Task Results
// ═══════════════════════════════════════════════════════════════════════════════

/// Result from an async Iris task
pub enum IrisTaskResult {
    /// Generated commit messages
    CommitMessages(Vec<GeneratedMessage>),
    /// Generated code review (markdown)
    ReviewContent(String),
    /// Generated PR description (markdown)
    PRContent(String),
    /// Generated changelog (markdown)
    ChangelogContent(String),
    /// Chat response from Iris
    ChatResponse(String),
    /// Error from the task
    Error(String),
}

// ═══════════════════════════════════════════════════════════════════════════════
// Studio Application
// ═══════════════════════════════════════════════════════════════════════════════

/// Main Iris Studio application
pub struct StudioApp {
    /// Application state
    pub state: StudioState,
    /// Git commit service for operations
    commit_service: Option<Arc<GitCommitService>>,
    /// Iris agent service for AI operations
    agent_service: Option<Arc<IrisAgentService>>,
    /// Channel receiver for async Iris results
    iris_result_rx: mpsc::UnboundedReceiver<IrisTaskResult>,
    /// Channel sender for async Iris results (kept for spawning tasks)
    iris_result_tx: mpsc::UnboundedSender<IrisTaskResult>,
    /// Last calculated layout for mouse hit testing
    last_layout: Option<LayoutAreas>,
    /// Whether an explicit initial mode was set
    explicit_mode_set: bool,
}

impl StudioApp {
    /// Create a new Studio application
    pub fn new(
        config: Config,
        repo: Option<Arc<GitRepo>>,
        commit_service: Option<Arc<GitCommitService>>,
        agent_service: Option<Arc<IrisAgentService>>,
    ) -> Self {
        let state = StudioState::new(config, repo);
        let (iris_result_tx, iris_result_rx) = mpsc::unbounded_channel();

        Self {
            state,
            commit_service,
            agent_service,
            iris_result_rx,
            iris_result_tx,
            last_layout: None,
            explicit_mode_set: false,
        }
    }

    /// Set explicit initial mode
    pub fn set_initial_mode(&mut self, mode: Mode) {
        self.state.switch_mode(mode);
        self.explicit_mode_set = true;
    }

    /// Update git status from repository
    pub fn refresh_git_status(&mut self) -> Result<()> {
        if let Some(repo) = &self.state.repo {
            // Get file info which includes staged files
            let files_info = repo.extract_files_info(false).ok();
            let unstaged = repo.get_unstaged_files().ok();

            let staged_files: Vec<std::path::PathBuf> = files_info
                .as_ref()
                .map(|f| {
                    f.staged_files
                        .iter()
                        .map(|s| s.path.clone().into())
                        .collect()
                })
                .unwrap_or_default();

            let modified_files: Vec<std::path::PathBuf> = unstaged
                .as_ref()
                .map(|f| f.iter().map(|s| s.path.clone().into()).collect())
                .unwrap_or_default();

            let status = GitStatus {
                branch: repo.get_current_branch().unwrap_or_default(),
                staged_count: staged_files.len(),
                staged_files,
                modified_count: modified_files.len(),
                modified_files,
                untracked_count: 0, // TODO: Separate untracked from modified
                untracked_files: Vec::new(),
                commits_ahead: 0, // TODO: Calculate from remote
                commits_behind: 0,
            };
            self.state.git_status = status;

            // Update file trees for components
            self.update_commit_file_tree();
            self.update_explore_file_tree();
            self.update_review_file_tree();

            // Load diffs into diff view
            self.load_staged_diffs(files_info.as_ref());

            // Sync initial file selection with diff view
            if let Some(path) = self.state.modes.commit.file_tree.selected_path() {
                self.state.modes.commit.diff_view.select_file_by_path(&path);
            }
        }
        Ok(())
    }

    /// Load staged file diffs into the diff view component
    fn load_staged_diffs(&mut self, files_info: Option<&crate::git::RepoFilesInfo>) {
        let Some(info) = files_info else { return };
        let Some(repo) = &self.state.repo else { return };

        // Get a proper unified diff with all headers using git
        if let Ok(diff_text) = repo.get_staged_diff_full() {
            let diffs = parse_diff(&diff_text);
            self.state.modes.commit.diff_view.set_diffs(diffs);
        } else {
            // Fallback: Build synthetic diff from file info
            let mut diffs = Vec::new();
            for f in &info.staged_files {
                let mut file_diff = FileDiff::new(&f.path);
                file_diff.is_new = matches!(f.change_type, crate::context::ChangeType::Added);
                file_diff.is_deleted = matches!(f.change_type, crate::context::ChangeType::Deleted);

                // Create a synthetic hunk from the diff lines
                if !f.diff.is_empty() && f.diff != "[Content excluded]" {
                    let hunk = DiffHunk {
                        header: "@@ Changes @@".to_string(),
                        lines: f
                            .diff
                            .lines()
                            .enumerate()
                            .map(|(i, line)| {
                                let content = line.strip_prefix(['+', '-', ' ']).unwrap_or(line);
                                if line.starts_with('+') {
                                    DiffLine::added(content, i + 1)
                                } else if line.starts_with('-') {
                                    DiffLine::removed(content, i + 1)
                                } else {
                                    DiffLine::context(content, i + 1, i + 1)
                                }
                            })
                            .collect(),
                        old_start: 1,
                        old_count: 0,
                        new_start: 1,
                        new_count: 0,
                    };
                    file_diff.hunks.push(hunk);
                }
                diffs.push(file_diff);
            }
            self.state.modes.commit.diff_view.set_diffs(diffs);
        }
    }

    /// Update explore mode file tree from repository
    fn update_explore_file_tree(&mut self) {
        // Build file tree from staged + modified files for now
        // In the future, we can scan the working directory
        let mut all_files = Vec::new();
        let mut statuses = Vec::new();

        for path in &self.state.git_status.staged_files {
            all_files.push(path.clone());
            statuses.push((path.clone(), FileGitStatus::Staged));
        }
        for path in &self.state.git_status.modified_files {
            if !all_files.contains(path) {
                all_files.push(path.clone());
            }
            statuses.push((path.clone(), FileGitStatus::Modified));
        }

        if !all_files.is_empty() {
            let tree_state = super::components::FileTreeState::from_paths(&all_files, &statuses);
            self.state.modes.explore.file_tree = tree_state;
        }
    }

    /// Run the TUI application
    pub fn run(&mut self) -> Result<ExitResult> {
        // Setup terminal
        enable_raw_mode()?;
        let mut stdout = io::stdout();
        execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
        let backend = CrosstermBackend::new(stdout);
        let mut terminal = Terminal::new(backend)?;

        // Run main loop
        let result = self.main_loop(&mut terminal);

        // Cleanup terminal
        disable_raw_mode()?;
        execute!(
            terminal.backend_mut(),
            LeaveAlternateScreen,
            DisableMouseCapture
        )?;
        terminal.show_cursor()?;

        result
    }

    fn main_loop(
        &mut self,
        terminal: &mut Terminal<CrosstermBackend<Stdout>>,
    ) -> Result<ExitResult> {
        // Refresh git status on start
        let _ = self.refresh_git_status();

        // Set initial mode based on repo state (only if no explicit mode was set)
        let current_mode = if self.explicit_mode_set {
            self.state.active_mode
        } else {
            let suggested_mode = self.state.suggest_initial_mode();
            self.state.switch_mode(suggested_mode);
            suggested_mode
        };

        // Auto-generate commit message if entering Commit mode with staged changes
        if current_mode == Mode::Commit && self.state.git_status.has_staged() {
            self.auto_generate_commit();
        }

        loop {
            // Check for completed Iris tasks
            self.check_iris_results();

            // Render if dirty
            if self.state.check_dirty() {
                terminal.draw(|frame| self.render(frame))?;
            }

            // Poll for events with timeout for animations
            if event::poll(Duration::from_millis(50))? {
                match event::read()? {
                    Event::Key(key) => {
                        // Only handle key press events
                        if key.kind == KeyEventKind::Press {
                            let action = handle_key_event(&mut self.state, key);

                            match action {
                                Action::Quit => return Ok(ExitResult::Quit),
                                Action::Redraw => self.state.mark_dirty(),
                                Action::Commit(message) => {
                                    return Ok(self.perform_commit(&message));
                                }
                                Action::IrisQuery(query) => {
                                    self.handle_iris_query(query);
                                }
                                Action::SwitchMode(mode) => {
                                    self.state.switch_mode(mode);
                                    // Trigger mode-specific data loading
                                    match mode {
                                        Mode::PR => self.update_pr_data(),
                                        Mode::Review => self.update_review_data(),
                                        Mode::Changelog => self.update_changelog_data(),
                                        Mode::Commit => {
                                            if self.state.git_status.has_staged() {
                                                self.auto_generate_commit();
                                            }
                                        }
                                        Mode::Explore => {}
                                    }
                                    self.state.mark_dirty();
                                }
                                Action::ReloadPrData => {
                                    self.update_pr_data();
                                }
                                Action::ReloadReviewData => {
                                    self.update_review_data();
                                }
                                Action::ReloadChangelogData => {
                                    self.update_changelog_data();
                                }
                                Action::None => {}
                            }
                        }
                    }
                    Event::Mouse(mouse) => {
                        self.handle_mouse_event(mouse);
                    }
                    Event::Resize(_, _) => {
                        // Terminal resized, trigger redraw
                        self.state.mark_dirty();
                    }
                    _ => {}
                }
            }

            // Tick animations
            self.state.tick();
        }
    }

    /// Check for completed Iris task results
    fn check_iris_results(&mut self) {
        use super::state::Modal;

        while let Ok(result) = self.iris_result_rx.try_recv() {
            match result {
                IrisTaskResult::CommitMessages(messages) => {
                    self.state.set_iris_idle();
                    self.state.modes.commit.generating = false;
                    // Store messages in both the old location and the new component
                    self.state.modes.commit.messages.clone_from(&messages);
                    self.state.modes.commit.current_index = 0;
                    self.state
                        .modes
                        .commit
                        .message_editor
                        .set_messages(messages);
                    self.state
                        .notify(Notification::success("Commit message generated"));
                    self.state.mark_dirty();
                }
                IrisTaskResult::ReviewContent(content) => {
                    self.state.set_iris_idle();
                    self.state.modes.review.review_content = content;
                    self.state.modes.review.review_scroll = 0;
                    self.state.modes.review.generating = false;
                    self.state
                        .notify(Notification::success("Code review generated"));
                    self.state.mark_dirty();
                }
                IrisTaskResult::PRContent(content) => {
                    self.state.set_iris_idle();
                    self.state.modes.pr.pr_content = content;
                    self.state.modes.pr.pr_scroll = 0;
                    self.state.modes.pr.generating = false;
                    self.state
                        .notify(Notification::success("PR description generated"));
                    self.state.mark_dirty();
                }
                IrisTaskResult::ChangelogContent(content) => {
                    self.state.set_iris_idle();
                    self.state.modes.changelog.changelog_content = content;
                    self.state.modes.changelog.changelog_scroll = 0;
                    self.state.modes.changelog.generating = false;
                    self.state
                        .notify(Notification::success("Changelog generated"));
                    self.state.mark_dirty();
                }
                IrisTaskResult::ChatResponse(response) => {
                    // Add response to chat state
                    if let Some(Modal::Chat(chat)) = &mut self.state.modal {
                        chat.add_iris_response(&response);
                    }
                    self.state.mark_dirty();
                }
                IrisTaskResult::Error(err) => {
                    self.state.set_iris_error(&err);
                    // Clear any generating states
                    self.state.modes.commit.generating = false;
                    self.state.modes.review.generating = false;
                    self.state.modes.pr.generating = false;
                    self.state.modes.changelog.generating = false;
                    // If we're in chat, add error as Iris response
                    if let Some(Modal::Chat(chat)) = &mut self.state.modal {
                        chat.is_responding = false;
                    }
                    self.state
                        .notify(Notification::error(format!("Iris error: {}", err)));
                    self.state.mark_dirty();
                }
            }
        }
    }

    /// Handle an Iris query request
    fn handle_iris_query(&self, query: super::events::IrisQueryRequest) {
        use super::events::IrisQueryRequest;

        match query {
            IrisQueryRequest::GenerateCommit { instructions } => {
                self.spawn_commit_generation(instructions);
            }
            IrisQueryRequest::GenerateReview => {
                self.spawn_review_generation();
            }
            IrisQueryRequest::GeneratePR => {
                self.spawn_pr_generation();
            }
            IrisQueryRequest::GenerateChangelog { from_ref, to_ref } => {
                self.spawn_changelog_generation(from_ref, to_ref);
            }
            IrisQueryRequest::SemanticBlame {
                file,
                start_line,
                end_line,
            } => {
                // TODO: Implement semantic blame
                let _ = (file, start_line, end_line);
            }
            IrisQueryRequest::Chat { message } => {
                self.spawn_chat_query(message);
            }
        }
    }

    /// Spawn a task for chat query - uses Iris agent with chat capability
    fn spawn_chat_query(&self, message: String) {
        use crate::agents::StructuredResponse;

        let Some(agent) = self.agent_service.clone() else {
            let tx = self.iris_result_tx.clone();
            let _ = tx.send(IrisTaskResult::ChatResponse(
                "Agent service not available".to_string(),
            ));
            return;
        };

        let tx = self.iris_result_tx.clone();
        let mode = self.state.active_mode;

        tokio::spawn(async move {
            // Build context-aware prompt with user's message
            let mode_context = match mode {
                Mode::Commit => "Current mode: Commit - working with staged changes",
                Mode::Review => "Current mode: Review - analyzing code changes",
                Mode::PR => "Current mode: PR - preparing pull request",
                Mode::Explore => "Current mode: Explore - navigating codebase",
                Mode::Changelog => "Current mode: Changelog - generating changelogs",
            };

            let prompt = format!("{}\n\nUser: {}", mode_context, message);

            // Execute with custom prompt to include user's message
            match agent.execute_task_with_prompt("chat", &prompt).await {
                Ok(response) => {
                    let text = match response {
                        StructuredResponse::PlainText(text) => text,
                        other => other.to_string(),
                    };
                    let _ = tx.send(IrisTaskResult::ChatResponse(text));
                }
                Err(e) => {
                    let _ = tx.send(IrisTaskResult::ChatResponse(format!(
                        "I encountered an error: {}",
                        e
                    )));
                }
            }
        });
    }

    /// Spawn a task for code review generation
    fn spawn_review_generation(&self) {
        use crate::agents::{StructuredResponse, TaskContext};

        let Some(agent) = self.agent_service.clone() else {
            let tx = self.iris_result_tx.clone();
            let _ = tx.send(IrisTaskResult::Error(
                "Agent service not available".to_string(),
            ));
            return;
        };

        let tx = self.iris_result_tx.clone();

        tokio::spawn(async move {
            // Use review context (staged changes only)
            let context = match TaskContext::for_review(None, None, None, false) {
                Ok(ctx) => ctx,
                Err(e) => {
                    let _ = tx.send(IrisTaskResult::Error(format!("Context error: {}", e)));
                    return;
                }
            };

            // Execute the review capability
            match agent.execute_task("review", context).await {
                Ok(response) => {
                    let review_text = match response {
                        StructuredResponse::MarkdownReview(review) => review.content,
                        StructuredResponse::PlainText(text) => text,
                        other => other.to_string(),
                    };
                    let _ = tx.send(IrisTaskResult::ReviewContent(review_text));
                }
                Err(e) => {
                    let _ = tx.send(IrisTaskResult::Error(format!("Review error: {}", e)));
                }
            }
        });
    }

    /// Spawn a task for PR description generation
    fn spawn_pr_generation(&self) {
        use crate::agents::{StructuredResponse, TaskContext};

        let Some(agent) = self.agent_service.clone() else {
            let tx = self.iris_result_tx.clone();
            let _ = tx.send(IrisTaskResult::Error(
                "Agent service not available".to_string(),
            ));
            return;
        };

        let tx = self.iris_result_tx.clone();
        let base_branch = self.state.modes.pr.base_branch.clone();

        tokio::spawn(async move {
            // Build context for PR (comparing current branch to base)
            let context = TaskContext::for_pr(Some(base_branch), None);

            // Execute the PR capability
            match agent.execute_task("pr", context).await {
                Ok(response) => {
                    let pr_text = match response {
                        StructuredResponse::PullRequest(pr) => pr.content,
                        StructuredResponse::PlainText(text) => text,
                        other => other.to_string(),
                    };
                    let _ = tx.send(IrisTaskResult::PRContent(pr_text));
                }
                Err(e) => {
                    let _ = tx.send(IrisTaskResult::Error(format!("PR error: {}", e)));
                }
            }
        });
    }

    /// Spawn a task for changelog generation
    fn spawn_changelog_generation(&self, from_ref: String, to_ref: String) {
        use crate::agents::{StructuredResponse, TaskContext};

        let Some(agent) = self.agent_service.clone() else {
            let tx = self.iris_result_tx.clone();
            let _ = tx.send(IrisTaskResult::Error(
                "Agent service not available".to_string(),
            ));
            return;
        };

        let tx = self.iris_result_tx.clone();

        tokio::spawn(async move {
            // Build context for changelog (comparing two refs)
            let context = TaskContext::for_changelog(from_ref, Some(to_ref));

            // Execute the changelog capability
            match agent.execute_task("changelog", context).await {
                Ok(response) => {
                    let changelog_text = match response {
                        StructuredResponse::Changelog(cl) => cl.content,
                        StructuredResponse::PlainText(text) => text,
                        other => other.to_string(),
                    };
                    let _ = tx.send(IrisTaskResult::ChangelogContent(changelog_text));
                }
                Err(e) => {
                    let _ = tx.send(IrisTaskResult::Error(format!("Changelog error: {}", e)));
                }
            }
        });
    }

    /// Auto-generate commit message on app start
    fn auto_generate_commit(&mut self) {
        // Don't regenerate if we already have messages
        if !self.state.modes.commit.messages.is_empty() {
            return;
        }

        self.state.set_iris_thinking("Analyzing changes...");
        self.state.modes.commit.generating = true;
        self.spawn_commit_generation(None);
    }

    /// Handle mouse events for panel focus and scrolling
    fn handle_mouse_event(&mut self, mouse: MouseEvent) {
        match mouse.kind {
            MouseEventKind::Down(MouseButton::Left) => {
                // Click to focus panel
                if let Some(panel) = self.panel_at(mouse.column, mouse.row)
                    && self.state.focused_panel != panel
                {
                    self.state.focused_panel = panel;
                    self.state.mark_dirty();
                }
            }
            MouseEventKind::ScrollUp => {
                // Scroll up in current panel
                self.scroll_focused_panel(-3);
            }
            MouseEventKind::ScrollDown => {
                // Scroll down in current panel
                self.scroll_focused_panel(3);
            }
            _ => {}
        }
    }

    /// Determine which panel contains the given coordinates
    fn panel_at(&self, x: u16, y: u16) -> Option<PanelId> {
        let Some(layout) = &self.last_layout else {
            return None;
        };

        for (i, panel_rect) in layout.panels.iter().enumerate() {
            if x >= panel_rect.x
                && x < panel_rect.x + panel_rect.width
                && y >= panel_rect.y
                && y < panel_rect.y + panel_rect.height
            {
                return match i {
                    0 => Some(PanelId::Left),
                    1 => Some(PanelId::Center),
                    2 => Some(PanelId::Right),
                    _ => None,
                };
            }
        }
        None
    }

    /// Scroll the focused panel by the given delta
    fn scroll_focused_panel(&mut self, delta: i32) {
        match self.state.active_mode {
            Mode::Explore => {
                match self.state.focused_panel {
                    PanelId::Left => {
                        // File tree scroll
                        if delta > 0 {
                            for _ in 0..delta {
                                self.state.modes.explore.file_tree.select_next();
                            }
                        } else {
                            for _ in 0..(-delta) {
                                self.state.modes.explore.file_tree.select_prev();
                            }
                        }
                    }
                    PanelId::Center => {
                        // Code view scroll
                        let scroll = &mut self.state.modes.explore.code_scroll;
                        if delta > 0 {
                            *scroll = scroll.saturating_add(delta as usize);
                        } else {
                            *scroll = scroll.saturating_sub((-delta) as usize);
                        }
                    }
                    PanelId::Right => {
                        // Context panel - no scroll yet
                    }
                }
            }
            Mode::Commit => {
                match self.state.focused_panel {
                    PanelId::Left => {
                        // Staged files tree scroll + sync diff view
                        if delta > 0 {
                            for _ in 0..delta {
                                self.state.modes.commit.file_tree.select_next();
                            }
                        } else {
                            for _ in 0..(-delta) {
                                self.state.modes.commit.file_tree.select_prev();
                            }
                        }
                        // Sync diff view to selected file
                        if let Some(path) = self.state.modes.commit.file_tree.selected_path() {
                            self.state.modes.commit.diff_view.select_file_by_path(&path);
                        }
                    }
                    PanelId::Center => {
                        // Message editor - textarea handles scrolling
                    }
                    PanelId::Right => {
                        // Diff view scroll
                        if delta > 0 {
                            self.state
                                .modes
                                .commit
                                .diff_view
                                .scroll_down(delta as usize);
                        } else {
                            self.state
                                .modes
                                .commit
                                .diff_view
                                .scroll_up((-delta) as usize);
                        }
                    }
                }
            }
            _ => {}
        }
        self.state.mark_dirty();
    }

    /// Spawn a task to generate a commit message
    fn spawn_commit_generation(&self, instructions: Option<String>) {
        use crate::agents::{StructuredResponse, TaskContext};

        let Some(agent) = self.agent_service.clone() else {
            let tx = self.iris_result_tx.clone();
            let _ = tx.send(IrisTaskResult::Error(
                "Agent service not available".to_string(),
            ));
            return;
        };

        // Build task prompt with optional instructions
        let task_prompt = if let Some(inst) = instructions {
            format!("Generate a commit message. User instructions: {}", inst)
        } else {
            "Generate a commit message for the staged changes.".to_string()
        };

        let tx = self.iris_result_tx.clone();

        tokio::spawn(async move {
            // Use standard commit context
            let context = TaskContext::for_gen();

            // Execute the commit capability
            match agent.execute_task("commit", context).await {
                Ok(response) => {
                    // Extract message from response
                    match response {
                        StructuredResponse::CommitMessage(msg) => {
                            let _ = tx.send(IrisTaskResult::CommitMessages(vec![msg]));
                        }
                        _ => {
                            let _ = tx.send(IrisTaskResult::Error(
                                "Unexpected response type from agent".to_string(),
                            ));
                        }
                    }
                }
                Err(e) => {
                    let _ = tx.send(IrisTaskResult::Error(format!("Agent error: {}", e)));
                }
            }
        });

        // Mark as unused for now - custom prompt not yet supported
        let _ = task_prompt;
    }

    fn perform_commit(&self, message: &str) -> ExitResult {
        if let Some(service) = &self.commit_service {
            match service.perform_commit(message) {
                Ok(result) => {
                    let output = crate::output::format_commit_result(&result, message);
                    ExitResult::Committed(output)
                }
                Err(e) => ExitResult::Error(e.to_string()),
            }
        } else {
            ExitResult::Error("Commit service not available".to_string())
        }
    }

    // ═══════════════════════════════════════════════════════════════════════════
    // Rendering
    // ═══════════════════════════════════════════════════════════════════════════

    fn render(&mut self, frame: &mut Frame) {
        let areas = calculate_layout(frame.area(), self.state.active_mode);

        self.render_header(frame, areas.header);
        self.render_tabs(frame, areas.tabs);
        self.render_panels(frame, &areas);
        self.render_status(frame, areas.status);

        // Store layout for mouse hit testing
        self.last_layout = Some(areas);

        // Render modal overlay on top of everything
        if self.state.modal.is_some() {
            render_modal(&self.state, frame, self.state.last_render);
        }
    }

    fn render_header(&self, frame: &mut Frame, area: Rect) {
        let branch = &self.state.git_status.branch;
        let staged = self.state.git_status.staged_count;

        let title = Span::styled(
            " Iris Studio ",
            Style::default()
                .fg(theme::ELECTRIC_PURPLE)
                .add_modifier(Modifier::BOLD),
        );

        let branch_info = if branch.is_empty() {
            Span::raw("")
        } else {
            Span::styled(
                format!(" {} ", branch),
                Style::default().fg(theme::NEON_CYAN),
            )
        };

        let staged_info = if staged > 0 {
            Span::styled(
                format!(" {} staged ", staged),
                Style::default().fg(theme::SUCCESS_GREEN),
            )
        } else {
            Span::raw("")
        };

        let line = Line::from(vec![title, branch_info, staged_info]);
        let header = Paragraph::new(line);
        frame.render_widget(header, area);
    }

    fn render_tabs(&self, frame: &mut Frame, area: Rect) {
        let mut spans = Vec::new();
        spans.push(Span::raw("  "));

        for mode in Mode::all() {
            let is_active = *mode == self.state.active_mode;
            let is_available = mode.is_available();

            let style = if is_active {
                theme::mode_active()
            } else if is_available {
                theme::mode_inactive()
            } else {
                Style::default().fg(theme::TEXT_MUTED)
            };

            let label = format!("[{}] {} ", mode.shortcut(), mode.display_name());
            spans.push(Span::styled(label, style));

            if is_active {
                spans.push(Span::styled(
                    "━━━",
                    Style::default().fg(theme::ELECTRIC_PURPLE),
                ));
            }

            spans.push(Span::raw("  "));
        }

        let tabs = Paragraph::new(Line::from(spans));
        frame.render_widget(tabs, area);
    }

    fn render_panels(&mut self, frame: &mut Frame, areas: &LayoutAreas) {
        let layout = get_mode_layout(self.state.active_mode);
        let panel_ids: Vec<_> = layout.panels.iter().map(|c| c.id).collect();
        let panel_areas: Vec<_> = areas.panels.clone();

        for (i, panel_area) in panel_areas.iter().enumerate() {
            if let Some(&panel_id) = panel_ids.get(i) {
                self.render_panel_content(frame, *panel_area, panel_id);
            }
        }
    }

    fn render_panel_content(&mut self, frame: &mut Frame, area: Rect, panel_id: PanelId) {
        match self.state.active_mode {
            Mode::Explore => render_explore_panel(&mut self.state, frame, area, panel_id),
            Mode::Commit => render_commit_panel(&mut self.state, frame, area, panel_id),
            Mode::Review => render_review_panel(&mut self.state, frame, area, panel_id),
            Mode::PR => render_pr_panel(&mut self.state, frame, area, panel_id),
            Mode::Changelog => render_changelog_panel(&mut self.state, frame, area, panel_id),
        }
    }

    /// Update commit mode file tree from git status
    fn update_commit_file_tree(&mut self) {
        let staged = &self.state.git_status.staged_files;
        let statuses: Vec<_> = staged
            .iter()
            .map(|p| (p.clone(), FileGitStatus::Staged))
            .collect();

        let tree_state = super::components::FileTreeState::from_paths(staged, &statuses);
        self.state.modes.commit.file_tree = tree_state;

        // Expand all by default for staged files (usually not too many)
        self.state.modes.commit.file_tree.expand_all();
    }

    /// Update review mode file tree from git status (staged + modified)
    fn update_review_file_tree(&mut self) {
        let mut all_files = Vec::new();
        let mut statuses = Vec::new();

        // Include both staged and modified files for review
        for path in &self.state.git_status.staged_files {
            all_files.push(path.clone());
            statuses.push((path.clone(), FileGitStatus::Staged));
        }
        for path in &self.state.git_status.modified_files {
            if !all_files.contains(path) {
                all_files.push(path.clone());
                statuses.push((path.clone(), FileGitStatus::Modified));
            }
        }

        let tree_state = super::components::FileTreeState::from_paths(&all_files, &statuses);
        self.state.modes.review.file_tree = tree_state;
        self.state.modes.review.file_tree.expand_all();

        // Also load diffs for review mode
        self.load_review_diffs();
    }

    /// Load diffs into review mode diff view
    fn load_review_diffs(&mut self) {
        let Some(repo) = &self.state.repo else { return };

        // Get staged diff first, then unstaged
        if let Ok(diff_text) = repo.get_staged_diff_full() {
            let diffs = parse_diff(&diff_text);
            self.state.modes.review.diff_view.set_diffs(diffs);
        }

        // Sync initial file selection
        if let Some(path) = self.state.modes.review.file_tree.selected_path() {
            self.state.modes.review.diff_view.select_file_by_path(&path);
        }
    }

    /// Update PR mode data - load commits and diff between refs
    pub fn update_pr_data(&mut self) {
        use super::state::PrCommit;

        // Clone the Arc to avoid borrow conflicts with self.state mutations
        let Some(repo) = self.state.repo.clone() else {
            return;
        };

        let base = self.state.modes.pr.base_branch.clone();
        let to = self.state.modes.pr.to_ref.clone();

        // Load commits between the refs
        match repo.get_commits_between_with_callback(&base, &to, |commit| {
            Ok(PrCommit {
                hash: commit.hash[..7.min(commit.hash.len())].to_string(),
                message: commit.message.lines().next().unwrap_or("").to_string(),
                author: commit.author.clone(),
            })
        }) {
            Ok(commits) => {
                self.state.modes.pr.commits = commits;
                self.state.modes.pr.selected_commit = 0;
                self.state.modes.pr.commit_scroll = 0;
            }
            Err(e) => {
                self.state.notify(Notification::warning(format!(
                    "Could not load commits: {}",
                    e
                )));
            }
        }

        // Load diff between the refs
        match repo.get_ref_diff_full(&base, &to) {
            Ok(diff_text) => {
                let diffs = parse_diff(&diff_text);
                self.state.modes.pr.diff_view.set_diffs(diffs);
            }
            Err(e) => {
                self.state
                    .notify(Notification::warning(format!("Could not load diff: {}", e)));
            }
        }

        self.state.mark_dirty();
    }

    /// Update Review mode data - load diff between `from_ref` and `to_ref`
    pub fn update_review_data(&mut self) {
        // Clone the Arc to avoid borrow conflicts with self.state mutations
        let Some(repo) = self.state.repo.clone() else {
            return;
        };

        let from = self.state.modes.review.from_ref.clone();
        let to = self.state.modes.review.to_ref.clone();

        // Load diff between the refs
        match repo.get_ref_diff_full(&from, &to) {
            Ok(diff_text) => {
                let diffs = parse_diff(&diff_text);
                self.state.modes.review.diff_view.set_diffs(diffs.clone());

                // Also update file tree from the diff files
                let files: Vec<std::path::PathBuf> = diffs
                    .iter()
                    .map(|d| std::path::PathBuf::from(&d.path))
                    .collect();
                let statuses: Vec<_> = files
                    .iter()
                    .map(|p| (p.clone(), FileGitStatus::Modified))
                    .collect();
                let tree_state = super::components::FileTreeState::from_paths(&files, &statuses);
                self.state.modes.review.file_tree = tree_state;
                self.state.modes.review.file_tree.expand_all();
            }
            Err(e) => {
                self.state
                    .notify(Notification::warning(format!("Could not load diff: {}", e)));
            }
        }

        self.state.mark_dirty();
    }

    /// Update Changelog mode data - load commits and diff between `from_ref` and `to_ref`
    pub fn update_changelog_data(&mut self) {
        use super::state::ChangelogCommit;

        // Clone the Arc to avoid borrow conflicts with self.state mutations
        let Some(repo) = self.state.repo.clone() else {
            return;
        };

        let from = self.state.modes.changelog.from_ref.clone();
        let to = self.state.modes.changelog.to_ref.clone();

        // Load commits between the refs
        match repo.get_commits_between_with_callback(&from, &to, |commit| {
            Ok(ChangelogCommit {
                hash: commit.hash[..7.min(commit.hash.len())].to_string(),
                message: commit.message.lines().next().unwrap_or("").to_string(),
                author: commit.author.clone(),
            })
        }) {
            Ok(commits) => {
                self.state.modes.changelog.commits = commits;
                self.state.modes.changelog.selected_commit = 0;
                self.state.modes.changelog.commit_scroll = 0;
            }
            Err(e) => {
                self.state.notify(Notification::warning(format!(
                    "Could not load commits: {}",
                    e
                )));
            }
        }

        // Load diff between the refs
        match repo.get_ref_diff_full(&from, &to) {
            Ok(diff_text) => {
                let diffs = parse_diff(&diff_text);
                self.state.modes.changelog.diff_view.set_diffs(diffs);
            }
            Err(e) => {
                self.state
                    .notify(Notification::warning(format!("Could not load diff: {}", e)));
            }
        }

        self.state.mark_dirty();
    }

    fn render_status(&self, frame: &mut Frame, area: Rect) {
        let mut spans = Vec::new();

        // Show notification if any
        if let Some(notification) = self.state.current_notification() {
            let style = match notification.level {
                super::state::NotificationLevel::Info => theme::dimmed(),
                super::state::NotificationLevel::Success => theme::success(),
                super::state::NotificationLevel::Warning => theme::warning(),
                super::state::NotificationLevel::Error => theme::error(),
            };
            spans.push(Span::styled(&notification.message, style));
        } else {
            // Default keybinding hints
            spans.push(Span::styled(
                "[?] help  [Tab] panel  [q] quit",
                theme::dimmed(),
            ));
        }

        // Right-align Iris status
        let iris_status = match &self.state.iris_status {
            IrisStatus::Idle => Span::styled("Iris: ready", theme::dimmed()),
            IrisStatus::Thinking { task, .. } => {
                let spinner = self.state.iris_status.spinner_char().unwrap_or('◎');
                Span::styled(
                    format!("{} {}", spinner, task),
                    Style::default().fg(theme::NEON_CYAN),
                )
            }
            IrisStatus::Error(msg) => Span::styled(format!("Error: {}", msg), theme::error()),
        };

        // Calculate spacing
        let left_len: usize = spans.iter().map(|s| s.content.len()).sum();
        let right_len = iris_status.content.len();
        let padding = area.width as usize - left_len - right_len - 2;
        let padding_str = " ".repeat(padding.max(1));

        spans.push(Span::raw(padding_str));
        spans.push(iris_status);

        let status = Paragraph::new(Line::from(spans));
        frame.render_widget(status, area);
    }
}

// ═══════════════════════════════════════════════════════════════════════════════
// Exit Result
// ═══════════════════════════════════════════════════════════════════════════════

/// Result of running the Studio application
#[derive(Debug)]
pub enum ExitResult {
    /// User quit normally
    Quit,
    /// User committed changes (with output message)
    Committed(String),
    /// An error occurred
    Error(String),
}

// ═══════════════════════════════════════════════════════════════════════════════
// Public Entry Point
// ═══════════════════════════════════════════════════════════════════════════════

/// Run Iris Studio
pub fn run_studio(
    config: Config,
    repo: Option<Arc<GitRepo>>,
    commit_service: Option<Arc<GitCommitService>>,
    agent_service: Option<Arc<IrisAgentService>>,
    initial_mode: Option<Mode>,
    from_ref: Option<String>,
    to_ref: Option<String>,
) -> Result<()> {
    let mut app = StudioApp::new(config, repo, commit_service, agent_service);

    // Set initial mode if specified
    if let Some(mode) = initial_mode {
        app.set_initial_mode(mode);
    }

    // Set comparison refs if specified (applies to Review, PR, and Changelog modes)
    if let Some(from) = from_ref {
        app.state.modes.review.from_ref = from.clone();
        app.state.modes.pr.base_branch = from.clone();
        app.state.modes.changelog.from_ref = from;
    }
    if let Some(to) = to_ref {
        app.state.modes.review.to_ref = to.clone();
        app.state.modes.pr.to_ref = to.clone();
        app.state.modes.changelog.to_ref = to;
    }

    // Run the app
    match app.run()? {
        ExitResult::Quit => {
            // Silent exit
            Ok(())
        }
        ExitResult::Committed(message) => {
            println!("{message}");
            Ok(())
        }
        ExitResult::Error(error) => {
            eprintln!("Error: {error}");
            Ok(())
        }
    }
}
