mod app;
mod filter;
mod model;
mod rpc;
mod ui;

#[cfg(debug_assertions)]
mod debug;

use anyhow::Result;
use app::{handle_event, AppEvent, AppState, Config};
use crossterm::{
    event::{Event, EventStream},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use futures::StreamExt;
use ratatui::{backend::CrosstermBackend, Terminal};
use std::io;
use std::time::{Duration, Instant};
use tokio::sync::mpsc;
use tokio::time::{interval, sleep};

// Performance tuning constants
const RENDER_INTERVAL_MS: u64 = 100;
const BATCH_SIZE: usize = 10;
const BATCH_TIMEOUT_MS: u64 = 50;
const MAX_FPS: u64 = 60;
const FRAME_TIME_MS: u64 = 1000 / MAX_FPS;

#[tokio::main]
async fn main() -> Result<()> {
    let config = Config::load()?;
    let mut app_state = AppState::new(config.clone());
    let mut terminal = setup_terminal()?;

    // Initialize debug mode if enabled
    #[cfg(debug_assertions)]
    initialize_debug_mode(&mut app_state).await?;

    // Setup event channels
    let (tx_sender, tx_receiver) = mpsc::channel(1000);
    let (event_sender, event_receiver) = mpsc::unbounded_channel();

    // Spawn RPC connection task (unless in debug simulation mode)
    if std::env::var("DEBUG_MODE").unwrap_or_default() != "1" {
        spawn_rpc_task(config.rpc_url.clone(), tx_sender.clone(), event_sender.clone());
    } else {
        // Spawn debug transaction generator if in debug simulation mode
        #[cfg(debug_assertions)]
        spawn_debug_generator(tx_sender.clone());

        // Keep event_sender alive in debug mode
        let _ = event_sender;
    }

    // Run main event loop
    let result = run_event_loop(
        &mut terminal,
        &mut app_state,
        tx_receiver,
        event_receiver,
        event_sender.clone(),
        config.rpc_url.clone(),
    ).await;

    restore_terminal(&mut terminal)?;
    result
}

async fn run_event_loop(
    terminal: &mut Terminal<CrosstermBackend<io::Stdout>>,
    app_state: &mut AppState,
    mut tx_receiver: mpsc::Receiver<model::Transaction>,
    mut event_receiver: mpsc::UnboundedReceiver<AppEvent>,
    event_sender: mpsc::UnboundedSender<AppEvent>,
    rpc_url: String,
) -> Result<()> {
    let mut input_events = EventStream::new();
    let mut render_interval = interval(Duration::from_millis(RENDER_INTERVAL_MS));

    let mut render_state = RenderState::new();
    let mut tx_batch = TransactionBatch::new();

    loop {
        tokio::select! {
            // Handle keyboard input with highest priority
            Some(Ok(Event::Key(key))) = input_events.next() => {
                handle_event(AppEvent::Input(key), app_state).await?;
                render_state.request_render();

                // Check if we need to fetch a transaction
                if let Some(tx_hash) = app_state.pending_tx_fetch.take() {
                    spawn_tx_fetch_task(rpc_url.clone(), tx_hash, event_sender.clone());
                }

                if app_state.should_quit {
                    return Ok(());
                }
            }

            // Batch process new transactions
            Some(tx) = tx_receiver.recv() => {
                if let Some(batch) = tx_batch.add(tx) {
                    for transaction in batch {
                        handle_event(AppEvent::Transaction(transaction), app_state).await?;
                    }
                    render_state.request_render();
                }
            }

            // Handle app events
            Some(event) = event_receiver.recv() => {
                handle_event(event, app_state).await?;
                render_state.request_render();
            }

            // Render tick
            _ = render_interval.tick() => {
                // Process any pending transactions
                if let Some(batch) = tx_batch.flush_if_timeout() {
                    for transaction in batch {
                        handle_event(AppEvent::Transaction(transaction), app_state).await?;
                    }
                    render_state.request_render();
                }

                // Render if needed and not too frequent
                if render_state.should_render() {
                    terminal.draw(|f| ui::render_ui(f, app_state))?;
                    render_state.mark_rendered();
                }
            }
        }
    }
}

/// Manages rendering state to optimize frame rate
struct RenderState {
    needs_render: bool,
    last_render: Instant,
}

impl RenderState {
    fn new() -> Self {
        Self {
            needs_render: true,
            last_render: Instant::now(),
        }
    }

    fn request_render(&mut self) {
        self.needs_render = true;
    }

    fn should_render(&self) -> bool {
        self.needs_render && self.last_render.elapsed() >= Duration::from_millis(FRAME_TIME_MS)
    }

    fn mark_rendered(&mut self) {
        self.last_render = Instant::now();
        self.needs_render = false;
    }
}

/// Manages batching of transactions for efficient processing
struct TransactionBatch {
    batch: Vec<model::Transaction>,
    timer: Instant,
}

impl TransactionBatch {
    fn new() -> Self {
        Self {
            batch: Vec::with_capacity(BATCH_SIZE),
            timer: Instant::now(),
        }
    }

    fn add(&mut self, tx: model::Transaction) -> Option<Vec<model::Transaction>> {
        self.batch.push(tx);

        if self.batch.len() >= BATCH_SIZE || self.timer.elapsed() > Duration::from_millis(BATCH_TIMEOUT_MS) {
            self.flush()
        } else {
            None
        }
    }

    fn flush_if_timeout(&mut self) -> Option<Vec<model::Transaction>> {
        if !self.batch.is_empty() && self.timer.elapsed() > Duration::from_millis(BATCH_TIMEOUT_MS) {
            self.flush()
        } else {
            None
        }
    }

    fn flush(&mut self) -> Option<Vec<model::Transaction>> {
        if self.batch.is_empty() {
            return None;
        }
        self.timer = Instant::now();
        Some(std::mem::take(&mut self.batch))
    }
}

fn spawn_tx_fetch_task(
    rpc_url: String,
    tx_hash: String,
    event_sender: mpsc::UnboundedSender<AppEvent>,
) {
    tokio::spawn(async move {
        match rpc::RpcClient::connect(&rpc_url).await {
            Ok(client) => {
                match client.fetch_transaction_by_hash(&tx_hash).await {
                    Ok(Some(tx)) => {
                        let _ = event_sender.send(AppEvent::TransactionFetched(tx));
                    }
                    Ok(None) => {
                        let _ = event_sender.send(AppEvent::TransactionNotFound(tx_hash));
                    }
                    Err(e) => {
                        let _ = event_sender.send(AppEvent::Disconnected(
                            format!("Failed to fetch transaction: {}", e)
                        ));
                    }
                }
            }
            Err(e) => {
                let _ = event_sender.send(AppEvent::Disconnected(
                    format!("Connection error: {}", e)
                ));
            }
        }
    });
}

fn spawn_rpc_task(
    rpc_url: String,
    tx_sender: mpsc::Sender<model::Transaction>,
    event_sender: mpsc::UnboundedSender<AppEvent>,
) {
    tokio::spawn(async move {
        loop {
            let _ = event_sender.send(AppEvent::Disconnected("Connecting to RPC endpoint...".to_string()));

            match rpc::RpcClient::connect(&rpc_url).await {
                Ok(client) => {
                    let _ = event_sender.send(AppEvent::Connected);

                    match client.subscribe_pending_txs().await {
                        Ok(mut rx) => {
                            while let Some(tx) = rx.recv().await {
                                if tx_sender.send(tx).await.is_err() {
                                    break; // Main loop has exited
                                }
                            }
                        }
                        Err(e) => {
                            let _ = event_sender.send(AppEvent::Disconnected(
                                format!("Subscription error: {}", e)
                            ));
                        }
                    }
                }
                Err(e) => {
                    let _ = event_sender.send(AppEvent::Disconnected(
                        format!("Connection error: {}", e)
                    ));
                }
            }

            sleep(Duration::from_secs(5)).await;
        }
    });
}

#[cfg(debug_assertions)]
async fn initialize_debug_mode(app_state: &mut AppState) -> Result<()> {
    if std::env::var("DEBUG_MODE").unwrap_or_default() == "1" {
        for tx in debug::create_sample_transactions(50) {
            handle_event(AppEvent::Transaction(tx), app_state).await?;
        }
        app_state.set_connected(true);
    }
    Ok(())
}

#[cfg(debug_assertions)]
fn spawn_debug_generator(tx_sender: mpsc::Sender<model::Transaction>) {
    if std::env::var("DEBUG_MODE").unwrap_or_default() == "1" {
        tokio::spawn(async move {
            loop {
                sleep(Duration::from_secs(2)).await;
                for transaction in debug::create_sample_transactions(1) {
                    let _ = tx_sender.send(transaction).await;
                }
            }
        });
    }
}

fn setup_terminal() -> Result<Terminal<CrosstermBackend<io::Stdout>>> {
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen)?;
    Ok(Terminal::new(CrosstermBackend::new(stdout))?)
}

fn restore_terminal(terminal: &mut Terminal<CrosstermBackend<io::Stdout>>) -> Result<()> {
    disable_raw_mode()?;
    execute!(terminal.backend_mut(), LeaveAlternateScreen)?;
    terminal.show_cursor()?;
    Ok(())
}