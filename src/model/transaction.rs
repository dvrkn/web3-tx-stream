use serde::{Deserialize, Serialize};
use std::borrow::Cow;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Transaction {
    pub hash: String,
    pub from: String,
    pub to: Option<String>,
    pub value: String, // ETH value
    pub gas_limit: String,
    pub gas_price: Option<String>,
    pub data: String,
    pub function_sig: Option<FunctionSignature>,
    pub timestamp: i64,
    // Receipt data (populated when fetching by hash or viewing details)
    pub block_number: Option<u64>,
    pub status: Option<bool>, // true = success, false = failed
    pub gas_used: Option<String>,
    pub effective_gas_price: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FunctionSignature {
    pub selector: String,
    pub name: String,
}

impl Transaction {
    /// Get a shortened version of the hash for display (avoids allocation when possible)
    pub fn short_hash(&self) -> Cow<'_, str> {
        if self.hash.len() > 10 {
            Cow::Owned(format!("{}...{}", &self.hash[0..6], &self.hash[self.hash.len()-4..]))
        } else {
            Cow::Borrowed(&self.hash)
        }
    }

    /// Get a shortened version of the from address for display
    pub fn short_from(&self) -> Cow<'_, str> {
        if self.from.len() > 10 {
            Cow::Owned(format!("{}...{}", &self.from[0..6], &self.from[self.from.len()-4..]))
        } else {
            Cow::Borrowed(&self.from)
        }
    }

    /// Get a shortened version of the to address for display
    pub fn short_to(&self) -> Cow<'_, str> {
        match &self.to {
            Some(addr) if addr.len() > 10 => {
                Cow::Owned(format!("{}...{}", &addr[0..6], &addr[addr.len()-4..]))
            }
            Some(addr) => Cow::Borrowed(addr.as_str()),
            None => Cow::Borrowed("Contract Creation"),
        }
    }

    /// Get the function name or "Unknown" if not decoded
    pub fn function_name(&self) -> &str {
        self.function_sig
            .as_ref()
            .map(|sig| sig.name.as_str())
            .unwrap_or("Unknown")
    }

    /// Format the timestamp as a human-readable string
    pub fn formatted_time(&self) -> String {
        use chrono::{DateTime, Local, TimeZone, Utc};

        let dt = Utc.timestamp_opt(self.timestamp, 0).unwrap();
        let local_dt: DateTime<Local> = DateTime::from(dt);
        local_dt.format("%H:%M:%S").to_string()
    }

    /// Get a short representation of the data field
    pub fn short_data(&self) -> &str {
        if self.data == "0x" || self.data.is_empty() {
            "-"
        } else if self.data.len() > 10 {
            // Return a slice instead of allocating new string
            // Note: For display purposes, we'll handle truncation in the UI
            &self.data[0..10.min(self.data.len())]
        } else {
            &self.data
        }
    }

    /// Check if transaction has data
    #[inline]
    pub fn has_data(&self) -> bool {
        self.data != "0x" && !self.data.is_empty()
    }

    /// Check if this is a contract creation
    #[inline]
    pub fn is_contract_creation(&self) -> bool {
        self.to.is_none()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_has_data() {
        let mut tx = Transaction {
            hash: "0x123".to_string(),
            from: "0x456".to_string(),
            to: Some("0x789".to_string()),
            value: "0.5".to_string(),
            gas_limit: "21000".to_string(),
            gas_price: Some("30".to_string()),
            data: "0x".to_string(),
            function_sig: None,
            timestamp: 0,
            block_number: None,
            status: None,
            gas_used: None,
            effective_gas_price: None,
        };

        // Empty data
        assert!(!tx.has_data());

        // Has data
        tx.data = "0xa9059cbb000000000000000000000000".to_string();
        assert!(tx.has_data());

        // Empty string
        tx.data = "".to_string();
        assert!(!tx.has_data());
    }

    #[test]
    fn test_short_methods_no_allocation() {
        let tx = Transaction {
            hash: "0x123".to_string(),
            from: "0x456".to_string(),
            to: Some("0x789".to_string()),
            value: "0.5".to_string(),
            gas_limit: "21000".to_string(),
            gas_price: Some("30".to_string()),
            data: "0x".to_string(),
            function_sig: None,
            timestamp: 0,
            block_number: None,
            status: None,
            gas_used: None,
            effective_gas_price: None,
        };

        // These should not allocate for short strings
        assert!(matches!(tx.short_hash(), Cow::Borrowed(_)));
        assert!(matches!(tx.short_from(), Cow::Borrowed(_)));
        assert!(matches!(tx.short_to(), Cow::Borrowed(_)));
    }
}