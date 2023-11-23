use sea_orm::entity::prelude::*;
use sea_orm::*;

use crate::{
    library::common::now,
    transaction::{common::Common, TransactionWithResult},
};

// The DeriveEntityModel macro does all the heavy lifting of defining an Entity with associating Model, Column and PrimaryKey.
#[derive(Clone, Debug, PartialEq, Eq, DeriveEntityModel)]
#[sea_orm(table_name = "tx_state")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub hash: String,
    pub block_hash: String,
    pub json: String,
    pub event_time: i64,
    pub created_at: i64,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}
impl ActiveModelBehavior for ActiveModel {}

impl Model {
    pub fn from(
        hash: &str,
        block_hash: &str,
        tx_res: &TransactionWithResult,
        json: String,
    ) -> ActiveModel {
        ActiveModel {
            hash: Set(hash.to_owned()),
            block_hash: Set(block_hash.to_owned()),
            json: Set(json),
            event_time: Set(tx_res.signed_tx.value.created_at()),
            created_at: Set(now()),
        }
    }
}
