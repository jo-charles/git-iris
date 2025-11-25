//! `SilkCircuit` TUI — Electric meets elegant
//!
//! A vibrant, modern TUI design using the full `SilkCircuit` Neon color palette.

use super::state::{EmojiMode, Mode, TuiState, UserInfoFocus};
use crate::ui::{
    BRIGHT_CYAN, CORAL, DEEP_PURPLE, DIM_GRAY, ELECTRIC_PURPLE, ELECTRIC_YELLOW, ERROR_RED,
    HIGHLIGHT, NEON_CYAN, PURE_PINK, SOFT_PINK, SOFT_WHITE, SUCCESS_GREEN, VOID,
};
use ratatui::{
    Frame,
    layout::{Alignment, Constraint, Direction, Layout, Margin, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Clear, List, ListItem, Padding, Paragraph, Wrap},
};
use unicode_width::UnicodeWidthStr;

const APP_VERSION: &str = env!("CARGO_PKG_VERSION");

// ═══════════════════════════════════════════════════════════════════════════════
// Box drawing characters for `SilkCircuit` aesthetic
// ═══════════════════════════════════════════════════════════════════════════════

const SEPARATOR_CHAR: &str = "─";
const DOT: &str = "•";

// ═══════════════════════════════════════════════════════════════════════════════
// Main UI rendering
// ═══════════════════════════════════════════════════════════════════════════════

pub fn draw_ui(f: &mut Frame, state: &mut TuiState) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .margin(1)
        .constraints([
            Constraint::Length(3), // Header (title + nav)
            Constraint::Length(1), // Gradient separator
            Constraint::Min(8),    // Main content area
            Constraint::Length(1), // Gradient separator
            Constraint::Length(3), // Footer (status + info)
        ])
        .split(f.area());

    draw_header(f, state, chunks[0]);
    draw_gradient_separator(f, chunks[1]);
    draw_main_content(f, state, chunks[2]);
    draw_gradient_separator(f, chunks[3]);
    draw_footer(f, state, chunks[4]);

    // Render popups on top
    match state.mode {
        Mode::SelectingEmoji => draw_emoji_popup(f, state),
        Mode::SelectingPreset => draw_preset_popup(f, state),
        Mode::EditingUserInfo => draw_user_info_popup(f, state),
        _ => {}
    }
}

// ═══════════════════════════════════════════════════════════════════════════════
// Header: Title + Navigation
// ═══════════════════════════════════════════════════════════════════════════════

fn draw_header(f: &mut Frame, state: &TuiState, area: Rect) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Length(1), Constraint::Length(2)])
        .split(area);

    draw_title(f, chunks[0]);
    draw_nav_bar(f, state, chunks[1]);
}

fn draw_title(f: &mut Frame, area: Rect) {
    // SilkCircuit gradient: Electric Purple -> Pure Pink -> Neon Cyan
    let gradient_colors = [
        ELECTRIC_PURPLE,
        Color::Rgb(243, 27, 255), // Purple-pink
        PURE_PINK,
        Color::Rgb(191, 128, 255), // Pink-cyan blend
        NEON_CYAN,
    ];

    let title_text = format!(" Iris v{APP_VERSION} ");

    // Build gradient title
    let text_chars: Vec<char> = title_text.chars().collect();
    let text_len = text_chars.len();
    let gradient_len = gradient_colors.len();

    let gradient_spans: Vec<Span> = text_chars
        .iter()
        .enumerate()
        .map(|(i, &c)| {
            let color_idx = if text_len > 1 {
                i * (gradient_len - 1) / (text_len - 1)
            } else {
                0
            };
            Span::styled(
                c.to_string(),
                Style::default()
                    .fg(gradient_colors[color_idx])
                    .add_modifier(Modifier::BOLD),
            )
        })
        .collect();

    let mut title_line = vec![
        Span::styled("", Style::default().fg(PURE_PINK)),
        Span::styled(" ", Style::default()),
    ];
    title_line.extend(gradient_spans);
    title_line.push(Span::styled(" ", Style::default().fg(ELECTRIC_PURPLE)));
    title_line.push(Span::styled(
        " electric commits",
        Style::default()
            .fg(PURPLE_MUTED)
            .add_modifier(Modifier::ITALIC),
    ));

    let title = Paragraph::new(Line::from(title_line)).alignment(Alignment::Center);
    f.render_widget(title, area);
}

// Purple muted color for inline use
const PURPLE_MUTED: Color = Color::Rgb(98, 114, 164);

