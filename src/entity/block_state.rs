use sea_orm::entity::prelude::*;
use sea_orm::*;

use crate::block::Block;
use crate::library::common::{as_timestamp, now};
// use block_state::Entity as BlockState;
// The DeriveEntityModel macro does all the heavy lifting of defining an Entity with associating Model, Column and PrimaryKey.
#[derive(Clone, Debug, PartialEq, Eq, DeriveEntityModel)]
#[sea_orm(table_name = "block_state")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub hash: String,
    pub number: i64,
    pub is_build: bool,
    pub json: String,
    pub event_time: i64,
    pub created_at: i64,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}
impl ActiveModelBehavior for ActiveModel {}

impl Model {
    pub fn from(hash: &str, block: &Block) -> ActiveModel {
        ActiveModel {
            hash: Set(hash.to_owned()),
            number: Set(block.header.number),
            json: Set(serde_json::to_string(block).unwrap()),
            is_build: Set(false),
            event_time: Set(as_timestamp(block.header.timestamp.as_str())),
            created_at: Set(now()),
        }
    }
}
