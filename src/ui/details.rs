use crate::model::Transaction;
use ratatui::prelude::*;
use ratatui::widgets::{Block, Borders, Clear, List, ListItem};

pub fn render_transaction_details(frame: &mut Frame, tx: &Transaction, scroll_offset: usize) {
    let area = centered_rect(90, 80, frame.area());

    // Clear the background
    frame.render_widget(Clear, area);

    // Create details text as list items
    let mut details: Vec<ListItem> = vec![];

    details.push(ListItem::new(Line::from("")));

    details.push(ListItem::new(Line::from(vec![
        Span::styled("Hash: ", Style::default().fg(Color::Yellow).bold()),
        Span::raw(&tx.hash),
    ])));
    details.push(ListItem::new(Line::from("")));

    details.push(ListItem::new(Line::from(vec![
        Span::styled("From: ", Style::default().fg(Color::Yellow).bold()),
        Span::raw(&tx.from),
    ])));
    details.push(ListItem::new(Line::from("")));

    // Add 'To' field
    if let Some(to) = &tx.to {
        details.push(ListItem::new(Line::from(vec![
            Span::styled("To: ", Style::default().fg(Color::Yellow).bold()),
            Span::raw(to),
        ])));
    } else {
        details.push(ListItem::new(Line::from(vec![
            Span::styled("To: ", Style::default().fg(Color::Yellow).bold()),
            Span::styled("Contract Creation", Style::default().fg(Color::Magenta).italic()),
        ])));
    }
    details.push(ListItem::new(Line::from("")));

    // Add value
    details.push(ListItem::new(Line::from(vec![
        Span::styled("Value: ", Style::default().fg(Color::Yellow).bold()),
        Span::styled(
            format!("{} ETH", tx.value),
            Style::default().fg(Color::Green),
        ),
    ])));
    details.push(ListItem::new(Line::from("")));

    // Add function information
    if let Some(func_sig) = &tx.function_sig {
        details.push(ListItem::new(Line::from(vec![
            Span::styled("Function: ", Style::default().fg(Color::Yellow).bold()),
            Span::styled(
                &func_sig.name,
                Style::default().fg(crate::model::decoder::get_function_color(&func_sig.name)),
            ),
        ])));
        details.push(ListItem::new(Line::from(vec![
            Span::styled("Selector: ", Style::default().fg(Color::Yellow).bold()),
            Span::raw(&func_sig.selector),
        ])));
    } else {
        details.push(ListItem::new(Line::from(vec![
            Span::styled("Function: ", Style::default().fg(Color::Yellow).bold()),
            Span::styled("Unknown", Style::default().fg(Color::Gray)),
        ])));
    }
    details.push(ListItem::new(Line::from("")));

    // Add data field
    if tx.has_data() {
        details.push(ListItem::new(Line::from(vec![
            Span::styled("Data: ", Style::default().fg(Color::Yellow).bold()),
        ])));

        // Format data with proper line wrapping for long data
        let data_str = &tx.data;
        if data_str.len() <= 66 {
            details.push(ListItem::new(Line::from(vec![
                Span::raw(data_str),
            ])));
        } else {
            // Break data into chunks of 66 characters
            for chunk in data_str.chars().collect::<Vec<_>>().chunks(66) {
                let chunk_str: String = chunk.iter().collect();
                details.push(ListItem::new(Line::from(vec![
                    Span::raw(chunk_str),
                ])));
            }
        }
    } else {
        details.push(ListItem::new(Line::from(vec![
            Span::styled("Data: ", Style::default().fg(Color::Yellow).bold()),
            Span::styled("(empty)", Style::default().fg(Color::DarkGray).italic()),
        ])));
    }
    details.push(ListItem::new(Line::from("")));

    // Add receipt data if available
    if let Some(block_num) = tx.block_number {
        details.push(ListItem::new(Line::from(vec![
            Span::styled("Block Number: ", Style::default().fg(Color::Yellow).bold()),
            Span::raw(block_num.to_string()),
        ])));
    }

    if let Some(status) = tx.status {
        let (status_text, status_color) = if status {
            ("Success ✓", Color::Green)
        } else {
            ("Failed ✗", Color::Red)
        };
        details.push(ListItem::new(Line::from(vec![
            Span::styled("Status: ", Style::default().fg(Color::Yellow).bold()),
            Span::styled(status_text, Style::default().fg(status_color).bold()),
        ])));
    }
    details.push(ListItem::new(Line::from("")));

    // Add gas information
    details.push(ListItem::new(Line::from(vec![
        Span::styled("Gas Limit: ", Style::default().fg(Color::Yellow).bold()),
        Span::raw(&tx.gas_limit),
    ])));

    if let Some(gas_used) = &tx.gas_used {
        details.push(ListItem::new(Line::from(vec![
            Span::styled("Gas Used: ", Style::default().fg(Color::Yellow).bold()),
            Span::raw(gas_used),
        ])));
    }

    if let Some(gas_price) = &tx.gas_price {
        details.push(ListItem::new(Line::from(vec![
            Span::styled("Gas Price: ", Style::default().fg(Color::Yellow).bold()),
            Span::raw(format!("{} Gwei", gas_price)),
        ])));
    }

    if let Some(effective_gas_price) = &tx.effective_gas_price {
        details.push(ListItem::new(Line::from(vec![
            Span::styled("Effective Gas Price: ", Style::default().fg(Color::Yellow).bold()),
            Span::raw(format!("{} Gwei", effective_gas_price)),
        ])));
    }

    // Calculate transaction cost if we have gas used and effective price
    if let (Some(gas_used), Some(price)) = (&tx.gas_used, &tx.effective_gas_price) {
        if let (Ok(gas), Ok(price_val)) = (gas_used.parse::<u128>(), price.parse::<u128>()) {
            let cost_wei = gas * price_val;
            let cost_eth = cost_wei as f64 / 1_000_000_000_000_000_000.0;
            details.push(ListItem::new(Line::from(vec![
                Span::styled("Transaction Cost: ", Style::default().fg(Color::Yellow).bold()),
                Span::styled(format!("{:.6} ETH", cost_eth), Style::default().fg(Color::Magenta)),
            ])));
        }
    }
    details.push(ListItem::new(Line::from("")));

    // Add timestamp
    details.push(ListItem::new(Line::from(vec![
        Span::styled("Time: ", Style::default().fg(Color::Yellow).bold()),
        Span::raw(tx.formatted_time()),
    ])));
    details.push(ListItem::new(Line::from("")));

    // Add footer instructions before calculating scroll
    details.push(ListItem::new(Line::from("")));
    details.push(ListItem::new(Line::from(vec![
        Span::styled(
            "Press ESC, Enter, or Q to close | ↑/↓ to scroll",
            Style::default().fg(Color::Gray).italic(),
        ),
    ])));

    // Add scroll indicator and instructions
    let total_lines = details.len();
    let visible_height = area.height.saturating_sub(2) as usize; // Subtract 2 for borders

    // Calculate max scroll offset
    let max_scroll = total_lines.saturating_sub(visible_height);
    let adjusted_scroll = scroll_offset.min(max_scroll);

    // Add scroll indicator to title if content is scrollable
    let title = if total_lines > visible_height {
        format!(" Transaction Details (Line {}/{}) ",
                adjusted_scroll + 1,
                total_lines - adjusted_scroll.min(visible_height))
    } else {
        " Transaction Details ".to_string()
    };

    // Get visible items based on scroll offset
    let visible_items: Vec<ListItem> = details
        .into_iter()
        .skip(adjusted_scroll)
        .take(visible_height)
        .collect();

    let list = List::new(visible_items)
        .block(
            Block::default()
                .title(title)
                .borders(Borders::ALL)
                .border_style(Style::default().fg(Color::Cyan))
                .border_type(ratatui::widgets::BorderType::Rounded),
        )
        .style(Style::default().bg(Color::Black));

    frame.render_widget(list, area);
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