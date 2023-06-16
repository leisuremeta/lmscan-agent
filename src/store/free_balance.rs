use std::{collections::{HashSet, HashMap}};
use crate::{library::common::{as_path_buf, as_json_byte_vec, now, as_byte_vec, from_ivec}, store::sled_store::init};

use super::sled_store::SledStore;
use dashmap::DashMap;
use lazy_static::lazy_static;
use sled::Db;


lazy_static! {
  static ref BALANCE: Db = init(as_path_buf("sled/free/input_tx"));

  static ref TEMP_INPUT: DashMap<String, HashSet<String>> = DashMap::new();
  static ref TOTAL_INPUT: Db = init(as_path_buf("sled/free/input_tx"));

  static ref TEMP_INPUT_WAL: DashMap<i64, HashMap<String, HashSet<String>>> = DashMap::new();
  static ref INPUT_WAL: Db = init(as_path_buf("sled/free/wal/input_tx"));
}
// 마지막 50번째 블록.
// 1] input Hashs   - HashMap<i64, HashMap<String, HashSet<String>>>
// 2] total balance - HashMap<i64, HashMap<String, BigDecimal>>

pub struct FreeBalanceStore {}

impl SledStore for FreeBalanceStore {
  fn spent_hashs(account_addr: &str) -> HashSet<String> {
    let value = TOTAL_INPUT.get(account_addr).unwrap_or_default().unwrap_or_default();
    // serde_json::from_slice::<HashSet<String>>(&value).unwrap_or_else(|_| HashSet::new());
    println!("spent_hashs - value: {:?}", value);
    from_ivec(&value)
  }

  fn insert(account_addr: String, value: HashSet<String>) {
    // let serialized_value = as_json_byte_vec(&value);
    let serialized_value = as_byte_vec(&value);
    TOTAL_INPUT.insert(account_addr, serialized_value).unwrap();
  }

  fn flush() -> bool {
    if let Ok(_) = TOTAL_INPUT.flush() {
      // TODO: INPUT_WAL_TEMP 를 정확히 50개 단위에서만 flush 하기 (TOTAL_INPUT 커밋과 디커플링) 
      // for (block_number, input_hash_info) in TEMP_INPUT_WAL.clone().into_iter() {
        // INPUT_WAL.insert(block_number.to_string(), as_json_byte_vec(&input_hash_info)).unwrap();
        // INPUT_WAL.insert(block_number.to_string(), as_byte_vec(&input_hash_info)).unwrap();
      // }

      // if let Err(err) = INPUT_WAL.flush() {
        // Self::rollback();
      // }
      return true
    }
    false
  }
}

impl FreeBalanceStore {
  pub fn insert_all(spent_txs_by_signer: HashMap<String, HashSet<String>>) {
    for (address, spent_txs) in spent_txs_by_signer.into_iter() {
      Self::insert(address, spent_txs);
    }
  }

  pub fn merge(snapshot_stage: i64, address: String, mut prev_input_hashs: HashSet<String>, new_input_hashs: HashSet<String>) {
    if new_input_hashs.is_empty() { 
      return; 
    }
    
    // Temporary write for Batch Write Ahead Logging 
    // TEMP_INPUT_WAL.entry(snapshot_stage)
    //     .or_insert_with(HashMap::new)
    //     .entry(address.clone())
    //     .or_insert_with(HashSet::new)
    //     .extend(new_input_hashs.clone());
    // let stage_info = Self::input_hashs_of_snapshot_stage(snapshot_stage);

    prev_input_hashs.extend(new_input_hashs);
    Self::insert(address, prev_input_hashs);
  }

  // temporary snapshots for rollback
  pub fn temporary_snapshots_of(addresses: &HashSet<String>) -> Option<(i64, HashMap<String, HashSet<String>>)> {
    TEMP_INPUT.clear();
    TEMP_INPUT_WAL.clear();

    for addr in addresses {
      TEMP_INPUT.insert(addr.clone(), Self::spent_hashs(&addr));
    }

    INPUT_WAL.last().unwrap().map(|(block_number, value)| { // last inserted block number
      let block_number = std::str::from_utf8(&block_number)
                                      .ok().unwrap()
                                      .parse::<i64>()
                                      .ok().unwrap();
      println!("value: {:?}", value);
      (block_number, from_ivec(&value))
    })
  }
  
  pub fn rollback() {
    // overwrite current update with previous version.
    for (key, val) in TEMP_INPUT.clone().into_iter() {
      TOTAL_INPUT.insert(key, as_byte_vec(&val)).unwrap();
    } 
    TOTAL_INPUT.flush().unwrap();
  }

  fn input_hashs_of_snapshot_stage(snapshot_stage: i64) -> HashMap<i64, HashMap<String, HashSet<String>>> {
    HashMap::from([(
      snapshot_stage, 
      match INPUT_WAL.get(snapshot_stage.to_string()).unwrap() {
        Some(value) => from_ivec(&value),
        None => HashMap::new(),
      }
    )])
  }
}


#[tokio::test]
async fn sled() {
  let addr_with_timestamp = format!("지창호_{}", now().to_string());
  let mut set = HashSet::new();
  set.insert("sdfioncxvsd".to_owned());
  set.insert("xcvxczzzx".to_owned());
  INPUT_WAL.insert("지창호", as_byte_vec(&set)).unwrap();

  let val = INPUT_WAL.get("지창호").unwrap().unwrap();
  let desirialized: HashSet<String> = from_ivec(&val);

  println!("desirialized: {:?}", desirialized);



  // let addr_with_timestamp = format!("지창호_{}", now().to_string());
  // let mut set = HashSet::new();
  // set.insert("1".to_owned());

  // WAL.insert(addr_with_timestamp.as_bytes(), as_json_byte_vec(&set)).unwrap();
  // println!("{}", WAL.scan_prefix("지창호").count());
  // WAL.scan_prefix("지창호").for_each(|res: Result<(sled::IVec, sled::IVec), sled::Error>| match res {
  //   Ok((k, v)) => {
  //     match (serde_json::from_slice::<String>(&k), serde_json::from_slice::<HashSet<String>>(&v)) {
  //         (Ok(deserialized_key), Ok(deserialized_value)) => {
  //             println!("s: {:?} - {:?}", deserialized_key, deserialized_value);
  //         }
  //         (Err(e), _) | (_, Err(e)) => {
  //             println!("Failed to deserialize: {:?}", e);
  //         }
  //     }
  //   }
  //   Err(_) => panic!(),

  // });


}
