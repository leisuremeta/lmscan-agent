use std::{collections::HashMap, ops::Sub};

use lmscan_agent::{entity::account_entity, library::common::get_request_always, model::balance_info::BalanceInfo};
use dotenvy::{dotenv, var};
use lmscan_agent::library::common::db_connn;
use sea_orm::*;
use log::info;

static BASE_URI: &str = "http://test.chain.leisuremeta.io";
// static BASE_URI: &str = "http://lmc.leisuremeta.io";

async fn get_account_balance_always(hash: &str) -> HashMap<String, BalanceInfo> {
  get_request_always(format!("{BASE_URI}/balance/{hash}?movable=free")).await
}

#[tokio::test]
async fn validate_account_balance() {
  dotenv().expect("Unable to load environment variables from .env file");
  let database_url = var("DATABASE_URL").expect("DATABASE_URL must be set.");
  let ref db = db_connn(database_url).await;

  let scan_accounts = account_entity::Entity::find().filter(account_entity::Column::Balance.gt(0)).all(db).await.unwrap();
  let mut success = 0;
  let mut fail = 0;
  for (count, scan_account) in scan_accounts.into_iter().enumerate() {
    println!("scan_account: {:?}", scan_account.address);
    let block_account_balance = get_account_balance_always(&scan_account.address).await;
    if let Some(block_info) = block_account_balance.get("LM") {
      // assert_eq!() 매크로는 둘이 같은 지를 체크하고 같지 않으면 에러를 냄.
      // assert_eq!(block_info.total_amount, scan_account.balance);

      if block_info.total_amount == scan_account.balance {
        success += 1;
        // println!("{success}개 성공");
      } else {
        fail += 1;
        println!("{fail}: {}", block_info.total_amount.clone() - scan_account.balance);
      }
    }
  }
  
  

}
