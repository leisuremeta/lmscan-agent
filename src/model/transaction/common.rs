use crate::tx_entity::ActiveModel;

use super::TransactionWithResult;

pub trait Common {
    fn created_at(&self) -> i64;
    fn from(
        &self,
        hash: String,
        block_hash: String,
        block_number: i64,
        tx: TransactionWithResult,
    ) -> ActiveModel;
}
