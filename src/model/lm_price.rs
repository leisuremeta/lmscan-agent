use std::collections::HashMap;

use bigdecimal::BigDecimal;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LmPrice {
    pub status: Status,
    pub data: HashMap<i32, Data>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Status {
    pub error_code: i32,
    pub error_message: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Data {
    pub id: u32,
    pub name: String,
    pub symbol: String,
    pub last_updated: String,
    pub quote: Currency,
    pub circulating_supply: BigDecimal,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Currency {
    #[serde(rename = "USD")]
    pub usd: USDCurrency,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct USDCurrency {
    pub price: BigDecimal,
    pub last_updated: String,
    pub market_cap: BigDecimal,
}