fn draw_nav_bar(f: &mut Frame, state: &TuiState, area: Rect) {
    // Electric keybinds with vibrant colors
    let nav_items: Vec<(&str, &str, Color, bool)> = vec![
        ("◀▶", "nav", NEON_CYAN, false),
        ("e", "edit", BRIGHT_CYAN, state.mode == Mode::EditingMessage),
        (
            "i",
            "instr",
            DEEP_PURPLE,
            state.mode == Mode::EditingInstructions,
        ),
        (
            "g",
            "emoji",
            ELECTRIC_YELLOW,
            state.mode == Mode::SelectingEmoji,
        ),
        ("p", "preset", CORAL, state.mode == Mode::SelectingPreset),
        (
            "u",
            "user",
            SUCCESS_GREEN,
            state.mode == Mode::EditingUserInfo,
        ),
        ("r", "regen", PURE_PINK, state.mode == Mode::Generating),
        ("⏎", "commit", SUCCESS_GREEN, false),
        ("esc", "exit", ERROR_RED, false),
    ];

    let nav_spans: Vec<Span> = nav_items
        .into_iter()
        .flat_map(|(key, desc, color, active)| {
            let (key_style, sep_style) = if active {
                (
                    Style::default()
                        .fg(VOID)
                        .bg(color)
                        .add_modifier(Modifier::BOLD),
                    Style::default().fg(color),
                )
            } else {
                (
                    Style::default().fg(color).add_modifier(Modifier::BOLD),
                    Style::default().fg(DIM_GRAY),
                )
            };

            vec![
                Span::styled(format!(" {key}"), key_style),
                Span::styled(":", sep_style),
                Span::styled(format!("{desc} "), Style::default().fg(SOFT_WHITE)),
            ]
        })
        .collect();

    let nav_bar = Paragraph::new(Line::from(nav_spans)).alignment(Alignment::Center);
    f.render_widget(nav_bar, area);
}

// ═══════════════════════════════════════════════════════════════════════════════
// Main Content: Commit Message + Instructions
// ═══════════════════════════════════════════════════════════════════════════════

fn draw_main_content(f: &mut Frame, state: &mut TuiState, area: Rect) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Min(5),    // Commit message
            Constraint::Length(5), // Instructions
        ])
        .split(area);

    draw_commit_message(f, state, chunks[0]);
    draw_instructions(f, state, chunks[1]);
}

fn draw_commit_message(f: &mut Frame, state: &mut TuiState, area: Rect) {
    let is_editing = state.mode == Mode::EditingMessage;

    // Build colorful title with counter and emoji indicator
    let title_spans = vec![
        Span::styled(" ", Style::default().fg(NEON_CYAN)),
        Span::styled(
            " commit ",
            Style::default()
                .fg(if is_editing { SOFT_WHITE } else { NEON_CYAN })
                .add_modifier(Modifier::BOLD),
        ),
        Span::styled(
            format!("{}", state.current_index + 1),
            Style::default().fg(PURE_PINK).add_modifier(Modifier::BOLD),
        ),
        Span::styled("/", Style::default().fg(DIM_GRAY)),
        Span::styled(
            format!("{}", state.messages.len()),
            Style::default().fg(CORAL),
        ),
        Span::styled(
            get_emoji_indicator(state),
            Style::default().fg(ELECTRIC_YELLOW),
        ),
        Span::styled(" ", Style::default()),
    ];

    let border_color = if is_editing { NEON_CYAN } else { DEEP_PURPLE };

    let message_block = Block::default()
        .borders(Borders::ALL)
        .border_style(Style::default().fg(border_color))
        .border_type(ratatui::widgets::BorderType::Rounded)
        .title(Line::from(title_spans))
        .padding(Padding::horizontal(1));

    if is_editing {
        state.message_textarea.set_block(message_block);
        state
            .message_textarea
            .set_style(Style::default().fg(SOFT_WHITE));
        state
            .message_textarea
            .set_cursor_style(Style::default().bg(PURE_PINK).fg(VOID));
        f.render_widget(&state.message_textarea, area);
    } else {
        let current_message = &state.messages[state.current_index];

        // Build colorful commit message display
        let emoji_prefix = state
            .get_current_emoji()
            .map_or(String::new(), |e| format!("{e} "));

        // Title line in bright cyan, body in soft white
        let mut lines: Vec<Line> = Vec::new();

        // Title with emoji - bright and bold
        lines.push(Line::from(vec![
            Span::styled(&emoji_prefix, Style::default().fg(ELECTRIC_YELLOW)),
            Span::styled(
                &current_message.title,
                Style::default()
                    .fg(BRIGHT_CYAN)
                    .add_modifier(Modifier::BOLD),
            ),
        ]));

        // Empty line
        lines.push(Line::from(""));

        // Body text in softer color
        for line in current_message.message.lines() {
            lines.push(Line::from(Span::styled(
                line,
                Style::default().fg(SOFT_WHITE),
            )));
        }

        let message = Paragraph::new(lines)
            .block(message_block)
            .wrap(Wrap { trim: true });
        f.render_widget(message, area);
    }
}

