use bigdecimal::BigDecimal;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use crate::{library::common::from_rawvalue_to_bigdecimal, transaction::TransactionWithResult};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct BalanceInfo {
    #[serde(
        rename = "totalAmount",
        deserialize_with = "from_rawvalue_to_bigdecimal"
    )]
    pub total_amount: BigDecimal, // BigDecimal
    pub unused: HashMap<String, TransactionWithResult>,
}

// TODO:
// 1. diff 굉장히 크게나는 계정들 수기 검사.
// 2. transaction 인풋 아웃풋 크기 비교 => logging
// Logging
//   1) TX Hash
//   2) sub_type
//   3) signer
//   4) 현재 balance
//   5) 결과 balance
//   6) input_sum/output_sum 대소 관계 분류 -> [Eq|Gt|Le]
//   Last line) 최종 balance.
