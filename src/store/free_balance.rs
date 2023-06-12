use std::collections::{HashSet, HashMap};

use crate::{library::common::as_path_buf, store::sled_store::init};

use super::sled_store::SledStore;
use lazy_static::lazy_static;
use sled::Db;


lazy_static! {
  static ref FREE: Db = init(as_path_buf("sled/free/input_tx"));
}

pub struct FreeBalanceStore {}

impl SledStore for FreeBalanceStore {
  fn spent_hashs(account_addr: &str) -> HashSet<String> {
    let value = FREE.get(account_addr).unwrap_or_default().unwrap_or_default();
    serde_json::from_slice::<HashSet<String>>(&value).unwrap_or_else(|_| HashSet::new())
  }
  fn insert(account_addr: String, value: HashSet<String>) {
    let serialized_value = serde_json::to_vec(&value).unwrap();
    FREE.insert(account_addr.as_bytes(), serialized_value).unwrap();
  }
  fn flush() {
    FREE.flush().unwrap();
  }
}

impl FreeBalanceStore {
  pub fn insert_all(spent_txs_by_signer: HashMap<String, HashSet<String>>) {
    for (address, spent_txs) in spent_txs_by_signer.into_iter() {
      Self::insert(address, spent_txs);
    }
  }
}


#[tokio::test]
async fn sled() {
  let mut set = HashSet::new();
  set.insert("sdfioncxvsd".to_owned());
  set.insert("xcvxczzzx".to_owned());
  FreeBalanceStore::insert("abc".to_owned(), set);
  FreeBalanceStore::flush();
  println!("res: {:?}", FREE.len());
  let res = FreeBalanceStore::spent_hashs("abc");

  println!("res: {:?}", res.len());
}