fn draw_instructions(f: &mut Frame, state: &mut TuiState, area: Rect) {
    let is_editing = state.mode == Mode::EditingInstructions;

    let title_spans = vec![
        Span::styled(" ", Style::default().fg(DEEP_PURPLE)),
        Span::styled(
            " instructions ",
            Style::default()
                .fg(if is_editing { SOFT_WHITE } else { DEEP_PURPLE })
                .add_modifier(Modifier::BOLD),
        ),
    ];

    let border_color = if is_editing {
        ELECTRIC_PURPLE
    } else {
        DIM_GRAY
    };

    let instructions_block = Block::default()
        .borders(Borders::ALL)
        .border_style(Style::default().fg(border_color))
        .border_type(ratatui::widgets::BorderType::Rounded)
        .title(Line::from(title_spans))
        .padding(Padding::horizontal(1));

    if is_editing {
        state.instructions_textarea.set_block(instructions_block);
        state
            .instructions_textarea
            .set_style(Style::default().fg(SOFT_WHITE));
        state
            .instructions_textarea
            .set_cursor_style(Style::default().bg(ELECTRIC_PURPLE).fg(SOFT_WHITE));
        f.render_widget(&state.instructions_textarea, area);
    } else {
        let display_text = if state.custom_instructions.is_empty() {
            "press i to add custom instructions...".to_string()
        } else {
            state.custom_instructions.clone()
        };

        let style = if state.custom_instructions.is_empty() {
            Style::default().fg(DIM_GRAY).add_modifier(Modifier::ITALIC)
        } else {
            Style::default().fg(SOFT_PINK)
        };

        let instructions = Paragraph::new(display_text)
            .block(instructions_block)
            .style(style)
            .wrap(Wrap { trim: true });
        f.render_widget(instructions, area);
    }
}

// ═══════════════════════════════════════════════════════════════════════════════
// Footer: Status + Info
// ═══════════════════════════════════════════════════════════════════════════════

fn draw_footer(f: &mut Frame, state: &mut TuiState, area: Rect) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Length(1), Constraint::Length(2)])
        .split(area);

    draw_info_bar(f, state, chunks[0]);
    draw_status(f, state, chunks[1]);
}

fn draw_info_bar(f: &mut Frame, state: &TuiState, area: Rect) {
    let preset_display = state.get_selected_preset_name_with_emoji();
    let emoji_display = match &state.emoji_mode {
        EmojiMode::None => "off".to_string(),
        EmojiMode::Auto => "auto".to_string(),
        EmojiMode::Custom(emoji) => emoji.clone(),
    };

    // Vibrant info bar with colored icons and labels
    let info_spans = vec![
        // User name
        Span::styled(" ", Style::default().fg(SUCCESS_GREEN)),
        Span::styled(
            format!(" {}", state.user_name),
            Style::default().fg(SUCCESS_GREEN),
        ),
        Span::styled(format!(" {DOT} "), Style::default().fg(DIM_GRAY)),
        // Email
        Span::styled(" ", Style::default().fg(CORAL)),
        Span::styled(format!(" {}", state.user_email), Style::default().fg(CORAL)),
        Span::styled(format!(" {DOT} "), Style::default().fg(DIM_GRAY)),
        // Emoji mode
        Span::styled(" ", Style::default().fg(ELECTRIC_YELLOW)),
        Span::styled(" emoji:", Style::default().fg(DIM_GRAY)),
        Span::styled(
            emoji_display,
            Style::default()
                .fg(ELECTRIC_YELLOW)
                .add_modifier(Modifier::BOLD),
        ),
        Span::styled(format!(" {DOT} "), Style::default().fg(DIM_GRAY)),
        // Preset
        Span::styled(" ", Style::default().fg(PURE_PINK)),
        Span::styled(format!(" {preset_display}"), Style::default().fg(PURE_PINK)),
    ];

    let info_bar = Paragraph::new(Line::from(info_spans)).alignment(Alignment::Center);
    f.render_widget(info_bar, area);
}

