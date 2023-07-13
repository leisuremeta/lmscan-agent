use std::collections::{HashMap, HashSet};

use itertools::Itertools;
use log::{info};
use sea_orm::{DatabaseConnection, EntityTrait, sea_query::Expr, QueryFilter, ColumnTrait, TransactionTrait, DatabaseTransaction, DatabaseBackend, Statement, ConnectionTrait};

use crate::{store::{free_balance::FreeBalanceStore, locked_balance::LockedBalanceStore, wal::{StateType, State}}, model::balance::Balance, balance_entity, block_state, state_daily};


pub struct StateBuilder {}

impl StateBuilder {
  pub async fn build(db: DatabaseConnection, snapshot_no: u64) -> HashMap<String, Balance>{
    if snapshot_no == 0 {
      info!("time travel didn't run - {snapshot_no}");
      return Self::get_balance_infos(&db).await
    }

    // if !Self::check_if_valid_stage_no(snapshot_no, &db).await {
    //   panic!("{snapshot_no} is not a checked snapshot stage number");
    // }

    println!("state building to {snapshot_no}");
    // Collect each state logs.
    let free_state_log = FreeBalanceStore::collect_log_limit(snapshot_no);
    let locked_state_log = LockedBalanceStore::collect_log_limit(snapshot_no);

    // let last_built_stage_no = std::cmp::max(free_last_built_stage_no, locked_last_built_stage_no);

    // Merge into total state log.
    let mut total_logs = free_state_log.clone();
    total_logs.extend(locked_state_log.clone());

    let account_state_info: HashMap<String, (Balance, HashSet<String>)> = 
          total_logs
            .clone()
            .into_iter()
            .into_group_map()
            .into_iter()
            .fold(HashMap::new(), |mut acc, (account, balances)| {
              let states: (Balance, HashSet<String>) = balances
                  .into_iter()
                  .fold((Balance::default(), HashSet::new()), |mut acc, state| {
                    match state {
                      StateType::Free((state_no, state)) => {
                        acc.0.update_free(state.balance, state_no);
                        acc.1.extend(state.input_hashs);
                      },
                      StateType::Locked((state_no, state)) => {
                        acc.0.update_locked(state.balance, state_no);
                        acc.1.extend(state.input_hashs);
                      },
                    }
                    acc
                  });
              acc.insert(account, states);
              acc
            });
    
    let txn = db.begin().await.unwrap();
    Self::cancel_block_built_states_gt(snapshot_no, &txn).await;
    Self::delete_all_balances(&txn).await;
    Self::insert_all_newly_built_balances(&account_state_info.clone(), &txn).await;

    let account_inputs: Vec<(String, HashSet<String>)> = 
        free_state_log
          .into_iter()
          .into_group_map()
          .into_iter()
          .map(|(k, v)| 
            (
              k,
              v.into_iter()
               .flat_map(|s| 
                 match s {
                   StateType::Free((_, s)) => s.input_hashs,
                   StateType::Locked(_) => HashSet::new(),
                 }
               )
               .collect()
            )  
          )
          .collect();
    FreeBalanceStore::overwrite_total_input(account_inputs);

    let locked_input_hashs: HashSet<String> =
        locked_state_log
          .into_iter()
          .flat_map(|(_, state)|
            match state {
              StateType::Locked((_, s)) => s.input_hashs,
              StateType::Free(_) => HashSet::new(),
            }
          )
          .collect();
    LockedBalanceStore::overwrite_total_input(locked_input_hashs);
    txn.commit().await.unwrap();

    account_state_info
      .into_iter()
      .map(|(addr, state)| (addr, state.0))
      .collect()
  }

  async fn check_if_valid_stage_no(stage_no: u64, db: &DatabaseConnection) -> bool {
    state_daily::Entity::find()
          .filter(state_daily::Column::BlockNumber.eq(stage_no))
          .one(db)
          .await
          .unwrap()
          .is_some()
  }

  async fn get_balance_infos(db: &DatabaseConnection) -> HashMap<String, Balance> {
    let balances = balance_entity::Entity::find().all(db).await.unwrap();
    balances.into_iter()
            .map(|b| (b.address, Balance::new(b.free, b.locked, b.block_number)))
            .collect()
  }

