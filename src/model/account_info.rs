use std::collections::HashMap;

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct AccountInfo {
    #[serde(rename = "ethAddress")]
    pub eth_address: Option<String>,
    pub guardian: Option<String>,
    #[serde(rename = "publicKeySummaries")]
    pub public_key_summaries: HashMap<String, Info>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Info {
    pub description: String,
    #[serde(rename = "addedAt")]
    pub added_at: String,
}