pub fn draw_status(f: &mut Frame, state: &mut TuiState, area: Rect) {
    let (spinner_with_space, status_content, color, content_width) =
        if let Some(spinner) = &mut state.spinner {
            spinner.tick()
        } else {
            (
                "  ".to_string(),
                state.status.clone(),
                SUCCESS_GREEN,
                state.status.width() + 2,
            )
        };

    #[allow(clippy::as_conversions)]
    let terminal_width = f.area().width as usize;

    let left_padding = if content_width >= terminal_width {
        0
    } else {
        (terminal_width - content_width) / 2
    };
    let right_padding = if content_width >= terminal_width {
        0
    } else {
        terminal_width - content_width - left_padding
    };

    // Spinner in electric purple, status in its designated color
    let status_line = Line::from(vec![
        Span::raw(" ".repeat(left_padding)),
        Span::styled(
            spinner_with_space,
            Style::default()
                .fg(ELECTRIC_PURPLE)
                .add_modifier(Modifier::BOLD),
        ),
        Span::styled(status_content, Style::default().fg(color)),
        Span::raw(" ".repeat(right_padding)),
    ]);

    let status_widget = Paragraph::new(vec![status_line]).alignment(Alignment::Left);
    f.render_widget(Clear, area);
    f.render_widget(status_widget, area);
}

// ═══════════════════════════════════════════════════════════════════════════════
// Visual Elements
// ═══════════════════════════════════════════════════════════════════════════════

/// Draw a gradient separator line (purple -> cyan)
fn draw_gradient_separator(f: &mut Frame, area: Rect) {
    #[allow(clippy::as_conversions)]
    let width = area.width as usize;

    // Create gradient from Electric Purple -> Pure Pink -> Neon Cyan
    let gradient_colors = [
        Color::Rgb(225, 53, 255),  // Electric Purple
        Color::Rgb(243, 27, 255),  // Purple-pink
        Color::Rgb(255, 0, 255),   // Pure Pink
        Color::Rgb(191, 128, 255), // Pink-cyan
        Color::Rgb(128, 255, 234), // Neon Cyan
    ];

    let spans: Vec<Span> = (0..width)
        .map(|i| {
            let color_idx = if width > 1 {
                i * (gradient_colors.len() - 1) / (width - 1)
            } else {
                0
            };
            Span::styled(
                SEPARATOR_CHAR,
                Style::default().fg(gradient_colors[color_idx]),
            )
        })
        .collect();

    let separator = Paragraph::new(Line::from(spans));
    f.render_widget(separator, area);
}

fn get_emoji_indicator(state: &TuiState) -> String {
    match &state.emoji_mode {
        EmojiMode::None => String::new(),
        EmojiMode::Auto => " ".to_string(),
        EmojiMode::Custom(e) => format!(" {e}"),
    }
}

// ═══════════════════════════════════════════════════════════════════════════════
// Popups — Floating panels with electric `SilkCircuit` styling
// ═══════════════════════════════════════════════════════════════════════════════

fn draw_emoji_popup(f: &mut Frame, state: &mut TuiState) {
    let area = centered_rect(50, 60, f.area());

    let popup_block = Block::default()
        .title(Line::from(vec![
            Span::styled(" ", Style::default().fg(ELECTRIC_YELLOW)),
            Span::styled(
                " select emoji ",
                Style::default()
                    .fg(ELECTRIC_YELLOW)
                    .add_modifier(Modifier::BOLD),
            ),
        ]))
        .borders(Borders::ALL)
        .border_style(Style::default().fg(ELECTRIC_YELLOW))
        .border_type(ratatui::widgets::BorderType::Rounded)
        .style(Style::default().bg(HIGHLIGHT));

    let items: Vec<ListItem> = state
        .emoji_list
        .iter()
        .map(|(emoji, description)| {
            ListItem::new(Line::from(vec![
                Span::styled(format!(" {emoji} "), Style::default().fg(ELECTRIC_YELLOW)),
                Span::styled(description, Style::default().fg(SOFT_WHITE)),
            ]))
        })
        .collect();

    let list = List::new(items).block(popup_block).highlight_style(
        Style::default()
            .bg(ELECTRIC_YELLOW)
            .fg(VOID)
            .add_modifier(Modifier::BOLD),
    );

    f.render_widget(Clear, area);
    f.render_stateful_widget(list, area, &mut state.emoji_list_state);
}

