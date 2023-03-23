
use serde::{Deserialize, Serialize};
use crate::transaction::TransactionWithResult;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct NftBalanceInfo {
  #[serde(rename = "tokenDefinitionId")]
  pub token_def_id: String,
  #[serde(rename = "txHash")]
  pub tx_hash: String,
  pub tx: TransactionWithResult,
}
