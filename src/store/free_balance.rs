use crate::{store::typed_sled::TypedSled, library::common::{from_ivec, now}, model::balance::Balance, service::api_service::ApiService};
use std::collections::{HashSet, HashMap, BTreeMap};
use crate::{store::sled_store::init};

use super::{sled_store::SledStore, wal::{State, StateType}};
use bigdecimal::BigDecimal;
use dashmap::DashMap;
use futures_util::lock;
use itertools::Itertools;
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
    todo!()
  }
}

impl FreeBalanceStore {
  pub fn merge (
    state_info: &mut HashMap<String, State>,
    entry: (String, BigDecimal)
  ) {
    let (address, free) = (entry.0, entry.1);
    
    state_info.entry(address.clone())
        .and_modify(|state| state.balance = free.clone() )
        .or_insert(State::new(free, HashSet::new()));
  }

  pub fn merge_with_inputs (
    state_info: &mut HashMap<String, State>,
    entry: (String, BigDecimal), 
    mut prev_input_hashs: HashSet<String>, 
    new_input_hashs: HashSet<String>
  ) {
    let (address, free) = (entry.0, entry.1);
    
    state_info.entry(address.clone())
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

  pub fn overwrite_total_input(account_inputs: Vec<(String, HashSet<String>)>) {
    TOTAL_INPUT.db.clear().unwrap();
    for (account, inputs) in account_inputs {
      TOTAL_INPUT.insert(account, inputs);
    }
    TOTAL_INPUT.flush().unwrap();
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
    WAL_INPUT.get(&snapshot_stage)
             .unwrap_or_default()
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

  pub fn collect_log_limit(limit_block_number: u64) -> Vec<(String, StateType)> {
    WAL_INPUT.db
        .iter()
        .filter_map(Result::ok)
        .map(|(key, val)| State::from(&key, &val))
        .collect::<BTreeMap<u64, HashMap<String, State>>>()
        .into_iter()
        .take_while(|(block_no, _)| *block_no <= limit_block_number)  
        .flat_map(|(state_no, account_state)| {
          account_state
              .into_iter()
              .collect::<Vec<(String, State)>>()
              .into_iter()
              .map(move |(addr,state)| (addr, (state_no, state)))
        })
        .map(|(k, v)| 
          (
            k, 
            StateType::Free(v),
          )
        )
        .collect()
  }
}




#[tokio::test]
async fn test() {
  
  WAL_INPUT.db
    .iter()
    .filter_map(Result::ok)
    .map(|(key, val)| State::from::<u64, HashMap<String, State>>(&key, &val))
    .sorted_by_key(|x| x.0)
    .into_iter()
    .for_each(|(key, _)| {
      println!("{key}");
    });

  // 마지막으로 체킹에 성공한 블록번호는 DB 에서 조회가능하므로 해당 블록넘버로 돌아갈수 있게 하는게 제일 적합.
  let rollback_stage_number: u64 = 20000;

  // let mut free: Vec<(String, Balance)> = WAL_INPUT.db
  //     .iter()
  //     .filter_map(Result::ok)
  //     .map(|(key, val)| State::from(&key, &val))
  //     .collect::<BTreeMap<u64, HashMap<String, State>>>()
  //     .into_iter()
  //     .take_while(|(block_no, _)| block_no <= &rollback_stage_number)  
  //     // .flat_map(|(_, stage_info)| stage_info.into_iter()) // 모든 stage_info 맵을 단일 스트림으로 변환
  //     .flat_map(|(_, v)| 
  //         v.into_iter().collect::<Vec<(String, State)>>()
  //     )
  //     .map(|(k, v)| 
  //       (
  //         k, 
  //         // Balance::new_with_free(v.balance),
  //       )
  //     )
  //     .collect();
  
  // let locked: Vec<(String, Balance)> = crate::store::locked_balance::LockedBalanceStore::collect_log_limit(rollback_stage_number);
  // free.extend(locked.clone());

  // let total_map: HashMap<String, Balance> = 
  //     free.into_iter()
  //         .into_group_map()
  //         .into_iter()
  //         .fold(HashMap::new(), |mut acc: HashMap<String, Balance>, (account, balances)| {
  //             let balance = balances.into_iter()
  //                     .fold(Balance::default(), |mut acc, balance| {
  //                       if balance.free() == BigDecimal::default() {
  //                         acc.locked = balance.locked();
  //                       } else {
  //                         acc.free = balance.free();
  //                       }
  //                       acc
  //                     });
  //             acc.insert(account, balance);
  //             acc
  //         });

  // for (account, balance) in total_map {
  //   println!("{account} - {:?}", balance);
  // }

  // let free_info: HashMap<String, Balance> = 
  //   free
  //     .into_iter()
  //     .fold(HashMap::new(), |mut acc, (account, balance)| {
  //       acc.entry(account)
  //           .and_modify(|prev: &mut Balance| {
  //             prev.free = balance.free();
  //           })
  //           .or_insert(balance);
  //       acc
  //     })
  //     .into_iter()
  //     // .map(|(account, built_state)| 
  //     //   (account, Balance::from_state(built_state))
  //     // )
  //     .collect();

  // let locked_info: HashMap<String, Balance> = 
  //   locked
  //     .into_iter()
  //     .fold(HashMap::new(), |mut acc, (account, balance)| {
  //       acc.entry(account)
  //           .and_modify(|prev: &mut Balance| {
  //             prev.locked = balance.locked();
  //           })
  //           .or_insert(balance);
  //       acc
  //     })
  //     .into_iter()
  //     // .map(|(account, built_state)| 
  //     //   (account, Balance::from_state(built_state))
  //     // )
  //     .collect();
  

  // println!("free_info - {:?}", free_info);

  // let mut wal_map: HashMap<String, State> = WAL_INPUT.db
  //     .into_iter()
  //     .filter_map(Result::ok)
  //     .map(|(key, val)| State::from(&key, &val))
  //     .collect::<BTreeMap<u64, HashMap<String, State>>>() 
  //     .into_iter()
  //     .fold(HashMap::new(), |mut acc, stage_info| {
  //       for (key, val) in stage_info {
  //         acc.entry(key)
  //             .and_modify(|prev| { 
  //               prev.merge(val.clone());
  //             })
  //             .or_insert(val);
  //       }
  //       acc
  //     });

  // assert_eq!(TOTAL_INPUT.db.len(),  wal_map.len());
  // println!("wal_map : {:?}", wal_map);
  

  // check build state is valid
  // for (account, balance) in total_map {
  //   let res = ApiService::get_free_balance(&account).await.unwrap();
  //   if res.is_none() {
  //     println!("{account} 의 잔고가 존재 하지 않습니다.");
  //     continue;
  //   }
  //   let res = res.unwrap();
  //   let balance_info = res.get("LM").unwrap();
  //   println!("{account} - {} - scan: {}, blc: {}",  balance.free() == balance_info.total_amount,  balance.free(), balance_info.total_amount);
  //   assert_eq!(balance.free(), balance_info.total_amount);

  //   let res = ApiService::get_locked_balance(&account).await.unwrap();
  //   if res.is_none() {
  //     println!("{account} 의 잔고가 존재 하지 않습니다.");
  //     continue;
  //   }
  //   let res = res.unwrap();
  //   let balance_info = res.get("LM").unwrap();
  //   println!("{account} - {} - scan: {}, blc: {}",  balance.locked() == balance_info.total_amount,  balance.locked(), balance_info.total_amount);
  //   assert_eq!(balance.locked(), balance_info.total_amount);
  // }

  // dd0b913a2d5d9059f64c1febe6c9e7a773a67979    
  // 3ce402227618ffd9fa975e4c6e5786bf55373986


  // TOTAL_INPUT.db
  //     .into_iter()
  //     .filter_map(Result::ok)
  //     .map(|(k, v)| State::from::<String, HashSet<String>>(&k, &v))
  //     .for_each(|(tot_key, tot_val)| {
  //       if let Some(wal_val) = wal_map.remove(&tot_key) {
  //         // println!("tot_key: {tot_key}");
  //         assert_eq!(wal_val.input_hashs, tot_val);
  //         println!("{} - wal_inputs = {:?}, tot_inputs = {:?}", wal_val.input_hashs == tot_val , wal_val.input_hashs, tot_val);
  //       } else {
  //         panic!();
  //       }
  //     });

  

}
