use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
// #[serde(rename_all(serialize = "snake_case", deserialize = "camelCase"))] 
pub struct NodeStatus {
    // #[serde(rename(serialize = "camelCase", deserialize = "networkId"))] 
    pub network_id: i32,
    pub genesis_hash: String,
    pub best_hash: String,
    pub number: u64,
}
