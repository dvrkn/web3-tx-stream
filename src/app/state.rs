use crate::model::Transaction;
use std::collections::VecDeque;
use std::time::Instant;

const DEFAULT_MAX_TRANSACTIONS: usize = 1000;
const VECDEQUE_SHRINK_THRESHOLD: usize = 2000; // Shrink if capacity exceeds this

pub struct AppState {
    pub transactions: VecDeque<Transaction>,
    pub max_transactions: usize,
    pub scroll_state: ScrollState,
    pub stats: Stats,
    pub config: Config,
    pub should_quit: bool,
    pub show_new_on_top: bool,
    pub show_details: bool,
    pub selected_transaction: Option<Transaction>,
    pub details_scroll_offset: usize,
}

pub struct ScrollState {
    pub offset: usize,
    pub selected: usize,
}

pub struct Stats {
    pub total_transactions: u64,
    pub start_time: i64,
    pub connected: bool,
    pub last_error: Option<String>,
    pub transactions_per_second: f32,
    pub memory_usage_mb: f32,
    pub last_perf_update: Instant,
}

#[derive(Clone)]
pub struct Config {
    pub rpc_url: String,
    pub reconnect_attempts: u32,
    pub reconnect_delay: u64,
    pub max_transactions: usize,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            rpc_url: std::env::var("BASE_RPC_URL")
                .unwrap_or_else(|_| "wss://base-rpc.publicnode.com".to_string()),
            reconnect_attempts: 10,
            reconnect_delay: 5000,
            max_transactions: DEFAULT_MAX_TRANSACTIONS,
        }
    }
}

impl Config {
    pub fn load() -> anyhow::Result<Self> {
        let mut config = Self::default();

        // Override from environment variables if present
        if let Ok(max_tx) = std::env::var("MAX_TRANSACTIONS") {
            config.max_transactions = max_tx.parse().unwrap_or(DEFAULT_MAX_TRANSACTIONS);
        }

        if let Ok(attempts) = std::env::var("RECONNECT_ATTEMPTS") {
            config.reconnect_attempts = attempts.parse().unwrap_or(10);
        }

        if let Ok(delay) = std::env::var("RECONNECT_DELAY_MS") {
            config.reconnect_delay = delay.parse().unwrap_or(5000);
        }

        Ok(config)
    }
}

impl AppState {
    pub fn new(config: Config) -> Self {
        let max_transactions = config.max_transactions;
        Self {
            transactions: VecDeque::with_capacity(max_transactions),
            max_transactions,
            scroll_state: ScrollState {
                offset: 0,
                selected: 0,
            },
            stats: Stats {
                total_transactions: 0,
                start_time: chrono::Utc::now().timestamp(),
                connected: false,
                last_error: None,
                transactions_per_second: 0.0,
                memory_usage_mb: 0.0,
                last_perf_update: Instant::now(),
            },
            config,
            should_quit: false,
            show_new_on_top: true, // Default to showing new transactions on top
            show_details: false,
            selected_transaction: None,
            details_scroll_offset: 0,
        }
    }

    pub fn add_transaction(&mut self, tx: Transaction) {
        if self.show_new_on_top {
            // Add new transactions at the front
            if self.transactions.len() >= self.max_transactions {
                self.transactions.pop_back(); // Remove oldest from back
            }
            self.transactions.push_front(tx);

            // When adding to front, shift selection down if not at top
            if !self.transactions.is_empty() && self.scroll_state.selected > 0 {
                self.scroll_state.selected += 1;
                self.scroll_state.offset = self.scroll_state.offset.saturating_add(1);
            }
        } else {
            // Add new transactions at the back (original behavior)
            if self.transactions.len() >= self.max_transactions {
                self.transactions.pop_front();
                // Adjust scroll position if we removed a transaction before the current view
                if self.scroll_state.selected > 0 {
                    self.scroll_state.selected = self.scroll_state.selected.saturating_sub(1);
                }
                if self.scroll_state.offset > 0 {
                    self.scroll_state.offset = self.scroll_state.offset.saturating_sub(1);
                }
            }
            self.transactions.push_back(tx);
        }

        self.stats.total_transactions += 1;

        // Optimize memory: shrink VecDeque if capacity is too large
        if self.transactions.capacity() > VECDEQUE_SHRINK_THRESHOLD &&
           self.transactions.len() < self.max_transactions / 2 {
            self.transactions.shrink_to_fit();
        }

        // Update performance metrics
        self.update_performance_stats();
    }

