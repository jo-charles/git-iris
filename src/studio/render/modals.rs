//! Modal rendering for Iris Studio

use ratatui::Frame;
use ratatui::layout::Rect;
use ratatui::style::{Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, Clear, Paragraph};
use std::time::Instant;

use crate::studio::state::{
    ChatRole, EmojiInfo, Modal, PresetInfo, RefSelectorTarget, SettingsField, SettingsState,
    StudioState,
};
use crate::studio::theme;

/// Render the currently active modal, if any
pub fn render_modal(state: &StudioState, frame: &mut Frame, last_render: Instant) {
    let Some(modal) = &state.modal else {
        return;
    };
    let area = frame.area();

    // Center modal in screen - chat is larger
    let (modal_width, modal_height) = match modal {
        Modal::Chat(_) => (
            80.min(area.width.saturating_sub(4)),
            (area.height * 3 / 4).min(area.height.saturating_sub(4)),
        ),
        Modal::Help => (70.min(area.width.saturating_sub(4)), 30),
        Modal::Instructions { .. } => (60.min(area.width.saturating_sub(4)), 8),
        Modal::Search { .. } => (60.min(area.width.saturating_sub(4)), 15),
        Modal::Confirm { .. } => (60.min(area.width.saturating_sub(4)), 6),
        Modal::RefSelector { .. } => (50.min(area.width.saturating_sub(4)), 15),
        Modal::PresetSelector { .. } => (70.min(area.width.saturating_sub(4)), 24),
        Modal::EmojiSelector { .. } => (55.min(area.width.saturating_sub(4)), 24),
        Modal::Settings(_) => (60.min(area.width.saturating_sub(4)), 18),
    };
    let modal_height = modal_height.min(area.height.saturating_sub(4));

    let x = (area.width.saturating_sub(modal_width)) / 2;
    let y = (area.height.saturating_sub(modal_height)) / 2;
    let modal_area = Rect::new(x, y, modal_width, modal_height);

    // Clear the area first
    frame.render_widget(Clear, modal_area);

    match modal {
        Modal::Help => render_help(frame, modal_area),
        Modal::Instructions { input } => render_instructions(frame, modal_area, input),
        Modal::Search {
            query,
            results,
            selected,
        } => {
            render_search(frame, modal_area, query, results, *selected);
        }
        Modal::Confirm { message, .. } => render_confirm(frame, modal_area, message),
        Modal::Chat(chat_state) => render_chat(frame, modal_area, chat_state, last_render),
        Modal::RefSelector {
            input,
            refs,
            selected,
            target,
        } => render_ref_selector(frame, modal_area, input, refs, *selected, *target),
        Modal::PresetSelector {
            input,
            presets,
            selected,
            scroll,
        } => render_preset_selector(frame, modal_area, input, presets, *selected, *scroll),
        Modal::EmojiSelector {
            input,
            emojis,
            selected,
            scroll,
        } => render_emoji_selector(frame, modal_area, input, emojis, *selected, *scroll),
        Modal::Settings(settings_state) => render_settings(frame, modal_area, settings_state),
    }
}

fn render_help(frame: &mut Frame, area: Rect) {
    let block = Block::default()
        .title(" Help ")
        .borders(Borders::ALL)
        .border_style(Style::default().fg(theme::ELECTRIC_PURPLE));
    let inner = block.inner(area);
    frame.render_widget(block, area);

    let help_text = vec![
        Line::from(Span::styled(
            "Global",
            Style::default()
                .fg(theme::NEON_CYAN)
                .add_modifier(Modifier::BOLD),
        )),
        Line::from("  q          Quit                 /   Chat with Iris"),
        Line::from("  ?          This help            Tab Next panel"),
        Line::from("  Shift+S    Settings             Shift+E  Explore mode"),
        Line::from("  Shift+C    Commit mode          Shift+R  Review mode"),
        Line::from("  Shift+P    PR mode              Shift+L  Changelog mode"),
        Line::from(""),
        Line::from(Span::styled(
            "Navigation (all modes)",
            Style::default()
                .fg(theme::NEON_CYAN)
                .add_modifier(Modifier::BOLD),
        )),
        Line::from("  j/k        Down/up              g/G  Top/bottom"),
        Line::from("  h/l        Collapse/expand      Enter Select"),
        Line::from(""),
        Line::from(Span::styled(
            "Commit Mode",
            Style::default()
                .fg(theme::NEON_CYAN)
                .add_modifier(Modifier::BOLD),
        )),
        Line::from("  r          Generate message     i   With instructions"),
        Line::from("  e          Edit message         n/p Cycle alternatives"),
        Line::from("  p          Select preset        g   Select emoji"),
        Line::from("  E          Toggle emoji         y   Copy message"),
        Line::from("  Enter      Commit changes"),
        Line::from(""),
        Line::from(Span::styled(
            "Review / PR / Changelog",
            Style::default()
                .fg(theme::NEON_CYAN)
                .add_modifier(Modifier::BOLD),
        )),
        Line::from("  f          Select from ref      t   Select to ref"),
        Line::from("  r          Generate             R   Reset"),
        Line::from(""),
        Line::from(Span::styled("Press any key to close", theme::dimmed())),
    ];
    let paragraph = Paragraph::new(help_text);
    frame.render_widget(paragraph, inner);
}

