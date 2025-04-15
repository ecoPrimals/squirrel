use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Style},
    text::{Line, Span, Text},
    widgets::{Block, Borders, List, ListItem, Paragraph},
    Frame,
};

use super::state::AiChatWidgetState;
use super::messages::ChatMessage;

/// Creates a formatted message for display in the chat UI
fn render_message(message: &ChatMessage) -> Text<'_> {
    let color = match message.role.as_str() {
        "user" => Color::LightBlue,
        "assistant" => Color::LightGreen,
        "system" => Color::LightYellow,
        _ => Color::Gray,
    };
    
    let content_str = message.content.clone();
    
    let header = Line::from(vec![
        Span::styled(
            format!("{}: ", message.role),
            Style::default().fg(color)
        )
    ]);
    
    let content = Line::from(vec![
        Span::raw(content_str),
    ]);
    
    Text::from(vec![header, content])
}

/// Draws the AI Chat widget in the terminal
pub fn draw_ai_chat(
    f: &mut Frame<'_>, 
    area: Rect, 
    state: &AiChatWidgetState, 
    is_focused: bool
) {
    // Create a layout for the chat area and input field
    let layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Min(5),      // Messages area
            Constraint::Length(3),   // Input field
            Constraint::Length(1),   // Status bar
        ])
        .split(area);
    
    // Draw chat messages area
    let messages_block = Block::default()
        .title("AI Chat")
        .borders(Borders::ALL)
        .style(Style::default().fg(if is_focused { Color::Cyan } else { Color::White }));
    
    // Convert messages to ListItems
    let messages: Vec<ListItem> = state.messages
        .iter()
        .map(|m| ListItem::new(render_message(m)))
        .collect();
    
    let messages_list = List::new(messages)
        .block(messages_block);
    
    f.render_widget(messages_list, layout[0]);
    
    // Draw input field
    let input_block = Block::default()
        .title("Message")
        .borders(Borders::ALL)
        .style(Style::default().fg(if state.input_focused { Color::Yellow } else { Color::White }));
    
    let input_paragraph = Paragraph::new(state.input.as_str())
        .block(input_block);
    
    f.render_widget(input_paragraph, layout[1]);
    
    // Draw status bar showing selected model and status
    let status_text = if state.is_sending {
        Text::from(vec![Line::from(vec![
            Span::styled("Sending...", Style::default().fg(Color::Yellow)),
        ])])
    } else if let Some(model) = state.models.get(state.selected_model) {
        Text::from(vec![Line::from(vec![
            Span::raw("Model: "),
            Span::styled(model.to_api_name(), Style::default().fg(Color::Green)),
        ])])
    } else {
        Text::from(vec![Line::from(vec![
            Span::styled("No model selected", Style::default().fg(Color::Red)),
        ])])
    };
    
    let status_paragraph = Paragraph::new(status_text);
    f.render_widget(status_paragraph, layout[2]);
} 