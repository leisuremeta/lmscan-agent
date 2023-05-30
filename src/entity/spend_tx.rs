use rust_decimal_macros::dec;
use sea_orm::entity::prelude::*;
use sea_orm::*;

use crate::library::common::{as_timestamp, now};
use crate::transaction::{CreateAccount, Extract};

#[derive(Clone, Debug, PartialEq, DeriveEntityModel)]
#[sea_orm(table_name = "spend_tx")]
pub struct Model {
  #[sea_orm(primary_key)]
  pub address: String,
  #[sea_orm(primary_key)]
  pub hash: String,
  pub created_at: i64,
}



#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}
impl ActiveModelBehavior for ActiveModel {}


impl Model {
  // pub fn from(tx: &CreateAccount) -> ActiveModel {
  //   ActiveModel {
  //   }
  // }
}

impl Extract for ActiveModel {
}