    fn update_performance_stats(&mut self) {
        let now = Instant::now();
        if now.duration_since(self.stats.last_perf_update).as_secs() >= 1 {
            // Calculate transactions per second
            let _elapsed = now.duration_since(self.stats.last_perf_update).as_secs_f32();
            let current_time = chrono::Utc::now().timestamp();
            let runtime = (current_time - self.stats.start_time) as f32;

            if runtime > 0.0 {
                self.stats.transactions_per_second = self.stats.total_transactions as f32 / runtime;
            }

            // Estimate memory usage (rough approximation)
            let tx_size = std::mem::size_of::<Transaction>() + 500; // Estimate avg string data
            let total_bytes = self.transactions.len() * tx_size;
            self.stats.memory_usage_mb = total_bytes as f32 / (1024.0 * 1024.0);

            self.stats.last_perf_update = now;
        }
    }

    pub fn scroll_up(&mut self) {
        if self.scroll_state.selected > 0 {
            self.scroll_state.selected = self.scroll_state.selected.saturating_sub(1);

            // Adjust offset if needed
            if self.scroll_state.selected < self.scroll_state.offset {
                self.scroll_state.offset = self.scroll_state.selected;
            }
        }
    }

    pub fn scroll_down(&mut self) {
        let max_selected = self.transactions.len().saturating_sub(1);
        if self.scroll_state.selected < max_selected {
            self.scroll_state.selected = (self.scroll_state.selected + 1).min(max_selected);

            // Adjust offset if needed (assuming viewport of ~20 items)
            let viewport_height = 20;
            if self.scroll_state.selected >= self.scroll_state.offset + viewport_height {
                self.scroll_state.offset = self.scroll_state.selected.saturating_sub(viewport_height - 1);
            }
        }
    }

    pub fn page_up(&mut self) {
        let page_size = 10;
        for _ in 0..page_size {
            self.scroll_up();
        }
    }

    pub fn page_down(&mut self) {
        let page_size = 10;
        for _ in 0..page_size {
            self.scroll_down();
        }
    }

    pub fn jump_to_top(&mut self) {
        self.scroll_state.offset = 0;
        self.scroll_state.selected = 0;
    }

    pub fn jump_to_bottom(&mut self) {
        let max_selected = self.transactions.len().saturating_sub(1);
        self.scroll_state.selected = max_selected;

        // Adjust offset to show the last page
        let viewport_height = 20;
        self.scroll_state.offset = max_selected.saturating_sub(viewport_height - 1);
    }


    pub fn set_connected(&mut self, connected: bool) {
        self.stats.connected = connected;
        if connected {
            self.stats.last_error = None;
        }
    }

    pub fn set_error(&mut self, error: String) {
        self.stats.last_error = Some(error);
        self.stats.connected = false;
    }


    pub fn quit(&mut self) {
        self.should_quit = true;
    }

    pub fn toggle_sort_order(&mut self) {
        self.show_new_on_top = !self.show_new_on_top;

        // Reverse the transaction order
        let mut temp: Vec<Transaction> = self.transactions.drain(..).collect();
        temp.reverse();
        self.transactions.extend(temp);

        // Reset scroll position
        self.scroll_state.offset = 0;
        self.scroll_state.selected = 0;
    }

    pub fn clear_transactions(&mut self) {
        self.transactions.clear();
        self.scroll_state.offset = 0;
        self.scroll_state.selected = 0;
        self.selected_transaction = None;
        self.show_details = false;
    }

    pub fn show_transaction_details(&mut self) {
        if let Some(tx) = self.transactions.get(self.scroll_state.selected) {
            // Debug: Write transaction info to file
            #[cfg(debug_assertions)]
            if std::env::var("DEBUG_MODE").unwrap_or_default() == "1" {
                use std::io::Write;
                if let Ok(mut file) = std::fs::OpenOptions::new()
                    .create(true)
                    .append(true)
                    .open("/tmp/tx_debug.log") {
                    let _ = writeln!(file, "Selected index: {}, has_data: {}, data: {:?}",
                        self.scroll_state.selected, tx.has_data(), &tx.data[..tx.data.len().min(50)]);
                }
            }
            self.selected_transaction = Some(tx.clone());
            self.show_details = true;
            self.details_scroll_offset = 0; // Reset scroll when opening details
        }
    }

    pub fn hide_transaction_details(&mut self) {
        self.show_details = false;
        self.selected_transaction = None;
        self.details_scroll_offset = 0; // Reset scroll when closing
    }

    pub fn scroll_details_up(&mut self) {
        self.details_scroll_offset = self.details_scroll_offset.saturating_sub(1);
    }

    pub fn scroll_details_down(&mut self) {
        self.details_scroll_offset = self.details_scroll_offset.saturating_add(1);
    }

    pub fn scroll_details_page_up(&mut self) {
        self.details_scroll_offset = self.details_scroll_offset.saturating_sub(10);
    }

    pub fn scroll_details_page_down(&mut self) {
        self.details_scroll_offset = self.details_scroll_offset.saturating_add(10);
    }
}