use std::{collections::HashMap, ops::Sub, fs::{OpenOptions, File}, path::Path, io::Write};

use lmscan_agent::{entity::account_entity, library::common::{get_request_always, get_request}, model::{balance_info::BalanceInfo, account_info::AccountInfo}};
use dotenvy::{dotenv, var};
use lmscan_agent::library::common::db_connn;
use sea_orm::*;
use log::info;

static BASE_URI: &str = "http://test.chain.leisuremeta.io";
// static BASE_URI: &str = "http://lmc.leisuremeta.io";

async fn get_account_balance(hash: &str) -> Result<Option<HashMap<String, BalanceInfo>>, String> {
  get_request(format!("{BASE_URI}/balance/{hash}?movable=free")).await
}

async fn get_account_always(address: &str) -> AccountInfo {
  get_request_always(format!("{BASE_URI}/account/{address}")).await
}



#[tokio::test]
async fn validate_account_balance() {
  dotenv().expect("Unable to load environment variables from .env file");
  let database_url = var("DATABASE_URL").expect("DATABASE_URL must be set.");
  let ref db = db_connn(database_url).await;

  let scan_accounts = account_entity::Entity::find()
                                                         //.filter(account_entity::Column::Balance.gt(0))
                                                         .all(db).await.unwrap();
  let mut success = 0;
  let mut fail = 0;
  let mut output_file = File::create(Path::new("balance check.txt"))
                                    // .append(true)
                                    // .open("")
                                    .expect("cannot open output file");
  for (count, scan_account) in scan_accounts.into_iter().enumerate() {
    println!("scan_account: {:?}", scan_account.address);
    match get_account_balance(&scan_account.address).await.ok() {
      Some(block_account_balance_opt) => {
        if block_account_balance_opt.is_none() {
          println!("balance is not found '{}'", scan_account.address);
          continue;
        }

        let block_account_balance = block_account_balance_opt.unwrap();
        if let Some(block_info) = block_account_balance.get("LM") {
          // assert_eq!() 매크로는 둘이 같은 지를 체크하고 같지 않으면 에러를 냄.
          // assert_eq!(block_info.total_amount, scan_account.balance);
    
          if block_info.total_amount != scan_account.balance {
            let line = format!("{}, {}, {}\n", 
                                        scan_account.address, 
                                        block_info.total_amount,
                                        scan_account.balance
                                     );
            println!("{line}");
            output_file.write(line.as_bytes()).expect("write failed");
          } 
        }
      }
      _ => ()
    }
  }
  
  

}
