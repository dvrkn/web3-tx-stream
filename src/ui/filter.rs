use crate::filter::FilterState;
use ratatui::prelude::*;
use ratatui::widgets::{Block, Borders, Clear, Paragraph};

/// Render the filter input popup when active
pub fn render_filter_input(frame: &mut Frame, filter: &FilterState) {
    if !filter.is_active() {
        return;
    }

    // Create a centered area for the filter input
    let area = centered_rect(60, 20, frame.area());

    // Clear the background
    frame.render_widget(Clear, area);

    // Create the input text with cursor
    let input_text = create_input_with_cursor(filter.query(), filter.cursor_position());

    // Create the filter input widget
    let input_widget = Paragraph::new(vec![
        Line::from(vec![
            Span::styled("Filter by Address", Style::default().fg(Color::Cyan).bold()),
        ]),
        Line::from(""),
        Line::from(vec![
            Span::raw("Enter text to filter by From/To addresses:"),
        ]),
        Line::from(""),
        Line::from(input_text),
        Line::from(""),
        Line::from(vec![
            Span::styled("Enter", Style::default().fg(Color::Green)),
            Span::raw(": Apply | "),
            Span::styled("Esc", Style::default().fg(Color::Red)),
            Span::raw(": Cancel | "),
            Span::styled("←→", Style::default().fg(Color::Yellow)),
            Span::raw(": Move cursor"),
        ]),
    ])
    .block(
        Block::default()
            .title(" Filter Input ")
            .borders(Borders::ALL)
            .border_style(Style::default().fg(Color::Yellow))
            .border_type(ratatui::widgets::BorderType::Rounded),
    )
    .style(Style::default().bg(Color::Black))
    .alignment(Alignment::Left);

    frame.render_widget(input_widget, area);
}

/// Create input text with visible cursor
fn create_input_with_cursor(text: &str, cursor_pos: usize) -> Vec<Span<'static>> {
    let mut spans = Vec::new();

    // Convert text to chars for proper cursor positioning
    let chars: Vec<char> = text.chars().collect();

    // Add text before cursor
    if cursor_pos > 0 {
        let before: String = chars[..cursor_pos.min(chars.len())].iter().collect();
        spans.push(Span::raw(before));
    }

    // Add cursor
    if cursor_pos < chars.len() {
        // Cursor on a character
        let cursor_char = chars[cursor_pos].to_string();
        spans.push(Span::styled(
            cursor_char,
            Style::default().bg(Color::White).fg(Color::Black),
        ));
        // Add text after cursor
        if cursor_pos + 1 < chars.len() {
            let after: String = chars[cursor_pos + 1..].iter().collect();
            spans.push(Span::raw(after));
        }
    } else {
        // Cursor at the end
        spans.push(Span::styled(
            " ",
            Style::default().bg(Color::White).fg(Color::Black),
        ));
    }

    spans
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