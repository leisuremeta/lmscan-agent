use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
#[serde_with::serde_as]
pub struct Block {
    pub header: Header,
    pub transaction_hashes: Vec<String>,
    pub votes: Vec<Votes>,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Header {
    pub number: i64,
    pub parent_hash: String,
    // state_root is not store to block entity
    pub transactions_root: Option<String>,
    pub timestamp: String,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Votes {
    pub v: i64,
    pub r: String,
    pub s: String,
}
