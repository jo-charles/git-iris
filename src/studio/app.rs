//! Main application for Iris Studio
//!
//! Event loop and rendering coordination.

use anyhow::Result;
use crossterm::event::{
    self, DisableMouseCapture, EnableMouseCapture, Event, KeyEventKind, MouseButton, MouseEventKind,
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
use std::collections::VecDeque;
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
use super::events::{
    AgentResult, BlameInfo, ContentPayload, ContentType, SemanticBlameResult, SideEffect,
    StudioEvent, TaskType,
};
use super::history::History;
use super::layout::{LayoutAreas, calculate_layout, get_mode_layout};
use super::reducer::reduce;
use super::render::{
    render_changelog_panel, render_commit_panel, render_explore_panel, render_modal,
    render_pr_panel, render_release_notes_panel, render_review_panel,
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
    /// Generated release notes (markdown)
    ReleaseNotesContent(String),
    /// Chat response from Iris
    ChatResponse(String),
    /// Chat-triggered update to current content
    ChatUpdate(ChatUpdateType),
    /// Tool call status update (for streaming tool calls to chat)
    ToolStatus { tool_name: String, message: String },
    /// Streaming text chunk received
    StreamingChunk {
        task_type: TaskType,
        chunk: String,
        aggregated: String,
    },
    /// Streaming completed
    StreamingComplete { task_type: TaskType },
    /// Semantic blame result
    SemanticBlame(SemanticBlameResult),
    /// Error from the task
    Error(String),
}

/// Type of content update triggered by chat
#[derive(Debug, Clone)]
pub enum ChatUpdateType {
    /// Update commit message
    CommitMessage(GeneratedMessage),
    /// Update PR description
    PRDescription(String),
    /// Update review content
    Review(String),
}

// ═══════════════════════════════════════════════════════════════════════════════
// Studio Application
// ═══════════════════════════════════════════════════════════════════════════════

/// Main Iris Studio application
pub struct StudioApp {
    /// Application state
    pub state: StudioState,
    /// History for all content, chat, and events
    pub history: History,
    /// Event queue for processing
    event_queue: VecDeque<StudioEvent>,
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
    /// Last mouse click info for double-click detection (time, x, y)
    last_click: Option<(std::time::Instant, u16, u16)>,
    /// Drag selection start info (panel, line number) for code view selection
    drag_start: Option<(PanelId, usize)>,
}

impl StudioApp {
    /// Create a new Studio application
    pub fn new(
        config: Config,
        repo: Option<Arc<GitRepo>>,
        commit_service: Option<Arc<GitCommitService>>,
        agent_service: Option<Arc<IrisAgentService>>,
    ) -> Self {
        // Build history with repo context if available
        let history = if let Some(ref r) = repo {
            let repo_path = r.repo_path().clone();
            let branch = r.get_current_branch().ok();
            History::with_repo(repo_path, branch)
        } else {
            History::new()
        };

        let state = StudioState::new(config, repo);
        let (iris_result_tx, iris_result_rx) = mpsc::unbounded_channel();

        Self {
            state,
            history,
            event_queue: VecDeque::new(),
            commit_service,
            agent_service,
            iris_result_rx,
            iris_result_tx,
            last_layout: None,
            explicit_mode_set: false,
            last_click: None,
            drag_start: None,
        }
    }

    /// Set explicit initial mode
    pub fn set_initial_mode(&mut self, mode: Mode) {
        self.state.switch_mode(mode);
        self.explicit_mode_set = true;
    }

    /// Push an event to the queue
    fn push_event(&mut self, event: StudioEvent) {
        self.event_queue.push_back(event);
    }

    /// Process all queued events through the reducer
    fn process_events(&mut self) -> Option<ExitResult> {
        while let Some(event) = self.event_queue.pop_front() {
            // Run through reducer which mutates state and returns effects
            let effects = reduce(&mut self.state, event, &mut self.history);

            // Execute side effects
            if let Some(result) = self.execute_effects(effects) {
                return Some(result);
            }
        }
        None
    }

    /// Execute side effects from reducer
    fn execute_effects(&mut self, effects: Vec<SideEffect>) -> Option<ExitResult> {
        use super::events::{AgentTask, DataType};

        for effect in effects {
            match effect {
                SideEffect::Quit => return Some(ExitResult::Quit),

                SideEffect::ExecuteCommit { message } => {
                    return Some(self.perform_commit(&message));
                }

                SideEffect::Redraw => {
                    self.state.mark_dirty();
                }

                SideEffect::RefreshGitStatus => {
                    let _ = self.refresh_git_status();
                }

                SideEffect::GitStage(path) => {
                    self.stage_file(&path.to_string_lossy());
                }

                SideEffect::GitUnstage(path) => {
                    self.unstage_file(&path.to_string_lossy());
                }

                SideEffect::GitStageAll => {
                    self.stage_all();
                }

                SideEffect::GitUnstageAll => {
                    self.unstage_all();
                }

                SideEffect::SaveSettings => {
                    self.save_settings();
                }

                SideEffect::CopyToClipboard(text) => match arboard::Clipboard::new() {
                    Ok(mut clipboard) => {
                        if let Err(e) = clipboard.set_text(&text) {
                            self.state
                                .notify(Notification::error(format!("Failed to copy: {e}")));
                        } else {
                            self.state
                                .notify(Notification::success("Copied to clipboard"));
                        }
                    }
                    Err(e) => {
                        self.state
                            .notify(Notification::error(format!("Clipboard unavailable: {e}")));
                    }
                },

                SideEffect::ShowNotification {
                    level,
                    message,
                    duration_ms: _,
                } => {
                    let notif = match level {
                        super::events::NotificationLevel::Info => Notification::info(&message),
                        super::events::NotificationLevel::Success => {
                            Notification::success(&message)
                        }
                        super::events::NotificationLevel::Warning => {
                            Notification::warning(&message)
                        }
                        super::events::NotificationLevel::Error => Notification::error(&message),
                    };
                    self.state.notify(notif);
                }

                SideEffect::SpawnAgent { task } => match task {
                    AgentTask::Commit {
                        instructions,
                        preset,
                        use_gitmoji,
                    } => {
                        self.spawn_commit_generation(instructions, preset, use_gitmoji);
                    }
                    AgentTask::Review { from_ref, to_ref } => {
                        self.spawn_review_generation(from_ref, to_ref);
                    }
                    AgentTask::PR {
                        base_branch,
                        to_ref,
                    } => {
                        self.spawn_pr_generation(base_branch, to_ref);
                    }
                    AgentTask::Changelog { from_ref, to_ref } => {
                        self.spawn_changelog_generation(from_ref, to_ref);
                    }
                    AgentTask::ReleaseNotes { from_ref, to_ref } => {
                        self.spawn_release_notes_generation(from_ref, to_ref);
                    }
                    AgentTask::Chat { message, context } => {
                        self.spawn_chat_query(message, context);
                    }
                    AgentTask::SemanticBlame { blame_info } => {
                        self.spawn_semantic_blame(blame_info);
                    }
                },

                SideEffect::GatherBlameAndSpawnAgent {
                    file,
                    start_line,
                    end_line,
                } => {
                    self.gather_blame_and_spawn(&file, start_line, end_line);
                }

                SideEffect::LoadData {
                    data_type,
                    from_ref,
                    to_ref,
                } => {
                    // Trigger data refresh for the mode
                    match data_type {
                        DataType::GitStatus | DataType::CommitDiff => {
                            let _ = self.refresh_git_status();
                        }
                        DataType::ReviewDiff => {
                            self.update_review_data(from_ref, to_ref);
                        }
                        DataType::PRDiff => {
                            self.update_pr_data(from_ref, to_ref);
                        }
                        DataType::ChangelogCommits => {
                            self.update_changelog_data(from_ref, to_ref);
                        }
                        DataType::ReleaseNotesCommits => {
                            self.update_release_notes_data(from_ref, to_ref);
                        }
                    }
                }
            }
        }
        None
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

            // Get untracked files
            let untracked_files: Vec<std::path::PathBuf> = repo
                .get_untracked_files()
                .unwrap_or_default()
                .into_iter()
                .map(std::path::PathBuf::from)
                .collect();

            // Get ahead/behind counts
            let (commits_ahead, commits_behind) = repo.get_ahead_behind();

            let status = GitStatus {
                branch: repo.get_current_branch().unwrap_or_default(),
                staged_count: staged_files.len(),
                staged_files,
                modified_count: modified_files.len(),
                modified_files,
                untracked_count: untracked_files.len(),
                untracked_files,
                commits_ahead,
                commits_behind,
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
        // Get all tracked files from the repository
        let Some(repo) = &self.state.repo else { return };
        let all_files: Vec<std::path::PathBuf> = match repo.get_all_tracked_files() {
            Ok(files) => files.into_iter().map(std::path::PathBuf::from).collect(),
            Err(e) => {
                eprintln!("Failed to get tracked files: {}", e);
                return;
            }
        };

        // Build status lookup from git status
        let mut statuses = Vec::new();
        for path in &self.state.git_status.staged_files {
            statuses.push((path.clone(), FileGitStatus::Staged));
        }
        for path in &self.state.git_status.modified_files {
            statuses.push((path.clone(), FileGitStatus::Modified));
        }
        for path in &self.state.git_status.untracked_files {
            statuses.push((path.clone(), FileGitStatus::Untracked));
        }

        if !all_files.is_empty() {
            let tree_state = super::components::FileTreeState::from_paths(&all_files, &statuses);
            self.state.modes.explore.file_tree = tree_state;
        }
    }

    /// Run the TUI application
    pub fn run(&mut self) -> Result<ExitResult> {
        // Install panic hook to ensure terminal is restored on panic
        let original_hook = std::panic::take_hook();
        std::panic::set_hook(Box::new(move |panic_info| {
            // Try to restore terminal
            let _ = disable_raw_mode();
            let _ = execute!(io::stdout(), LeaveAlternateScreen, DisableMouseCapture);
            // Print panic info to stderr
            eprintln!("\n\n=== PANIC ===\n{}\n", panic_info);
            original_hook(panic_info);
        }));

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

        // Auto-generate content based on initial mode
        match current_mode {
            Mode::Commit => {
                if self.state.git_status.has_staged() {
                    self.auto_generate_commit();
                }
            }
            Mode::PR => {
                self.update_pr_data(None, None);
                self.auto_generate_pr();
            }
            Mode::Review => {
                self.update_review_data(None, None);
                self.auto_generate_review();
            }
            Mode::Changelog => {
                self.update_changelog_data(None, None);
                self.auto_generate_changelog();
            }
            Mode::ReleaseNotes => {
                self.update_release_notes_data(None, None);
                self.auto_generate_release_notes();
            }
            Mode::Explore => {}
        }

        loop {
            // Check for completed Iris tasks
            self.check_iris_results();

            // Process any queued events through reducer
            if let Some(result) = self.process_events() {
                return Ok(result);
            }

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
                            // Push to event queue - reducer will handle via existing handlers
                            self.push_event(StudioEvent::KeyPressed(key));
                        }
                    }
                    Event::Mouse(mouse) => {
                        match mouse.kind {
                            MouseEventKind::Down(MouseButton::Left) => {
                                let now = std::time::Instant::now();
                                let is_double_click =
                                    self.last_click.is_some_and(|(time, lx, ly)| {
                                        now.duration_since(time).as_millis() < 400
                                            && mouse.column.abs_diff(lx) <= 2
                                            && mouse.row.abs_diff(ly) <= 1
                                    });

                                // Handle click based on what was clicked
                                if let Some(panel) = self.panel_at(mouse.column, mouse.row) {
                                    // Focus panel if not focused
                                    if self.state.focused_panel != panel {
                                        self.state.focused_panel = panel;
                                        self.state.mark_dirty();
                                    }

                                    // Start drag selection for code view
                                    if let Some(line) =
                                        self.code_view_line_at(panel, mouse.column, mouse.row)
                                    {
                                        self.drag_start = Some((panel, line));
                                        // Clear any existing selection and set cursor
                                        self.update_code_selection(panel, line, line);
                                    } else {
                                        self.drag_start = None;
                                    }

                                    // Handle file tree clicks
                                    self.handle_file_tree_click(
                                        panel,
                                        mouse.column,
                                        mouse.row,
                                        is_double_click,
                                    );
                                }

                                // Update last click for double-click detection
                                self.last_click = Some((now, mouse.column, mouse.row));
                            }
                            MouseEventKind::Drag(MouseButton::Left) => {
                                // Extend selection while dragging
                                if let Some((start_panel, start_line)) = self.drag_start
                                    && let Some(panel) = self.panel_at(mouse.column, mouse.row)
                                    && panel == start_panel
                                    && let Some(current_line) =
                                        self.code_view_line_at(panel, mouse.column, mouse.row)
                                {
                                    let (sel_start, sel_end) = if current_line < start_line {
                                        (current_line, start_line)
                                    } else {
                                        (start_line, current_line)
                                    };
                                    self.update_code_selection(panel, sel_start, sel_end);
                                }
                            }
                            MouseEventKind::Up(MouseButton::Left) => {
                                // Finalize drag selection
                                self.drag_start = None;
                            }
                            _ => {}
                        }
                        // Push scroll events to queue
                        self.push_event(StudioEvent::Mouse(mouse));
                    }
                    Event::Resize(_, _) => {
                        // Terminal resized, trigger redraw
                        self.state.mark_dirty();
                    }
                    _ => {}
                }
            }

            // Push tick event for animations
            self.push_event(StudioEvent::Tick);
        }
    }

    /// Check for completed Iris task results
    /// Convert async Iris results to events and push to queue
    fn check_iris_results(&mut self) {
        use super::state::Modal;

        while let Ok(result) = self.iris_result_rx.try_recv() {
            let event = match result {
                IrisTaskResult::CommitMessages(messages) => StudioEvent::AgentComplete {
                    task_type: TaskType::Commit,
                    result: AgentResult::CommitMessages(messages),
                },

                IrisTaskResult::ReviewContent(content) => StudioEvent::AgentComplete {
                    task_type: TaskType::Review,
                    result: AgentResult::ReviewContent(content),
                },

                IrisTaskResult::PRContent(content) => StudioEvent::AgentComplete {
                    task_type: TaskType::PR,
                    result: AgentResult::PRContent(content),
                },

                IrisTaskResult::ChangelogContent(content) => StudioEvent::AgentComplete {
                    task_type: TaskType::Changelog,
                    result: AgentResult::ChangelogContent(content),
                },

                IrisTaskResult::ReleaseNotesContent(content) => StudioEvent::AgentComplete {
                    task_type: TaskType::ReleaseNotes,
                    result: AgentResult::ReleaseNotesContent(content),
                },

                IrisTaskResult::ChatResponse(response) => StudioEvent::AgentComplete {
                    task_type: TaskType::Chat,
                    result: AgentResult::ChatResponse(response),
                },

                IrisTaskResult::ChatUpdate(update) => {
                    let (content_type, content) = match update {
                        ChatUpdateType::CommitMessage(msg) => {
                            (ContentType::CommitMessage, ContentPayload::Commit(msg))
                        }
                        ChatUpdateType::PRDescription(content) => (
                            ContentType::PRDescription,
                            ContentPayload::Markdown(content),
                        ),
                        ChatUpdateType::Review(content) => {
                            (ContentType::CodeReview, ContentPayload::Markdown(content))
                        }
                    };
                    StudioEvent::UpdateContent {
                        content_type,
                        content,
                    }
                }

                IrisTaskResult::SemanticBlame(result) => StudioEvent::AgentComplete {
                    task_type: TaskType::SemanticBlame,
                    result: AgentResult::SemanticBlame(result),
                },

                IrisTaskResult::ToolStatus { tool_name, message } => {
                    // Tool status updates - move current tool to history, set new current
                    let tool_desc = format!("{} - {}", tool_name, message);
                    if let Some(Modal::Chat(chat)) = &mut self.state.modal {
                        // Move previous tool to history
                        if let Some(prev) = chat.current_tool.take() {
                            chat.tool_history.push(prev);
                        }
                        chat.current_tool = Some(tool_desc.clone());
                    }
                    if let Some(prev) = self.state.chat_state.current_tool.take() {
                        self.state.chat_state.tool_history.push(prev);
                    }
                    self.state.chat_state.current_tool = Some(tool_desc);
                    self.state.mark_dirty();
                    continue; // Already handled, skip event push
                }

                IrisTaskResult::StreamingChunk {
                    task_type,
                    chunk,
                    aggregated,
                } => StudioEvent::StreamingChunk {
                    task_type,
                    chunk,
                    aggregated,
                },

                IrisTaskResult::StreamingComplete { task_type } => {
                    StudioEvent::StreamingComplete { task_type }
                }

                IrisTaskResult::Error(err) => {
                    // Determine which task failed based on what's currently generating
                    let task_type = if self.state.modes.commit.generating {
                        TaskType::Commit
                    } else if self.state.modes.review.generating {
                        TaskType::Review
                    } else if self.state.modes.pr.generating {
                        TaskType::PR
                    } else if self.state.modes.changelog.generating {
                        TaskType::Changelog
                    } else if self.state.modes.release_notes.generating {
                        TaskType::ReleaseNotes
                    } else if matches!(&self.state.modal, Some(Modal::Chat(c)) if c.is_responding) {
                        TaskType::Chat
                    } else if self.state.modes.explore.blame_loading {
                        TaskType::SemanticBlame
                    } else {
                        // Default to commit if we can't determine
                        TaskType::Commit
                    };

                    StudioEvent::AgentError {
                        task_type,
                        error: err,
                    }
                }
            };

            self.push_event(event);
        }
    }
    /// Spawn a task for chat query - uses Iris agent with chat capability
    fn spawn_chat_query(&self, message: String, context: super::events::ChatContext) {
        use crate::agents::StructuredResponse;
        use crate::agents::status::IRIS_STATUS;
        use crate::agents::tools::{ContentUpdate, create_content_update_channel};
        use crate::studio::state::{ChatMessage, ChatRole, Modal};
        use std::sync::Arc;
        use std::sync::atomic::{AtomicBool, Ordering};

        let Some(agent) = self.agent_service.clone() else {
            let tx = self.iris_result_tx.clone();
            let _ = tx.send(IrisTaskResult::ChatResponse(
                "Agent service not available".to_string(),
            ));
            return;
        };

        // Create content update channel for tool-based updates
        let (content_tx, mut content_rx) = create_content_update_channel();

        // Capture context before spawning async task
        let tx = self.iris_result_tx.clone();
        let tx_status = self.iris_result_tx.clone();
        let tx_updates = self.iris_result_tx.clone();
        let mode = context.mode;

        // Extract conversation history from chat modal
        let chat_history: Vec<ChatMessage> = if let Some(Modal::Chat(chat)) = &self.state.modal {
            chat.messages.clone()
        } else {
            Vec::new()
        };

        // Use context content if provided, otherwise extract from state
        let current_content = context
            .current_content
            .or_else(|| self.get_current_content_for_chat());

        // Flag to signal when the main task is done
        let is_done = Arc::new(AtomicBool::new(false));
        let is_done_clone = is_done.clone();
        let is_done_updates = is_done.clone();

        // Spawn a status polling task
        tokio::spawn(async move {
            use crate::agents::status::IrisPhase;
            let mut last_tool: Option<String> = None;

            while !is_done_clone.load(Ordering::Relaxed) {
                let status = IRIS_STATUS.get_current();

                // Check if we're in a tool execution phase
                if let IrisPhase::ToolExecution {
                    ref tool_name,
                    ref reason,
                } = status.phase
                {
                    // Only send if it's a new tool
                    if last_tool.as_ref() != Some(tool_name) {
                        let _ = tx_status.send(IrisTaskResult::ToolStatus {
                            tool_name: tool_name.clone(),
                            message: reason.clone(),
                        });
                        last_tool = Some(tool_name.clone());
                    }
                }

                tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;
            }
        });

        // Spawn a task to listen for content updates from tools
        tokio::spawn(async move {
            while !is_done_updates.load(Ordering::Relaxed) {
                match content_rx.try_recv() {
                    Ok(update) => {
                        let chat_update = match update {
                            ContentUpdate::Commit {
                                emoji,
                                title,
                                message,
                            } => {
                                tracing::info!("Content update tool: commit - {}", title);
                                ChatUpdateType::CommitMessage(crate::types::GeneratedMessage {
                                    emoji,
                                    title,
                                    message,
                                })
                            }
                            ContentUpdate::PR { content } => {
                                tracing::info!("Content update tool: PR");
                                ChatUpdateType::PRDescription(content)
                            }
                            ContentUpdate::Review { content } => {
                                tracing::info!("Content update tool: review");
                                ChatUpdateType::Review(content)
                            }
                        };
                        let _ = tx_updates.send(IrisTaskResult::ChatUpdate(chat_update));
                    }
                    Err(tokio::sync::mpsc::error::TryRecvError::Empty) => {
                        tokio::time::sleep(tokio::time::Duration::from_millis(50)).await;
                    }
                    Err(tokio::sync::mpsc::error::TryRecvError::Disconnected) => break,
                }
            }
        });

        tokio::spawn(async move {
            // Build comprehensive context (universal chat across all modes)
            let mode_context = format!(
                "Current Mode: {:?}\nYou are Iris, a helpful git assistant. You have access to all generated content across modes and can help with commit messages, PR descriptions, code reviews, changelogs, and release notes.",
                mode
            );

            // Build conversation history string
            let history_str = if chat_history.is_empty() {
                String::new()
            } else {
                let mut hist = String::from("\n## Conversation History\n");
                for msg in &chat_history {
                    match msg.role {
                        ChatRole::User => hist.push_str(&format!("User: {}\n", msg.content)),
                        ChatRole::Iris => hist.push_str(&format!("Iris: {}\n", msg.content)),
                    }
                }
                hist
            };

            // Build current content section
            let content_section = if let Some(content) = &current_content {
                format!("\n## Current Content\n```\n{}\n```\n", content)
            } else {
                String::new()
            };

            // Tool-based update instructions
            let update_instructions = r"
## Response Guidelines
- Be concise - don't repeat content the user already sees
- When updating content, briefly explain what you changed

## Content Update Tools
You have tools to update content. When the user asks you to modify, change, update, or rewrite content:

1. **update_commit** - Update the commit message (emoji, title, message)
2. **update_pr** - Update the PR description (content)
3. **update_review** - Update the code review (content)

Simply call the appropriate tool with the new content. Do NOT echo back the full content in your response - the tool will update it directly.";

            let prompt = format!(
                "{}{}{}{}\n\n## Current Request\nUser: {}",
                mode_context, content_section, history_str, update_instructions, message
            );

            // Execute with streaming and content update tools
            let streaming_tx = tx.clone();
            let on_chunk = move |chunk: &str, aggregated: &str| {
                let _ = streaming_tx.send(IrisTaskResult::StreamingChunk {
                    task_type: TaskType::Chat,
                    chunk: chunk.to_string(),
                    aggregated: aggregated.to_string(),
                });
            };

            match agent
                .execute_chat_streaming(&prompt, content_tx, on_chunk)
                .await
            {
                Ok(response) => {
                    // Signal streaming complete
                    let _ = tx.send(IrisTaskResult::StreamingComplete {
                        task_type: TaskType::Chat,
                    });

                    let text = match response {
                        StructuredResponse::PlainText(text) => text,
                        other => other.to_string(),
                    };

                    tracing::debug!("Chat response received, length: {}", text.len());
                    let _ = tx.send(IrisTaskResult::ChatResponse(text));
                }
                Err(e) => {
                    let _ = tx.send(IrisTaskResult::ChatResponse(format!(
                        "I encountered an error: {}",
                        e
                    )));
                }
            }

            // Signal that we're done so the status polling task stops
            is_done.store(true, Ordering::Relaxed);
        });
    }

    /// Get ALL generated content for chat context (universal across modes)
    fn get_current_content_for_chat(&self) -> Option<String> {
        let mut sections = Vec::new();

        // Commit message
        let commit = &self.state.modes.commit;
        if let Some(msg) = commit.messages.get(commit.current_index) {
            let formatted = crate::types::format_commit_message(msg);
            if !formatted.trim().is_empty() {
                sections.push(format!("## Commit Message\n{}", formatted));
            }
        }

        // Code review
        let review = &self.state.modes.review.review_content;
        if !review.is_empty() {
            let preview = if review.len() > 500 {
                format!("{}...", &review[..500])
            } else {
                review.clone()
            };
            sections.push(format!("## Code Review\n{}", preview));
        }

        // PR description
        let pr = &self.state.modes.pr.pr_content;
        if !pr.is_empty() {
            let preview = if pr.len() > 500 {
                format!("{}...", &pr[..500])
            } else {
                pr.clone()
            };
            sections.push(format!("## PR Description\n{}", preview));
        }

        // Changelog
        let cl = &self.state.modes.changelog.changelog_content;
        if !cl.is_empty() {
            let preview = if cl.len() > 500 {
                format!("{}...", &cl[..500])
            } else {
                cl.clone()
            };
            sections.push(format!("## Changelog\n{}", preview));
        }

        // Release notes
        let rn = &self.state.modes.release_notes.release_notes_content;
        if !rn.is_empty() {
            let preview = if rn.len() > 500 {
                format!("{}...", &rn[..500])
            } else {
                rn.clone()
            };
            sections.push(format!("## Release Notes\n{}", preview));
        }

        if sections.is_empty() {
            None
        } else {
            Some(sections.join("\n\n"))
        }
    }

    /// Spawn a task for code review generation with streaming
    fn spawn_review_generation(&self, from_ref: String, to_ref: String) {
        use crate::agents::{StructuredResponse, TaskContext};

        let Some(agent) = self.agent_service.clone() else {
            let tx = self.iris_result_tx.clone();
            let _ = tx.send(IrisTaskResult::Error(
                "Agent service not available".to_string(),
            ));
            return;
        };

        let tx = self.iris_result_tx.clone();
        let streaming_tx = tx.clone();

        tokio::spawn(async move {
            // Use review context with specified refs
            let context = match TaskContext::for_review(None, Some(from_ref), Some(to_ref), false) {
                Ok(ctx) => ctx,
                Err(e) => {
                    let _ = tx.send(IrisTaskResult::Error(format!("Context error: {}", e)));
                    return;
                }
            };

            // Execute with streaming - send chunks as they arrive
            let on_chunk = {
                let tx = streaming_tx.clone();
                move |chunk: &str, aggregated: &str| {
                    let _ = tx.send(IrisTaskResult::StreamingChunk {
                        task_type: TaskType::Review,
                        chunk: chunk.to_string(),
                        aggregated: aggregated.to_string(),
                    });
                }
            };

            match agent
                .execute_task_streaming("review", context, on_chunk)
                .await
            {
                Ok(response) => {
                    // Send streaming complete
                    let _ = tx.send(IrisTaskResult::StreamingComplete {
                        task_type: TaskType::Review,
                    });

                    // Also send final content
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

    /// Spawn a task for PR description generation with streaming
    fn spawn_pr_generation(&self, base_branch: String, _to_ref: String) {
        use crate::agents::{StructuredResponse, TaskContext};

        let Some(agent) = self.agent_service.clone() else {
            let tx = self.iris_result_tx.clone();
            let _ = tx.send(IrisTaskResult::Error(
                "Agent service not available".to_string(),
            ));
            return;
        };

        let tx = self.iris_result_tx.clone();
        let streaming_tx = tx.clone();

        tokio::spawn(async move {
            // Build context for PR (comparing current branch to base)
            let context = TaskContext::for_pr(Some(base_branch), None);

            // Execute with streaming
            let on_chunk = {
                let tx = streaming_tx.clone();
                move |chunk: &str, aggregated: &str| {
                    let _ = tx.send(IrisTaskResult::StreamingChunk {
                        task_type: TaskType::PR,
                        chunk: chunk.to_string(),
                        aggregated: aggregated.to_string(),
                    });
                }
            };

            match agent.execute_task_streaming("pr", context, on_chunk).await {
                Ok(response) => {
                    let _ = tx.send(IrisTaskResult::StreamingComplete {
                        task_type: TaskType::PR,
                    });

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

    /// Spawn a task for changelog generation with streaming
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
        let streaming_tx = tx.clone();

        tokio::spawn(async move {
            // Build context for changelog (comparing two refs)
            let context = TaskContext::for_changelog(from_ref, Some(to_ref));

            // Execute with streaming
            let on_chunk = {
                let tx = streaming_tx.clone();
                move |chunk: &str, aggregated: &str| {
                    let _ = tx.send(IrisTaskResult::StreamingChunk {
                        task_type: TaskType::Changelog,
                        chunk: chunk.to_string(),
                        aggregated: aggregated.to_string(),
                    });
                }
            };

            match agent
                .execute_task_streaming("changelog", context, on_chunk)
                .await
            {
                Ok(response) => {
                    let _ = tx.send(IrisTaskResult::StreamingComplete {
                        task_type: TaskType::Changelog,
                    });

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

    /// Spawn a task for release notes generation with streaming
    fn spawn_release_notes_generation(&self, from_ref: String, to_ref: String) {
        use crate::agents::{StructuredResponse, TaskContext};

        let Some(agent) = self.agent_service.clone() else {
            let tx = self.iris_result_tx.clone();
            let _ = tx.send(IrisTaskResult::Error(
                "Agent service not available".to_string(),
            ));
            return;
        };

        let tx = self.iris_result_tx.clone();
        let streaming_tx = tx.clone();

        tokio::spawn(async move {
            // Build context for release notes (comparing two refs)
            let context = TaskContext::for_changelog(from_ref, Some(to_ref));

            // Execute with streaming
            let on_chunk = {
                let tx = streaming_tx.clone();
                move |chunk: &str, aggregated: &str| {
                    let _ = tx.send(IrisTaskResult::StreamingChunk {
                        task_type: TaskType::ReleaseNotes,
                        chunk: chunk.to_string(),
                        aggregated: aggregated.to_string(),
                    });
                }
            };

            match agent
                .execute_task_streaming("release_notes", context, on_chunk)
                .await
            {
                Ok(response) => {
                    let _ = tx.send(IrisTaskResult::StreamingComplete {
                        task_type: TaskType::ReleaseNotes,
                    });

                    let release_notes_text = match response {
                        StructuredResponse::ReleaseNotes(rn) => rn.content,
                        StructuredResponse::PlainText(text) => text,
                        other => other.to_string(),
                    };
                    let _ = tx.send(IrisTaskResult::ReleaseNotesContent(release_notes_text));
                }
                Err(e) => {
                    let _ = tx.send(IrisTaskResult::Error(format!("Release notes error: {}", e)));
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
        let preset = self.state.modes.commit.preset.clone();
        let use_gitmoji = self.state.modes.commit.use_gitmoji;
        self.spawn_commit_generation(None, preset, use_gitmoji);
    }

    /// Auto-generate code review on mode entry
    fn auto_generate_review(&mut self) {
        // Don't regenerate if we already have content
        if !self.state.modes.review.review_content.is_empty() {
            return;
        }

        // Need diffs to review
        if self.state.modes.review.diff_view.file_paths().is_empty() {
            return;
        }

        self.state.set_iris_thinking("Reviewing code changes...");
        self.state.modes.review.generating = true;
        let from_ref = self.state.modes.review.from_ref.clone();
        let to_ref = self.state.modes.review.to_ref.clone();
        self.spawn_review_generation(from_ref, to_ref);
    }

    /// Auto-generate PR description on mode entry
    fn auto_generate_pr(&mut self) {
        // Don't regenerate if we already have content
        if !self.state.modes.pr.pr_content.is_empty() {
            return;
        }

        // Need commits to describe
        if self.state.modes.pr.commits.is_empty() {
            return;
        }

        self.state.set_iris_thinking("Drafting PR description...");
        self.state.modes.pr.generating = true;
        let base_branch = self.state.modes.pr.base_branch.clone();
        let to_ref = self.state.modes.pr.to_ref.clone();
        self.spawn_pr_generation(base_branch, to_ref);
    }

    /// Auto-generate changelog on mode entry
    fn auto_generate_changelog(&mut self) {
        // Don't regenerate if we already have content
        if !self.state.modes.changelog.changelog_content.is_empty() {
            return;
        }

        // Need commits to generate from
        if self.state.modes.changelog.commits.is_empty() {
            return;
        }

        let from_ref = self.state.modes.changelog.from_ref.clone();
        let to_ref = self.state.modes.changelog.to_ref.clone();

        self.state.set_iris_thinking("Generating changelog...");
        self.state.modes.changelog.generating = true;
        self.spawn_changelog_generation(from_ref, to_ref);
    }

    /// Auto-generate release notes on mode entry
    fn auto_generate_release_notes(&mut self) {
        // Don't regenerate if we already have content
        if !self
            .state
            .modes
            .release_notes
            .release_notes_content
            .is_empty()
        {
            return;
        }

        // Need commits to generate from
        if self.state.modes.release_notes.commits.is_empty() {
            return;
        }

        let from_ref = self.state.modes.release_notes.from_ref.clone();
        let to_ref = self.state.modes.release_notes.to_ref.clone();

        self.state.set_iris_thinking("Generating release notes...");
        self.state.modes.release_notes.generating = true;
        self.spawn_release_notes_generation(from_ref, to_ref);
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

    /// Handle mouse click in panels (file tree, code view, etc.)
    fn handle_file_tree_click(&mut self, panel: PanelId, _x: u16, y: u16, is_double_click: bool) {
        let Some(layout) = &self.last_layout else {
            return;
        };

        // Get the panel rect
        let panel_idx = match panel {
            PanelId::Left => 0,
            PanelId::Center => 1,
            PanelId::Right => 2,
        };

        let Some(panel_rect) = layout.panels.get(panel_idx) else {
            return;
        };

        // Calculate row within panel (accounting for border and title)
        // Panel has 1 row border on each side
        let inner_y = y.saturating_sub(panel_rect.y + 1);

        // Determine which component to update based on mode and panel
        match (self.state.active_mode, panel) {
            // ─────────────────────────────────────────────────────────────────
            // Explore Mode
            // ─────────────────────────────────────────────────────────────────
            (Mode::Explore, PanelId::Left) => {
                let file_tree = &mut self.state.modes.explore.file_tree;
                let (changed, is_dir) = file_tree.handle_click(inner_y as usize);

                if is_double_click && is_dir {
                    file_tree.toggle_expand();
                } else if is_double_click && !is_dir {
                    // Double-click on file: load it and focus code view
                    if let Some(path) = file_tree.selected_path() {
                        self.state.modes.explore.current_file = Some(path.clone());
                        if let Err(e) = self.state.modes.explore.code_view.load_file(&path) {
                            self.state.notify(Notification::warning(format!(
                                "Could not load file: {}",
                                e
                            )));
                        }
                        self.state.focused_panel = PanelId::Center;
                    }
                } else if changed && !is_dir {
                    // Single click on file: load it into code view
                    if let Some(path) = file_tree.selected_path() {
                        self.state.modes.explore.current_file = Some(path.clone());
                        if let Err(e) = self.state.modes.explore.code_view.load_file(&path) {
                            self.state.notify(Notification::warning(format!(
                                "Could not load file: {}",
                                e
                            )));
                        }
                    }
                }
                self.state.mark_dirty();
            }
            (Mode::Explore, PanelId::Center) => {
                // Code view: click to select line
                let code_view = &mut self.state.modes.explore.code_view;
                if code_view.select_by_row(inner_y as usize) {
                    self.state.mark_dirty();
                }
            }
            // ─────────────────────────────────────────────────────────────────
            // Commit Mode
            // ─────────────────────────────────────────────────────────────────
            (Mode::Commit, PanelId::Left) => {
                let file_tree = &mut self.state.modes.commit.file_tree;
                let (changed, is_dir) = file_tree.handle_click(inner_y as usize);

                if is_double_click && is_dir {
                    file_tree.toggle_expand();
                } else if is_double_click && !is_dir {
                    // Double-click on file: focus on diff panel
                    if let Some(path) = file_tree.selected_path() {
                        self.state.modes.commit.diff_view.select_file_by_path(&path);
                        self.state.focused_panel = PanelId::Right;
                    }
                } else if changed {
                    // Single click: sync diff view
                    if let Some(path) = file_tree.selected_path() {
                        self.state.modes.commit.diff_view.select_file_by_path(&path);
                    }
                }
                self.state.mark_dirty();
            }
            // ─────────────────────────────────────────────────────────────────
            // Review Mode
            // ─────────────────────────────────────────────────────────────────
            (Mode::Review, PanelId::Left) => {
                let file_tree = &mut self.state.modes.review.file_tree;
                let (changed, is_dir) = file_tree.handle_click(inner_y as usize);

                if is_double_click && is_dir {
                    file_tree.toggle_expand();
                } else if is_double_click && !is_dir {
                    // Double-click on file: focus on diff panel
                    if let Some(path) = file_tree.selected_path() {
                        self.state.modes.review.diff_view.select_file_by_path(&path);
                        self.state.focused_panel = PanelId::Center;
                    }
                } else if changed {
                    // Single click: sync diff view
                    if let Some(path) = file_tree.selected_path() {
                        self.state.modes.review.diff_view.select_file_by_path(&path);
                    }
                }
                self.state.mark_dirty();
            }
            // ─────────────────────────────────────────────────────────────────
            // PR Mode
            // ─────────────────────────────────────────────────────────────────
            (Mode::PR, PanelId::Left) => {
                let file_tree = &mut self.state.modes.pr.file_tree;
                let (changed, is_dir) = file_tree.handle_click(inner_y as usize);

                if is_double_click && is_dir {
                    file_tree.toggle_expand();
                } else if (changed || is_double_click)
                    && let Some(path) = file_tree.selected_path()
                {
                    self.state.modes.pr.diff_view.select_file_by_path(&path);
                }
                self.state.mark_dirty();
            }
            _ => {}
        }
    }

    /// Get the line number at a mouse position in the code view (1-indexed)
    /// Returns None if not in a code view area
    fn code_view_line_at(&self, panel: PanelId, _x: u16, y: u16) -> Option<usize> {
        let layout = self.last_layout.as_ref()?;

        // Get the panel rect
        let panel_idx = match panel {
            PanelId::Left => 0,
            PanelId::Center => 1,
            PanelId::Right => 2,
        };

        let panel_rect = layout.panels.get(panel_idx)?;

        // Calculate row within panel (accounting for border)
        let inner_y = y.saturating_sub(panel_rect.y + 1) as usize;

        // Only handle code view panels based on mode
        match (self.state.active_mode, panel) {
            (Mode::Explore, PanelId::Center) => {
                let code_view = &self.state.modes.explore.code_view;
                let target_line = code_view.scroll_offset() + inner_y + 1;
                if target_line <= code_view.line_count() {
                    Some(target_line)
                } else {
                    None
                }
            }
            _ => None,
        }
    }

    /// Update code view selection range (for mouse drag)
    fn update_code_selection(&mut self, panel: PanelId, start: usize, end: usize) {
        if let (Mode::Explore, PanelId::Center) = (self.state.active_mode, panel) {
            // Update code view state
            self.state.modes.explore.code_view.set_selected_line(start);
            if start == end {
                // Single line - set anchor for potential drag extension
                self.state.modes.explore.selection_anchor = Some(start);
                self.state.modes.explore.code_view.clear_selection();
                self.state.modes.explore.selection = None;
            } else {
                // Multi-line selection from drag
                self.state.modes.explore.code_view.set_selection(start, end);
                self.state.modes.explore.selection = Some((start, end));
            }
            // Update current line for semantic blame
            self.state.modes.explore.current_line = start;
            self.state.mark_dirty();
        }
    }

    /// Spawn a task to generate a commit message
    fn spawn_commit_generation(
        &self,
        _instructions: Option<String>,
        preset: String,
        use_gitmoji: bool,
    ) {
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
            // Use standard commit context
            let context = TaskContext::for_gen();

            // Execute commit capability with style overrides
            let preset_opt = if preset == "default" {
                None
            } else {
                Some(preset.as_str())
            };

            match agent
                .execute_task_with_style("commit", context, preset_opt, Some(use_gitmoji))
                .await
            {
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
    }

    /// Gather blame information from git and spawn the semantic blame agent
    fn gather_blame_and_spawn(&self, file: &std::path::Path, start_line: usize, end_line: usize) {
        use std::fs;
        use std::process::Command;

        let Some(repo) = &self.state.repo else {
            let tx = self.iris_result_tx.clone();
            let _ = tx.send(IrisTaskResult::Error(
                "Repository not available".to_string(),
            ));
            return;
        };

        // Read the file content for the specified range
        let code_content = match fs::read_to_string(file) {
            Ok(content) => {
                let lines: Vec<&str> = content.lines().collect();
                if start_line == 0 || start_line > lines.len() {
                    let tx = self.iris_result_tx.clone();
                    let _ = tx.send(IrisTaskResult::Error("Invalid line range".to_string()));
                    return;
                }
                let end = end_line.min(lines.len());
                lines[(start_line - 1)..end].join("\n")
            }
            Err(e) => {
                let tx = self.iris_result_tx.clone();
                let _ = tx.send(IrisTaskResult::Error(format!("Could not read file: {}", e)));
                return;
            }
        };

        // Get repo path
        let repo_path = repo.repo_path();

        // Run git blame with porcelain format to get commit info
        let output = Command::new("git")
            .args([
                "-C",
                &repo_path.to_string_lossy(),
                "blame",
                "-L",
                &format!("{},{}", start_line, end_line),
                "--porcelain",
                &file.to_string_lossy(),
            ])
            .output();

        let (commit_hash, author, commit_date, commit_message) = match output {
            Ok(output) if output.status.success() => {
                let blame_output = String::from_utf8_lossy(&output.stdout);
                parse_blame_porcelain(&blame_output)
            }
            Ok(output) => {
                let err = String::from_utf8_lossy(&output.stderr);
                let tx = self.iris_result_tx.clone();
                let _ = tx.send(IrisTaskResult::Error(format!("Git blame failed: {}", err)));
                return;
            }
            Err(e) => {
                let tx = self.iris_result_tx.clone();
                let _ = tx.send(IrisTaskResult::Error(format!("Could not run git: {}", e)));
                return;
            }
        };

        let blame_info = BlameInfo {
            file: file.to_path_buf(),
            start_line,
            end_line,
            commit_hash,
            author,
            commit_date,
            commit_message,
            code_content,
        };

        // Now spawn the semantic blame agent
        self.spawn_semantic_blame(blame_info);
    }

    /// Spawn the semantic blame agent to explain why the code exists
    fn spawn_semantic_blame(&self, blame_info: BlameInfo) {
        use crate::agents::StructuredResponse;

        let Some(agent) = self.agent_service.clone() else {
            let tx = self.iris_result_tx.clone();
            let _ = tx.send(IrisTaskResult::Error(
                "Agent service not available".to_string(),
            ));
            return;
        };

        let tx = self.iris_result_tx.clone();

        tokio::spawn(async move {
            // Build context with blame info
            let context_text = format!(
                "File: {}\nLines: {}-{}\nCommit: {} by {} on {}\nMessage: {}\n\nCode:\n{}",
                blame_info.file.display(),
                blame_info.start_line,
                blame_info.end_line,
                blame_info.commit_hash,
                blame_info.author,
                blame_info.commit_date,
                blame_info.commit_message,
                blame_info.code_content
            );

            // Execute semantic_blame capability
            match agent
                .execute_task_with_prompt("semantic_blame", &context_text)
                .await
            {
                Ok(response) => match response {
                    StructuredResponse::SemanticBlame(explanation) => {
                        let result = SemanticBlameResult {
                            file: blame_info.file,
                            start_line: blame_info.start_line,
                            end_line: blame_info.end_line,
                            commit_hash: blame_info.commit_hash,
                            author: blame_info.author,
                            commit_date: blame_info.commit_date,
                            commit_message: blame_info.commit_message,
                            explanation,
                        };
                        let _ = tx.send(IrisTaskResult::SemanticBlame(result));
                    }
                    _ => {
                        let _ = tx.send(IrisTaskResult::Error(
                            "Unexpected response type from agent".to_string(),
                        ));
                    }
                },
                Err(e) => {
                    let _ = tx.send(IrisTaskResult::Error(format!(
                        "Semantic blame error: {}",
                        e
                    )));
                }
            }
        });
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
        let modified = self.state.git_status.modified_count;

        // Create gradient title "◆ Iris Studio"
        let mut spans: Vec<Span> = Vec::new();
        spans.push(Span::styled(
            " ◆ ",
            Style::default().fg(theme::ELECTRIC_PURPLE),
        ));

        // Gradient text for "Iris Studio"
        let title_text = "Iris Studio";
        #[allow(clippy::cast_precision_loss)]
        for (i, c) in title_text.chars().enumerate() {
            let position = i as f32 / (title_text.len() - 1).max(1) as f32;
            spans.push(Span::styled(
                c.to_string(),
                Style::default()
                    .fg(theme::gradient_purple_cyan(position))
                    .add_modifier(Modifier::BOLD),
            ));
        }

        spans.push(Span::raw(" "));

        // Branch info with git icon
        if !branch.is_empty() {
            spans.push(Span::styled("⎇ ", Style::default().fg(theme::TEXT_DIM)));
            spans.push(Span::styled(
                format!("{} ", branch),
                Style::default()
                    .fg(theme::NEON_CYAN)
                    .add_modifier(Modifier::BOLD),
            ));
        }

        // Staged count
        if staged > 0 {
            spans.push(Span::styled(
                format!("✓{} ", staged),
                Style::default().fg(theme::SUCCESS_GREEN),
            ));
        }

        // Modified count
        if modified > 0 {
            spans.push(Span::styled(
                format!("○{} ", modified),
                Style::default().fg(theme::ELECTRIC_YELLOW),
            ));
        }

        let line = Line::from(spans);
        let header = Paragraph::new(line);
        frame.render_widget(header, area);
    }

    fn render_tabs(&self, frame: &mut Frame, area: Rect) {
        let mut spans = Vec::new();
        spans.push(Span::raw(" "));

        for (idx, mode) in Mode::all().iter().enumerate() {
            let is_active = *mode == self.state.active_mode;
            let is_available = mode.is_available();

            if is_active {
                // Active tab with gradient underline effect
                spans.push(Span::styled(
                    format!(" {} ", mode.shortcut()),
                    Style::default()
                        .fg(theme::ELECTRIC_PURPLE)
                        .add_modifier(Modifier::BOLD),
                ));
                // Mode name with gradient
                let name = mode.display_name();
                #[allow(clippy::cast_precision_loss)]
                for (i, c) in name.chars().enumerate() {
                    let position = i as f32 / (name.len() - 1).max(1) as f32;
                    spans.push(Span::styled(
                        c.to_string(),
                        Style::default()
                            .fg(theme::gradient_purple_cyan(position))
                            .add_modifier(Modifier::BOLD),
                    ));
                }
                spans.push(Span::raw(" "));
                // Underline with gradient
                spans.push(Span::styled(
                    "━",
                    Style::default().fg(theme::ELECTRIC_PURPLE),
                ));
                spans.push(Span::styled("━", Style::default().fg(theme::SOFT_PURPLE)));
                spans.push(Span::styled("━", Style::default().fg(theme::NEON_CYAN)));
            } else if is_available {
                spans.push(Span::styled(
                    format!(" {} ", mode.shortcut()),
                    Style::default().fg(theme::TEXT_MUTED),
                ));
                spans.push(Span::styled(
                    mode.display_name().to_string(),
                    theme::mode_inactive(),
                ));
            } else {
                spans.push(Span::styled(
                    format!(" {} {} ", mode.shortcut(), mode.display_name()),
                    Style::default().fg(theme::TEXT_MUTED),
                ));
            }

            // Separator between tabs
            if idx < Mode::all().len() - 1 {
                spans.push(Span::styled(" │ ", Style::default().fg(theme::TEXT_MUTED)));
            }
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
            Mode::ReleaseNotes => {
                render_release_notes_panel(&mut self.state, frame, area, panel_id);
            }
        }
    }

    /// Update commit mode file tree from git status
    /// Shows either changed files (staged/unstaged) or all tracked files based on toggle
    fn update_commit_file_tree(&mut self) {
        let mut statuses = Vec::new();

        // Build status map for known changed files
        for path in &self.state.git_status.staged_files {
            statuses.push((path.clone(), FileGitStatus::Staged));
        }
        for path in &self.state.git_status.modified_files {
            if !self.state.git_status.staged_files.contains(path) {
                statuses.push((path.clone(), FileGitStatus::Modified));
            }
        }
        for path in &self.state.git_status.untracked_files {
            statuses.push((path.clone(), FileGitStatus::Untracked));
        }

        let all_files: Vec<std::path::PathBuf> = if self.state.modes.commit.show_all_files {
            // Show all tracked files from the repository
            let Some(repo) = &self.state.repo else {
                return;
            };
            match repo.get_all_tracked_files() {
                Ok(files) => files.into_iter().map(std::path::PathBuf::from).collect(),
                Err(e) => {
                    eprintln!("Failed to get tracked files: {}", e);
                    return;
                }
            }
        } else {
            // Show only changed files (staged + modified + untracked)
            let mut files = Vec::new();
            for path in &self.state.git_status.staged_files {
                files.push(path.clone());
            }
            for path in &self.state.git_status.modified_files {
                if !files.contains(path) {
                    files.push(path.clone());
                }
            }
            for path in &self.state.git_status.untracked_files {
                if !files.contains(path) {
                    files.push(path.clone());
                }
            }
            files
        };

        let tree_state = super::components::FileTreeState::from_paths(&all_files, &statuses);
        self.state.modes.commit.file_tree = tree_state;

        // Expand all by default (usually not too many files)
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
    pub fn update_pr_data(&mut self, from_ref: Option<String>, to_ref: Option<String>) {
        use super::state::PrCommit;

        // Clone the Arc to avoid borrow conflicts with self.state mutations
        let Some(repo) = self.state.repo.clone() else {
            return;
        };

        // Use provided refs or fall back to state
        let base = from_ref.unwrap_or_else(|| self.state.modes.pr.base_branch.clone());
        let to = to_ref.unwrap_or_else(|| self.state.modes.pr.to_ref.clone());

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
    pub fn update_review_data(&mut self, from_ref: Option<String>, to_ref: Option<String>) {
        // Clone the Arc to avoid borrow conflicts with self.state mutations
        let Some(repo) = self.state.repo.clone() else {
            return;
        };

        // Use provided refs or fall back to state
        let from = from_ref.unwrap_or_else(|| self.state.modes.review.from_ref.clone());
        let to = to_ref.unwrap_or_else(|| self.state.modes.review.to_ref.clone());

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
    pub fn update_changelog_data(&mut self, from_ref: Option<String>, to_ref: Option<String>) {
        use super::state::ChangelogCommit;

        // Clone the Arc to avoid borrow conflicts with self.state mutations
        let Some(repo) = self.state.repo.clone() else {
            return;
        };

        // Use provided refs or fall back to state
        let from = from_ref.unwrap_or_else(|| self.state.modes.changelog.from_ref.clone());
        let to = to_ref.unwrap_or_else(|| self.state.modes.changelog.to_ref.clone());

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

    /// Update release notes mode data when refs change
    pub fn update_release_notes_data(&mut self, from_ref: Option<String>, to_ref: Option<String>) {
        use super::state::ChangelogCommit;

        // Clone the Arc to avoid borrow conflicts with self.state mutations
        let Some(repo) = self.state.repo.clone() else {
            return;
        };

        // Use provided refs or fall back to state
        let from = from_ref.unwrap_or_else(|| self.state.modes.release_notes.from_ref.clone());
        let to = to_ref.unwrap_or_else(|| self.state.modes.release_notes.to_ref.clone());

        // Load commits between the refs
        match repo.get_commits_between_with_callback(&from, &to, |commit| {
            Ok(ChangelogCommit {
                hash: commit.hash[..7.min(commit.hash.len())].to_string(),
                message: commit.message.lines().next().unwrap_or("").to_string(),
                author: commit.author.clone(),
            })
        }) {
            Ok(commits) => {
                self.state.modes.release_notes.commits = commits;
                self.state.modes.release_notes.selected_commit = 0;
                self.state.modes.release_notes.commit_scroll = 0;
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
                self.state.modes.release_notes.diff_view.set_diffs(diffs);
            }
            Err(e) => {
                self.state
                    .notify(Notification::warning(format!("Could not load diff: {}", e)));
            }
        }

        self.state.mark_dirty();
    }

    // ═══════════════════════════════════════════════════════════════════════════
    // Staging Operations
    // ═══════════════════════════════════════════════════════════════════════════

    /// Stage a single file
    fn stage_file(&mut self, path: &str) {
        let Some(repo) = &self.state.repo else {
            self.state
                .notify(Notification::error("No repository available"));
            return;
        };

        match repo.stage_file(std::path::Path::new(path)) {
            Ok(()) => {
                self.state
                    .notify(Notification::success(format!("Staged: {}", path)));
                let _ = self.refresh_git_status();
            }
            Err(e) => {
                self.state
                    .notify(Notification::error(format!("Failed to stage: {}", e)));
            }
        }
        self.state.mark_dirty();
    }

    /// Unstage a single file
    fn unstage_file(&mut self, path: &str) {
        let Some(repo) = &self.state.repo else {
            self.state
                .notify(Notification::error("No repository available"));
            return;
        };

        match repo.unstage_file(std::path::Path::new(path)) {
            Ok(()) => {
                self.state
                    .notify(Notification::success(format!("Unstaged: {}", path)));
                let _ = self.refresh_git_status();
            }
            Err(e) => {
                self.state
                    .notify(Notification::error(format!("Failed to unstage: {}", e)));
            }
        }
        self.state.mark_dirty();
    }

    /// Stage all files
    fn stage_all(&mut self) {
        let Some(repo) = &self.state.repo else {
            self.state
                .notify(Notification::error("No repository available"));
            return;
        };

        match repo.stage_all() {
            Ok(()) => {
                self.state.notify(Notification::success("Staged all files"));
                let _ = self.refresh_git_status();
            }
            Err(e) => {
                self.state
                    .notify(Notification::error(format!("Failed to stage all: {}", e)));
            }
        }
        self.state.mark_dirty();
    }

    /// Unstage all files
    fn unstage_all(&mut self) {
        let Some(repo) = &self.state.repo else {
            self.state
                .notify(Notification::error("No repository available"));
            return;
        };

        match repo.unstage_all() {
            Ok(()) => {
                self.state
                    .notify(Notification::success("Unstaged all files"));
                let _ = self.refresh_git_status();
            }
            Err(e) => {
                self.state
                    .notify(Notification::error(format!("Failed to unstage all: {}", e)));
            }
        }
        self.state.mark_dirty();
    }

    /// Save settings from the settings modal to config file
    fn save_settings(&mut self) {
        use crate::studio::state::Modal;

        let settings = if let Some(Modal::Settings(s)) = &self.state.modal {
            s.clone()
        } else {
            return;
        };

        if !settings.modified {
            self.state.notify(Notification::info("No changes to save"));
            return;
        }

        // Update config
        let mut config = self.state.config.clone();
        config.default_provider.clone_from(&settings.provider);
        config.use_gitmoji = settings.use_gitmoji;
        config
            .instruction_preset
            .clone_from(&settings.instruction_preset);

        // Update provider config
        if let Some(provider_config) = config.providers.get_mut(&settings.provider) {
            provider_config.model.clone_from(&settings.model);
            if let Some(api_key) = &settings.api_key_actual {
                provider_config.api_key.clone_from(api_key);
            }
        }

        // Save to file
        match config.save() {
            Ok(()) => {
                self.state.config = config;
                // Clear the modified flag
                if let Some(Modal::Settings(s)) = &mut self.state.modal {
                    s.modified = false;
                    s.error = None;
                }
                self.state.notify(Notification::success("Settings saved"));
            }
            Err(e) => {
                if let Some(Modal::Settings(s)) = &mut self.state.modal {
                    s.error = Some(format!("Save failed: {}", e));
                }
                self.state
                    .notify(Notification::error(format!("Failed to save: {}", e)));
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
            // Context-aware keybinding hints based on mode and panel
            let hints = self.get_context_hints();
            spans.push(Span::styled(hints, theme::dimmed()));
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

    /// Get context-aware keybinding hints based on mode and focused panel
    fn get_context_hints(&self) -> String {
        let base = "[?]help [Tab]panel [q]quit";

        match self.state.active_mode {
            Mode::Commit => match self.state.focused_panel {
                PanelId::Left => format!("{} · [↑↓]nav [s]stage [u]unstage [a]all [U]reset", base),
                PanelId::Center => format!(
                    "{} · [e]edit [r]regen [p]preset [g]emoji [←→]msg [Enter]commit",
                    base
                ),
                PanelId::Right => format!("{} · [↑↓]scroll [n/p]file []/[]hunk", base),
            },
            Mode::Review | Mode::PR | Mode::Changelog | Mode::ReleaseNotes => {
                match self.state.focused_panel {
                    PanelId::Left => format!("{} · [f/t]set refs [r]generate", base),
                    PanelId::Center => format!("{} · [↑↓]scroll [y]copy [r]generate", base),
                    PanelId::Right => format!("{} · [↑↓]scroll", base),
                }
            }
            Mode::Explore => match self.state.focused_panel {
                PanelId::Left => format!("{} · [↑↓]nav [Enter]open", base),
                PanelId::Center => {
                    format!("{} · [↑↓]nav [v]select [y]copy [Y]copy file [w]why", base)
                }
                PanelId::Right => format!("{} · [c]chat", base),
            },
        }
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
    // Enable file logging for debugging (TUI owns stdout, so logs go to file only)
    // Only set up default log file if one wasn't specified via CLI (-l --log-file)
    if !crate::logger::has_log_file()
        && let Err(e) = crate::logger::set_log_file(crate::cli::LOG_FILE)
    {
        eprintln!("Warning: Could not set up log file: {}", e);
    }
    // Disable stdout logging - TUI owns the terminal, but debug goes to file
    crate::logger::set_log_to_stdout(false);
    tracing::info!("Iris Studio starting");

    let mut app = StudioApp::new(config, repo, commit_service, agent_service);

    // Set initial mode if specified
    if let Some(mode) = initial_mode {
        app.set_initial_mode(mode);
    }

    // Set comparison refs if specified (applies to Review, PR, Changelog, and Release Notes modes)
    if let Some(from) = from_ref {
        app.state.modes.review.from_ref = from.clone();
        app.state.modes.pr.base_branch = from.clone();
        app.state.modes.changelog.from_ref = from.clone();
        app.state.modes.release_notes.from_ref = from;
    }
    if let Some(to) = to_ref {
        app.state.modes.review.to_ref = to.clone();
        app.state.modes.pr.to_ref = to.clone();
        app.state.modes.changelog.to_ref = to.clone();
        app.state.modes.release_notes.to_ref = to;
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

/// Parse git blame porcelain output to extract commit info
fn parse_blame_porcelain(output: &str) -> (String, String, String, String) {
    let mut commit_hash = String::new();
    let mut author = String::new();
    let mut commit_time = String::new();
    let mut summary = String::new();

    for line in output.lines() {
        if commit_hash.is_empty()
            && line.len() >= 40
            && line.chars().take(40).all(|c| c.is_ascii_hexdigit())
        {
            commit_hash = line.split_whitespace().next().unwrap_or("").to_string();
        } else if let Some(rest) = line.strip_prefix("author ") {
            author = rest.to_string();
        } else if let Some(rest) = line.strip_prefix("author-time ") {
            if let Ok(timestamp) = rest.parse::<i64>() {
                commit_time = chrono::DateTime::from_timestamp(timestamp, 0).map_or_else(
                    || "Unknown date".to_string(),
                    |dt| dt.format("%Y-%m-%d %H:%M").to_string(),
                );
            }
        } else if let Some(rest) = line.strip_prefix("summary ") {
            summary = rest.to_string();
        }
    }

    if commit_hash.is_empty() {
        commit_hash = "Unknown".to_string();
    }
    if author.is_empty() {
        author = "Unknown".to_string();
    }
    if commit_time.is_empty() {
        commit_time = "Unknown date".to_string();
    }

    (commit_hash, author, commit_time, summary)
}
