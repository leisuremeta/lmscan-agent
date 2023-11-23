use sea_orm::entity::prelude::*;

#[derive(Clone, Debug, PartialEq, DeriveEntityModel)]
#[sea_orm(table_name = "state_daily")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub address: String,
    #[sea_orm(primary_key)]
    pub date: String,
    pub free: BigDecimal,
    pub locked: BigDecimal,
    pub created_at: i64,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}
impl ActiveModelBehavior for ActiveModel {}