fn render_instructions(frame: &mut Frame, area: Rect, input: &str) {
    let block = Block::default()
        .title(" Instructions for Iris ")
        .borders(Borders::ALL)
        .border_style(Style::default().fg(theme::NEON_CYAN));
    let inner = block.inner(area);
    frame.render_widget(block, area);

    let lines = vec![
        Line::from(Span::styled(
            "Enter instructions for commit message generation:",
            theme::dimmed(),
        )),
        Line::from(""),
        Line::from(vec![
            Span::styled("> ", Style::default().fg(theme::ELECTRIC_PURPLE)),
            Span::styled(input, Style::default().fg(theme::TEXT_PRIMARY)),
            Span::styled("█", Style::default().fg(theme::NEON_CYAN)),
        ]),
        Line::from(""),
        Line::from(Span::styled(
            "Press Enter to generate, Esc to cancel",
            theme::dimmed(),
        )),
    ];
    let paragraph = Paragraph::new(lines);
    frame.render_widget(paragraph, inner);
}

fn render_search(frame: &mut Frame, area: Rect, query: &str, results: &[String], selected: usize) {
    let block = Block::default()
        .title(" Search Files ")
        .borders(Borders::ALL)
        .border_style(Style::default().fg(theme::NEON_CYAN));
    let inner = block.inner(area);
    frame.render_widget(block, area);

    // Filter results by query
    let filtered: Vec<_> = results
        .iter()
        .filter(|r| query.is_empty() || r.to_lowercase().contains(&query.to_lowercase()))
        .collect();

    let visible_height = inner.height.saturating_sub(4) as usize;

    let mut lines = vec![
        Line::from(vec![
            Span::styled("Filter: ", theme::dimmed()),
            Span::styled(query, Style::default().fg(theme::TEXT_PRIMARY)),
            Span::styled("█", Style::default().fg(theme::NEON_CYAN)),
        ]),
        Line::from(""),
    ];

    if filtered.is_empty() {
        lines.push(Line::from(Span::styled(
            if results.is_empty() {
                "No files in current mode"
            } else {
                "No matching files"
            },
            theme::dimmed(),
        )));
    } else {
        // Calculate scroll offset to keep selection visible
        let scroll = if selected >= visible_height {
            selected - visible_height + 1
        } else {
            0
        };

        for (i, file) in filtered
            .iter()
            .enumerate()
            .skip(scroll)
            .take(visible_height)
        {
            let is_selected = i == selected;
            let prefix = if is_selected { "▸ " } else { "  " };
            let style = if is_selected {
                Style::default()
                    .fg(theme::NEON_CYAN)
                    .add_modifier(Modifier::BOLD)
            } else {
                Style::default().fg(theme::TEXT_PRIMARY)
            };
            lines.push(Line::from(Span::styled(
                format!("{}{}", prefix, file),
                style,
            )));
        }
    }

    // Add footer
    lines.push(Line::from(""));
    lines.push(Line::from(vec![
        Span::styled("↑↓", Style::default().fg(theme::NEON_CYAN)),
        Span::styled(" navigate  ", theme::dimmed()),
        Span::styled("Enter", Style::default().fg(theme::NEON_CYAN)),
        Span::styled(" jump  ", theme::dimmed()),
        Span::styled("Esc", Style::default().fg(theme::NEON_CYAN)),
        Span::styled(" cancel", theme::dimmed()),
    ]));

    let paragraph = Paragraph::new(lines);
    frame.render_widget(paragraph, inner);
}

