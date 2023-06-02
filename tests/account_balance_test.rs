use std::{fs::File, path::Path, io::Write, collections::HashMap};

use lmscan_agent::{entity::account_entity, service::api_service::ApiService, model::balance_info::BalanceInfo};
use dotenvy::{dotenv, var};
use lmscan_agent::library::common::db_connn;
use sea_orm::*;


#[tokio::test]
async fn account_balance_test() {
  dotenv().expect("Unable to load environment variables from .env file");
  let database_url = var("DATABASE_URL").expect("DATABASE_URL must be set.");
  let ref db = db_connn(database_url).await;

  let scan_accounts = account_entity::Entity::find()
                                                        //  .filter(account_entity::Column::Balance.ne(0))
                                                         .all(db).await.unwrap();
  let scan_accounts = scan_accounts.into_iter()
                                                                .filter(|acc| acc.created_at != acc.event_time);
  let mut success = 0;
  let mut fail = 0;
  let mut output_file = File::create(Path::new("balance check.txt"))
                                    // .append(true)
                                    // .open("")
                                    .expect("cannot open output file");
  output_file.write(format!("address, blc_balance, scan_balance, equal, diff\n").as_bytes()).expect("write failed");
  for (count, scan_account) in scan_accounts.enumerate() {
    let address = scan_account.address.clone();
    println!("scan_account: {:?}", address);
    // match ApiService::get_account_balance(&scan_account.address).await {
    match ApiService::get_request_until::<String, Option<HashMap<String, BalanceInfo>>>(format!("http://lmc.leisuremeta.io/balance/{address}?movable=free") , 100).await {
      Some(block_account_balance_opt) => {
        // if block_account_balance_opt.is_none() {
        //   println!("balance doesn't exist '{}'", scan_account.address);
        //   continue;
        // }

        let block_account_balance = block_account_balance_opt.unwrap();
        if let Some(block_info) = block_account_balance.get("LM") {
          let line = format!("{}, {}, {}, {}, {}\n", 
                    scan_account.address, 
                    block_info.total_amount,
                    scan_account.balance.clone(),
                    block_info.total_amount.clone() == scan_account.balance,
                    if block_info.total_amount.clone() > scan_account.balance { block_info.total_amount.clone() - scan_account.balance } else { scan_account.balance - block_info.total_amount.clone() }
                  );
          output_file.write(line.as_bytes()).expect("write failed");
        }
      }
      None => println!("balance doesn't exist '{}'", scan_account.address)
    }
  }
}

