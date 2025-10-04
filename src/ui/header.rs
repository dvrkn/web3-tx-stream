use crate::app::{Config, Stats};
use ratatui::prelude::*;
use ratatui::widgets::{Block, Borders, Paragraph};

pub fn render_header(frame: &mut Frame, area: Rect, stats: &Stats, config: &Config) {
    let runtime = format_runtime(stats.start_time);
    let connection_status = if stats.connected {
        ("✓", Color::Green)
    } else {
        ("✗", Color::Red)
    };

    // Format the RPC URL to show only the domain/important part
    let rpc_display = format_rpc_url(&config.rpc_url);

    let header_text = vec![
        Line::from(vec![
            Span::styled("Web3TxStream", Style::default().fg(Color::Cyan).bold()),
            Span::raw(" | "),
            Span::styled(rpc_display, Style::default().fg(Color::Yellow)),
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

fn format_rpc_url(url: &str) -> String {
    // Extract the meaningful part of the URL
    // Remove protocol (ws://, wss://, http://, https://)
    let without_protocol = url
        .strip_prefix("wss://")
        .or_else(|| url.strip_prefix("ws://"))
        .or_else(|| url.strip_prefix("https://"))
        .or_else(|| url.strip_prefix("http://"))
        .unwrap_or(url);

    // If it's a known provider, simplify it
    if without_protocol.contains("base-rpc.publicnode.com") {
        "Base PublicNode".to_string()
    } else if without_protocol.contains("base-mainnet") {
        "Base Mainnet".to_string()
    } else if without_protocol.contains("base-sepolia") || without_protocol.contains("base-testnet") {
        "Base Sepolia".to_string()
    } else if without_protocol.contains("mainnet.infura.io") {
        "Ethereum Mainnet (Infura)".to_string()
    } else if without_protocol.contains("polygon-rpc.com") {
        "Polygon".to_string()
    } else if without_protocol.contains("arb1.arbitrum.io") {
        "Arbitrum One".to_string()
    } else if without_protocol.contains("optimism.io") {
        "Optimism".to_string()
    } else if without_protocol.contains("localhost") || without_protocol.contains("127.0.0.1") {
        "Local Node".to_string()
    } else {
        // For other URLs, just show the domain
        without_protocol
            .split('/')
            .next()
            .unwrap_or(without_protocol)
            .to_string()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_format_rpc_url() {
        assert_eq!(format_rpc_url("wss://base-rpc.publicnode.com"), "Base PublicNode");
        assert_eq!(format_rpc_url("https://base-mainnet.g.alchemy.com/v2/abc"), "Base Mainnet");
        assert_eq!(format_rpc_url("ws://localhost:8545"), "Local Node");
        assert_eq!(format_rpc_url("wss://custom.provider.com/rpc"), "custom.provider.com");
        assert_eq!(format_rpc_url("http://127.0.0.1:8545"), "Local Node");
        assert_eq!(format_rpc_url("wss://mainnet.infura.io/ws/v3/key"), "Ethereum Mainnet (Infura)");
    }
}