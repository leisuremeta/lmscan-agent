use std::collections::HashSet;
use std::hash::Hash;
use std::{collections::HashMap};
use std::time::Duration;
use std::vec;

use lmscan_agent::transaction::{TransactionWithResult, Common, Job, AdditionalEntity, ExtractEntity, AdditionalEntityKey};
use rust_decimal::Decimal;
use rust_decimal::prelude::FromPrimitive;
use rust_decimal_macros::dec;
use sea_orm::sea_query::{OnConflict, Expr};

use lmscan_agent::{model::{block::Block, node_status::NodeStatus}, entity::*};
use sea_orm::{Database, DatabaseConnection, ConnectOptions};
use sea_orm::*;
use lmscan_agent::summary;
use lmscan_agent::block_state::{Entity as BlockState};
use lmscan_agent::tx_state::{Entity as TxState};
use lmscan_agent::tx_entity::{Entity as TxEntity};
use lmscan_agent::block_entity::Entity as BlockEntity;
use lmscan_agent::library::common::*;
use lmscan_agent::model::lm_price::LmPrice;
use itertools::Itertools; 

use log::{error, info, LevelFilter};

extern crate dotenvy;
use dotenvy::{dotenv, var};
use tokio::time::sleep;

static DOWNLOAD_BATCH_UNIT: u32 = 50;
static BUILD_BATCH_UNIT: u64 = 50;
static BASE_URI: &str = "http://lmc.leisuremeta.io";
// static BASE_URI: &str = "http://test.chain.leisuremeta.io";


async fn get_last_saved_lm_price(db: &DatabaseConnection) -> Option<summary::Model> {
  summary::Entity::find().order_by_desc(summary::Column::BlockNumber).one(db).await.unwrap()
}

async fn get_node_status_always() -> NodeStatus {
  get_request_always(format!("{BASE_URI}/status")).await
}

async fn get_block_always(hash: &str) -> Block {
  get_request_always(format!("{BASE_URI}/block/{hash}")).await
}

async fn get_tx_always(hash: &str) -> TransactionWithResult {
  get_request_always(format!("{BASE_URI}/tx/{hash}")).await
}

async fn get_last_built_or_genesis_block_hash(node_status: &NodeStatus, db: &DatabaseConnection) -> String {
  match get_last_built_block(db).await {
    Some(block) => block.hash,
    None => node_status.genesis_hash.to_owned(),
  }
}

async fn get_last_saved_or_genesis_block_hash(node_status: &NodeStatus, db: &DatabaseConnection) -> String {
  match get_last_saved_block(db).await {
    Some(block) => block.hash,
    None => node_status.genesis_hash.to_owned(),
  }
}

async fn get_last_saved_summary(db: &DatabaseConnection) -> Option<summary::Model> {
  summary::Entity::find().order_by_desc(summary::Column::Id)
                         .one(db).await.unwrap()
}

async fn get_last_saved_block(db: &DatabaseConnection) -> Option<block_state::Model> {
  BlockState::find().order_by_desc(block_state::Column::Number)
                    .one(db).await.unwrap()
}

async fn get_last_built_block(db: &DatabaseConnection) -> Option<block_state::Model> {
  BlockState::find().filter(block_state::Column::IsBuild.eq(true))
                    .order_by_desc(block_state::Column::Number)
                    .one(db).await.unwrap()
} 


async fn get_tx_states_in_block_hashs(block_hashs: Vec<String>, db: &DatabaseConnection) -> HashMap<String, Vec<tx_state::Model>> {
  let mut map: HashMap<String, Vec<tx_state::Model>> = HashMap::new();
  TxState::find().filter(tx_state::Column::BlockHash.is_in(block_hashs))
                  .order_by_asc(tx_state::Column::EventTime)
                  .all(db).await.unwrap().into_iter().for_each(|tx| {
                    match map.get_mut(&tx.block_hash) {
                      Some(vec) => { vec.push(tx) },
                      None => { map.insert(tx.block_hash.clone(), vec![tx]); } ,
                    }
                  });
  map
}

