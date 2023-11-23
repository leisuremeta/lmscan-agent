use std::collections::HashMap;

use crate::entity::{balance_entity, block_daily, block_state, state_daily};
use crate::library::common;
use crate::model::balance::Balance;
use crate::model::balance_info::BalanceInfo;
use bigdecimal::BigDecimal;
use chrono::{Duration, Utc};
use futures_util::Future;
use itertools::Itertools;
use log::info;
use sea_orm::*;

use super::api_service::ApiService;
use log::error;

pub async fn daily_snapshot(db: &DatabaseConnection) {
    let now = Utc::now();
    let date = now.format("%Y-%m-%d").to_string();
    let start_of_day = (now - Duration::days(10))
        .date_naive()
        .and_hms_opt(15, 0, 0)
        .unwrap()
        .timestamp();
    let end_of_day = now
        .date_naive()
        .and_hms_opt(14, 59, 59)
        .unwrap()
        .timestamp();

    let balances = balance_entity::Entity::find()
        .filter(
            Condition::all()
                .add(balance_entity::Column::UpdatedAt.between(start_of_day, end_of_day)),
        )
        .all(db)
        .await
        .unwrap();
    if balances.is_empty() {
        info!("{now} 기준 Fungible 타입 트랜잭션이 발생하지 않았습니다.");
        return;
    }

    let balances: Vec<(String, Balance)> = balances
        .into_iter()
        .map(|b| (b.address.clone(), Balance::from(b.clone())))
        .collect();
    let mut synced_balances = Vec::new();
    for (address, balance) in balances.into_iter() {
        let (scan_free, scan_locked) = (balance.free(), balance.locked());

        let blc_free = balance_from_blockchain(|| ApiService::get_free_balance(&address)).await;
        let blc_locked = balance_from_blockchain(|| ApiService::get_locked_balance(&address)).await;

        if match (scan_free == blc_free, scan_locked == blc_locked) {
      (true,  true)  => true,
      (false, true)  => log_and_continue(format!("{address} 계정의 Free balacne 가 맞지 않습니다. {blc_free} != {scan_free}")),
      (true,  false) => log_and_continue(format!("{address} 계정의 Locked balacne 가 맞지 않습니다. {blc_locked} != {scan_locked}")),
      (false, false) => log_and_continue(format!("{address} 계정의 Balacne 가 모두 맞지 않습니다. Free: {blc_free} != {scan_free}, Locked: {blc_locked} != {scan_locked}")),
    } {
      synced_balances.push((address, balance));
    }
    }

    let latest_built_block = block_state::Entity::find()
        .filter(block_state::Column::IsBuild.eq(true))
        .order_by_asc(block_state::Column::Number)
        .one(db)
        .await
        .unwrap()
        .unwrap();

    let txn = db.begin().await.unwrap();
    let daily_states: Vec<state_daily::ActiveModel> = synced_balances
        .into_iter()
        .map(|(address, balance)| state_daily::ActiveModel {
            address: Set(address),
            date: Set(date.clone()),
            free: Set(balance.free()),
            locked: Set(balance.locked()),
            created_at: Set(common::now()),
        })
        .collect();

    let daily_block = block_daily::ActiveModel {
        date: Set(date),
        hash: Set(latest_built_block.hash),
        number: Set(latest_built_block.number),
        created_at: Set(common::now()),
    };

    let outer_vec: Vec<Vec<state_daily::ActiveModel>> = daily_states
        .into_iter()
        .chunks(100)
        .into_iter()
        .map(|m| m.collect())
        .collect();

    for daily_states in outer_vec {
        if let Err(err) = state_daily::Entity::insert_many(daily_states)
            .exec(&txn)
            .await
        {
            println!("state_daily err: {err}");
        }
    }

    if let Err(err) = daily_block.insert(&txn).await {
        println!("daily_block err: {err}");
    }

    txn.commit().await.unwrap();
}

async fn balance_from_blockchain<F, Fut>(balance_fetcher: F) -> BigDecimal
where
    F: Fn() -> Fut,
    Fut: Future<Output = Result<Option<HashMap<String, BalanceInfo>>, String>>,
{
    let response = balance_fetcher().await.unwrap().unwrap();
    let total_amount = response
        .get("LM")
        .ok_or("LM not found in response")
        .unwrap()
        .total_amount
        .clone();
    total_amount
}

fn log_and_continue(message: String) -> bool {
    error!("{}", message);
    false
}
