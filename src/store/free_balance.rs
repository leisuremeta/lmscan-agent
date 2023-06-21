use crate::{store::typed_sled::TypedSled};
use std::collections::{HashSet, HashMap};
use crate::{store::sled_store::init, model::balance::Balance};

use super::{sled_store::SledStore, wal::State};
use bigdecimal::BigDecimal;
use dashmap::DashMap;
use lazy_static::lazy_static;

// TODO: Define type alias.
type Address = String;

lazy_static! {
  static ref WAL_INPUT:   TypedSled<u64, HashMap<String, State>> = TypedSled::new(init("sled/free/wal/input_tx"));    // for snapshot & time_travel
  static ref TOTAL_INPUT: TypedSled<String, HashSet<String>> = TypedSled::new(init("sled/free/input_tx"));      // for current state building.
  static ref TEMP_INPUT:  DashMap<String, HashSet<String>> = DashMap::new();  // for rollback.
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
    if let Ok(_) = TOTAL_INPUT.flush() {
      if let Ok(_) = WAL_INPUT.flush() {
        return true;
      }
      Self::total_input_rollback();
    }
    false
  }
}

impl FreeBalanceStore {
  pub fn merge (
    snapshot_stage: u64, 
    entry: (String, BigDecimal), 
    mut prev_input_hashs: HashSet<String>, 
    new_input_hashs: HashSet<String>
  ) {
    let (address, free) = (entry.0, entry.1);
    
    let mut state_info = Self::log_of_snapshot_stage(snapshot_stage);
    state_info.entry(address.clone())
        .and_modify(|log| log.update(free.clone(), new_input_hashs.clone()))
        .or_insert(State::new(free, new_input_hashs.clone()));

    Self::wal_into_stage(snapshot_stage, state_info);
    
    prev_input_hashs.extend(new_input_hashs);
    Self::insert(address, prev_input_hashs);
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
  
  fn log_of_snapshot_stage(snapshot_stage: u64) -> HashMap<String, State> {
    WAL_INPUT.get(&snapshot_stage)
             .unwrap_or_default()
  }
}


#[tokio::test]
async fn test() {
  // WAL_INPUT.insert("지창호", as_byte_vec(&set)).unwrap();
  let config = sled::Config::new().temporary(true);
  let db: sled::Db = config.open().expect("open");

  let mut balance_log_info = HashSet::new(); // Put some dummy data here.
  balance_log_info.insert("sdiofnwioneflsdf".to_string());
  balance_log_info.insert("xcvnoinowqwenois".to_string());

  let sled: TypedSled<String, HashSet<String>> = TypedSled::new(db);
  sled.insert("지창호".to_string(), balance_log_info);

  match sled.get(&"지창호".to_string()) {
    Some(des_val)  => println!("des_val: {:?}", des_val),
    None => println!("empty"),
  }
  
  // let serialized = bincode::serialize(&balance_log_info).unwrap();
  // let deserialized: HashMap<String, BalanceLog> = bincode::deserialize(&serialized).unwrap();
  // println!("len: {}", TOTAL_INPUT.len());
  
  // db.insert(as_byte_vec(&"지창호".to_string()), as_byte_vec(&balance_log_info)).unwrap();

  // let val = db.get(as_byte_vec(&"지창호".to_string())).unwrap().unwrap();
  // println!("{:?}", from_ivec::<HashSet<String>>(&val));
  // db
  //     .into_iter()
  //     .inspect(|x| 
  //       if let Err(err) = x {
  //         println!("err: {err}");
  //       }
  //     )
  //     .filter_map(|res| res.ok())
  //     .for_each(|(key, value)| {
  //       let key: String = from_ivec(&key);
  //       let value: HashSet<String> = from_ivec(&value);
  //       println!("key : {:?}", key);
  //       println!("value : {:?}", value);

  //       // println!("{key} - {:?}", value);
  //     });

  // println!("{:?}", deserialized);
}
