use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Span, Text, Line},
    widgets::{Block, Borders, List, ListItem, Paragraph},
    Frame
};

use super::state::AiChatWidgetState;

/// Render the AI chat widget
pub fn render(f: &mut Frame, area: Rect, state: &mut AiChatWidgetState) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Min(1),
            Constraint::Length(3),
        ].as_ref())
        .split(area);
    
    // Render the messages
    render_messages(f, chunks[0], state);
    
    // Render the input area
    render_input(f, chunks[1], state);
}

/// Render the message history
fn render_messages(f: &mut Frame, area: Rect, state: &mut AiChatWidgetState) {
    let messages: Vec<ListItem> = state.messages.iter().map(|message| {
        let color = match message.role.as_str() {
            "user" => Color::LightBlue,
            "assistant" => Color::LightGreen,
            "system" => Color::LightYellow,
            _ => Color::Gray,
        };
        
        let header = Line::from(vec![
            Span::styled(
                format!("{}: ", message.role),
                Style::default().fg(color).add_modifier(Modifier::BOLD)
            )
        ]);
        
        let content = Line::from(vec![
            Span::raw(&message.content),
        ]);
        
        ListItem::new(Text::from(vec![header, content]))
    }).collect();
    
    let messages_list = List::new(messages)
        .block(Block::default().borders(Borders::ALL).title("Chat"))
        .highlight_style(Style::default().add_modifier(Modifier::BOLD))
        .highlight_symbol(">> ");
    
    f.render_stateful_widget(messages_list, area, &mut state.list_state);
}

/// Render the input area with model selector
fn render_input(f: &mut Frame, area: Rect, state: &AiChatWidgetState) {
    let chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Min(20),
            Constraint::Length(20),
        ].as_ref())
        .split(area);
    
    // Input text box
    let input_style = if state.input_focused {
        Style::default().fg(Color::Yellow)
    } else {
        Style::default()
    };
    
    let input_block = Block::default()
        .borders(Borders::ALL)
        .title(if state.is_sending {
            "Sending..."
        } else if state.generating_response {
            "Generating response..."
        } else {
            "Type a message"
        });
    
    let input = Paragraph::new(state.input.as_str())
        .style(input_style)
        .block(input_block);
    
    f.render_widget(input, chunks[0]);
    
    // Model selector
    let model_text = match state.get_selected_model() {
        Ok(model) => format!("Model: {}", model),
        Err(_) => "Model: None Selected".to_string(),
    };
    
    let model_block = Block::default()
        .borders(Borders::ALL)
        .title("AI Model");
    
    let model_selector = Paragraph::new(model_text)
        .style(Style::default().fg(Color::Cyan))
        .block(model_block);
    
    f.render_widget(model_selector, chunks[1]);
} 