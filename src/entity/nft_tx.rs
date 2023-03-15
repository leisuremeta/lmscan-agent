use sea_orm::entity::prelude::*;
use sea_orm::*;

use crate::{transaction::{MintNft, Extract, TransferNft, EntrustNft, DisposeEntrustedNft}, tx_entity, library::common::now};


#[derive(Clone, Debug, PartialEq, Eq, DeriveEntityModel)]
#[sea_orm(table_name = "nft")]
pub struct Model {
  #[sea_orm(primary_key)]
  pub tx_hash: String,
  pub token_id: String,
  pub action: String,
  pub from_addr: String,
  pub to_addr: String,
  pub event_time: i64,
  pub created_at: i64,
}


#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}
impl ActiveModelBehavior for ActiveModel {}


impl Model {
  pub fn from<T: NftTx>(tx: &T, tx_entity: &tx_entity::ActiveModel) -> ActiveModel {
    ActiveModel {
      tx_hash: tx_entity.hash.clone(),
      token_id: Set(tx.token_id()),
      action: Set(tx.sub_type()),
      from_addr: tx_entity.from_addr.clone(),
      to_addr: Set(tx_entity.to_addr.clone().unwrap().first().unwrap_or(&"".to_string()).to_string()),
      event_time: tx_entity.event_time.clone(),
      created_at: Set(now()),
    }
  }
}

impl Extract for ActiveModel {}

pub trait NftTx {
  fn token_id(&self) -> String;
  fn sub_type(&self) -> String;
}

impl NftTx for MintNft {
  fn token_id(&self) -> String {
    self.token_id.clone()
  }
  fn sub_type(&self) -> String {
    String::from("MintNft")
  }
}

impl NftTx for TransferNft {
  fn token_id(&self) -> String {
    self.token_id.clone()
  }
  fn sub_type(&self) -> String {
    String::from("TransferNft")
  }
}

impl NftTx for EntrustNft {
  fn token_id(&self) -> String {
    self.token_id.clone()
  }
  fn sub_type(&self) -> String {
    String::from("EntrustNft")
  }
}

impl NftTx for DisposeEntrustedNft {
  fn token_id(&self) -> String {
    self.token_id.clone()
  }
  fn sub_type(&self) -> String {
    String::from("DisposeEntrustedNft")
  }
}
