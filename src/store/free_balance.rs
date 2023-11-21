use super::{sled_store::init, typed_sled::TypedSled};
use std::collections::{HashMap, HashSet};

use super::{sled_store::SledStore, wal::State};
use bigdecimal::BigDecimal;
use dashmap::DashMap;
use lazy_static::lazy_static;

lazy_static! {
  static ref WAL_INPUT: TypedSled<u64, HashMap<String, State>> = TypedSled::new(init("sled/free/wal/input_tx"));    // for snapshot & time_travel
  static ref TOTAL_INPUT: TypedSled<String, HashSet<String>> = TypedSled::new(init("sled/free/input_tx"));      // for current state building.
  static ref TEMP_INPUT: DashMap<String, HashSet<String>> = DashMap::new();  // for rollback.
}
pub struct FreeBalanceStore {}

impl SledStore for FreeBalanceStore {
    fn spent_hashs(account: &str) -> HashSet<String> {
        TOTAL_INPUT.get(&account.to_owned()).unwrap_or_default()
    }

    fn insert(account: String, value: HashSet<String>) {
        TOTAL_INPUT.insert(account, value);
    }

    fn flush() -> bool {
        todo!()
    }
}

impl FreeBalanceStore {
    pub fn merge(state_info: &mut HashMap<String, State>, entry: (String, BigDecimal)) {
        let (address, free) = (entry.0, entry.1);

        state_info
            .entry(address.clone())
            .and_modify(|state| state.balance = free.clone())
            .or_insert(State::new(free, HashSet::new()));
    }

    pub fn merge_with_inputs(
        state_info: &mut HashMap<String, State>,
        entry: (String, BigDecimal),
        mut prev_input_hashs: HashSet<String>,
        new_input_hashs: HashSet<String>,
    ) {
        let (address, free) = (entry.0, entry.1);

        state_info
            .entry(address.clone())
            .and_modify(|state| state.update(free.clone(), new_input_hashs.clone()))
            .or_insert(State::new(free, new_input_hashs.clone()));

        prev_input_hashs.extend(new_input_hashs);
        Self::total_input_insert(address, prev_input_hashs);
    }

    // temporary snapshots for rollback
    pub fn temporary_snapshot_of(addresses: &HashSet<String>) {
        TEMP_INPUT.clear();

        for addr in addresses {
            TEMP_INPUT.insert(addr.clone(), Self::spent_hashs(&addr));
        }
    }

    pub fn rollback(snapshot_stage: u64) {
        Self::total_input_rollback();
        Self::wal_rollback(snapshot_stage);
    }

    fn total_input_insert(account: String, value: HashSet<String>) {
        TOTAL_INPUT.insert(account, value);
    }

    // write ahead logging
    fn wal_into_stage(stage_number: u64, state_info: HashMap<String, State>) {
        WAL_INPUT.insert(stage_number, state_info);
    }

    // overwrite current update with previous version.
    fn total_input_rollback() {
        for (key, val) in TEMP_INPUT.clone().into_iter() {
            TOTAL_INPUT.insert(key, val);
        }
        TOTAL_INPUT.flush().unwrap();
    }

    fn wal_rollback(snapshot_stage: u64) {
        WAL_INPUT.remove(&snapshot_stage);
    }

    pub fn log_of_snapshot_stage(snapshot_stage: u64) -> HashMap<String, State> {
        WAL_INPUT.get(&snapshot_stage).unwrap_or_default()
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
            Self::total_input_rollback();
        }
        false
    }
}
