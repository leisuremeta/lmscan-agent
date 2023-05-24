use std::{fs::File, path::Path, io::Write, collections::{HashMap, HashSet}};

use bigdecimal::BigDecimal;
use itertools::Itertools;
use lmscan_agent::{entity::account_entity, service::api_service::ApiService, block_state, library::common::parse_from_json_str, transaction::{TransactionWithResult, Job, Common}, block_entity, tx_state};
use dotenvy::{dotenv, var};
use lmscan_agent::library::common::db_connn;
use sea_orm::{*, sea_query::Expr};
use lmscan_agent::tx_state::{Entity as TxState};

#[tokio::test]
async fn balance_local_build() {
  dotenv().expect("Unable to load environment variables from .env file");
  let database_url = var("DATABASE_URL").expect("DATABASE_URL must be set.");
  let ref db = db_connn(database_url).await;

  let mut output_file = File::create(Path::new("balance_local_build.txt"))
                                    // .append(true)
                                    // .open("")
                                    .expect("cannot open output file");
  output_file.write(format!("address, blc_balance, scan_balance, equal, diff\n").as_bytes()).expect("write failed");
  let result = build_saved_state_proc(db, HashMap::new(), &mut HashMap::new()).await;
  output_file.write(format!("builded account total count: {}\n", result.keys().count()).as_bytes()).expect("write failed");

  let scan_accounts = account_entity::Entity::find().all(db).await.unwrap();
  output_file.write(format!("scan account total count: {}\n", scan_accounts.len()).as_bytes()).expect("write failed");

  for scan_account in scan_accounts.into_iter() {
    let scan_addr = &scan_account.address;
    if let Some(scan_balance) = result.get(scan_addr) {
      // println!("{count} - scan {key} - {scan_balance}");
      match ApiService::get_account_balance(scan_addr).await {
        Ok(block_account_balance_opt) => {
          if block_account_balance_opt.is_none() {
            println!("balance doesn't exist '{}'", scan_addr);
            continue;
          }

          let block_account_balance = block_account_balance_opt.unwrap();
          if let Some(block_info) = block_account_balance.get("LM") {
            println!("block_info.total_amount: {}", block_info.total_amount);
            let line = format!("{}, {}, {}, {}, {}\n", 
                      scan_addr, 
                      block_info.total_amount,
                      scan_balance,
                      block_info.total_amount.clone() == scan_balance.clone(),
                      if block_info.total_amount.clone() > scan_balance.clone() { block_info.total_amount.clone() - scan_balance.clone() } else { scan_balance.clone() - block_info.total_amount.clone() }
                    );
            output_file.write(line.as_bytes()).expect("write failed");
          }
        }
        Err(err) => println!("request err: {err}")
      }
    }
  }
}


async fn build_saved_state_proc(db: &DatabaseConnection, mut account_balance_info: HashMap<String, BigDecimal>, nft_owner_info: &mut HashMap<String, String>) -> HashMap<String, BigDecimal>{
  
  while let Some(block_states) = get_block_states_not_built_order_by_asc_limit(db).await  {
    let mut cloned_account_balance_info = account_balance_info.clone();
    let mut block_entities = vec![];
    let mut balance_updated_accounts = HashSet::new();

    let block_hashs = block_states.iter().map(|b|b.hash.clone()).collect::<Vec<String>>();
    let mut txs_in_block = get_tx_states_in_block_hashs(block_hashs, db).await;

    for block_state in block_states.iter() {
      let block = parse_from_json_str(block_state.json.as_str());
      block_entities.push(block_entity::Model::from(&block, block_state.hash.clone()));
      println!("{}", block.header.number);
      if let Some(tx_states_in_block) = txs_in_block.remove(&block_state.hash) {
        let iter = tx_states_in_block.iter().map(|state| (state, parse_from_json_str::<TransactionWithResult>(state.json.as_str())))
                                      .sorted_by_key(|(_, tx_res)| tx_res.signed_tx.value.created_at());

        for (tx_state, tx_res) in iter {
          balance_updated_accounts.extend(tx_res.update_account_balance_info(&mut cloned_account_balance_info).await);
          // transfered_nft_token_ids.extend(tx_res.update_nft_owner_info(nft_owner_info));
          
        }
      }
    }

    // let this_time_updated_nft_owners = extract_updated_nft_owners(&nft_owner_info, transfered_nft_token_ids);
    // let this_time_updated_balance_accounts = extract_updated_balance_accounts(&cloned_account_balance_info, balance_updated_accounts);
    // let addresses = extract_addresses(additional_entity_store.get(&AdditionalEntityKey::CreateAccount));
    // let token_ids = extract_token_ids(additional_entity_store.get(&AdditionalEntityKey::CreateNftFile));
    match finish_all_block_states(block_states, db).await {
      true => account_balance_info = cloned_account_balance_info,
      false => panic!("finish_all_block_states error"),
    }
  } 
  account_balance_info
}


async fn get_block_states_not_built_order_by_asc_limit(db: &DatabaseConnection) -> Option<Vec<block_state::Model>> {
  block_state::Entity::find()
                      .filter(block_state::Column::IsBuild.eq(false))
                      .order_by_asc(block_state::Column::Number)
                      .paginate(db, BUILD_BATCH_UNIT).fetch_and_next().await.unwrap()
}

static BUILD_BATCH_UNIT: u64 = 50;


async fn get_tx_states_in_block_hashs(block_hashs: Vec<String>, db: &DatabaseConnection) -> HashMap<String, Vec<tx_state::Model>> {

  TxState::find().filter(tx_state::Column::BlockHash.is_in(block_hashs))
                  .order_by_asc(tx_state::Column::EventTime)
                  .all(db).await.unwrap()
                  .into_iter()
                  .fold(HashMap::new(), |mut acc, tx| {
                    acc.entry(tx.block_hash.clone()).or_insert_with(Vec::new).push(tx);
                    acc
                  })
}

async fn finish_all_block_states(block_states: Vec<block_state::Model>, db: &DatabaseConnection) -> bool {
  if block_states.is_empty() { return true }
  if let Err(err) = block_state::Entity::update_many()
                                        .col_expr(block_state::Column::IsBuild, Expr::value(true))
                                        .filter(block_state::Column::Hash.is_in(block_states.iter().map(|b| b.hash.clone())
                                        .collect::<Vec<String>>()))
                                        .exec(db).await {
    return false;
  }
  true
}