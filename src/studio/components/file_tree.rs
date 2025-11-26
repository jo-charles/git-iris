//! File tree component for Iris Studio
//!
//! Hierarchical file browser with git status indicators.

use ratatui::Frame;
use ratatui::layout::Rect;
use ratatui::style::{Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{
    Block, Borders, Paragraph, Scrollbar, ScrollbarOrientation, ScrollbarState,
};
use std::collections::HashSet;
use std::path::{Path, PathBuf};

use crate::studio::theme;

// ═══════════════════════════════════════════════════════════════════════════════
// Git Status
// ═══════════════════════════════════════════════════════════════════════════════

/// Git status for a file
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum FileGitStatus {
    #[default]
    Normal,
    Staged,
    Modified,
    Untracked,
    Deleted,
    Renamed,
    Conflict,
}

impl FileGitStatus {
    /// Get the indicator character for this status
    pub fn indicator(self) -> &'static str {
        match self {
            Self::Normal => " ",
            Self::Staged => "●",
            Self::Modified => "○",
            Self::Untracked => "?",
            Self::Deleted => "✕",
            Self::Renamed => "→",
            Self::Conflict => "!",
        }
    }

    /// Get the style for this status
    pub fn style(self) -> Style {
        match self {
            Self::Normal => theme::dimmed(),
            Self::Staged => theme::git_staged(),
            Self::Modified => theme::git_modified(),
            Self::Untracked => theme::git_untracked(),
            Self::Deleted => theme::git_deleted(),
            Self::Renamed => theme::git_staged(),
            Self::Conflict => theme::error(),
        }
    }
}

// ═══════════════════════════════════════════════════════════════════════════════
// Tree Node
// ═══════════════════════════════════════════════════════════════════════════════

/// A node in the file tree
#[derive(Debug, Clone)]
pub struct TreeNode {
    /// File or directory name
    pub name: String,
    /// Full path from repository root
    pub path: PathBuf,
    /// Is this a directory?
    pub is_dir: bool,
    /// Git status (for files)
    pub git_status: FileGitStatus,
    /// Depth in tree (for indentation)
    pub depth: usize,
    /// Children (for directories)
    pub children: Vec<TreeNode>,
}

impl TreeNode {
    /// Create a new file node
    pub fn file(name: impl Into<String>, path: impl Into<PathBuf>, depth: usize) -> Self {
        Self {
            name: name.into(),
            path: path.into(),
            is_dir: false,
            git_status: FileGitStatus::Normal,
            depth,
            children: Vec::new(),
        }
    }

    /// Create a new directory node
    pub fn dir(name: impl Into<String>, path: impl Into<PathBuf>, depth: usize) -> Self {
        Self {
            name: name.into(),
            path: path.into(),
            is_dir: true,
            git_status: FileGitStatus::Normal,
            depth,
            children: Vec::new(),
        }
    }

    /// Set git status
    pub fn with_status(mut self, status: FileGitStatus) -> Self {
        self.git_status = status;
        self
    }

    /// Add a child node
    pub fn add_child(&mut self, child: TreeNode) {
        self.children.push(child);
    }

    /// Sort children (directories first, then alphabetically)
    pub fn sort_children(&mut self) {
        self.children.sort_by(|a, b| match (a.is_dir, b.is_dir) {
            (true, false) => std::cmp::Ordering::Less,
            (false, true) => std::cmp::Ordering::Greater,
            _ => a.name.to_lowercase().cmp(&b.name.to_lowercase()),
        });
        for child in &mut self.children {
            child.sort_children();
        }
    }
}

// ═══════════════════════════════════════════════════════════════════════════════
// Flattened Entry (for rendering)
// ═══════════════════════════════════════════════════════════════════════════════

/// A flattened view of the tree for rendering
#[derive(Debug, Clone)]
pub struct FlatEntry {
    pub name: String,
    pub path: PathBuf,
    pub is_dir: bool,
    pub git_status: FileGitStatus,
    pub depth: usize,
    pub is_expanded: bool,
    pub has_children: bool,
}

// ═══════════════════════════════════════════════════════════════════════════════
// File Tree State
// ═══════════════════════════════════════════════════════════════════════════════

/// File tree widget state
#[derive(Debug, Clone)]
pub struct FileTreeState {
    /// Root nodes of the tree
    root: Vec<TreeNode>,
    /// Expanded directories (by path)
    expanded: HashSet<PathBuf>,
    /// Currently selected index in flat view
    selected: usize,
    /// Scroll offset
    scroll_offset: usize,
    /// Cached flat view
    flat_cache: Vec<FlatEntry>,
    /// Cache is dirty flag
    cache_dirty: bool,
}

