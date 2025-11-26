//! Layout management for Iris Studio
//!
//! Panel layout calculations and constraints.

#![allow(dead_code)] // Layout helpers are scaffolded for responsive features

use ratatui::layout::{Constraint, Direction, Layout, Rect};

use super::state::{Mode, PanelId};

// ═══════════════════════════════════════════════════════════════════════════════
// Layout Configuration
// ═══════════════════════════════════════════════════════════════════════════════

/// Panel configuration for layout
#[derive(Debug, Clone)]
pub struct PanelConfig {
    /// Panel identifier
    pub id: PanelId,
    /// Panel title
    pub title: &'static str,
    /// Whether the panel can receive focus
    pub focusable: bool,
    /// Minimum width in characters
    pub min_width: u16,
}

/// Layout definition for a mode
#[derive(Debug, Clone)]
pub struct ModeLayout {
    /// Panel configurations (left to right)
    pub panels: Vec<PanelConfig>,
    /// Width constraints for each panel
    pub constraints: Vec<Constraint>,
}

impl ModeLayout {
    /// Get panel config by ID
    pub fn get_panel(&self, id: PanelId) -> Option<&PanelConfig> {
        self.panels.iter().find(|p| p.id == id)
    }
}

// ═══════════════════════════════════════════════════════════════════════════════
// Mode Layouts
// ═══════════════════════════════════════════════════════════════════════════════

/// Get the layout for a specific mode
pub fn get_mode_layout(mode: Mode) -> ModeLayout {
    match mode {
        Mode::Explore => explore_layout(),
        Mode::Commit => commit_layout(),
        Mode::Review => review_layout(),
        Mode::PR => pr_layout(),
        Mode::Changelog => changelog_layout(),
    }
}

fn explore_layout() -> ModeLayout {
    ModeLayout {
        panels: vec![
            PanelConfig {
                id: PanelId::Left,
                title: "Files",
                focusable: true,
                min_width: 20,
            },
            PanelConfig {
                id: PanelId::Center,
                title: "Code",
                focusable: true,
                min_width: 40,
            },
            PanelConfig {
                id: PanelId::Right,
                title: "Context",
                focusable: true,
                min_width: 25,
            },
        ],
        constraints: vec![
            Constraint::Percentage(20),
            Constraint::Percentage(50),
            Constraint::Percentage(30),
        ],
    }
}

fn commit_layout() -> ModeLayout {
    ModeLayout {
        panels: vec![
            PanelConfig {
                id: PanelId::Left,
                title: "Staged",
                focusable: true,
                min_width: 20,
            },
            PanelConfig {
                id: PanelId::Center,
                title: "Message",
                focusable: true,
                min_width: 40,
            },
            PanelConfig {
                id: PanelId::Right,
                title: "Diff",
                focusable: true,
                min_width: 25,
            },
        ],
        constraints: vec![
            Constraint::Percentage(18),
            Constraint::Percentage(42),
            Constraint::Percentage(40),
        ],
    }
}

fn review_layout() -> ModeLayout {
    ModeLayout {
        panels: vec![
            PanelConfig {
                id: PanelId::Left,
                title: "Files",
                focusable: true,
                min_width: 20,
            },
            PanelConfig {
                id: PanelId::Center,
                title: "Review",
                focusable: true,
                min_width: 40,
            },
            PanelConfig {
                id: PanelId::Right,
                title: "Diff",
                focusable: true,
                min_width: 25,
            },
        ],
        constraints: vec![
            Constraint::Percentage(18),
            Constraint::Percentage(42),
            Constraint::Percentage(40),
        ],
    }
}

fn pr_layout() -> ModeLayout {
    ModeLayout {
        panels: vec![
            PanelConfig {
                id: PanelId::Left,
                title: "Commits",
                focusable: true,
                min_width: 20,
            },
            PanelConfig {
                id: PanelId::Center,
                title: "PR",
                focusable: true,
                min_width: 40,
            },
            PanelConfig {
                id: PanelId::Right,
                title: "Diff",
                focusable: true,
                min_width: 25,
            },
        ],
        constraints: vec![
            Constraint::Percentage(18),
            Constraint::Percentage(42),
            Constraint::Percentage(40),
        ],
    }
}

fn changelog_layout() -> ModeLayout {
    ModeLayout {
        panels: vec![
            PanelConfig {
                id: PanelId::Left,
                title: "Range",
                focusable: true,
                min_width: 15,
            },
            PanelConfig {
                id: PanelId::Center,
                title: "Changes",
                focusable: true,
                min_width: 35,
            },
            PanelConfig {
                id: PanelId::Right,
                title: "Changelog",
                focusable: true,
                min_width: 30,
            },
        ],
        constraints: vec![
            Constraint::Percentage(15),
            Constraint::Percentage(40),
            Constraint::Percentage(45),
        ],
    }
}

// ═══════════════════════════════════════════════════════════════════════════════
// Layout Calculation
// ═══════════════════════════════════════════════════════════════════════════════

/// Calculated layout areas
#[derive(Debug, Clone)]
pub struct LayoutAreas {
    /// Header area (title bar)
    pub header: Rect,
    /// Mode tabs area
    pub tabs: Rect,
    /// Main content area (panels)
    pub content: Rect,
    /// Individual panel areas
    pub panels: Vec<Rect>,
    /// Status bar area
    pub status: Rect,
}

/// Calculate layout areas for the given terminal size
pub fn calculate_layout(area: Rect, mode: Mode) -> LayoutAreas {
    // Main vertical split: header, tabs, content, status
    let main_chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(1), // Header
            Constraint::Length(2), // Tabs
            Constraint::Min(10),   // Content
            Constraint::Length(1), // Status
        ])
        .split(area);

    let header = main_chunks[0];
    let tabs = main_chunks[1];
    let content = main_chunks[2];
    let status = main_chunks[3];

    // Get mode-specific layout
    let mode_layout = get_mode_layout(mode);

    // Split content into panels
    let panel_chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints(mode_layout.constraints.clone())
        .split(content);

    let panels = panel_chunks.to_vec();

    LayoutAreas {
        header,
        tabs,
        content,
        panels,
        status,
    }
}

/// Calculate inner area for panel content (with border padding)
pub fn panel_inner(area: Rect) -> Rect {
    // Account for border (1 char each side)
    Rect {
        x: area.x.saturating_add(1),
        y: area.y.saturating_add(1),
        width: area.width.saturating_sub(2),
        height: area.height.saturating_sub(2),
    }
}

// ═══════════════════════════════════════════════════════════════════════════════
// Responsive Helpers
// ═══════════════════════════════════════════════════════════════════════════════

/// Check if terminal is wide enough for three-panel layout
pub fn is_wide_layout(width: u16) -> bool {
    width >= 100
}

/// Check if terminal is too narrow for comfortable use
pub fn is_narrow_layout(width: u16) -> bool {
    width < 80
}

/// Get appropriate layout constraints based on terminal width
pub fn responsive_constraints(width: u16, mode: Mode) -> Vec<Constraint> {
    let default = get_mode_layout(mode).constraints;

    if is_narrow_layout(width) {
        // Narrow: hide right panel, expand center
        vec![
            Constraint::Percentage(25),
            Constraint::Percentage(75),
            Constraint::Length(0),
        ]
    } else if !is_wide_layout(width) {
        // Medium: slightly different ratios
        vec![
            Constraint::Percentage(22),
            Constraint::Percentage(48),
            Constraint::Percentage(30),
        ]
    } else {
        default
    }
}
