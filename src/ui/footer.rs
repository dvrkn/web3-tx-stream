use crate::app::AppState;
use ratatui::prelude::*;
use ratatui::widgets::{Block, Borders, Paragraph};

pub fn render_footer(frame: &mut Frame, area: Rect, state: &AppState) {
    let config = &state.config;
    let stats = &state.stats;

    // First line: navigation keys
    let line1 = vec![
        Span::styled("Navigation: ", Style::default().fg(Color::Cyan).bold()),
        Span::raw("↑↓/jk: Scroll | "),
        Span::raw("Enter: Details | "),
        Span::raw("g/G: Top/Bottom | "),
        Span::raw("PgUp/PgDn: Page"),
    ];

    // Second line: commands
    let line2 = vec![
        Span::styled("Commands: ", Style::default().fg(Color::Cyan).bold()),
        Span::raw("q: Quit | "),
        Span::raw("r: Reconnect | "),
        Span::raw("c: Clear | "),
        Span::raw("t: Toggle Sort "),
        Span::styled(
            if state.show_new_on_top { "[New↑]" } else { "[New↓]" },
            Style::default().fg(Color::Yellow),
        ),
    ];

    // Third line: connection status
    let line3 = if !stats.connected {
        if let Some(error) = &stats.last_error {
            // Show error message (including "Connecting..." status)
            vec![
                Span::styled("Status: ", Style::default().fg(Color::Cyan).bold()),
                Span::styled(
                    truncate_string(error, 100),
                    Style::default().fg(if error.contains("Connecting") { Color::Yellow } else { Color::Red }),
                ),
            ]
        } else {
            vec![
                Span::styled("Status: ", Style::default().fg(Color::Cyan).bold()),
                Span::styled(
                    "Disconnected",
                    Style::default().fg(Color::Red),
                ),
            ]
        }
    } else {
        vec![
            Span::styled("Status: ", Style::default().fg(Color::Cyan).bold()),
            Span::styled(
                format!("Connected to {}", truncate_url(&config.rpc_url)),
                Style::default().fg(Color::Green),
            ),
        ]
    };

    let footer_text = vec![
        Line::from(line1),
        Line::from(line2),
        Line::from(line3),
    ];

    let footer_widget = Paragraph::new(footer_text)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .border_style(Style::default().fg(Color::DarkGray)),
        )
        .style(Style::default().fg(Color::Gray));

    frame.render_widget(footer_widget, area);
}

fn truncate_string(s: &str, max_len: usize) -> String {
    if s.len() <= max_len {
        s.to_string()
    } else {
        format!("{}...", &s[..max_len - 3])
    }
}

fn truncate_url(url: &str) -> String {
    // Extract domain from URL for display
    if let Some(start) = url.find("://") {
        let domain_start = start + 3;
        if let Some(end) = url[domain_start..].find('/') {
            return url[domain_start..domain_start + end].to_string();
        } else {
            return url[domain_start..].to_string();
        }
    }
    truncate_string(url, 40)
}