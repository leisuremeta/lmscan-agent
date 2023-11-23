use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct NodeStatus {
    pub network_id: i32,
    pub genesis_hash: String,
    pub best_hash: String,
    pub number: u64,
}
