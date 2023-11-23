use std::collections::{HashMap, HashSet};

use super::{sled_store::init, typed_sled::TypedSled};
use bigdecimal::BigDecimal;
use dashmap::DashSet;
use lazy_static::lazy_static;

use super::wal::State;

lazy_static! {
    static ref WAL_INPUT: TypedSled<u64, HashMap<String, State>> =
        TypedSled::new(init("sled/locked/wal/input_tx"));
    static ref TOTAL_INPUT: TypedSled<String, ()> = TypedSled::new(init("sled/locked/input_tx"));
    static ref TEMP_INPUT: DashSet<String> = DashSet::new();
}

pub struct LockedBalanceStore {}

impl LockedBalanceStore {
    pub fn contains(input_hash: &String) -> bool {
        TOTAL_INPUT.contains(input_hash)
    }

    pub fn insert0(state_info: &mut HashMap<String, State>, entry: (String, BigDecimal)) {
        let (address, locked) = (entry.0, entry.1);

        state_info
            .entry(address.clone())
            .and_modify(|state| state.balance = locked.clone())
            .or_insert(State::new_with_iterable(locked, HashSet::new()));

        // TEMP_INPUT.insert(input_hash.clone());  // for rollback
        // TOTAL_INPUT.insert(input_hash, ());
    }

    pub fn insert(
        state_info: &mut HashMap<String, State>,
        entry: (String, BigDecimal),
        input_hash: String,
    ) {
        let (address, locked) = (entry.0, entry.1);

        let new_input_hash = [input_hash.clone(); 1];
        state_info
            .entry(address.clone())
            .and_modify(|state| state.update(locked.clone(), new_input_hash.clone()))
            .or_insert(State::new_with_iterable(locked, new_input_hash));

        TEMP_INPUT.insert(input_hash.clone()); // for rollback
        TOTAL_INPUT.insert(input_hash, ());
    }

    pub fn flush(snapshot_stage: u64, state_info: HashMap<String, State>) -> bool {
        if state_info.is_empty() {
            return true;
        }
        if let Ok(_) = TOTAL_INPUT.flush() {
            Self::wal_into_stage(snapshot_stage, state_info);
            if let Ok(_) = WAL_INPUT.flush() {
                return true;
            }
            // Self::total_input_rollback();
        }
        false
    }

    pub fn rollback() {
        TEMP_INPUT
            .iter()
            .for_each(|input_hash| TOTAL_INPUT.remove(input_hash.key()));
    }

    pub fn log_of_snapshot_stage(snapshot_stage: u64) -> HashMap<String, State> {
        WAL_INPUT.get(&snapshot_stage).unwrap_or_default()
    }

    pub fn temporary_snapshot_of() {
        TEMP_INPUT.clear();
    }

    // write ahead logging
    fn wal_into_stage(stage_number: u64, state_info: HashMap<String, State>) {
        WAL_INPUT.insert(stage_number, state_info);
    }

    pub fn wal_input_db() -> sled::Db {
        WAL_INPUT.db.clone()
    }
}

#[tokio::test]
async fn test() {}
