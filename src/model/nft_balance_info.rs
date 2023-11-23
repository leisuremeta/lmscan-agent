use crate::transaction::TransactionWithResult;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct NftBalanceInfo {
    #[serde(rename = "tokenDefinitionId")]
    pub token_def_id: String,
    #[serde(rename = "txHash")]
    pub tx_hash: String,
    pub tx: TransactionWithResult,
}
