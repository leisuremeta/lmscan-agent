use std::collections::HashMap;
use serde::{Deserialize, Serialize};
use crate::transaction::TransactionWithResult;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct BlockInfo {
  #[serde(rename = "totalAmmount")]
  pub total_amount: rust_decimal::Decimal,
  pub unused: HashMap<String, TransactionWithResult>,
}
