use sea_orm::{entity::prelude::*, ActiveValue::NotSet, Set};

use crate::library::common::now;

#[derive(Clone, Debug, DeriveEntityModel)]
#[sea_orm(table_name = "summary")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub id: i64,
    pub lm_price: rust_decimal::Decimal,
    pub block_number: i64,
    pub total_tx_size: i64,
    // pub tx_count_in_latest24h: i64, //tx_count_in_latest24h
    pub total_accounts: i64,
    pub total_balance: BigDecimal,
    pub created_at: i64,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}
impl ActiveModelBehavior for ActiveModel {}

impl Model {
    pub fn from(
        block_number: i64,
        lm_price: Decimal,
        total_accounts: i64,
        total_tx_size: i64,
        total_balance: BigDecimal,
    ) -> ActiveModel {
        ActiveModel {
            id: NotSet,
            lm_price: Set(lm_price),
            block_number: Set(block_number),
            total_tx_size: Set(total_tx_size),
            total_accounts: Set(total_accounts),
            total_balance: Set(total_balance),
            created_at: Set(now()),
        }
    }
}
