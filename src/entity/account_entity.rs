use sea_orm::entity::prelude::*;
use sea_orm::*;

use crate::library::common::{as_timestamp, now};
use crate::transaction::account_transaction::CreateAccount;

#[derive(Clone, Debug, PartialEq, DeriveEntityModel)]
#[sea_orm(table_name = "account")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub address: String,
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
            event_time: Set(as_timestamp(&tx.created_at)),
            created_at: Set(now()),
        }
    }
}

impl Related<super::tx_entity::Entity> for Entity {
    fn to() -> RelationDef {
        super::account_mapper::Relation::Hash.def()
    }

    fn via() -> Option<RelationDef> {
        Some(super::account_mapper::Relation::Address.def().rev())
    }
}
