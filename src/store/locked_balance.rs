use crate::{library::common::as_path_buf, store::sled_store::init};

use lazy_static::lazy_static;
use sled::Db;


lazy_static! {
  static ref LOCKED: Db = init(as_path_buf("sled/locked/input_tx"));
}

pub struct LockedBalanceStore {}

impl LockedBalanceStore {
  pub fn contains(input_hash: &str) -> bool {
    LOCKED.contains_key(input_hash).unwrap()
  }
  pub fn insert(input_hash: String) {
    LOCKED.insert(input_hash.as_bytes(), "".as_bytes()).unwrap();
  }
  pub fn flush() {
    LOCKED.flush().unwrap();
  }
}
