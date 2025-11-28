//! Chat modal rendering

use ratatui::Frame;
use ratatui::layout::Rect;
use ratatui::widgets::{Block, Borders, Paragraph};
use std::time::Instant;

use super::super::chat;
use crate::studio::state::ChatState;
use crate::studio::theme;

pub fn render(frame: &mut Frame, area: Rect, chat_state: &ChatState, last_render: Instant) {
    let block = Block::default()
        .title(" ◈ Chat with Iris ")
        .title_bottom(chat::help_footer())
        .borders(Borders::ALL)
        .border_style(theme::keyword());
    let inner = block.inner(area);
    frame.render_widget(block, area);

    // Split inner area: messages area and input area
    let input_height = 3u16;
    let messages_height = inner.height.saturating_sub(input_height);
    let content_width = inner.width.saturating_sub(2) as usize;

    let messages_area = Rect::new(inner.x, inner.y, inner.width, messages_height);
    let input_area = Rect::new(
        inner.x,
        inner.y + messages_height,
        inner.width,
        input_height,
    );

    // Render messages using chat module
    let lines = chat::render_messages(chat_state, content_width, last_render.elapsed().as_millis());
    let total_lines = lines.len();
    let visible_lines = messages_height as usize;

    // Calculate max scroll offset (can't scroll past content)
    let max_scroll = total_lines.saturating_sub(visible_lines);

    // If auto_scroll is enabled, scroll to bottom; otherwise use the current offset (clamped)
    let effective_scroll = if chat_state.auto_scroll {
        max_scroll
    } else {
        chat_state.scroll_offset.min(max_scroll)
    };

    let messages_paragraph = Paragraph::new(lines)
        .scroll((effective_scroll as u16, 0))
        .wrap(ratatui::widgets::Wrap { trim: false });
    frame.render_widget(messages_paragraph, messages_area);

    // Render input box
    let input_block = Block::default()
        .borders(Borders::TOP)
        .border_style(theme::dimmed());
    let input_inner = input_block.inner(input_area);
    frame.render_widget(input_block, input_area);

    let cursor_visible = last_render.elapsed().as_millis() % 1000 < 500;
    let input_line = chat::render_input_line(&chat_state.input, cursor_visible);
    frame.render_widget(Paragraph::new(input_line), input_inner);
}