fn render_confirm(frame: &mut Frame, area: Rect, message: &str) {
    let block = Block::default()
        .title(" Confirm ")
        .borders(Borders::ALL)
        .border_style(Style::default().fg(theme::ELECTRIC_YELLOW));
    let inner = block.inner(area);
    frame.render_widget(block, area);

    let lines = vec![
        Line::from(message),
        Line::from(""),
        Line::from("Press y/n to confirm"),
    ];
    let paragraph = Paragraph::new(lines);
    frame.render_widget(paragraph, inner);
}

fn render_chat(
    frame: &mut Frame,
    area: Rect,
    chat_state: &crate::studio::state::ChatState,
    last_render: Instant,
) {
    let block = Block::default()
        .title(" ◈ Chat with Iris ")
        .borders(Borders::ALL)
        .border_style(Style::default().fg(theme::ELECTRIC_PURPLE));
    let inner = block.inner(area);
    frame.render_widget(block, area);

    // Split inner area: messages area and input area
    let input_height = 3u16;
    let messages_height = inner.height.saturating_sub(input_height);

    let messages_area = Rect::new(inner.x, inner.y, inner.width, messages_height);
    let input_area = Rect::new(
        inner.x,
        inner.y + messages_height,
        inner.width,
        input_height,
    );

    // Render messages
    let mut lines: Vec<Line> = Vec::new();
    for msg in &chat_state.messages {
        let (prefix, style) = match msg.role {
            ChatRole::User => ("You: ", Style::default().fg(theme::NEON_CYAN)),
            ChatRole::Iris => ("Iris: ", Style::default().fg(theme::ELECTRIC_PURPLE)),
        };

        // Add each line of the message with proper wrapping
        for (i, content_line) in msg.content.lines().enumerate() {
            if i == 0 {
                lines.push(Line::from(vec![
                    Span::styled(prefix, style.add_modifier(Modifier::BOLD)),
                    Span::styled(content_line, Style::default().fg(theme::TEXT_PRIMARY)),
                ]));
            } else {
                lines.push(Line::from(Span::styled(
                    format!("      {}", content_line),
                    Style::default().fg(theme::TEXT_PRIMARY),
                )));
            }
        }
        lines.push(Line::from("")); // Spacing between messages
    }

    // Show typing indicator if Iris is responding
    if chat_state.is_responding {
        let spinner = theme::SPINNER_BRAILLE
            [last_render.elapsed().as_millis() as usize / 80 % theme::SPINNER_BRAILLE.len()];
        lines.push(Line::from(vec![
            Span::styled(
                format!("{} ", spinner),
                Style::default().fg(theme::ELECTRIC_PURPLE),
            ),
            Span::styled("Iris is thinking...", theme::dimmed()),
        ]));
    }

    // Empty state
    if chat_state.messages.is_empty() && !chat_state.is_responding {
        lines.push(Line::from(Span::styled(
            "Ask Iris anything about your code...",
            theme::dimmed(),
        )));
    }

    let messages_paragraph = Paragraph::new(lines).scroll((chat_state.scroll_offset as u16, 0));
    frame.render_widget(messages_paragraph, messages_area);

    // Render input box
    let input_block = Block::default()
        .borders(Borders::TOP)
        .border_style(Style::default().fg(theme::TEXT_DIM));

    let input_inner = input_block.inner(input_area);
    frame.render_widget(input_block, input_area);

    let input_line = Line::from(vec![
        Span::styled("> ", Style::default().fg(theme::ELECTRIC_PURPLE)),
        Span::styled(&chat_state.input, Style::default().fg(theme::TEXT_PRIMARY)),
        Span::styled("█", Style::default().fg(theme::NEON_CYAN)),
    ]);
    let input_paragraph = Paragraph::new(input_line);
    frame.render_widget(input_paragraph, input_inner);
}

