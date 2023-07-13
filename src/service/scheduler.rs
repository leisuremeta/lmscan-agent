
use std::collections::HashMap;

use bigdecimal::BigDecimal;
use chrono::{Utc, Duration};
use futures_util::Future;
use itertools::Itertools;
use log::info;
use once_cell::sync::OnceCell;
use crate::entity::{balance_entity, block_state, state_daily};
use crate::library::common;
use crate::model::balance::Balance;
use crate::model::balance_info::BalanceInfo;
use sea_orm::*;

use super::api_service::ApiService;
use log::error;
use clokwerk::{TimeUnits, AsyncScheduler, Job};
use chrono_tz::Asia::Seoul;
use std::time::Duration as StdDuration;


static DB: OnceCell<DatabaseConnection> = OnceCell::new();

pub struct SchedulerHandler {}

impl SchedulerHandler{
  pub fn init(db: DatabaseConnection) {
    DB.set(db).unwrap();
  }

  pub async fn run() {
    tokio::spawn(async move {
      println!("1");
      let mut scheduler = AsyncScheduler::with_tz(Seoul);
  
      scheduler
        // .every(1.day())
        .every(10.seconds())
        // .at("00:00") // KST 00:00
        .run(|| async {
          Self::check_invalid_balance_accounts().await
        });  
  
      loop {
        println!("3");
        scheduler.run_pending().await;
        tokio::time::sleep(StdDuration::from_millis(1000)).await;
      }
    });
  }

  pub async fn check_invalid_balance_accounts() {
    println!("2");
    let latest_built_block = block_state::Entity::find()
                                                .filter(block_state::Column::IsBuild.eq(true))
                                                .order_by_desc(block_state::Column::Number)
                                                .one(DB.get().unwrap())
                                                .await.unwrap().unwrap();

    let last_validated_state_block_no = state_daily::Entity::find()
                                                        .order_by_desc(state_daily::Column::BlockNumber)
                                                        .one(DB.get().unwrap())
                                                        .await.unwrap()
                                                        .map_or(0, |b| b.block_number);

    let balances = balance_entity::Entity::find()
                                                      .filter(balance_entity::Column::BlockNumber.gt(last_validated_state_block_no))
                                                      .all(DB.get().unwrap())
                                                      .await.unwrap();
    if balances.is_empty() {
      info!("Fungible 타입 트랜잭션이 발생하지 않았습니다.");
      return; 
    }
    let balances: Vec<(String, Balance)> = balances.into_iter().map(|b| (b.address.clone(), Balance::from(b.clone()))).collect();
  
    let mut unsynced_balances = Vec::new();
    for (address, balance) in balances.iter() {
      let (scan_free, scan_locked) = (balance.free(), balance.locked());
      
      println!("address: {address}");
      let blc_free = balance_from_blockchain(|| ApiService::get_free_balance(address)).await;
      let blc_locked = balance_from_blockchain(|| ApiService::get_locked_balance(address)).await;
  
      if match (scan_free == blc_free, scan_locked == blc_locked) {
        (true,  true)  => false,
        (false, true)  => log_and_continue(format!("{address} 계정의 Free balacne 가 맞지 않습니다. {blc_free} != {scan_free}")),
        (true,  false) => log_and_continue(format!("{address} 계정의 Locked balacne 가 맞지 않습니다. {blc_locked} != {scan_locked}")),
        (false, false) => log_and_continue(format!("{address} 계정의 Balacne 가 모두 맞지 않습니다. Free: {blc_free} != {scan_free}, Locked: {blc_locked} != {scan_locked}")),
      } {
        unsynced_balances.push(address.clone());
      }
    }
  
    for  _ in 0..3 {
      if unsynced_balances.is_empty() {
        break;
      }
      unsynced_balances = filter_unmatching_account(&unsynced_balances, DB.get().unwrap()).await;
    }
  
    let synced_balances: Vec<(String, Balance)> = balances.into_iter().filter(|(addr, _)| !unsynced_balances.contains(addr)).collect();
    let daily_states: Vec<state_daily::ActiveModel> = 
        synced_balances
          .into_iter()
          .map(|(address, balance)| 
            state_daily::ActiveModel {
              address: Set(address),
              free: Set(balance.free()),
              locked: Set(balance.locked()),
              block_number: Set(latest_built_block.number),
              created_at: Set(common::now()),
            }
          )
          .collect();
    
    let outer_vec: Vec<Vec<state_daily::ActiveModel>> = 
        daily_states
          .into_iter()
          .chunks(100)
          .into_iter()
          .map(|m| m.collect())
          .collect();
  
    let txn = DB.get().unwrap().begin().await.unwrap();
    for daily_states in outer_vec {
      if let Err(err) = state_daily::Entity::insert_many(daily_states)
                                                   .exec(&txn).await {
        panic!("state_daily err: {err}");
      }
    }
    txn.commit().await.unwrap();
  }
}


async fn filter_unmatching_account(accounts: &Vec<String>, db: &DatabaseConnection) -> Vec<String> {
  let mut new_accounts = Vec::new();
  for address in accounts.iter() {
    let balance = balance_entity::Entity::find_by_id(address.clone()).one(db).await.unwrap().unwrap();
    let (scan_free, scan_locked) = (balance.free, balance.locked);

    let blc_free = balance_from_blockchain(|| ApiService::get_free_balance(address)).await;
    let blc_locked = balance_from_blockchain(|| ApiService::get_locked_balance(address)).await;

    match (scan_free == blc_free, scan_locked == blc_locked) {
      (true,  true)  => new_accounts.push(address.clone()),
      (false, true)  => { log_and_continue(format!("{} 계정의 Free balacne 가 맞지 않습니다. {} != {}", address, blc_free, scan_free)); },
      (true,  false) => { log_and_continue(format!("{} 계정의 Locked balacne 가 맞지 않습니다. {} != {}", address, blc_locked, scan_locked)); },
      (false, false) => { log_and_continue(format!("{} 계정의 Balacne 가 모두 맞지 않습니다. Free: {} != {}, Locked: {} != {}", address, blc_free, scan_free, blc_locked, scan_locked)); },
    };
  }
  new_accounts
}

async fn balance_from_blockchain<F, Fut>(balance_fetcher: F) -> BigDecimal
where
  F: Fn() -> Fut,
  Fut: Future<Output = Result<Option<HashMap<String, BalanceInfo>>, String>>,
{
  match balance_fetcher().await.unwrap() {
    Some(response) => 
      response.get("LM")
        .ok_or("LM not found in response")
        .unwrap()
        .total_amount
        .clone(),
    None => BigDecimal::default(),
  }
}

fn log_and_continue(message: String) -> bool {
  error!("{}", message);
  false
}
