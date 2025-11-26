//! Main application for Iris Studio
//!
//! Event loop and rendering coordination.

use anyhow::Result;
use crossterm::event::{self, Event, KeyEventKind};
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
use ratatui::widgets::{Block, Borders, Paragraph};
use std::io::{self, Stdout};
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::mpsc;

use crate::agents::IrisAgentService;
use crate::config::Config;
use crate::git::GitRepo;
use crate::services::GitCommitService;
use crate::types::GeneratedMessage;

use super::components::{
    FileGitStatus, parse_diff, render_diff_view, render_file_tree, render_message_editor,
};
use super::events::{Action, handle_key_event};
use super::layout::{LayoutAreas, calculate_layout, get_mode_layout};
use super::state::{GitStatus, IrisStatus, Mode, Notification, PanelId, StudioState};
use super::theme;

// ═══════════════════════════════════════════════════════════════════════════════
// Async Task Results
// ═══════════════════════════════════════════════════════════════════════════════

/// Result from an async Iris task
pub enum IrisTaskResult {
    /// Generated commit messages
    CommitMessages(Vec<GeneratedMessage>),
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
        }
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

            // Load diffs into diff view
            self.load_staged_diffs(&files_info);
        }
        Ok(())
    }

    /// Load staged file diffs into the diff view component
    fn load_staged_diffs(&mut self, files_info: &Option<crate::git::RepoFilesInfo>) {
        if let Some(info) = files_info {
            // Combine all staged file diffs into a single diff string
            let combined_diff: String = info
                .staged_files
                .iter()
                .map(|f| {
                    // Build a git-style diff header for each file
                    let header = format!("diff --git a/{} b/{}\n", f.path, f.path);
                    format!("{}{}\n", header, f.diff)
                })
                .collect();

            // Parse the combined diff and load into view
            let diffs = parse_diff(&combined_diff);
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
        execute!(stdout, EnterAlternateScreen)?;
        let backend = CrosstermBackend::new(stdout);
        let mut terminal = Terminal::new(backend)?;

        // Run main loop
        let result = self.main_loop(&mut terminal);

        // Cleanup terminal
        disable_raw_mode()?;
        execute!(terminal.backend_mut(), LeaveAlternateScreen)?;
        terminal.show_cursor()?;

        result
    }

    fn main_loop(
        &mut self,
        terminal: &mut Terminal<CrosstermBackend<Stdout>>,
    ) -> Result<ExitResult> {
        // Refresh git status on start
        let _ = self.refresh_git_status();

        // Set initial mode based on repo state
        let suggested_mode = self.state.suggest_initial_mode();
        self.state.switch_mode(suggested_mode);

        loop {
            // Check for completed Iris tasks
            self.check_iris_results();

            // Render if dirty
            if self.state.check_dirty() {
                terminal.draw(|frame| self.render(frame))?;
            }

            // Poll for events with timeout for animations
            if event::poll(Duration::from_millis(50))?
                && let Event::Key(key) = event::read()?
            {
                // Only handle key press events
                if key.kind == KeyEventKind::Press {
                    let action = handle_key_event(&mut self.state, key);

                    match action {
                        Action::Quit => return Ok(ExitResult::Quit),
                        Action::Redraw => self.state.mark_dirty(),
                        Action::Commit(message) => {
                            return self.perform_commit(&message);
                        }
                        Action::IrisQuery(query) => {
                            self.handle_iris_query(query);
                        }
                        Action::None => {}
                    }
                }
            }

            // Tick animations
            self.state.tick();
        }
    }

    /// Check for completed Iris task results
    fn check_iris_results(&mut self) {
        while let Ok(result) = self.iris_result_rx.try_recv() {
            match result {
                IrisTaskResult::CommitMessages(messages) => {
                    self.state.set_iris_idle();
                    // Store messages in both the old location and the new component
                    self.state.modes.commit.messages = messages.clone();
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
                IrisTaskResult::Error(err) => {
                    self.state.set_iris_error(&err);
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
            IrisQueryRequest::SemanticBlame {
                file,
                start_line,
                end_line,
            } => {
                // TODO: Implement semantic blame
                let _ = (file, start_line, end_line);
            }
        }
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

    fn perform_commit(&self, message: &str) -> Result<ExitResult> {
        if let Some(service) = &self.commit_service {
            match service.perform_commit(message) {
                Ok(result) => {
                    let output = crate::output::format_commit_result(&result, message);
                    Ok(ExitResult::Committed(output))
                }
                Err(e) => Ok(ExitResult::Error(e.to_string())),
            }
        } else {
            Ok(ExitResult::Error(
                "Commit service not available".to_string(),
            ))
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
            Mode::Explore => self.render_explore_panel(frame, area, panel_id),
            Mode::Commit => self.render_commit_panel(frame, area, panel_id),
            _ => {
                // Placeholder for unimplemented modes
                let block = Block::default()
                    .title(" Coming Soon ")
                    .borders(Borders::ALL)
                    .border_style(theme::unfocused_border());
                let inner = block.inner(area);
                frame.render_widget(block, area);

                let text = Paragraph::new("This mode is not yet implemented")
                    .style(Style::default().fg(theme::TEXT_DIM));
                frame.render_widget(text, inner);
            }
        }
    }

    fn render_explore_panel(&mut self, frame: &mut Frame, area: Rect, panel_id: PanelId) {
        let is_focused = panel_id == self.state.focused_panel;

        match panel_id {
            PanelId::Left => {
                // File tree
                render_file_tree(
                    frame,
                    area,
                    &mut self.state.modes.explore.file_tree,
                    "Files",
                    is_focused,
                );
            }
            PanelId::Center => {
                // Code view (placeholder for now - will be CodeView component)
                let block = Block::default()
                    .title(" Code ")
                    .borders(Borders::ALL)
                    .border_style(if is_focused {
                        theme::focused_border()
                    } else {
                        theme::unfocused_border()
                    });
                let inner = block.inner(area);
                frame.render_widget(block, area);

                let file_name = self.state.modes.explore.current_file.as_ref().map_or_else(
                    || "No file selected".to_string(),
                    |p| p.display().to_string(),
                );

                let text = Paragraph::new(format!("{}\n\nSelect a file from the tree", file_name))
                    .style(Style::default().fg(theme::TEXT_PRIMARY));
                frame.render_widget(text, inner);
            }
            PanelId::Right => {
                // Context panel
                let block = Block::default()
                    .title(" Context ")
                    .borders(Borders::ALL)
                    .border_style(if is_focused {
                        theme::focused_border()
                    } else {
                        theme::unfocused_border()
                    });
                let inner = block.inner(area);
                frame.render_widget(block, area);

                let text =
                    Paragraph::new("Semantic context\n\nSelect code and press 'w' to ask why")
                        .style(Style::default().fg(theme::TEXT_DIM));
                frame.render_widget(text, inner);
            }
        }
    }

    fn render_commit_panel(&mut self, frame: &mut Frame, area: Rect, panel_id: PanelId) {
        let is_focused = panel_id == self.state.focused_panel;

        match panel_id {
            PanelId::Left => {
                // Render staged files using FileTree component
                render_file_tree(
                    frame,
                    area,
                    &mut self.state.modes.commit.file_tree,
                    "Staged Files",
                    is_focused,
                );
            }
            PanelId::Center => {
                // Render diff view
                render_diff_view(
                    frame,
                    area,
                    &self.state.modes.commit.diff_view,
                    "Changes",
                    is_focused,
                );
            }
            PanelId::Right => {
                // Render message editor
                render_message_editor(
                    frame,
                    area,
                    &self.state.modes.commit.message_editor,
                    "Commit Message",
                    is_focused,
                );
            }
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
) -> Result<()> {
    let mut app = StudioApp::new(config, repo, commit_service, agent_service);

    // Set initial mode if specified
    if let Some(mode) = initial_mode {
        app.state.switch_mode(mode);
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
