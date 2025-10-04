use ratatui::prelude::*;
use ratatui::widgets::{Block, Borders, Clear, Paragraph};

/// Render quit confirmation dialog
pub fn render_quit_confirmation(frame: &mut Frame) {
    // Create a centered area for the confirmation dialog
    let area = centered_rect(40, 20, frame.area());

    // Clear the background
    frame.render_widget(Clear, area);

    // Create the confirmation dialog
    let confirmation_widget = Paragraph::new(vec![
        Line::from(vec![
            Span::styled("Quit Application?", Style::default().fg(Color::Yellow).bold()),
        ]),
        Line::from(""),
        Line::from("Are you sure you want to quit?"),
        Line::from(""),
        Line::from(vec![
            Span::styled("Y", Style::default().fg(Color::Green).bold()),
            Span::raw(": Yes  |  "),
            Span::styled("N", Style::default().fg(Color::Red).bold()),
            Span::raw(": No"),
        ]),
    ])
    .block(
        Block::default()
            .title(" Confirm ")
            .borders(Borders::ALL)
            .border_style(Style::default().fg(Color::Red))
            .border_type(ratatui::widgets::BorderType::Rounded),
    )
    .style(Style::default().bg(Color::Black))
    .alignment(Alignment::Center);

    frame.render_widget(confirmation_widget, area);
}

/// Helper function to create a centered rect
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