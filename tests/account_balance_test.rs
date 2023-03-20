use std::collections::HashMap;

use lmscan_agent::{entity::account_entity, library::common::get_request_always, model::balance_info::BalanceInfo};
use dotenvy::{dotenv, var};
use lmscan_agent::library::common::db_connn;
use log::info;
use sea_orm::EntityTrait;

// static BASE_URI: &str = "http://test.chain.leisuremeta.io";
static BASE_URI: &str = "http://lmc.leisuremeta.io";

async fn get_account_balance_always(hash: &str) -> HashMap<String, BalanceInfo> {
  get_request_always(format!("{BASE_URI}/balance/{hash}?movable=free")).await
}

#[tokio::test]
async fn validate_account_balance() {

  info!("00000");
  dotenv().expect("Unable to load environment variables from .env file");
  println!("aaaa");
  let database_url = var("DATABASE_URL").expect("DATABASE_URL must be set.");
  println!("bbbb");
  let ref db = db_connn(database_url).await;
  println!("11111");
  let scan_accounts = account_entity::Entity::find().all(db).await.unwrap();
  println!("22222");
  for (count, scan_account) in scan_accounts.into_iter().enumerate() {
    println!("33333");
    let block_account_balance = get_account_balance_always(&scan_account.address).await;
    println!("4444");
    if let Some(block_info) = block_account_balance.get("LM") {
      // assert_eq!() 매크로는 둘이 같은 지를 체크하고 같지 않으면 에러를 냄.
      assert_eq!(block_info.total_amount, scan_account.balance);
    }
    info!("{count}개 성공");
  }
}
