use std::collections::HashMap;
use bigdecimal::BigDecimal;
use serde::{Deserialize, Serialize};
use crate::transaction::TransactionWithResult;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct BalanceInfo {
  #[serde(rename = "totalAmount")]
  pub total_amount: BigDecimal,
  pub unused: HashMap<String, TransactionWithResult>,
}
