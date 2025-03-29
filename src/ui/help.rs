use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph},
    Frame,
};

/// Draw the help screen
pub(super) fn draw_help(f: &mut Frame) {
    let area = centered_rect(60, 60, f.size());

    let help_text = vec![
        Line::from(Span::styled("Help", Style::default().add_modifier(Modifier::BOLD))),
        Line::from(""),
        Line::from(Span::raw("Navigation:")),
        Line::from(Span::raw("  Tab, Right - Next tab")),
        Line::from(Span::raw("  Shift+Tab, Left - Previous tab")),
        Line::from(Span::raw("  1-6 - Select tab")),
        Line::from(""),
        Line::from(Span::raw("Scrolling:")),
        Line::from(Span::raw("  Up, Down - Scroll current view")),
        Line::from(Span::raw("  Page Up, Page Down - Scroll page")),
        Line::from(Span::raw("  Home, End - Scroll to top/bottom")),
        Line::from(""),
        Line::from(Span::raw("Other:")),
        Line::from(Span::raw("  ? - Toggle help")),
        Line::from(Span::raw("  q, Esc - Quit")),
        Line::from(Span::raw("  r - Refresh data")),
    ];

    let help_block = Block::default()
        .borders(Borders::ALL)
        .title("Help");

    let help_paragraph = Paragraph::new(help_text)
        .block(help_block)
        .style(Style::default());

    f.render_widget(help_paragraph, area);
}

/// Helper function to create a centered rect (moved here as it's only used by help)
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