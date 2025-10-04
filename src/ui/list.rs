use crate::app::AppState;
use ratatui::prelude::*;
use ratatui::widgets::{Block, Borders, Cell, Row, Table};

pub fn render_transaction_list(
    frame: &mut Frame,
    area: Rect,
    state: &AppState,
) {
    let filtered_transactions = state.get_filtered_transactions();
    let scroll_state = &state.scroll_state;
    // Check if any transaction has data to decide if we need the data column
    let show_data_column = filtered_transactions.iter().any(|tx| tx.has_data());

    // Define table headers dynamically
    let mut header_cells = vec!["Time", "Hash", "From", "To", "Value (ETH)", "Function"];
    if show_data_column {
        header_cells.push("Data");
    }

    let headers = Row::new(header_cells)
        .style(Style::default().fg(Color::Cyan).bold())
        .bottom_margin(1);

    // Convert transactions to table rows
    let visible_height = area.height.saturating_sub(4) as usize; // Account for borders and header

    let rows: Vec<Row> = filtered_transactions
        .iter()
        .enumerate()
        .skip(scroll_state.offset)
        .take(visible_height)
        .map(|(absolute_index, &tx)| {
            // Check if this row is selected
            let is_selected = absolute_index == scroll_state.selected;
            let style = if is_selected {
                Style::default().bg(Color::DarkGray).fg(Color::White)
            } else {
                Style::default()
            };

            let function_color = if is_selected {
                Color::White  // Override color when selected for better visibility
            } else {
                crate::model::decoder::get_function_color(tx.function_name())
            };

            // Style contract creation differently
            let to_style = if tx.is_contract_creation() {
                Style::default().fg(Color::Magenta).italic()
            } else {
                Style::default()
            };

            let mut cells = vec![
                Cell::from(tx.formatted_time()),
                Cell::from(tx.short_hash().into_owned()),
                Cell::from(tx.short_from().into_owned()),
                Cell::from(tx.short_to().into_owned()).style(to_style),
                Cell::from(tx.value.as_str()),
                Cell::from(tx.function_name()).style(Style::default().fg(function_color)),
            ];

            if show_data_column {
                let data_display = if tx.short_data().len() > 10 {
                    format!("{}...", tx.short_data())
                } else {
                    tx.short_data().to_string()
                };
                cells.push(Cell::from(data_display).style(Style::default().fg(Color::DarkGray)));
            }

            Row::new(cells).style(style)
        })
        .collect();

    // Define column widths dynamically - use better allocation
    let mut widths = vec![
        Constraint::Length(8),   // Time (HH:MM:SS)
        Constraint::Length(15),  // Hash (0x123...abc)
        Constraint::Length(15),  // From (0x123...abc)
        Constraint::Length(20),  // To (0x123...abc or "Contract Creation")
        Constraint::Min(10),     // Value (flexible for different ETH amounts)
        Constraint::Min(15),     // Function (flexible for function names)
    ];

    if show_data_column {
        widths.push(Constraint::Min(10));  // Data
    }

    // Create title with filter indicator
    let title = if state.filter.has_query() {
        format!(
            " Transactions [{}/{}] (Filtered: {}/{}) [Filter: {}] ",
            if filtered_transactions.is_empty() { 0 } else { scroll_state.selected + 1 },
            filtered_transactions.len(),
            filtered_transactions.len(),
            state.transactions.len(),
            state.filter.query()
        )
    } else {
        format!(
            " Transactions [{}/{}] (Showing {}-{}) ",
            if filtered_transactions.is_empty() { 0 } else { scroll_state.selected + 1 },
            filtered_transactions.len(),
            if filtered_transactions.is_empty() { 0 } else { scroll_state.offset + 1 },
            (scroll_state.offset + visible_height).min(filtered_transactions.len())
        )
    };

    // Create the table
    let table = Table::new(rows, widths)
        .header(headers)
        .block(
            Block::default()
                .title(title)
                .borders(Borders::ALL)
                .border_style(Style::default().fg(
                    if state.filter.has_query() { Color::Yellow } else { Color::DarkGray }
                )),
        )
        .column_spacing(1);

    frame.render_widget(table, area);
}