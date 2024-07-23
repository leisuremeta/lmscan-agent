use sea_orm::entity::prelude::*;

#[derive(Clone, Debug, PartialEq, DeriveEntityModel)]
#[sea_orm(table_name = "spend_tx")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub target: String,
    #[sea_orm(primary_key)]
    pub hash: String,
    pub token: String,
    pub t: u8,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}
impl ActiveModelBehavior for ActiveModel {}
