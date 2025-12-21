use axum::Json;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::{Path, PathBuf};

use crate::ApiError;

/// Information about a strategy file
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StrategyInfo {
    /// Name of the strategy (filename without .lua extension)
    pub name: String,
    /// Full path to the strategy file
    pub path: String,
    /// Category (e.g., "examples" or "custom")
    pub category: String,
}

/// Response containing list of available strategies
#[derive(Debug, Serialize)]
pub struct StrategyListResponse {
    pub strategies: Vec<StrategyInfo>,
}

/// Information about a trading symbol
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SymbolInfo {
    /// Symbol ticker (e.g., "BTCUSDT")
    pub symbol: String,
    /// Human-readable name
    pub name: String,
    /// Category (crypto, stock, forex, etc.)
    pub category: String,
}

/// Response containing list of available symbols
#[derive(Debug, Serialize)]
pub struct SymbolListResponse {
    pub symbols: Vec<SymbolInfo>,
}

/// List all available Lua strategies
///
/// Scans the lua-strategies directory and returns information about all .lua files found.
pub async fn list_strategies() -> Result<Json<StrategyListResponse>, ApiError> {
    // Try to find lua-strategies directory from current dir, parent, or project root
    let base_path = if PathBuf::from("lua-strategies").exists() {
        PathBuf::from("lua-strategies")
    } else if PathBuf::from("../lua-strategies").exists() {
        PathBuf::from("../lua-strategies")
    } else if PathBuf::from("../../lua-strategies").exists() {
        PathBuf::from("../../lua-strategies")
    } else {
        // Return empty list if not found
        return Ok(Json(StrategyListResponse {
            strategies: vec![],
        }));
    };

    let mut strategies = Vec::new();

    // Scan examples directory
    let examples_path = base_path.join("examples");
    if examples_path.exists() && examples_path.is_dir() {
        if let Ok(entries) = fs::read_dir(&examples_path) {
            for entry in entries.flatten() {
                if let Some(strategy) = process_strategy_file(&entry.path(), "examples") {
                    strategies.push(strategy);
                }
            }
        }
    }

    // Scan root directory for custom strategies
    if let Ok(entries) = fs::read_dir(&base_path) {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_file() {
                if let Some(strategy) = process_strategy_file(&path, "custom") {
                    strategies.push(strategy);
                }
            }
        }
    }

    // Sort strategies by category and name
    strategies.sort_by(|a, b| {
        match a.category.cmp(&b.category) {
            std::cmp::Ordering::Equal => a.name.cmp(&b.name),
            other => other,
        }
    });

    Ok(Json(StrategyListResponse { strategies }))
}

/// Process a potential strategy file and extract info
fn process_strategy_file(path: &Path, category: &str) -> Option<StrategyInfo> {
    // Only process .lua files
    if path.extension()? != "lua" {
        return None;
    }

    let name = path
        .file_stem()?
        .to_str()?
        .to_string();

    let path_str = path
        .to_str()?
        .to_string();

    Some(StrategyInfo {
        name,
        path: path_str,
        category: category.to_string(),
    })
}

/// List commonly traded symbols
///
/// Returns a curated list of popular trading symbols across different categories.
pub async fn list_symbols() -> Result<Json<SymbolListResponse>, ApiError> {
    let symbols = vec![
        // Crypto - Major
        SymbolInfo {
            symbol: "BTCUSDT".to_string(),
            name: "Bitcoin / Tether".to_string(),
            category: "Crypto - Major".to_string(),
        },
        SymbolInfo {
            symbol: "ETHUSDT".to_string(),
            name: "Ethereum / Tether".to_string(),
            category: "Crypto - Major".to_string(),
        },
        SymbolInfo {
            symbol: "BNBUSDT".to_string(),
            name: "Binance Coin / Tether".to_string(),
            category: "Crypto - Major".to_string(),
        },
        SymbolInfo {
            symbol: "SOLUSDT".to_string(),
            name: "Solana / Tether".to_string(),
            category: "Crypto - Major".to_string(),
        },
        SymbolInfo {
            symbol: "XRPUSDT".to_string(),
            name: "Ripple / Tether".to_string(),
            category: "Crypto - Major".to_string(),
        },
        // Crypto - Alt
        SymbolInfo {
            symbol: "ADAUSDT".to_string(),
            name: "Cardano / Tether".to_string(),
            category: "Crypto - Alt".to_string(),
        },
        SymbolInfo {
            symbol: "DOGEUSDT".to_string(),
            name: "Dogecoin / Tether".to_string(),
            category: "Crypto - Alt".to_string(),
        },
        SymbolInfo {
            symbol: "AVAXUSDT".to_string(),
            name: "Avalanche / Tether".to_string(),
            category: "Crypto - Alt".to_string(),
        },
        SymbolInfo {
            symbol: "DOTUSDT".to_string(),
            name: "Polkadot / Tether".to_string(),
            category: "Crypto - Alt".to_string(),
        },
        SymbolInfo {
            symbol: "MATICUSDT".to_string(),
            name: "Polygon / Tether".to_string(),
            category: "Crypto - Alt".to_string(),
        },
        // Stocks - Tech
        SymbolInfo {
            symbol: "AAPL".to_string(),
            name: "Apple Inc.".to_string(),
            category: "Stocks - Tech".to_string(),
        },
        SymbolInfo {
            symbol: "MSFT".to_string(),
            name: "Microsoft Corp.".to_string(),
            category: "Stocks - Tech".to_string(),
        },
        SymbolInfo {
            symbol: "GOOGL".to_string(),
            name: "Alphabet Inc.".to_string(),
            category: "Stocks - Tech".to_string(),
        },
        SymbolInfo {
            symbol: "AMZN".to_string(),
            name: "Amazon.com Inc.".to_string(),
            category: "Stocks - Tech".to_string(),
        },
        SymbolInfo {
            symbol: "TSLA".to_string(),
            name: "Tesla Inc.".to_string(),
            category: "Stocks - Tech".to_string(),
        },
        // Forex - Major
        SymbolInfo {
            symbol: "EURUSD".to_string(),
            name: "Euro / US Dollar".to_string(),
            category: "Forex - Major".to_string(),
        },
        SymbolInfo {
            symbol: "GBPUSD".to_string(),
            name: "British Pound / US Dollar".to_string(),
            category: "Forex - Major".to_string(),
        },
        SymbolInfo {
            symbol: "USDJPY".to_string(),
            name: "US Dollar / Japanese Yen".to_string(),
            category: "Forex - Major".to_string(),
        },
    ];

    Ok(Json(SymbolListResponse { symbols }))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_process_strategy_file() {
        let path = PathBuf::from("lua-strategies/examples/ema_crossover.lua");
        let info = process_strategy_file(&path, "examples");

        assert!(info.is_some());
        let info = info.unwrap();
        assert_eq!(info.name, "ema_crossover");
        assert_eq!(info.category, "examples");
        assert!(info.path.contains("ema_crossover.lua"));
    }

    #[test]
    fn test_process_non_lua_file() {
        let path = PathBuf::from("lua-strategies/README.md");
        let info = process_strategy_file(&path, "custom");

        assert!(info.is_none());
    }

    #[tokio::test]
    async fn test_list_strategies() {
        // This test will work if the lua-strategies directory exists
        let result = list_strategies().await;
        assert!(result.is_ok());
    }
}