fn render_ref_selector(
    frame: &mut Frame,
    area: Rect,
    input: &str,
    refs: &[String],
    selected: usize,
    target: RefSelectorTarget,
) {
    let title = match target {
        RefSelectorTarget::ReviewFrom => " Select Review From Ref ",
        RefSelectorTarget::ReviewTo => " Select Review To Ref ",
        RefSelectorTarget::PrFrom => " Select PR Base (From) ",
        RefSelectorTarget::PrTo => " Select PR Target (To) ",
        RefSelectorTarget::ChangelogFrom => " Select Changelog From ",
        RefSelectorTarget::ChangelogTo => " Select Changelog To ",
        RefSelectorTarget::ReleaseNotesFrom => " Select Release Notes From ",
        RefSelectorTarget::ReleaseNotesTo => " Select Release Notes To ",
    };

    let block = Block::default()
        .title(title)
        .borders(Borders::ALL)
        .border_style(Style::default().fg(theme::NEON_CYAN));
    let inner = block.inner(area);
    frame.render_widget(block, area);

    // Filter refs based on input
    let filtered: Vec<_> = refs
        .iter()
        .filter(|r| input.is_empty() || r.to_lowercase().contains(&input.to_lowercase()))
        .collect();

    // Build lines
    let mut lines = vec![
        Line::from(vec![
            Span::styled("Filter: ", theme::dimmed()),
            Span::styled(input, Style::default().fg(theme::TEXT_PRIMARY)),
            Span::styled("█", Style::default().fg(theme::NEON_CYAN)),
        ]),
        Line::from(""),
    ];

    // Show filtered refs
    for (idx, branch) in filtered.iter().take(inner.height as usize - 4).enumerate() {
        let is_selected = idx == selected;
        let prefix = if is_selected { "▸ " } else { "  " };
        let style = if is_selected {
            Style::default()
                .fg(theme::NEON_CYAN)
                .add_modifier(Modifier::BOLD)
        } else {
            Style::default().fg(theme::TEXT_PRIMARY)
        };
        lines.push(Line::from(vec![
            Span::styled(prefix, style),
            Span::styled((*branch).clone(), style),
        ]));
    }

    if filtered.is_empty() {
        lines.push(Line::from(Span::styled(
            "No matching refs",
            theme::dimmed(),
        )));
    }

    // Hint at bottom
    lines.push(Line::from(""));
    lines.push(Line::from(vec![
        Span::styled("↑↓", Style::default().fg(theme::NEON_CYAN)),
        Span::styled(" navigate  ", theme::dimmed()),
        Span::styled("Enter", Style::default().fg(theme::NEON_CYAN)),
        Span::styled(" select  ", theme::dimmed()),
        Span::styled("Esc", Style::default().fg(theme::NEON_CYAN)),
        Span::styled(" cancel", theme::dimmed()),
    ]));

    let paragraph = Paragraph::new(lines);
    frame.render_widget(paragraph, inner);
}

fn render_preset_selector(
    frame: &mut Frame,
    area: Rect,
    input: &str,
    presets: &[PresetInfo],
    selected: usize,
    scroll: usize,
) {
    let block = Block::default()
        .title(" Select Commit Style Preset ")
        .borders(Borders::ALL)
        .border_style(Style::default().fg(theme::ELECTRIC_PURPLE));
    let inner = block.inner(area);
    frame.render_widget(block, area);

    // Filter presets based on input
    let filtered: Vec<_> = presets
        .iter()
        .filter(|p| {
            input.is_empty()
                || p.name.to_lowercase().contains(&input.to_lowercase())
                || p.key.to_lowercase().contains(&input.to_lowercase())
        })
        .collect();

    // Calculate visible area for presets (height minus header and footer)
    let visible_height = inner.height.saturating_sub(5) as usize;

    // Build lines
    let mut lines = vec![
        Line::from(vec![
            Span::styled("Filter: ", theme::dimmed()),
            Span::styled(input, Style::default().fg(theme::TEXT_PRIMARY)),
            Span::styled("█", Style::default().fg(theme::NEON_CYAN)),
        ]),
        Line::from(""),
    ];

    // Show filtered presets with scroll offset
    for (idx, preset) in filtered
        .iter()
        .enumerate()
        .skip(scroll)
        .take(visible_height)
    {
        let is_selected = idx == selected;
        let prefix = if is_selected { "▸ " } else { "  " };
        let style = if is_selected {
            Style::default()
                .fg(theme::NEON_CYAN)
                .add_modifier(Modifier::BOLD)
        } else {
            Style::default().fg(theme::TEXT_PRIMARY)
        };
        let desc_style = if is_selected {
            Style::default().fg(theme::TEXT_DIM)
        } else {
            theme::dimmed()
        };

        lines.push(Line::from(vec![
            Span::styled(prefix, style),
            Span::styled(&preset.emoji, style),
            Span::raw(" "),
            Span::styled(&preset.name, style),
            Span::styled(" - ", desc_style),
            Span::styled(&preset.description, desc_style),
        ]));
    }

    if filtered.is_empty() {
        lines.push(Line::from(Span::styled(
            "No matching presets",
            theme::dimmed(),
        )));
    }

    // Hint at bottom with scroll indicator
    lines.push(Line::from(""));
    let scroll_hint = if filtered.len() > visible_height {
        format!(" ({}/{})", selected + 1, filtered.len())
    } else {
        String::new()
    };
    lines.push(Line::from(vec![
        Span::styled("↑↓", Style::default().fg(theme::ELECTRIC_PURPLE)),
        Span::styled(" navigate  ", theme::dimmed()),
        Span::styled("Enter", Style::default().fg(theme::ELECTRIC_PURPLE)),
        Span::styled(" select  ", theme::dimmed()),
        Span::styled("Esc", Style::default().fg(theme::ELECTRIC_PURPLE)),
        Span::styled(" cancel", theme::dimmed()),
        Span::styled(scroll_hint, Style::default().fg(theme::TEXT_DIM)),
    ]));

    let paragraph = Paragraph::new(lines);
    frame.render_widget(paragraph, inner);
}