async fn get_tx_states_by_block_hash(block_hash: String, db: &DatabaseConnection) -> Vec<tx_state::Model> {
  TxState::find().filter(tx_state::Column::BlockHash.eq(block_hash))
                  .order_by_asc(tx_state::Column::EventTime)
                  .all(db).await.unwrap()
}

async fn get_block_states_not_built_order_by_asc_limit(db: &DatabaseConnection) -> Option<Vec<block_state::Model>> {
  block_state::Entity::find()
                      .filter(block_state::Column::IsBuild.eq(false))
                      .order_by_asc(block_state::Column::Number)
                      .paginate(db, BUILD_BATCH_UNIT).fetch_and_next().await.unwrap()
}

async fn get_total_accounts(db: &DatabaseConnection) -> Option<i64> {
  let query = format!(
    r#"SELECT
         COUNT(account.address) AS count
       FROM
         account;"#);

  match db.query_one(Statement::from_string(DatabaseBackend::Postgres,query.to_owned())).await.ok() {
    Some(res) if res.is_some() => {
      return Some(res.unwrap().try_get::<i64>("", "count").unwrap());
    }
    _ => None
  }
}

async fn get_account_balance_infos(db: &DatabaseConnection) -> HashMap<String, i128> {
  let accounts = account_entity::Entity::find().all(db).await.unwrap();
  accounts.into_iter().map(|account| (account.address.to_owned(), account.balance.mantissa())).collect::<HashMap<String, i128>>()
}

async fn get_nft_owner_infos(db: &DatabaseConnection) -> HashMap<String, String> {
  let nft_files = nft_file::Entity::find().all(db).await.unwrap();
  nft_files.into_iter().map(|nft| (nft.token_id, nft.owner)).collect::<HashMap<String, String>>()
}

async fn get_newly_accumumlated_tx_json_size(db: &DatabaseConnection, last_summary_opt: Option<summary::Model>) -> Option<i64> {
  match last_summary_opt {
    Some(last_summay) => {
      let query = block_entity::Entity::find().select_only()
                                                .column_as(block_entity::Column::Hash, "hash")
                                                .filter(block_entity::Column::Number.gt(last_summay.block_number))
                                                .build(DbBackend::Postgres);

      let block_hashs = match db.query_one(Statement::from_string(DatabaseBackend::Postgres,query.to_string())).await.ok() {
        Some(vec) => {
          vec.into_iter().map(|r| r.try_get::<String>("", "hash").unwrap()).into_iter().join(",")
        }
        _ => panic!(),
      };
      
      if block_hashs.is_empty() {
        return Some(last_summay.total_tx_size)
      }

      let query = format!(
        r#"select 
               CAST(pg_column_size(json) AS BIGINT) AS size
           from 
               tx
           where 
               block_hash in ({block_hashs});"#);
    
      match db.query_one(Statement::from_string(DatabaseBackend::Postgres,query.to_owned())).await.ok() {
        Some(res) if res.is_some() => {
          let new_txs_size = res.unwrap().try_get::<i64>("", "size").unwrap();
          return Some(last_summay.total_tx_size + new_txs_size)
        }
        _ => None
      }
    },
    None => {
      let query = format!(
        r#"select 
               CAST(pg_column_size(json) AS BIGINT) AS size
           from 
               tx;"#);
    
      match db.query_all(Statement::from_string(DatabaseBackend::Postgres,query.to_owned())).await.ok() {
        Some(vec) => {
          let new_txs_size = vec.into_iter().map(|r| r.try_get::<i64>("", "size").unwrap()).sum();
          return Some(new_txs_size)
        }
        _ => None
      }
    },
  }
}

async fn get_lm_price(db: &DatabaseConnection, api_key: String) -> Option<Decimal> {
  let lm_token_id = 20315;
  let coin_market: LmPrice = get_request_header_always(format!("https://pro-api.coinmarketcap.com/v2/cryptocurrency/quotes/latest?id={lm_token_id}"), &api_key).await;
  if coin_market.status.error_code == 0 {
    match coin_market.data.get(&lm_token_id) {
      Some(data) => return Some(Decimal::from_f32(data.quote.usd.price).unwrap_or_default()),
      None => {
        error!("coin market api returned response error (code: {}, message: {})", 
        coin_market.status.error_code, coin_market.status.error_message.unwrap_or_default());
      },
    }
  } 
  match get_last_saved_lm_price(db).await {
    Some(latest_price) => Some(latest_price.lm_price),
    None => Some(dec!(0.0)),
  }
}


async fn save_all_block_states(block_states: Vec<block_state::ActiveModel>, txn: &DatabaseTransaction) {
  if block_states.is_empty() { return }
  if let Err(err) = block_state::Entity::insert_many(block_states)
                                               .on_conflict(OnConflict::column(block_state::Column::Hash).do_nothing().to_owned())
                                               .exec(txn).await {
    info!("save_all_block_states err - {err}");
  }
}

async fn save_all_tx_states(txs: Vec<tx_state::ActiveModel>, txn: &DatabaseTransaction) {
  if txs.is_empty() { return }
  if let Err(err) = TxState::insert_many(txs)
                                  .on_conflict(OnConflict::column(tx_state::Column::Hash).do_nothing().to_owned())
                                  .exec(txn).await {
    info!("save_all_tx_states err - {err}");
  }
}

async fn save_all_blocks(block_entities: Vec<block_entity::ActiveModel>, db: &DatabaseTransaction) -> bool {
  if block_entities.is_empty() { return true }
  if let Err(err) = BlockEntity::insert_many(block_entities)
                                        .on_conflict(OnConflict::column(block_entity::Column::Hash).do_nothing().to_owned())
                                        .exec(db).await {
    return err != DbErr::RecordNotInserted;
  }
  true
}

async fn save_all_txs(tx_entities: Vec<tx_entity::ActiveModel>, db: &DatabaseTransaction) -> bool {
  if tx_entities.is_empty() { return true }
  if let Err(err) = TxEntity::insert_many(tx_entities)
                                            .on_conflict(OnConflict::column(tx_entity::Column::Hash).do_nothing().to_owned())
                                            .exec(db).await {
    return err != DbErr::RecordNotInserted;
  }
  true
}

async fn save_all_nft_txs(nft_tx_opt: Option<AdditionalEntity>, txn: &DatabaseTransaction) -> bool {
  if let Some(nft_tx) = nft_tx_opt {
    match nft_tx {
      AdditionalEntity::NftTx(vec) => {
        match nft_tx::Entity::insert_many(vec)
            .on_conflict(OnConflict::column(nft_tx::Column::TxHash).do_nothing().to_owned())
            .exec(txn).await.err() {
              Some(err) if err != DbErr::RecordNotInserted => {
                error!("save_all_create_nft_file: {err}");
                return false;
              },
              _ => (),
            }
        }
      _ => panic!("invalid params")
    }
  }
  true
}

async fn firstly_save_all_create_event(create_account_event_opt: Option<AdditionalEntity>, 
                                       create_nft_file_event_opt: Option<AdditionalEntity>, 
                                       db: &DatabaseConnection) {
  let txn = db.begin().await.unwrap();
  match create_nft_file_event_opt {
    Some(AdditionalEntity::CreateNftFile(vec)) => {
      let outer_vec: Vec<Vec<nft_file::ActiveModel>> = vec.into_iter().chunks(10).into_iter().map(|x| x.collect()).collect();
      for vec in outer_vec {
        match nft_file::Entity::insert_many(vec)
                              .on_conflict(OnConflict::column(nft_file::Column::TokenId).do_nothing().to_owned())
                              .exec(&txn).await.err() {
          Some(err) if err != DbErr::RecordNotInserted => {
            error!("save_all_create_nft_file: {err}");
            panic!()
          },
          _ => (),
        }
      }
    }
    _ => (),
  };

  match create_account_event_opt {
    Some(AdditionalEntity::CreateAccount(vec)) => {
      match account_entity::Entity::insert_many(vec)
                                    .on_conflict(OnConflict::column(account_entity::Column::Address).do_nothing().to_owned())
                                    .exec(&txn).await.err() {
        Some(err) if err != DbErr::RecordNotInserted  => {
          error!("save_all_create_account: {err}");
          panic!()
        }
        _ => (),
      }               
    }
    _ => (),
  };
  txn.commit().await.unwrap();
}


async fn update_all_nft_file_owner(token_id_owner_info: HashMap<String, String>, txn: &DatabaseTransaction) -> bool {
  if token_id_owner_info.is_empty() { return true }
  let token_id_owner_info = token_id_owner_info.iter()
                                                       .map(|(token_id, owner)| format!("('{token_id}','{owner}')"))
                                                       .collect::<Vec<String>>().join(",");
  let query = format!(
    r#"update nft_file
      set owner = nv.owner
      from 
        ( values 
          {token_id_owner_info} 
        ) as nv (token_id, owner)
      where nft_file.token_id = nv.token_id;"#);

  match txn.query_one(Statement::from_string(DatabaseBackend::Postgres, query.to_owned())).await {
    Err(err) => {
      error!("update_all_nft_file_owner err: {err}");
      panic!()
    },
    _ => true
  }
}


async fn update_all_account_balance_info(account_balance_info: HashMap<String, i128>, db: &DatabaseTransaction) -> bool {
  if account_balance_info.is_empty() { return true }
  let balance_info = account_balance_info.iter()
                                          .map(|(address, balance)| format!("('{address}',{balance})"))
                                          .collect::<Vec<String>>().join(",");
  let query = format!(
    r#"update account
      set balance = nv.balance
      from 
        ( values 
          {balance_info} 
        ) as nv (address, balance)
      where account.address = nv.address;"#);

  match db.query_one(Statement::from_string(DatabaseBackend::Postgres,query.to_owned())).await {
    Err(err) => {
      error!("update_all_account_balance_info err: {err}");
      false
    },
    _ => true
  }
}

async fn finish_all_block_states(block_states: Vec<block_state::Model>, db: &DatabaseTransaction) -> bool {
  if block_states.is_empty() { return true }
  if let Err(err) = block_state::Entity::update_many()
                                        .col_expr(block_state::Column::IsBuild, Expr::value(true))
                                        .filter(block_state::Column::Hash.is_in(block_states.iter().map(|b| b.hash.clone())
                                        .collect::<Vec<String>>()))
                                        .exec(db).await {
    error!("finish_all_block_states fail : {err}");
    return false;
  }
  true
}


async fn remove_firstly_saved_create_events(addresses: Vec<String>, token_ids: Vec<String>, db: &DatabaseConnection) {
  if !addresses.is_empty() {
    account_entity::Entity::delete_many()
      .filter(account_entity::Column::Address.is_in(addresses))
      .exec(db).await.unwrap();
  }
  if !token_ids.is_empty() {
    nft_file::Entity::delete_many()
      .filter(nft_file::Column::TokenId.is_in(token_ids))
      .exec(db).await.unwrap();
  }
}


fn extract_updated_nft_owners(nft_owner_info: &HashMap<String, String>, transfered_token_id: HashSet<String>) -> HashMap<String, String> {
  let mut this_time_updated_nft_owners = HashMap::new();
  nft_owner_info.iter().filter(|(k,_)| transfered_token_id.contains(*k)).for_each(|(k, v)| {
    this_time_updated_nft_owners.insert(k.clone(), v.clone());
  });
  this_time_updated_nft_owners
}

fn extract_updated_balance_accounts(account_balance_info: &HashMap<String, i128>, balanced_updated_accounts: HashSet<String>) -> HashMap<String, i128> {
  let mut this_time_updated_balance_accounts = HashMap::new();
  account_balance_info.iter().filter(|(k, _)|  balanced_updated_accounts.contains(*k)).for_each(|(k, v)| {
    this_time_updated_balance_accounts.insert(k.clone(), *v);
  });
  this_time_updated_balance_accounts
}

fn extract_token_ids(create_nft_file_opt: Option<&AdditionalEntity>) -> Vec<String> {
  let mut token_ids = vec![];
  create_nft_file_opt.map(|x| match x {
    AdditionalEntity::CreateNftFile(vec) => vec.iter().for_each(|x| token_ids.push(x.token_id.clone().unwrap())),
    _ => (),
  });
  token_ids
}

fn extract_addresses(create_account_opt: Option<&AdditionalEntity>) -> Vec<String> {
  let mut addresses = vec![];
  create_account_opt.map(|x| match x {
    AdditionalEntity::CreateAccount(vec) => vec.iter().for_each(|x| addresses.push(x.address.clone().unwrap())),
    _ => (),
  });
  addresses
}


async fn summary_loop(db: DatabaseConnection, api_key: String) {
  tokio::spawn(async move {
    loop {
      let last_summary_opt = get_last_saved_summary(&db).await;
      match (get_last_built_block(&db).await, 
             get_lm_price(&db, api_key.clone()).await, 
             get_total_accounts(&db).await,
             get_newly_accumumlated_tx_json_size(&db, last_summary_opt).await) 
      {
        (Some(last_built_block), Some(lm_price), Some(total_accounts), Some(total_tx_size)) => {
          let summary = summary::Model::from(last_built_block.number, lm_price, total_accounts, total_tx_size);
          if let Err(err) = summary::Entity::insert(summary).exec(&db).await {
            error!("summary loop failed {}", err);
            panic!();
          }
        },
        _ => {
          error!("summary loop is skiped.")
        }
      }
      sleep(Duration::from_secs(60 * 10)).await;
    }
  }).await.unwrap();
}


async fn save_diff_state_proc(mut curr_block_hash: String, target_hash: String, db: &DatabaseConnection) {
  info!("save_diff_state_proc started");
  let mut is_conitnue = !curr_block_hash.eq(&target_hash);

  let mut block_counter = 0;
  let mut block_states = vec![];
  let mut txs = vec![];
  
  while is_conitnue {
    let block = get_block_always(&curr_block_hash.to_owned()).await;
    let block_state = block_state::Model::from(curr_block_hash.as_str(), &block);
    block_states.push(block_state);
    
    for tx_hash in &block.transaction_hashes {
      let tx_result = get_tx_always(tx_hash).await;
      let tx_state = tx_state::Model::from(tx_hash.as_str(), curr_block_hash.as_str(), &tx_result);
      txs.push(tx_state);
    }
    
    block_counter += 1;
    curr_block_hash = block.header.parent_hash.clone();
    is_conitnue = !curr_block_hash.eq(&target_hash);

    if !is_conitnue || block_counter == DOWNLOAD_BATCH_UNIT {
      let txn = db.begin().await.unwrap();
      save_all_block_states(block_states.clone(), &txn).await;
      save_all_tx_states(txs.clone(), &txn).await;

      block_counter = 0;
      block_states.clear();
      txs.clear();
      txn.commit().await.unwrap();
    }
  }
  info!("save_diff_state_proc ended");
}


async fn build_saved_state_proc(db: &DatabaseConnection, account_balance_info: &mut HashMap<String, i128>, nft_owner_info: &mut HashMap<String, String>) {
  info!("build_saved_state_proc started");
  while let Some(block_states) = get_block_states_not_built_order_by_asc_limit(db).await  {
    let mut tx_entities = vec![];
    let mut block_entities = vec![];
    let mut additional_entity_store = HashMap::new();
    let mut balance_updated_accounts = HashSet::new();
    let mut transfered_nft_token_ids = HashSet::new();

    let block_hashs = block_states.iter().map(|b|b.hash.clone()).collect::<Vec<String>>();
    let mut txs_in_block = get_tx_states_in_block_hashs(block_hashs, db).await;

    for block_state in block_states.iter() {
      let block = parse_from_json_str(block_state.json.as_str());
      block_entities.push(block_entity::Model::from(&block, block_state.hash.clone()));
      
      if let Some(tx_states_in_block) = txs_in_block.remove(&block_state.hash) {
        for tx_state in tx_states_in_block {
          let tx_res = parse_from_json_str::<TransactionWithResult>(tx_state.json.as_str());
          balance_updated_accounts.extend(tx_res.update_account_balance_info(account_balance_info));
          transfered_nft_token_ids.extend(tx_res.update_nft_owner_info(nft_owner_info));
          
          let tx = &tx_res.signed_tx.value;
          let tx_entity = tx.from(tx_state.hash, tx_res.signed_tx.sig.account, 
                                               tx_state.block_hash, block_state.number, 
                                               tx_state.json, tx_res.result);
          tx.extract_additional_entity(&tx_entity, &mut additional_entity_store).await;
          tx_entities.push(tx_entity);
        }
      }
    }

    let this_time_updated_nft_owners = extract_updated_nft_owners(&nft_owner_info, transfered_nft_token_ids);
    let this_time_updated_balance_accounts = extract_updated_balance_accounts(&account_balance_info, balance_updated_accounts);
    let addresses = extract_addresses(additional_entity_store.get(&AdditionalEntityKey::CreateAccount));
    let token_ids = extract_token_ids(additional_entity_store.get(&AdditionalEntityKey::CreateNftFile));
    
    firstly_save_all_create_event(additional_entity_store.remove(&AdditionalEntityKey::CreateAccount),
                                 additional_entity_store.remove(&AdditionalEntityKey::CreateNftFile),
                                                            &db).await;
    
    let save_res = &db.transaction::<_, (), DbErr>(|txn| {
      Box::pin(async move {
        if !save_all_blocks(block_entities, txn).await ||
           !save_all_txs(tx_entities, txn).await ||
           !save_all_nft_txs(additional_entity_store.remove(&AdditionalEntityKey::NftTx), txn).await ||
           !update_all_account_balance_info(this_time_updated_balance_accounts, txn).await ||
           !update_all_nft_file_owner(this_time_updated_nft_owners, txn).await ||
           !finish_all_block_states(block_states, txn).await
        {
          return Err(DbErr::Query(RuntimeErr::Internal("Force Rollback!".to_owned())))
        }    
        Ok(())
      })
    })
    .await;

    if let Err(err) = save_res {
      remove_firstly_saved_create_events(addresses, token_ids, &db).await;
      error!("save transaction err: {err}");
    } 
  } 
  info!("build_saved_state_proc ended");
}


async fn block_check_loop(db: DatabaseConnection) {
  tokio::spawn(async move {
    let mut account_balance_info = get_account_balance_infos(&db).await;
    let mut nft_owner_info = get_nft_owner_infos(&db).await;
    build_saved_state_proc(&db, &mut account_balance_info, &mut nft_owner_info).await;
    loop {
      info!("block_check_loop start");
      let ref node_status = get_node_status_always().await;
      let target_hash = get_last_built_or_genesis_block_hash(node_status, &db).await;
      // let target_hash = get_last_saved_or_genesis_block_hash(node_status, &db).await;
      save_diff_state_proc(node_status.best_hash.clone(), target_hash, &db).await;
            
      // 테스트 전용 로직.
      // let ref node_status = get_node_status_always().await;
      // let last_saved_block = get_last_saved_block(&db).await.unwrap();
      // let target_hash = get_last_built_or_genesis_block_hash(node_status, &db).await;
      // println!("save_diff_state_proc started");
      // save_diff_state_proc(last_saved_block.hash, target_hash, &db).await;
      // println!("save_diff_state_proc ended");

      build_saved_state_proc(&db, &mut account_balance_info, &mut nft_owner_info).await;
      sleep(Duration::from_secs(5)).await;
      info!("block_check_loop end");
    }
  }).await.unwrap()
}


#[tokio::main]
async fn main() {
  dotenv().expect("Unable to load environment variables from .env file");
  log4rs::init_file(var("LOG_CONFIG_FILE_PATH").unwrap(), Default::default()).unwrap();

  let database_url = var("DATABASE_URL").expect("DATABASE_URL must be set.");
  let coin_market_api_key = var("COIN_MARKET_API_KEY").expect("COIN_MARKET_API_KEY must be set.");

  let db = db_connn(database_url).await;

  // tokio::join!(
  //   summary_loop(db.clone(), coin_market_api_key),
  //   block_check_loop(db),
  // );
}

