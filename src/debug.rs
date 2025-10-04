use crate::model::{FunctionSignature, Transaction};

pub fn create_sample_transactions(count: usize) -> Vec<Transaction> {
    let mut transactions = Vec::new();
    let functions = [("transfer", "0xa9059cbb"),
        ("swap", "0x38ed1739"),
        ("approve", "0x095ea7b3"),
        ("mint", "0x40c10f19"),
        ("burn", "0x42966c68")];

    for i in 0..count {
        let (name, selector) = &functions[i % functions.len()];

        // Generate more realistic transaction data
        let data = if i % 3 == 0 {
            // Some transactions have no data (simple ETH transfers)
            "0x".to_string()
        } else {
            // Generate function call data with parameters
            format!("{}000000000000000000000000{:040x}00000000000000000000000000000000000000000000000000000000{:08x}",
                selector, i * 7, i * 1000)
        };

        transactions.push(Transaction {
            hash: format!("0x{:064x}", i),
            from: format!("0x{:040x}", i * 2),
            to: Some(format!("0x{:040x}", i * 3)),
            value: format!("{:.4}", i as f64 * 0.001),
            gas_limit: format!("{}", 21000 + i * 100),
            gas_price: Some(format!("{}", 30 + i)),
            data: data.clone(),
            function_sig: if data.len() > 10 {
                Some(FunctionSignature {
                    selector: selector.to_string(),
                    name: name.to_string(),
                })
            } else {
                None
            },
            timestamp: chrono::Utc::now().timestamp() - (count - i) as i64,
            block_number: None,
            status: None,
            gas_used: None,
            effective_gas_price: None,
        });
    }
    transactions
}