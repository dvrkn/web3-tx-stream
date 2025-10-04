pub mod details;
pub mod footer;
pub mod header;
pub mod list;

use crate::app::AppState;
use ratatui::prelude::*;
use ratatui::Frame;

pub fn render_ui(frame: &mut Frame, state: &AppState) {
    // Create main layout
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3),  // Header
            Constraint::Min(10),    // Transaction list
            Constraint::Length(4),  // Footer (3 lines + border for status)
        ])
        .split(frame.area());

    // Render components
    header::render_header(frame, chunks[0], &state.stats, &state.config);
    list::render_transaction_list(
        frame,
        chunks[1],
        &state.transactions,
        &state.scroll_state,
    );
    footer::render_footer(frame, chunks[2], state);

    // Render transaction details popup if active
    if state.show_details {
        if let Some(ref tx) = state.selected_transaction {
            details::render_transaction_details(frame, tx, state.details_scroll_offset);
        }
    }
}