fn render_emoji_selector(
    frame: &mut Frame,
    area: Rect,
    input: &str,
    emojis: &[EmojiInfo],
    selected: usize,
    scroll: usize,
) {
    let block = Block::default()
        .title(" Select Emoji ")
        .borders(Borders::ALL)
        .border_style(Style::default().fg(theme::ELECTRIC_YELLOW));
    let inner = block.inner(area);
    frame.render_widget(block, area);

    // Filter emojis based on input
    let filtered: Vec<_> = emojis
        .iter()
        .filter(|e| {
            input.is_empty()
                || e.key.to_lowercase().contains(&input.to_lowercase())
                || e.description.to_lowercase().contains(&input.to_lowercase())
                || e.emoji.contains(input)
        })
        .collect();

    // Calculate visible area for emojis (height minus header and footer)
    let visible_height = inner.height.saturating_sub(5) as usize;

    // Build lines
    let mut lines = vec![
        Line::from(vec![
            Span::styled("Filter: ", theme::dimmed()),
            Span::styled(input, Style::default().fg(theme::TEXT_PRIMARY)),
            Span::styled("█", Style::default().fg(theme::NEON_CYAN)),
        ]),
        Line::from(""),
    ];

    // Show filtered emojis with scroll offset
    for (idx, emoji_info) in filtered
        .iter()
        .enumerate()
        .skip(scroll)
        .take(visible_height)
    {
        let is_selected = idx == selected;
        let prefix = if is_selected { "▸ " } else { "  " };
        let style = if is_selected {
            Style::default()
                .fg(theme::NEON_CYAN)
                .add_modifier(Modifier::BOLD)
        } else {
            Style::default().fg(theme::TEXT_PRIMARY)
        };
        let desc_style = if is_selected {
            Style::default().fg(theme::TEXT_DIM)
        } else {
            theme::dimmed()
        };

        // Special styling for None and Auto options
        let emoji_style = if emoji_info.key == "none" || emoji_info.key == "auto" {
            Style::default()
                .fg(theme::ELECTRIC_PURPLE)
                .add_modifier(Modifier::BOLD)
        } else {
            Style::default().fg(theme::ELECTRIC_YELLOW)
        };

        lines.push(Line::from(vec![
            Span::styled(prefix, style),
            Span::styled(&emoji_info.emoji, emoji_style),
            Span::raw("  "),
            Span::styled(&emoji_info.key, style),
            Span::styled(" - ", desc_style),
            Span::styled(&emoji_info.description, desc_style),
        ]));
    }

    if filtered.is_empty() {
        lines.push(Line::from(Span::styled(
            "No matching emojis",
            theme::dimmed(),
        )));
    }

    // Hint at bottom with scroll indicator
    lines.push(Line::from(""));
    let scroll_hint = if filtered.len() > visible_height {
        format!(" ({}/{})", selected + 1, filtered.len())
    } else {
        String::new()
    };
    lines.push(Line::from(vec![
        Span::styled("↑↓", Style::default().fg(theme::ELECTRIC_YELLOW)),
        Span::styled(" navigate  ", theme::dimmed()),
        Span::styled("Enter", Style::default().fg(theme::ELECTRIC_YELLOW)),
        Span::styled(" select  ", theme::dimmed()),
        Span::styled("Esc", Style::default().fg(theme::ELECTRIC_YELLOW)),
        Span::styled(" cancel", theme::dimmed()),
        Span::styled(scroll_hint, Style::default().fg(theme::TEXT_DIM)),
    ]));

    let emoji_paragraph = Paragraph::new(lines);
    frame.render_widget(emoji_paragraph, inner);
}