fn draw_preset_popup(f: &mut Frame, state: &mut TuiState) {
    let area = centered_rect(70, 70, f.area());

    let popup_block = Block::default()
        .title(Line::from(vec![
            Span::styled(" ", Style::default().fg(CORAL)),
            Span::styled(
                " select preset ",
                Style::default().fg(CORAL).add_modifier(Modifier::BOLD),
            ),
        ]))
        .borders(Borders::ALL)
        .border_style(Style::default().fg(CORAL))
        .border_type(ratatui::widgets::BorderType::Rounded)
        .style(Style::default().bg(HIGHLIGHT));

    let items: Vec<ListItem> = state
        .preset_list
        .iter()
        .map(|(_, emoji, name, description)| {
            ListItem::new(Line::from(vec![
                Span::styled(format!(" {emoji} "), Style::default().fg(ELECTRIC_YELLOW)),
                Span::styled(
                    format!("{name} "),
                    Style::default().fg(NEON_CYAN).add_modifier(Modifier::BOLD),
                ),
                Span::styled(description, Style::default().fg(DIM_GRAY)),
            ]))
        })
        .collect();

    let list = List::new(items).block(popup_block).highlight_style(
        Style::default()
            .bg(CORAL)
            .fg(VOID)
            .add_modifier(Modifier::BOLD),
    );

    f.render_widget(Clear, area);
    f.render_stateful_widget(list, area, &mut state.preset_list_state);
}

fn draw_user_info_popup(f: &mut Frame, state: &mut TuiState) {
    let area = centered_rect(50, 30, f.area());

    let popup_block = Block::default()
        .title(Line::from(vec![
            Span::styled(" ", Style::default().fg(SUCCESS_GREEN)),
            Span::styled(
                " edit user info ",
                Style::default()
                    .fg(SUCCESS_GREEN)
                    .add_modifier(Modifier::BOLD),
            ),
        ]))
        .borders(Borders::ALL)
        .border_style(Style::default().fg(SUCCESS_GREEN))
        .border_type(ratatui::widgets::BorderType::Rounded)
        .style(Style::default().bg(HIGHLIGHT));

    let inner_area = area.inner(Margin::new(1, 1));

    let popup_chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Length(3), Constraint::Length(3)])
        .split(inner_area);

    let name_active = state.user_info_focus == UserInfoFocus::Name;
    let email_active = state.user_info_focus == UserInfoFocus::Email;

    state.user_name_textarea.set_block(
        Block::default()
            .borders(Borders::ALL)
            .border_style(Style::default().fg(if name_active { NEON_CYAN } else { DIM_GRAY }))
            .border_type(ratatui::widgets::BorderType::Rounded)
            .title(Span::styled(
                "  name ",
                Style::default().fg(if name_active { NEON_CYAN } else { DIM_GRAY }),
            )),
    );

    state.user_email_textarea.set_block(
        Block::default()
            .borders(Borders::ALL)
            .border_style(Style::default().fg(if email_active { CORAL } else { DIM_GRAY }))
            .border_type(ratatui::widgets::BorderType::Rounded)
            .title(Span::styled(
                "  email ",
                Style::default().fg(if email_active { CORAL } else { DIM_GRAY }),
            )),
    );

    if name_active {
        state
            .user_name_textarea
            .set_cursor_style(Style::default().bg(NEON_CYAN).fg(VOID));
    }
    if email_active {
        state
            .user_email_textarea
            .set_cursor_style(Style::default().bg(CORAL).fg(VOID));
    }

    f.render_widget(Clear, area);
    f.render_widget(popup_block, area);
    f.render_widget(&state.user_name_textarea, popup_chunks[0]);
    f.render_widget(&state.user_email_textarea, popup_chunks[1]);
}

// ═══════════════════════════════════════════════════════════════════════════════
// Layout Helpers
// ═══════════════════════════════════════════════════════════════════════════════

/// Create a centered rectangle with given percent width and height
fn centered_rect(percent_x: u16, percent_y: u16, r: Rect) -> Rect {
    let popup_layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Percentage((100 - percent_y) / 2),
            Constraint::Percentage(percent_y),
            Constraint::Percentage((100 - percent_y) / 2),
        ])
        .split(r);

    Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage((100 - percent_x) / 2),
            Constraint::Percentage(percent_x),
            Constraint::Percentage((100 - percent_x) / 2),
        ])
        .split(popup_layout[1])[1]
}
