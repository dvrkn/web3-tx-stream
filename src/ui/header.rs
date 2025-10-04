use crate::app::{Config, Stats};
use ratatui::prelude::*;
use ratatui::widgets::{Block, Borders, Paragraph};

pub fn render_header(frame: &mut Frame, area: Rect, stats: &Stats, _config: &Config) {
    let runtime = format_runtime(stats.start_time);
    let connection_status = if stats.connected {
        ("✓", Color::Green)
    } else {
        ("✗", Color::Red)
    };

    let header_text = vec![
        Line::from(vec![
            Span::styled("Base Transaction Sniffer", Style::default().fg(Color::Cyan).bold()),
            Span::raw(" | "),
            Span::raw("Connected: "),
            Span::styled(connection_status.0, Style::default().fg(connection_status.1)),
            Span::raw(" | "),
            Span::raw(format!("TX: {} | ", format_number(stats.total_transactions))),
            Span::raw(format!("TPS: {:.1} | ", stats.transactions_per_second)),
            Span::raw(format!("Mem: {:.1}MB | ", stats.memory_usage_mb)),
            Span::raw(format!("Runtime: {}", runtime)),
        ]),
    ];

    let header_widget = Paragraph::new(header_text)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .border_style(Style::default().fg(Color::DarkGray)),
        )
        .style(Style::default().bg(Color::Black))
        .alignment(Alignment::Center);

    frame.render_widget(header_widget, area);
}

fn format_runtime(start_time: i64) -> String {
    let now = chrono::Utc::now().timestamp();
    let elapsed = now - start_time;

    let hours = elapsed / 3600;
    let minutes = (elapsed % 3600) / 60;
    let seconds = elapsed % 60;

    if hours > 0 {
        format!("{}h {}m", hours, minutes)
    } else if minutes > 0 {
        format!("{}m {}s", minutes, seconds)
    } else {
        format!("{}s", seconds)
    }
}

fn format_number(n: u64) -> String {
    if n >= 1_000_000 {
        format!("{:.1}M", n as f64 / 1_000_000.0)
    } else if n >= 1_000 {
        format!("{:.1}K", n as f64 / 1_000.0)
    } else {
        n.to_string()
    }
}