use sea_orm::entity::prelude::*;
use sea_orm::*;

use crate::library::common::as_timestamp;

#[derive(Clone, Debug, PartialEq, DeriveEntityModel)]
#[sea_orm(table_name = "account_mapper")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub address: String,
    #[sea_orm(primary_key)]
    pub hash: String,
    pub event_time: i64,
}

// #[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(
        belongs_to = "super::account_entity::Entity",
        from = "Column::Address",
        to = "super::account_entity::Column::Address"
    )]
    Address,
    #[sea_orm(
        belongs_to = "super::tx_entity::Entity",
        from = "Column::Hash",
        to = "super::tx_entity::Column::Hash"
    )]
    Hash,
}

impl ActiveModelBehavior for ActiveModel {}

impl Model {
    pub fn from(address: String, hash: String, time: String) -> ActiveModel {
        ActiveModel {
            address: Set(address),
            hash: Set(hash),
            event_time: Set(as_timestamp(&time)),
        }
    }
}