impl Default for FileTreeState {
    fn default() -> Self {
        Self::new()
    }
}

impl FileTreeState {
    /// Create new empty file tree state
    pub fn new() -> Self {
        Self {
            root: Vec::new(),
            expanded: HashSet::new(),
            selected: 0,
            scroll_offset: 0,
            flat_cache: Vec::new(),
            cache_dirty: true,
        }
    }

    /// Set root nodes
    pub fn set_root(&mut self, root: Vec<TreeNode>) {
        self.root = root;
        self.cache_dirty = true;
        self.selected = 0;
        self.scroll_offset = 0;
    }

    /// Build tree from a list of file paths
    pub fn from_paths(paths: &[PathBuf], git_statuses: &[(PathBuf, FileGitStatus)]) -> Self {
        let mut state = Self::new();
        let mut root_nodes: Vec<TreeNode> = Vec::new();

        // Build status lookup
        let status_map: std::collections::HashMap<_, _> = git_statuses.iter().cloned().collect();

        for path in paths {
            let components: Vec<_> = path.components().collect();
            insert_path(&mut root_nodes, &components, 0, path, &status_map);
        }

        // Sort all nodes
        for node in &mut root_nodes {
            node.sort_children();
        }

        // Sort root level
        root_nodes.sort_by(|a, b| match (a.is_dir, b.is_dir) {
            (true, false) => std::cmp::Ordering::Less,
            (false, true) => std::cmp::Ordering::Greater,
            _ => a.name.to_lowercase().cmp(&b.name.to_lowercase()),
        });

        state.root = root_nodes;
        state.cache_dirty = true;
        // Auto-expand first 2 levels for visibility
        state.expand_to_depth(2);
        state
    }

    /// Get flat view (rebuilds cache if needed)
    pub fn flat_view(&mut self) -> &[FlatEntry] {
        if self.cache_dirty {
            self.rebuild_cache();
        }
        &self.flat_cache
    }

    /// Rebuild the flat cache
    fn rebuild_cache(&mut self) {
        self.flat_cache.clear();
        let root_clone = self.root.clone();
        for node in &root_clone {
            self.flatten_node(node);
        }
        self.cache_dirty = false;
    }

    /// Flatten a node into the cache
    fn flatten_node(&mut self, node: &TreeNode) {
        let is_expanded = self.expanded.contains(&node.path);

        self.flat_cache.push(FlatEntry {
            name: node.name.clone(),
            path: node.path.clone(),
            is_dir: node.is_dir,
            git_status: node.git_status,
            depth: node.depth,
            is_expanded,
            has_children: !node.children.is_empty(),
        });

        if is_expanded {
            let children = node.children.clone();
            for child in &children {
                self.flatten_node(child);
            }
        }
    }

    /// Get selected entry
    pub fn selected_entry(&mut self) -> Option<FlatEntry> {
        self.ensure_cache();
        self.flat_cache.get(self.selected).cloned()
    }

    /// Ensure cache is up to date
    fn ensure_cache(&mut self) {
        if self.cache_dirty {
            self.rebuild_cache();
        }
    }

    /// Get selected path
    pub fn selected_path(&mut self) -> Option<PathBuf> {
        self.selected_entry().map(|e| e.path)
    }

    /// Move selection up
    pub fn select_prev(&mut self) {
        if self.selected > 0 {
            self.selected -= 1;
            self.ensure_visible();
        }
    }

    /// Move selection down
    pub fn select_next(&mut self) {
        let len = self.flat_view().len();
        if self.selected + 1 < len {
            self.selected += 1;
            self.ensure_visible();
        }
    }

    /// Jump to first item
    pub fn select_first(&mut self) {
        self.selected = 0;
        self.scroll_offset = 0;
    }

    /// Jump to last item
    pub fn select_last(&mut self) {
        let len = self.flat_view().len();
        if len > 0 {
            self.selected = len - 1;
        }
    }

    /// Page up
    pub fn page_up(&mut self, page_size: usize) {
        self.selected = self.selected.saturating_sub(page_size);
        self.ensure_visible();
    }

    /// Page down
    pub fn page_down(&mut self, page_size: usize) {
        let len = self.flat_view().len();
        self.selected = (self.selected + page_size).min(len.saturating_sub(1));
        self.ensure_visible();
    }

