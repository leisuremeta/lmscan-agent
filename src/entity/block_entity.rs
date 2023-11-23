use crate::{
    block::Block,
    library::common::{as_timestamp, now},
};
use sea_orm::entity::prelude::*;
use sea_orm::*;

// use crate::block_state::Model as BlockState;

#[derive(Clone, Debug, PartialEq, Eq, DeriveEntityModel)]
#[sea_orm(table_name = "block")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub hash: String,
    pub number: i64,
    pub parent_hash: String,
    pub tx_count: i64,
    pub event_time: i64,
    pub created_at: i64,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}
impl ActiveModelBehavior for ActiveModel {}

impl Model {
    pub fn from(block: &Block, block_hash: String) -> ActiveModel {
        ActiveModel {
            hash: Set(block_hash),
            number: Set(block.header.number),
            parent_hash: Set(block.header.parent_hash.clone()),
            tx_count: Set(block.transaction_hashes.len() as i64),
            event_time: Set(as_timestamp(&block.header.timestamp)),
            created_at: Set(now()),
        }
    }
}
