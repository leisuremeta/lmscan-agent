use sea_orm::entity::prelude::*;
use sea_orm::*;

use crate::library::common::{as_timestamp, now};
use crate::transaction::{account_transaction::CreateAccount, Extract};

#[derive(Clone, Debug, PartialEq, DeriveEntityModel)]
#[sea_orm(table_name = "account")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub address: String,
    // pub balance: BigDecimal,
    // pub amount: rust_decimal::Decimal,
    pub event_time: i64,
    pub created_at: i64,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}
impl ActiveModelBehavior for ActiveModel {}

impl Model {
    pub fn from(tx: &CreateAccount) -> ActiveModel {
        ActiveModel {
            address: Set(tx.account.to_owned()),
            // balance: Set(BigDecimal::from(0)),
            // amount: Set(dec!(0.0)),
            event_time: Set(as_timestamp(&tx.created_at)),
            created_at: Set(now()),
        }
    }
}

impl Extract for ActiveModel {}
