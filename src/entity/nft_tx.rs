use sea_orm::entity::prelude::*;
use sea_orm::*;

use crate::{
    library::common::now, transaction::token_transaction::TokenTx, tx_entity
};

#[derive(Clone, Debug, PartialEq, Eq, DeriveEntityModel)]
#[sea_orm(table_name = "nft")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub tx_hash: String,
    pub token_id: String,
    pub action: String,
    pub from_addr: String,
    pub to_addr: String,
    pub event_time: i64,
    pub created_at: i64,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}
impl ActiveModelBehavior for ActiveModel {}

impl Model {
    pub fn from(tx: &TokenTx, tx_entity: &tx_entity::ActiveModel, from: String, to: String) -> ActiveModel {
        ActiveModel {
            tx_hash: tx_entity.hash.clone(),
            token_id: Set(tx.token_id()),
            action: Set(tx.sub_type()),
            from_addr: Set(from),
            to_addr: Set(to),
            event_time: tx_entity.event_time.clone(),
            created_at: Set(now()),
        }
    }
}
