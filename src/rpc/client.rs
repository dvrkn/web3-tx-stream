use alloy::providers::{Provider, ProviderBuilder, WsConnect};
use alloy::rpc::types::{Transaction as AlloyTransaction, TransactionReceipt};
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

    /// Fetch a specific transaction by hash with receipt data
    pub async fn fetch_transaction_by_hash(&self, tx_hash: &str) -> Result<Option<crate::model::Transaction>> {
        // Create provider
        let ws = WsConnect::new(&self.rpc_url);
        let provider = ProviderBuilder::new()
            .on_ws(ws)
            .await
            .context("Failed to connect to WebSocket")?;

        // Parse the transaction hash
        let hash = tx_hash.parse().context("Invalid transaction hash")?;

        // Try to fetch the transaction
        let tx_data = provider.get_transaction_by_hash(hash).await
            .context("Failed to fetch transaction")?;

        if let Some(tx) = tx_data {
            // Parse basic transaction data
            let mut transaction = parse_transaction(tx)?;

            // Try to fetch the receipt for additional data
            if let Ok(Some(receipt)) = provider.get_transaction_receipt(hash).await {
                transaction = enhance_with_receipt(transaction, receipt);
            }

            Ok(Some(transaction))
        } else {
            Ok(None)
        }
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
        block_number: None,
        status: None,
        gas_used: None,
        effective_gas_price: None,
    })
}

fn enhance_with_receipt(mut tx: crate::model::Transaction, receipt: TransactionReceipt) -> crate::model::Transaction {
    // Add receipt data to transaction
    tx.block_number = receipt.block_number;

    // Check status (successful if status is true/1)
    tx.status = Some(receipt.status());

    // Format gas used
    tx.gas_used = Some(receipt.gas_used.to_string());

    // Format effective gas price (it's always present in receipts)
    tx.effective_gas_price = Some(receipt.effective_gas_price.to_string());

    tx
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