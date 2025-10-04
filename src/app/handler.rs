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
    if state.show_details {
        handle_details_navigation(key, state)
    } else {
        handle_main_navigation(key, state)
    }
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

/// Handle navigation in main transaction list
fn handle_main_navigation(key: KeyEvent, state: &mut AppState) -> Result<()> {
    use KeyCode::*;

    match key.code {
        // Quit application
        Char('q') | Esc => state.quit(),
        Char('c') if key.modifiers.contains(KeyModifiers::CONTROL) => state.quit(),

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