  async fn cancel_block_built_states_gt(block_num: u64, db: &DatabaseTransaction) {
    if let Err(err) = block_state::Entity::update_many()
                                          .col_expr(block_state::Column::IsBuild, Expr::value(false))
                                          .filter(block_state::Column::Number.gt(block_num))
                                          .exec(db).await {
      panic!("cancel_block_build_states_gt fail : {err}");
    }
  }

  async fn delete_all_balances(db: &DatabaseTransaction) {
    if let Err(err) = balance_entity::Entity::delete_many()
                          .exec(db).await {
      panic!("cancel_block_build_states_gt fail : {err}");
    }
  }

  async fn insert_all_newly_built_balances(balance_info: &HashMap<String, (Balance, HashSet<String>)>, db: &DatabaseTransaction){
    if balance_info.is_empty() { return }
    let balance_info = balance_info.iter()
                                           .map(|(addr, (b, _))| 
                                             format!("('{addr}',{},{},{})", b.free(), b.locked(), b.block_number()))
                                           .collect::<Vec<String>>()
                                           .join(",");
  
    let query = format!(
      r#"INSERT INTO balance (address,free,locked,block_number)
        VALUES {balance_info}
        ON CONFLICT (address) 
        DO UPDATE 
        SET 
          free = EXCLUDED.free,
          locked = EXCLUDED.locked,
          block_number = EXCLUDED.block_number;"#
      );
  
    if let Err(err) = db.execute(Statement::from_string(DatabaseBackend::Postgres,query.to_owned())).await {    
      panic!("update_all_account_balance_info fail {balance_info}:{err}");
    }
  }  
  
}

#[tokio::test] 
async fn test() {
  let snapshot_no = 2050;
  let free_state_log = FreeBalanceStore::collect_log_limit(snapshot_no);
  let locked_state_log = LockedBalanceStore::collect_log_limit(snapshot_no);

  // let last_built_stage_no = std::cmp::max(free_last_built_stage_no, locked_last_built_stage_no);

  // Merge into total state log.
  let mut total_logs = free_state_log.clone();
  total_logs.extend(locked_state_log.clone());


  let account_state_info: HashMap<String, (Balance, HashSet<String>)> = 
      free_state_log
          .clone()
          .into_iter()
          .into_group_map()
          .into_iter()
          .fold(HashMap::new(), |mut acc, (account, balances)| {
            let states: (Balance, HashSet<String>) = balances
                .into_iter()
                .fold((Balance::default(), HashSet::new()), |mut acc, state| {
                  match state {
                    StateType::Free((state_no, state)) => {
                      acc.0.update_free(state.balance, state_no);
                      acc.1.extend(state.input_hashs);
                    },
                    StateType::Locked((state_no, state)) => {
                      acc.0.update_locked(state.balance, state_no);
                      acc.1.extend(state.input_hashs);
                    },
                  }
                  acc
                });
            acc.insert(account, states);
            acc
          });
  

  let account_inputs: Vec<(String, HashSet<String>)> = 
        account_state_info
          .iter()
          .map(|(addr, state)| (addr.clone(), state.1.clone()))
          .collect();
  
  println!("account_inputs: {:?}", account_inputs);
  // FreeBalanceStore::overwrite_total_input(account_inputs);

  let locked_input_hashs: HashSet<String> =
      locked_state_log
        .into_iter()
        .flat_map(|(_, state)|
          match state {
            StateType::Locked((_, s)) => s.input_hashs,
            StateType::Free(_) => panic!(),
          }
        )
        .collect();
  
  println!("locked_input_hashs: {:?}", locked_input_hashs);
  // LockedBalanceStore::overwrite_total_input(locked_input_hashs);

  let res: HashMap<String, Balance> = account_state_info.into_iter().map(|(addr, state)| (addr, state.0)).collect();

  let database_url = dotenvy::var("DATABASE_URL").expect("DATABASE_URL must be set.");
  let db = crate::library::common::db_connn(database_url).await;
  let db_res = StateBuilder::get_balance_infos(&db).await;

  assert_eq!(res.len(), db_res.len());
  for (a, b) in res {
    println!("{a}");
    match db_res.get(&a) {
      Some(db_b) => assert_eq!(db_b.free(), b.free()),
      None => panic!(),
    }

    match db_res.get(&a) {
      Some(db_b) => assert_eq!(db_b.locked(), b.locked()),
      None => panic!(),
    }
  }
  // println!("result: {:?}", res);
  // println!("db_res: {:?}", db_res);
  // assert_eq!(res, db_res);
}