fn render_settings(frame: &mut Frame, area: Rect, state: &SettingsState) {
    let title = if state.modified {
        " Settings * "
    } else {
        " Settings "
    };

    let block = Block::default()
        .title(title)
        .borders(Borders::ALL)
        .border_style(Style::default().fg(theme::ELECTRIC_PURPLE));
    let inner = block.inner(area);
    frame.render_widget(block, area);

    let mut lines = Vec::new();

    // Header
    lines.push(Line::from(Span::styled(
        "Configure Iris settings",
        theme::dimmed(),
    )));
    lines.push(Line::from(""));

    // Render each field
    for (idx, field) in SettingsField::all().iter().enumerate() {
        let is_selected = idx == state.selected_field;
        let value = state.get_field_value(*field);

        let prefix = if is_selected { "> " } else { "  " };
        let label_style = if is_selected {
            Style::default()
                .fg(theme::NEON_CYAN)
                .add_modifier(Modifier::BOLD)
        } else {
            Style::default().fg(theme::TEXT_PRIMARY)
        };

        let value_style = if is_selected {
            Style::default().fg(theme::TEXT_PRIMARY)
        } else {
            theme::dimmed()
        };

        // Show input buffer when editing
        let display_value = if state.editing && is_selected {
            match field {
                SettingsField::ApiKey => format!("{}█", "*".repeat(state.input_buffer.len())),
                _ => format!("{}█", state.input_buffer),
            }
        } else {
            value
        };

        // Add indicator for cyclable fields
        let suffix = match field {
            SettingsField::Provider
            | SettingsField::UseGitmoji
            | SettingsField::InstructionPreset => " [←→]",
            SettingsField::Model | SettingsField::ApiKey => " [edit]",
        };

        lines.push(Line::from(vec![
            Span::styled(prefix, label_style),
            Span::styled(format!("{:12}", field.display_name()), label_style),
            Span::styled(display_value, value_style),
            Span::styled(suffix, theme::dimmed()),
        ]));
    }

    // Error message if any
    if let Some(error) = &state.error {
        lines.push(Line::from(""));
        lines.push(Line::from(Span::styled(
            error,
            Style::default().fg(theme::ERROR_RED),
        )));
    }

    // Footer hints
    lines.push(Line::from(""));
    lines.push(Line::from(""));

    if state.editing {
        lines.push(Line::from(vec![
            Span::styled("Enter", Style::default().fg(theme::NEON_CYAN)),
            Span::styled(" confirm  ", theme::dimmed()),
            Span::styled("Esc", Style::default().fg(theme::NEON_CYAN)),
            Span::styled(" cancel", theme::dimmed()),
        ]));
    } else {
        lines.push(Line::from(vec![
            Span::styled("↑↓", Style::default().fg(theme::ELECTRIC_PURPLE)),
            Span::styled(" navigate  ", theme::dimmed()),
            Span::styled("Enter/Space", Style::default().fg(theme::ELECTRIC_PURPLE)),
            Span::styled(" edit  ", theme::dimmed()),
            Span::styled("s", Style::default().fg(theme::SUCCESS_GREEN)),
            Span::styled(" save  ", theme::dimmed()),
            Span::styled("Esc", Style::default().fg(theme::ELECTRIC_PURPLE)),
            Span::styled(" close", theme::dimmed()),
        ]));
    }

    let paragraph = Paragraph::new(lines);
    frame.render_widget(paragraph, inner);
}
