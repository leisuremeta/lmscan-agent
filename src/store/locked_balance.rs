use std::collections::HashSet;

use crate::store::{sled_store::init, typed_sled::TypedSled};
use bigdecimal::BigDecimal;
use dashmap::DashSet;
use lazy_static::lazy_static;

use super::wal::State;

lazy_static! {
  static ref WAL_INPUT:   TypedSled<u64, State> = TypedSled::new(init("sled/locked/wal/input_tx"));
  static ref TOTAL_INPUT: TypedSled<String, ()> = TypedSled::new(init("sled/locked/input_tx"));
  static ref TEMP_INPUT:  DashSet<String> = DashSet::new();
}

pub struct LockedBalanceStore {}

impl LockedBalanceStore {
  pub fn contains(input_hash: &str) -> bool {
    TOTAL_INPUT.contains(input_hash)
  }

  pub fn insert(snapshot_stage: u64, entry: (String, BigDecimal), input_hash: String) {
    // TODO: Write ahead logging
    let mut state_info = Self::log_of_snapshot_stage(snapshot_stage);
    state_info.update(entry.1, HashSet::from([input_hash.clone(); 1]));
    WAL_INPUT.insert(snapshot_stage, state_info);

    TEMP_INPUT.insert(input_hash.clone());  // for rollback
    TOTAL_INPUT.insert(input_hash, ());

  }

  pub fn flush() -> bool {
    match TOTAL_INPUT.flush() {
      Ok(_) => true,
      Err(_) => false,
    }
  }

  pub fn rollback() {
    TEMP_INPUT
        .iter()
        .for_each(|input_hash| 
          TOTAL_INPUT.remove(input_hash.key()))
  }
  
  fn log_of_snapshot_stage(snapshot_stage: u64) -> State {
    WAL_INPUT.get(&snapshot_stage).unwrap_or_default()
  }
  
}
