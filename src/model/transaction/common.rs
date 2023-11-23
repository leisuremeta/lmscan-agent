use crate::tx_entity::ActiveModel;

use super::TransactionWithResult;

pub trait Common {
    fn created_at(&self) -> i64;
    fn network_id(&self) -> i64;
    fn from(
        &self,
        hash: String,
        from_account: String,
        block_hash: String,
        block_number: i64,
        json: String,
        tx: TransactionWithResult,
    ) -> ActiveModel;
}
