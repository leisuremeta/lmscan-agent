use std::{collections::HashSet, fs::File, io::Write, path::Path};

use bigdecimal::BigDecimal;
use dotenvy::{dotenv, var};
use lmscan_agent::library::common::db_connn;
use lmscan_agent::{account_entity, balance_entity, service::api_service::ApiService};
use sea_orm::*;

#[tokio::test]
async fn free() {
    dotenv().expect("Unable to load environment variables from .env file");
    let database_url = var("DATABASE_URL").expect("DATABASE_URL must be set.");
    let ref db = db_connn(database_url).await;

    let scan_accounts: HashSet<String> = account_entity::Entity::find()
        .all(db)
        .await
        .unwrap()
        .into_iter()
        .map(|a| a.address)
        .collect();
    let balances = balance_entity::Entity::find().all(db).await.unwrap();
    let balances: Vec<(String, BigDecimal)> = balances
        .into_iter()
        .filter(|b| scan_accounts.contains(&b.address))
        .map(|b| (b.address, b.free))
        .collect();

    let mut output_file =
        File::create(Path::new("balance check.txt")).expect("cannot open output file");
    output_file
        .write(format!("address, blc_balance, scan_balance, equal, diff\n").as_bytes())
        .expect("write failed");
    for (_count, (address, free)) in balances.into_iter().enumerate() {
        println!("{address}");
        if let Some(block_account_balance_opt) = ApiService::get_free_balance(&address).await.ok() {
            if block_account_balance_opt.is_none() {
                println!("balance doesn't exist '{}'", address);
                continue;
            }

            let block_account_balance = block_account_balance_opt.unwrap();
            if let Some(block_info) = block_account_balance.get("LM") {
                let line = format!(
                    "{}, {}, {}, {}, {}\n",
                    address,
                    block_info.total_amount,
                    free.clone(),
                    block_info.total_amount.clone() == free,
                    if block_info.total_amount.clone() > free {
                        block_info.total_amount.clone() - free
                    } else {
                        free - block_info.total_amount.clone()
                    }
                );
                output_file.write(line.as_bytes()).expect("write failed");
            }
        }
    }
}

#[tokio::test]
async fn locked() {
    dotenv().expect("Unable to load environment variables from .env file");
    let database_url = var("DATABASE_URL").expect("DATABASE_URL must be set.");
    let ref db = db_connn(database_url).await;

    let scan_accounts: HashSet<String> = account_entity::Entity::find()
        .all(db)
        .await
        .unwrap()
        .into_iter()
        .map(|a| a.address)
        .collect();
    let balances = balance_entity::Entity::find()
        .filter(Condition::all().add(balance_entity::Column::Free.gt(0)))
        .all(db)
        .await
        .unwrap();
    let balances: Vec<(String, BigDecimal)> = balances
        .into_iter()
        .filter(|b| scan_accounts.contains(&b.address))
        .map(|b| (b.address, b.locked))
        .collect();

    let mut output_file =
        File::create(Path::new("locked_balance_check.txt")).expect("cannot open output file");
    output_file
        .write(format!("address, blc_balance, scan_balance, equal, diff\n").as_bytes())
        .expect("write failed");
    for (_count, (address, locked)) in balances.into_iter().enumerate() {
        println!("{address}");
        if let Some(block_account_balance_opt) = ApiService::get_locked_balance(&address).await.ok()
        {
            if block_account_balance_opt.is_none() {
                println!("balance doesn't exist '{}'", address);
                continue;
            }

            let block_account_balance = block_account_balance_opt.unwrap();
            if let Some(block_info) = block_account_balance.get("LM") {
                let line = format!(
                    "{}, {}, {}, {}, {}\n",
                    address,
                    block_info.total_amount,
                    locked.clone(),
                    block_info.total_amount.clone() == locked,
                    if block_info.total_amount.clone() > locked {
                        block_info.total_amount.clone() - locked
                    } else {
                        locked - block_info.total_amount.clone()
                    }
                );
                output_file.write(line.as_bytes()).expect("write failed");
            }
        }
    }
}
