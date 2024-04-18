use sea_orm::entity::prelude::*;

#[derive(Clone, Debug, PartialEq, Eq, DeriveEntityModel)]
#[sea_orm(table_name = "tx")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub hash: String,
    pub token_type: String,
    pub tx_type: String, // col_name : type
    pub sub_type: String,
    pub block_hash: String,
    pub block_number: i64,
    pub event_time: i64,
    pub created_at: i64,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}
impl ActiveModelBehavior for ActiveModel {}

impl Related<super::account_entity::Entity> for Entity {
    fn to() -> RelationDef {
        super::account_mapper::Relation::Address.def()
    }

    fn via() -> Option<RelationDef> {
        Some(super::account_mapper::Relation::Hash.def().rev())
    }
}