    /// Toggle expansion of selected item
    pub fn toggle_expand(&mut self) {
        if let Some(entry) = self.selected_entry()
            && entry.is_dir
        {
            if self.expanded.contains(&entry.path) {
                self.expanded.remove(&entry.path);
            } else {
                self.expanded.insert(entry.path);
            }
            self.cache_dirty = true;
        }
    }

    /// Expand selected directory
    pub fn expand(&mut self) {
        if let Some(entry) = self.selected_entry()
            && entry.is_dir
            && !self.expanded.contains(&entry.path)
        {
            self.expanded.insert(entry.path);
            self.cache_dirty = true;
        }
    }

    /// Collapse selected directory (or parent)
    pub fn collapse(&mut self) {
        if let Some(entry) = self.selected_entry() {
            if entry.is_dir && self.expanded.contains(&entry.path) {
                self.expanded.remove(&entry.path);
                self.cache_dirty = true;
            } else if entry.depth > 0 {
                // Find and select parent
                let parent_path = entry.path.parent().map(Path::to_path_buf);
                if let Some(parent) = parent_path {
                    self.expanded.remove(&parent);
                    self.cache_dirty = true;
                    // Find parent in flat view and select it
                    let flat = self.flat_view();
                    for (i, e) in flat.iter().enumerate() {
                        if e.path == parent {
                            self.selected = i;
                            break;
                        }
                    }
                }
            }
        }
    }

    /// Expand all directories
    pub fn expand_all(&mut self) {
        self.expand_all_recursive(&self.root.clone());
        self.cache_dirty = true;
    }

    fn expand_all_recursive(&mut self, nodes: &[TreeNode]) {
        for node in nodes {
            if node.is_dir {
                self.expanded.insert(node.path.clone());
                self.expand_all_recursive(&node.children);
            }
        }
    }

    /// Collapse all directories
    pub fn collapse_all(&mut self) {
        self.expanded.clear();
        self.cache_dirty = true;
        self.selected = 0;
    }

    /// Expand directories up to a certain depth
    pub fn expand_to_depth(&mut self, max_depth: usize) {
        self.expand_to_depth_recursive(&self.root.clone(), 0, max_depth);
        self.cache_dirty = true;
    }

    fn expand_to_depth_recursive(
        &mut self,
        nodes: &[TreeNode],
        current_depth: usize,
        max_depth: usize,
    ) {
        if current_depth >= max_depth {
            return;
        }
        for node in nodes {
            if node.is_dir {
                self.expanded.insert(node.path.clone());
                self.expand_to_depth_recursive(&node.children, current_depth + 1, max_depth);
            }
        }
    }

    /// Ensure selected item is visible (stub for future scroll viewport tracking)
    #[allow(clippy::unused_self)]
    fn ensure_visible(&mut self) {
        // Will be adjusted based on render area height
    }

    /// Update scroll offset based on area height
    pub fn update_scroll(&mut self, visible_height: usize) {
        if visible_height == 0 {
            return;
        }

        // Ensure selected is within scroll view
        if self.selected < self.scroll_offset {
            self.scroll_offset = self.selected;
        } else if self.selected >= self.scroll_offset + visible_height {
            self.scroll_offset = self.selected - visible_height + 1;
        }
    }

    /// Get current scroll offset
    pub fn scroll_offset(&self) -> usize {
        self.scroll_offset
    }

    /// Get selected index
    pub fn selected_index(&self) -> usize {
        self.selected
    }
}

