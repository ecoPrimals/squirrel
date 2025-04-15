//! Rendering functionality for the chat widget
use ratatui::{
    backend::Backend,
    Frame,
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span, Text},
    widgets::{Block, Borders, Paragraph, Wrap},
    prelude::Alignment,
};

use super::state::ChatState;

/// Render the chat widget - this function is used by the UI to render the chat widget
pub fn render<B: Backend>(f: &mut Frame<'_>, area: Rect, chat_state: &ChatState) {
    chat_state.render::<B>(f, area);
}

/// Render help overlay on the screen
pub(crate) fn render_help_overlay(f: &mut Frame<'_>, area: Rect) {
    let block = Block::default()
        .title("Help")
        .borders(Borders::ALL)
        .style(Style::default().bg(Color::DarkGray));
    
    let inner_area = block.inner(area);
    f.render_widget(Block::default().style(Style::default().bg(Color::DarkGray)), area);
    f.render_widget(block, area);

    let help_text = Text::from(vec![
        Line::from(Span::styled("Keyboard Controls:", Style::default().add_modifier(Modifier::BOLD))),
        Line::from(""),
        Line::from(Span::styled("Global Controls:", Style::default().add_modifier(Modifier::UNDERLINED))),
        Line::from("q         : Quit application"),
        Line::from("?         : Toggle help overlay"),
        Line::from("Up        : Scroll up"),
        Line::from("Down      : Scroll down"),
        Line::from("PgUp      : Scroll up by page"),
        Line::from("PgDown    : Scroll down by page"),
        Line::from("Home      : Scroll to top"),
        Line::from("End       : Scroll to bottom"),
        Line::from("Ctrl+K    : Clear chat history"),
        Line::from(""),
        Line::from(Span::styled("Normal Mode:", Style::default().add_modifier(Modifier::UNDERLINED))),
        Line::from("i         : Enter editing mode"),
        Line::from("g         : Go to top of chat history"),
        Line::from("G         : Go to bottom of chat history"),
        Line::from(""),
        Line::from(Span::styled("Editing Mode:", Style::default().add_modifier(Modifier::UNDERLINED))),
        Line::from("Enter     : Send message"),
        Line::from("Esc       : Return to normal mode"),
        Line::from("Left/Right: Move cursor"),
        Line::from("Backspace : Delete character"),
        Line::from("Any key   : Type character"),
        Line::from(""),
        Line::from(Span::styled("Chat History:", Style::default().add_modifier(Modifier::UNDERLINED))),
        Line::from("Your chat history is automatically saved between sessions."),
        Line::from("Press Ctrl+K to clear all history if needed."),
        Line::from(""),
        Line::from("Press any key to close help"),
    ]);

    let paragraph = Paragraph::new(help_text)
        .style(Style::default().bg(Color::DarkGray))
        .wrap(Wrap { trim: true });
    
    f.render_widget(paragraph, inner_area);
}

/// Render a chat message
pub(crate) fn render_message(f: &mut Frame<'_>, area: Rect, msg: &str, is_user: bool, is_selected: bool) {
    // Use different styles for user vs AI messages
    let user_style = Style::default().fg(Color::Green);
    let ai_style = Style::default().fg(Color::Blue);
    let selected_style = Style::default().fg(Color::Yellow);
    
    // Choose the style based on whether it's a user message and whether it's selected
    let style = if is_selected {
        selected_style
    } else if is_user {
        user_style
    } else {
        ai_style
    };
    
    // Create a paragraph for the message with proper wrapping
    let text = Text::from(Line::from(msg));
    
    // Calculate the required height based on message content and area width
    // This ensures proper wrapping for long messages
    let line_count = msg.chars().filter(|&c| c == '\n').count() + 1;
    
    // Estimate additional lines from wrapping (rough estimate)
    let wrap_count = (msg.len() / area.width as usize).max(1);
    let total_lines = line_count + wrap_count;
    
    let paragraph = Paragraph::new(text)
        .style(style)
        .wrap(Wrap { trim: false }) // Enable wrapping without trimming
        .block(Block::default().borders(Borders::NONE));
    
    // Render the paragraph in the provided area
    f.render_widget(paragraph, area);
    
    // Log the rendering details for debugging long message issues
    log::debug!("Rendered message with {} chars, estimated {} lines in area width {}", 
        msg.len(), total_lines, area.width);
} 