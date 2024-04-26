use sea_orm::entity::prelude::*;

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
