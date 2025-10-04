use crate::model::Transaction;

/// Filter state management - Single Responsibility: Managing filter state and logic
#[derive(Debug, Clone, Default)]
pub struct FilterState {
    /// Current filter query
    query: String,
    /// Whether filter mode is active
    active: bool,
    /// Cursor position in the input
    cursor_position: usize,
}

impl FilterState {
    pub fn new() -> Self {
        Self::default()
    }

    /// Activate filter mode
    pub fn activate(&mut self) {
        self.active = true;
        self.cursor_position = self.query.len();
    }

    /// Deactivate filter mode
    pub fn deactivate(&mut self) {
        self.active = false;
    }

    /// Clear the filter
    pub fn clear(&mut self) {
        self.query.clear();
        self.cursor_position = 0;
    }

    /// Check if filter is active
    pub fn is_active(&self) -> bool {
        self.active
    }

    /// Check if filter has a query
    pub fn has_query(&self) -> bool {
        !self.query.is_empty()
    }

    /// Get the current query
    pub fn query(&self) -> &str {
        &self.query
    }

    /// Get cursor position
    pub fn cursor_position(&self) -> usize {
        self.cursor_position
    }

    /// Add a character at cursor position
    pub fn add_char(&mut self, c: char) {
        self.query.insert(self.cursor_position, c);
        self.cursor_position += 1;
    }

    /// Remove character before cursor (backspace)
    pub fn delete_char_before_cursor(&mut self) {
        if self.cursor_position > 0 {
            self.query.remove(self.cursor_position - 1);
            self.cursor_position -= 1;
        }
    }

    /// Remove character at cursor (delete)
    pub fn delete_char_at_cursor(&mut self) {
        if self.cursor_position < self.query.len() {
            self.query.remove(self.cursor_position);
        }
    }

    /// Move cursor left
    pub fn move_cursor_left(&mut self) {
        self.cursor_position = self.cursor_position.saturating_sub(1);
    }

    /// Move cursor right
    pub fn move_cursor_right(&mut self) {
        self.cursor_position = (self.cursor_position + 1).min(self.query.len());
    }

    /// Move cursor to start
    pub fn move_cursor_to_start(&mut self) {
        self.cursor_position = 0;
    }

    /// Move cursor to end
    pub fn move_cursor_to_end(&mut self) {
        self.cursor_position = self.query.len();
    }

    /// Check if the query looks like a transaction hash
    pub fn is_transaction_hash(&self) -> bool {
        // Transaction hash is 0x followed by 64 hex characters
        if self.query.len() == 66 && self.query.starts_with("0x") {
            // Check if the rest are valid hex characters
            self.query[2..].chars().all(|c| c.is_ascii_hexdigit())
        } else {
            false
        }
    }

    /// Check if a transaction matches the filter
    pub fn matches(&self, transaction: &Transaction) -> bool {
        if self.query.is_empty() {
            return true;
        }

        let query_lower = self.query.to_lowercase();

        // Check if query matches transaction hash exactly (for hash searches)
        if transaction.hash.to_lowercase() == query_lower {
            return true;
        }

        // Check if query matches from address
        if transaction.from.to_lowercase().contains(&query_lower) {
            return true;
        }

        // Check if query matches to address
        if let Some(to) = &transaction.to {
            if to.to_lowercase().contains(&query_lower) {
                return true;
            }
        }

        // Check if query partially matches hash (for non-exact hash searches)
        if !self.is_transaction_hash() && transaction.hash.to_lowercase().contains(&query_lower) {
            return true;
        }

        false
    }
}

/// Filter statistics for UI display
#[allow(dead_code)]
pub struct FilterStats {
    pub total_transactions: usize,
    pub filtered_transactions: usize,
}

#[allow(dead_code)]
impl FilterStats {
    pub fn new(total: usize, filtered: usize) -> Self {
        Self {
            total_transactions: total,
            filtered_transactions: filtered,
        }
    }

    pub fn display_text(&self) -> String {
        if self.filtered_transactions == self.total_transactions {
            format!("{} transactions", self.total_transactions)
        } else {
            format!("{}/{} transactions", self.filtered_transactions, self.total_transactions)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_filter_matches() {
        let filter = FilterState {
            query: "0x123".to_string(),
            active: true,
            cursor_position: 5,
        };

        let tx = Transaction {
            hash: "0xabc".to_string(),
            from: "0x123456".to_string(),
            to: Some("0x789".to_string()),
            value: "1.0".to_string(),
            gas_limit: "21000".to_string(),
            gas_price: Some("30".to_string()),
            data: "0x".to_string(),
            function_sig: None,
            timestamp: 0,
        };

        assert!(filter.matches(&tx));
    }

    #[test]
    fn test_filter_input_operations() {
        let mut filter = FilterState::new();

        filter.add_char('0');
        filter.add_char('x');
        assert_eq!(filter.query(), "0x");
        assert_eq!(filter.cursor_position(), 2);

        filter.move_cursor_left();
        filter.add_char('_');
        assert_eq!(filter.query(), "0_x");

        filter.delete_char_before_cursor();
        assert_eq!(filter.query(), "0x");
    }
}