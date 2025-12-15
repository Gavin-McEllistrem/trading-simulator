// Configuration structures and loading

use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, Serialize)]
pub struct DataSourceConfig {
    #[serde(rename = "type")]
    pub source_type: DataSourceType,

    #[serde(flatten)]
    pub specific: Option<DataSourceSpecific>,
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum DataSourceType {
    Binance,
    Alpaca,
    Simulated,
    Csv,
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(untagged)]
pub enum DataSourceSpecific {
    Binance(BinanceConfig),
    Alpaca(AlpacaConfig),
    Simulated(SimulatedConfig),
    Csv(CsvConfig),
}

#[derive(Debug, Deserialize, Serialize)]
pub struct BinanceConfig {
    pub symbols: Vec<String>,
    pub interval: String,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct AlpacaConfig {
    pub api_key_env: String,
    pub secret_key_env: String,
    pub symbols: Vec<String>,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct SimulatedConfig {
    pub symbol: String,
    pub starting_price: f64,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct CsvConfig {
    pub path: String,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct StorageConfig {
    pub window_size: usize,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct EngineConfig {
    pub data_source: DataSourceConfig,
    pub storage: StorageConfig,
}
