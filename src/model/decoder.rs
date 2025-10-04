use once_cell::sync::Lazy;
use std::collections::HashMap;

use super::FunctionSignature;

// Common function signatures on Ethereum-compatible chains
static SIGNATURES: Lazy<HashMap<&'static str, &'static str>> = Lazy::new(|| {
    let mut m = HashMap::new();

    // ERC20 functions
    m.insert("0xa9059cbb", "transfer");
    m.insert("0x23b872dd", "transferFrom");
    m.insert("0x095ea7b3", "approve");
    m.insert("0xdd62ed3e", "allowance");
    m.insert("0x70a08231", "balanceOf");
    m.insert("0x18160ddd", "totalSupply");

    // Uniswap/DEX functions
    m.insert("0x38ed1739", "swapExactTokensForTokens");
    m.insert("0x7ff36ab5", "swapExactETHForTokens");
    m.insert("0x18cbafe5", "swapExactTokensForETH");
    m.insert("0x4a25d94a", "swapTokensForExactETH");
    m.insert("0xfb3bdb41", "swapETHForExactTokens");
    m.insert("0x5c11d795", "swapExactTokensForTokensSupportingFeeOnTransferTokens");
    m.insert("0xb6f9de95", "swapExactETHForTokensSupportingFeeOnTransferTokens");
    m.insert("0x791ac947", "swapExactTokensForETHSupportingFeeOnTransferTokens");

    // Liquidity functions
    m.insert("0xe8e33700", "addLiquidity");
    m.insert("0xf305d719", "addLiquidityETH");
    m.insert("0xbaa2abde", "removeLiquidity");
    m.insert("0x02751cec", "removeLiquidityETH");
    m.insert("0xaf2979eb", "removeLiquidityETHSupportingFeeOnTransferTokens");
    m.insert("0xded9382a", "removeLiquidityETHWithPermit");
    m.insert("0x2195995c", "removeLiquidityWithPermit");

    // NFT functions
    m.insert("0x42842e0e", "safeTransferFrom");
    m.insert("0xb88d4fde", "safeTransferFromWithData");
    m.insert("0x23b872dd", "transferFrom"); // Same as ERC20
    m.insert("0x6352211e", "ownerOf");
    m.insert("0x081812fc", "getApproved");
    m.insert("0xa22cb465", "setApprovalForAll");
    m.insert("0xe985e9c5", "isApprovedForAll");
    m.insert("0x40c10f19", "mint");
    m.insert("0x42966c68", "burn");

    // WETH functions
    m.insert("0xd0e30db0", "deposit");
    m.insert("0x2e1a7d4d", "withdraw");

    // Multicall
    m.insert("0xac9650d8", "multicall");
    m.insert("0x5ae401dc", "multicallWithDeadline");

    // Bridge functions
    m.insert("0x3ceda011", "bridgeETH");
    m.insert("0xd92d0bd7", "bridgeERC20");
    m.insert("0x8eb388f3", "bridgeNativeToken");

    // Staking functions
    m.insert("0xa694fc3a", "stake");
    m.insert("0x2e17de78", "unstake");
    m.insert("0x3d18b912", "getReward");
    m.insert("0xe9fad8ee", "exit");
    m.insert("0x379607f5", "claim");

    // Governance
    m.insert("0x15373e3d", "castVote");
    m.insert("0x56781388", "castVoteWithReason");
    m.insert("0x7b3c71d3", "castVoteWithReasonAndParams");
    m.insert("0xc9d27afe", "castVoteBySig");
    m.insert("0xea0217cf", "propose");
    m.insert("0x40e58ee5", "cancel");
    m.insert("0xfe0d94c1", "execute");
    m.insert("0x2656227d", "queue");

    // Other common functions
    m.insert("0x3ccfd60b", "withdraw");
    m.insert("0x1249c58b", "mint");
    m.insert("0x853828b6", "withdrawAll");
    m.insert("0x1cff79cd", "execute");
    m.insert("0x9059cbb2", "transfer");

    m
});

/// Decode a function signature from transaction data
pub fn decode_function(data: &str) -> Option<FunctionSignature> {
    // Check if data is long enough to contain a function selector
    if data.len() < 10 {
        return None;
    }

    // Extract the function selector (first 4 bytes = 8 hex chars + 0x)
    let selector = &data[0..10];

    // Look up the function name
    SIGNATURES.get(selector).map(|name| FunctionSignature {
        selector: selector.to_string(),
        name: name.to_string(),
    })
}

/// Get a color for a function based on its type
pub fn get_function_color(function_name: &str) -> ratatui::style::Color {
    use ratatui::style::Color;

    match function_name {
        // Transfers - green
        "transfer" | "transferFrom" | "safeTransferFrom" => Color::Green,

        // Swaps/Trading - blue
        name if name.contains("swap") => Color::Blue,

        // Liquidity - cyan
        name if name.contains("Liquidity") => Color::Cyan,

        // Approvals - yellow
        "approve" | "setApprovalForAll" => Color::Yellow,

        // Minting - magenta
        "mint" | "deposit" => Color::Magenta,

        // Withdrawals/Burns - red
        "withdraw" | "withdrawAll" | "burn" | "exit" => Color::Red,

        // Bridge - light blue
        name if name.contains("bridge") => Color::LightBlue,

        // Staking - light green
        "stake" | "unstake" | "getReward" | "claim" => Color::LightGreen,

        // Governance - light yellow
        name if name.contains("Vote") || name == "propose" || name == "execute" => Color::LightYellow,

        // Unknown - gray
        _ => Color::Gray,
    }
}