/// Helper to insert a path into the tree structure
fn insert_path(
    nodes: &mut Vec<TreeNode>,
    components: &[std::path::Component<'_>],
    depth: usize,
    full_path: &Path,
    status_map: &std::collections::HashMap<PathBuf, FileGitStatus>,
) {
    if components.is_empty() {
        return;
    }

    let name = components[0].as_os_str().to_string_lossy().to_string();
    let is_last = components.len() == 1;

    // Build path up to this point
    let current_path: PathBuf = components[..1].iter().collect();

    // Find or create node
    let node_idx = nodes.iter().position(|n| n.name == name);

    if is_last {
        // This is a file
        let status = status_map.get(full_path).copied().unwrap_or_default();
        if node_idx.is_none() {
            nodes.push(TreeNode::file(name, full_path, depth).with_status(status));
        }
    } else {
        // This is a directory
        let idx = if let Some(idx) = node_idx {
            idx
        } else {
            nodes.push(TreeNode::dir(name, current_path, depth));
            nodes.len() - 1
        };

        insert_path(
            &mut nodes[idx].children,
            &components[1..],
            depth + 1,
            full_path,
            status_map,
        );
    }
}

// ═══════════════════════════════════════════════════════════════════════════════
// Rendering
// ═══════════════════════════════════════════════════════════════════════════════

/// Render the file tree widget
pub fn render_file_tree(
    frame: &mut Frame,
    area: Rect,
    state: &mut FileTreeState,
    title: &str,
    focused: bool,
) {
    let block = Block::default()
        .title(format!(" {} ", title))
        .borders(Borders::ALL)
        .border_style(if focused {
            theme::focused_border()
        } else {
            theme::unfocused_border()
        });

    let inner = block.inner(area);
    frame.render_widget(block, area);

    if inner.height == 0 || inner.width == 0 {
        return;
    }

    let visible_height = inner.height as usize;
    state.update_scroll(visible_height);

    // Get values we need before borrowing flat view
    let scroll_offset = state.scroll_offset();
    let selected = state.selected_index();

    // Now get the flat view
    let flat = state.flat_view().to_vec(); // Clone to avoid borrow issues
    let flat_len = flat.len();

    let lines: Vec<Line> = flat
        .iter()
        .enumerate()
        .skip(scroll_offset)
        .take(visible_height)
        .map(|(i, entry)| {
            let is_selected = i == selected;
            render_entry(entry, is_selected, inner.width as usize)
        })
        .collect();

    let paragraph = Paragraph::new(lines);
    frame.render_widget(paragraph, inner);

    // Render scrollbar if needed
    if flat_len > visible_height {
        let scrollbar = Scrollbar::new(ScrollbarOrientation::VerticalRight)
            .begin_symbol(None)
            .end_symbol(None);

        let mut scrollbar_state = ScrollbarState::new(flat_len).position(scroll_offset);

        frame.render_stateful_widget(
            scrollbar,
            area.inner(ratatui::layout::Margin {
                vertical: 1,
                horizontal: 0,
            }),
            &mut scrollbar_state,
        );
    }
}

/// Render a single tree entry
fn render_entry(entry: &FlatEntry, is_selected: bool, width: usize) -> Line<'static> {
    let indent = "  ".repeat(entry.depth);

    // Icon
    let icon = if entry.is_dir {
        if entry.is_expanded { "▼" } else { "▶" }
    } else {
        get_file_icon(&entry.name)
    };

    // Git status indicator
    let status_indicator = entry.git_status.indicator();
    let status_style = entry.git_status.style();

    // Selection marker
    let marker = if is_selected { "▶" } else { " " };
    let marker_style = if is_selected {
        Style::default().fg(theme::ELECTRIC_PURPLE)
    } else {
        Style::default()
    };

    // Name style
    let name_style = if is_selected {
        theme::selected()
    } else if entry.is_dir {
        Style::default()
            .fg(theme::NEON_CYAN)
            .add_modifier(Modifier::BOLD)
    } else {
        Style::default().fg(theme::TEXT_PRIMARY)
    };

    // Build the line
    let content = format!("{}{} {} {}", indent, icon, entry.name, status_indicator);
    let display_width = content.chars().count() + 2; // +2 for marker
    let padding = if display_width < width {
        " ".repeat(width - display_width)
    } else {
        String::new()
    };

    Line::from(vec![
        Span::styled(marker, marker_style),
        Span::raw(" "),
        Span::raw(indent),
        Span::styled(
            format!("{} ", icon),
            if entry.is_dir {
                Style::default().fg(theme::NEON_CYAN)
            } else {
                Style::default().fg(theme::TEXT_DIM)
            },
        ),
        Span::styled(entry.name.clone(), name_style),
        Span::raw(" "),
        Span::styled(status_indicator, status_style),
        Span::raw(padding),
    ])
}

/// Get icon for file based on extension
fn get_file_icon(name: &str) -> &'static str {
    let ext = name.rsplit('.').next().unwrap_or("");
    match ext.to_lowercase().as_str() {
        "rs" => "◆",
        "toml" => "◇",
        "md" => "▪",
        "json" => "◈",
        "yaml" | "yml" => "◈",
        "js" | "jsx" => "◎",
        "ts" | "tsx" => "◉",
        "py" => "○",
        "go" => "●",
        "sh" | "bash" | "zsh" => "▸",
        "lock" => "◌",
        "gitignore" => "◦",
        "dockerfile" => "▣",
        _ => "·",
    }
}
