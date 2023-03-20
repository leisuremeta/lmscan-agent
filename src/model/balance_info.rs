use std::collections::HashMap;
use serde::{Deserialize, Serialize};
use crate::transaction::TransactionWithResult;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct BalanceInfo {
  #[serde(rename = "totalAmount")]
  pub total_amount: rust_decimal::Decimal,
  pub unused: HashMap<String, TransactionWithResult>,
}
