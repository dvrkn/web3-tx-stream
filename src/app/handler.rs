use crate::app::AppState;
use crate::model::Transaction;
use anyhow::Result;
use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

pub enum AppEvent {
    Input(KeyEvent),
    Transaction(Transaction),
    Connected,
    Disconnected(String),
}

impl AppEvent {
    /// Process the event and update application state
    pub async fn process(self, state: &mut AppState) -> Result<()> {
        match self {
            Self::Input(key) => handle_key_event(key, state),
            Self::Transaction(tx) => {
                state.add_transaction(tx);
                Ok(())
            }
            Self::Connected => {
                state.set_connected(true);
                Ok(())
            }
            Self::Disconnected(error) => {
                state.set_error(error);
                Ok(())
            }
        }
    }
}

/// Handle keyboard events - Single Responsibility: keyboard input processing
fn handle_key_event(key: KeyEvent, state: &mut AppState) -> Result<()> {
    // Priority order: quit confirmation > filter mode > details view > main navigation
    if state.quit_confirmation {
        handle_quit_confirmation(key, state)
    } else if state.filter.is_active() {
        handle_filter_input(key, state)
    } else if state.show_details {
        handle_details_navigation(key, state)
    } else {
        handle_main_navigation(key, state)
    }
}

/// Handle quit confirmation dialog
fn handle_quit_confirmation(key: KeyEvent, state: &mut AppState) -> Result<()> {
    use KeyCode::*;

    match key.code {
        Char('y') | Char('Y') => state.quit(),
        Char('n') | Char('N') | Esc => state.quit_confirmation = false,
        _ => {}
    }
    Ok(())
}

/// Handle navigation when details view is active
fn handle_details_navigation(key: KeyEvent, state: &mut AppState) -> Result<()> {
    use KeyCode::*;

    match key.code {
        // Close details
        Esc | Enter | Char('q') => state.hide_transaction_details(),

        // Vertical scrolling
        Up | Char('k') => state.scroll_details_up(),
        Down | Char('j') => state.scroll_details_down(),

        // Page navigation
        PageUp => state.scroll_details_page_up(),
        PageDown => state.scroll_details_page_down(),

        // Jump to top
        Home | Char('g') => state.details_scroll_offset = 0,

        _ => {}
    }
    Ok(())
}

/// Handle filter input mode
fn handle_filter_input(key: KeyEvent, state: &mut AppState) -> Result<()> {
    use KeyCode::*;

    match key.code {
        // Clear filter and exit filter mode on Escape
        Esc => {
            state.filter.clear();
            state.filter.deactivate();
            // Reset scroll position
            state.scroll_state.offset = 0;
            state.scroll_state.selected = 0;
        }

        // Submit filter (keep it active)
        Enter => {
            state.filter.deactivate();
            // Reset scroll position when filter is applied
            state.scroll_state.offset = 0;
            state.scroll_state.selected = 0;
        }

        // Character input
        Char(c) => state.filter.add_char(c),

        // Editing
        Backspace => state.filter.delete_char_before_cursor(),
        Delete => state.filter.delete_char_at_cursor(),

        // Cursor movement
        Left => state.filter.move_cursor_left(),
        Right => state.filter.move_cursor_right(),
        Home => state.filter.move_cursor_to_start(),
        End => state.filter.move_cursor_to_end(),

        _ => {}
    }
    Ok(())
}

/// Handle navigation in main transaction list
fn handle_main_navigation(key: KeyEvent, state: &mut AppState) -> Result<()> {
    use KeyCode::*;

    match key.code {
        // Show quit confirmation
        Char('q') => state.quit_confirmation = true,
        // Escape behavior depends on filter status
        Esc => {
            if state.filter.has_query() {
                // If filter is active, clear it
                state.filter.clear();
                state.filter.deactivate();
                state.scroll_state.offset = 0;
                state.scroll_state.selected = 0;
            } else {
                // If no filter, show quit confirmation
                state.quit_confirmation = true;
            }
        }
        Char('c') if key.modifiers.contains(KeyModifiers::CONTROL) => state.quit_confirmation = true,

        // Activate filter mode
        Char('/') => {
            state.filter.activate();
        }

        // Clear filter
        Char('\\') => {
            state.filter.clear();
            state.filter.deactivate();
            // Reset scroll position
            state.scroll_state.offset = 0;
            state.scroll_state.selected = 0;
        }

        // Vertical scrolling
        Up | Char('k') => state.scroll_up(),
        Down | Char('j') => state.scroll_down(),

        // Page navigation
        PageUp => state.page_up(),
        PageDown => state.page_down(),

        // Jump navigation
        Home | Char('g') => state.jump_to_top(),
        End | Char('G') => state.jump_to_bottom(),

        // Actions
        Char('r') => state.set_connected(false), // Trigger reconnect
        Char('t') => state.toggle_sort_order(),
        Char('c') => state.clear_transactions(),
        Char('C') if key.modifiers.contains(KeyModifiers::SHIFT) => state.clear_transactions(),
        Enter => state.show_transaction_details(),

        _ => {}
    }
    Ok(())
}

// Public interface for handling events
pub async fn handle_event(event: AppEvent, state: &mut AppState) -> Result<()> {
    event.process(state).await
}