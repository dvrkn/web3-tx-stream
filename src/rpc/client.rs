use alloy::providers::{Provider, ProviderBuilder, WsConnect};
use alloy::rpc::types::Transaction as AlloyTransaction;
use anyhow::{Context, Result};
use tokio::sync::mpsc;

pub struct RpcClient {
    rpc_url: String,
}

impl RpcClient {
    pub async fn connect(url: &str) -> Result<Self> {
        Ok(Self {
            rpc_url: url.to_string(),
        })
    }

    pub async fn subscribe_pending_txs(&self) -> Result<mpsc::UnboundedReceiver<crate::model::Transaction>> {
        let (tx, rx) = mpsc::unbounded_channel();

        // Create a new provider for the subscription
        let ws = WsConnect::new(&self.rpc_url);
        let provider = ProviderBuilder::new()
            .on_ws(ws)
            .await
            .context("Failed to connect to WebSocket")?;

        // Spawn a task to handle subscriptions
        tokio::spawn(async move {
            // Subscribe to pending transactions
            match provider.subscribe_pending_transactions().await {
                Ok(mut sub) => {
                    loop {
                        match sub.recv().await {
                            Ok(tx_hash) => {
                                // Fetch full transaction details
                                if let Ok(Some(tx_data)) = provider.get_transaction_by_hash(tx_hash).await {
                                    if let Ok(parsed_tx) = parse_transaction(tx_data) {
                                        let _ = tx.send(parsed_tx);
                                    }
                                }
                            }
                            Err(_) => {
                                // Subscription error - connection likely dropped
                                break;
                            }
                        }
                    }
                }
                Err(_) => {
                    // Failed to subscribe - will be handled by caller
                }
            }
        });

        Ok(rx)
    }
}

fn parse_transaction(tx: AlloyTransaction) -> Result<crate::model::Transaction> {
    use crate::model::Transaction;

    let hash = format!("{:#x}", tx.hash);
    let from = format!("{:#x}", tx.from);
    let to = tx.to.map(|addr| format!("{:#x}", addr));

    // Handle value field
    let value = format_ether(tx.value);

    // Get gas limit
    let gas_limit = tx.gas.to_string();

    // Get gas price (might be None for EIP-1559 txs)
    let gas_price = tx.gas_price.map(|p| p.to_string());

    // Get input data
    let data = format!("0x{}", hex::encode(tx.input.as_ref()));

    let function_sig = crate::model::decoder::decode_function(&data);
    let timestamp = chrono::Utc::now().timestamp();

    Ok(Transaction {
        hash,
        from,
        to,
        value,
        gas_limit,
        gas_price,
        data,
        function_sig,
        timestamp,
    })
}

fn format_ether(wei: alloy::primitives::U256) -> String {
    // Convert wei to ether (1 ether = 10^18 wei)
    const WEI_PER_ETHER: u128 = 1_000_000_000_000_000_000;

    // Convert U256 to u128 (safe for most transaction values)
    let wei_u128 = wei.to::<u128>();

    if wei_u128 == 0 {
        return "0.0000".to_string();
    }

    let ether = wei_u128 / WEI_PER_ETHER;
    let remainder = wei_u128 % WEI_PER_ETHER;

    // Get first 6 decimal places for better precision
    let decimal_part = (remainder * 1_000_000) / WEI_PER_ETHER;

    // Format with appropriate precision
    if ether > 0 {
        format!("{}.{:04}", ether, decimal_part / 100) // Show 4 decimals for large values
    } else {
        format!("0.{:06}", decimal_part) // Show 6 decimals for small values
    }
}