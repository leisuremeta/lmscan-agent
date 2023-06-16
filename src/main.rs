use std::collections::{HashSet, HashMap};
use std::time::Duration;
use std::vec;

use bigdecimal::BigDecimal;
use lmscan_agent::model::balance::Balance;
use lmscan_agent::service::api_service::ApiService;
use lmscan_agent::service::finder_service::Finder;


use lmscan_agent::store::free_balance::FreeBalanceStore;
use lmscan_agent::store::locked_balance::LockedBalanceStore;
use lmscan_agent::store::sled_store::SledStore;
use lmscan_agent::transaction::{TransactionWithResult, Common, Job, AdditionalEntity, ExtractEntity, AdditionalEntityKey};
use rust_decimal::Decimal;
use rust_decimal::prelude::FromPrimitive;
use rust_decimal_macros::dec;
use sea_orm::sea_query::{OnConflict, Expr};

use lmscan_agent::{model::{block::Block, node_status::NodeStatus}, entity::*};
use sea_orm::DatabaseConnection;
use sea_orm::*;
use lmscan_agent::summary;
use lmscan_agent::block_state::{Entity as BlockState};
use lmscan_agent::tx_state::{Entity as TxState};
use lmscan_agent::tx_entity::{Entity as TxEntity};
use lmscan_agent::block_entity::Entity as BlockEntity;
use lmscan_agent::library::common::*;
use lmscan_agent::model::lm_price::LmPrice;
use itertools::Itertools; 

use log::error;

extern crate dotenvy;
use dotenvy::{dotenv, var};
use tokio::time::sleep;

static DOWNLOAD_BATCH_UNIT: u32 = 50;
static BUILD_BATCH_UNIT: u64 = 50;

async fn get_last_saved_lm_price(db: &DatabaseConnection) -> Option<summary::Model> {
  summary::Entity::find().order_by_desc(summary::Column::BlockNumber).one(db).await.unwrap()
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

async fn get_block_states_after_order_by_asc_limit(db: &DatabaseConnection, num: i64) -> Option<Vec<block_state::Model>> {
  block_state::Entity::find()
                      .filter(block_state::Column::Number.gt(num))
                      .order_by_asc(block_state::Column::Number)
                      .paginate(db, BUILD_BATCH_UNIT).fetch_and_next().await.unwrap()
}

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

async fn get_tx_states_by_block_hash(block_hash: String, db: &DatabaseConnection) -> Vec<tx_state::Model> {
  TxState::find().filter(tx_state::Column::BlockHash.eq(block_hash))
                  .order_by_asc(tx_state::Column::EventTime)
                  .all(db).await.unwrap()
}

async fn get_block_states_not_built_order_by_number_asc_limit(db: &DatabaseConnection) -> Option<Vec<block_state::Model>> {
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

async fn get_balance_infos(db: &DatabaseConnection) -> HashMap<String, Balance> {
  let balances = balance_entity::Entity::find().all(db).await.unwrap();
  balances.into_iter()
          .map(|b| (b.address.clone(), Balance::new(b.free, b.locked)))
          .collect::<HashMap<String, Balance>>()
}

async fn get_nft_owner_infos(db: &DatabaseConnection) -> HashMap<String, String> {
  let nft_owners = nft_owner::Entity::find().all(db).await.unwrap();
  nft_owners.into_iter().map(|nft| (nft.token_id, nft.owner)).collect::<HashMap<String, String>>()
}

async fn get_newly_accumumlated_tx_json_size(db: &DatabaseConnection, last_summary_opt: Option<summary::Model>) -> Option<i64> {
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
  // match last_summary_opt {
  //   Some(last_summay) => {
  //     let query = block_entity::Entity::find().select_only()
  //                                               .column_as(block_entity::Column::Hash, "hash")
  //                                               .filter(block_entity::Column::Number.gt(last_summay.block_number))
  //                                               .build(DbBackend::Postgres);

  //     let block_hashs = match db.query_one(Statement::from_string(DatabaseBackend::Postgres,query.to_string())).await.ok() {
  //       Some(vec) => {
  //         vec.into_iter().map(|r| r.try_get::<String>("", "hash").unwrap()).into_iter().join(",")
  //       }
  //       _ => panic!(),
  //     };
      
  //     if block_hashs.is_empty() {
  //       return Some(last_summay.total_tx_size)
  //     }

  //     let query = format!(
  //       r#"select 
  //              CAST(pg_column_size(json) AS BIGINT) AS size
  //          from 
  //              tx
  //          where 
  //              block_hash in ({block_hashs});"#);
    
  //     match db.query_one(Statement::from_string(DatabaseBackend::Postgres,query.to_owned())).await.ok() {
  //       Some(res) if res.is_some() => {
  //         let new_txs_size = res.unwrap().try_get::<i64>("", "size").unwrap();
  //         return Some(last_summay.total_tx_size + new_txs_size)
  //       }
  //       _ => Some(last_summay.total_tx_size)
  //     }
  //   },
  //   None => {
  //     let query = format!(
  //       r#"select 
  //              CAST(pg_column_size(json) AS BIGINT) AS size
  //          from 
  //              tx;"#);
    
  //     match db.query_all(Statement::from_string(DatabaseBackend::Postgres,query.to_owned())).await.ok() {
  //       Some(vec) => {
  //         let new_txs_size = vec.into_iter().map(|r| r.try_get::<i64>("", "size").unwrap()).sum();
  //         return Some(new_txs_size)
  //       }
  //       _ => None
  //     }
  //   },
  // }
}

async fn get_lm_price(db: &DatabaseConnection, api_key: String) -> Option<Decimal> {
  let lm_token_id = 20315;
  let coin_market: LmPrice = ApiService::get_request_header_always(format!("https://pro-api.coinmarketcap.com/v2/cryptocurrency/quotes/latest?id={lm_token_id}"), &api_key).await;
  if coin_market.status.error_code == 0 {
    match coin_market.data.get(&lm_token_id) {
      Some(data) => 
        return Some(Decimal::from_f32(data.quote.usd.price).unwrap_or_default()),
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
  match block_state::Entity::insert_many(block_states)
                            .on_conflict(OnConflict::column(block_state::Column::Hash).do_nothing().to_owned())
                            .exec(txn).await {
    Err(err) if err != DbErr::RecordNotInserted => {
      error!("save_all_block_states err - {err}");
      panic!("save_all_block_states err : {err}")
    }
    _ => ()
  }
}

async fn save_all_tx_states(txs: Vec<tx_state::ActiveModel>, txn: &DatabaseTransaction) {
  if txs.is_empty() { return }
  match TxState::insert_many(txs)
                .on_conflict(OnConflict::column(tx_state::Column::Hash).do_nothing().to_owned())
                .exec(txn).await {
    Err(err) if err != DbErr::RecordNotInserted => {
      error!("save_all_tx_states err - {err}");
      panic!("save_all_tx_states err : {err}")
    }
    _ => ()
  }
}

async fn save_all_blocks(block_entities: Vec<block_entity::ActiveModel>, db: &DatabaseTransaction) -> bool {
  if block_entities.is_empty() { return true }
  if let Err(err) = BlockEntity::insert_many(block_entities)
                                        .on_conflict(OnConflict::column(block_entity::Column::Hash).do_nothing().to_owned())
                                        .exec(db).await {
    //  panic!("save_all_txs: {err}")
    return err != DbErr::RecordNotInserted;
  }
  true
}

async fn save_all_txs(tx_entities: Vec<tx_entity::ActiveModel>, db: &DatabaseTransaction) -> bool {
  if tx_entities.is_empty() { return true }
  if let Err(err) = TxEntity::insert_many(tx_entities)
                                            .on_conflict(OnConflict::column(tx_entity::Column::Hash).do_nothing().to_owned())
                                            .exec(db).await {
    // panic!("save_all_txs: {err}")
    return err != DbErr::RecordNotInserted;
  }
  true
}

async fn save_all_nft_txs(nft_tx_opt: Option<AdditionalEntity>, txn: &DatabaseTransaction) -> bool {
  if let Some(nft_tx) = nft_tx_opt {
    match nft_tx {
      AdditionalEntity::NftTx(vec) if !vec.is_empty() => {
        match nft_tx::Entity::insert_many(vec)
                             .on_conflict(OnConflict::column(nft_tx::Column::TxHash).do_nothing().to_owned())
                             .exec(txn).await.err() {
              Some(err) if err != DbErr::RecordNotInserted => {
                // panic!("save_all_nft_txs: {err}");
                error!("save_all_nft_txs: {err}");
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

async fn save_all_accounts(create_account_tx_opt: Option<AdditionalEntity>, txn: &DatabaseTransaction) -> bool {
  match create_account_tx_opt {
    Some(AdditionalEntity::CreateAccount(vec)) if !vec.is_empty() => { 
      match account_entity::Entity::insert_many(vec)
                                  .on_conflict(OnConflict::column(account_entity::Column::Address).do_nothing().to_owned())
                                  .exec(txn).await.err() {
        Some(err) if err != DbErr::RecordNotInserted => {
          // info!("create_account_event: {:?}", vec);
          error!("save_all_create_account: {err}");
          return false
        }
        _ => (),
      }
    }
    _ => (),
  }
  true
}

async fn save_all_nft_files(create_nft_file_event_opt: Option<AdditionalEntity>, txn: &DatabaseTransaction) -> bool {
  match create_nft_file_event_opt {
    Some(AdditionalEntity::CreateNftFile(vec)) if !vec.is_empty() => {
      let outer_vec: Vec<Vec<nft_file::ActiveModel>> = vec.into_iter()
                                                          .chunks(10)
                                                          .into_iter()
                                                          .map(|x| x.collect())
                                                          .collect();
      for vec in outer_vec {
        match nft_file::Entity::insert_many(vec)
                              .on_conflict(OnConflict::column(nft_file::Column::TokenId).do_nothing().to_owned())
                              .exec(txn).await.err() {
          Some(err) if err != DbErr::RecordNotInserted => {
            // panic!("create_nft_file_event_opt firstly_save_all_create_event: {err}");
            error!("save_all_create_nft_file: {err}");
            return false;
          },
          _ => (),
        }
      }
    }
    _ => (),
  };
  true
}



async fn update_all_nft_owner(owner_info: HashMap<String, String>, txn: &DatabaseTransaction) -> bool {
  if owner_info.is_empty() { return true }
  let owner_info = owner_info.iter()
                                     .map(|(token_id, owner)| format!("('{token_id}','{owner}')"))
                                     .collect::<Vec<String>>().join(",");
  let query = format!(
    r#"update nft_owner
      set owner = nv.owner
      from 
        ( values 
          {owner_info} 
        ) as nv (token_id, owner)
      where nft_owner.token_id = nv.token_id;"#);

  match txn.query_one(Statement::from_string(DatabaseBackend::Postgres, query.to_owned())).await {
    Err(err) => {
      error!("update_all_nft_file_owner err: {err}");
      false
      // panic!("update_all_nft_file_owner: {err}")
    },
    _ => true
  }
}

async fn update_all_balance_info(balance_info: HashMap<String, Balance>, db: &DatabaseTransaction) -> bool {
  if balance_info.is_empty() { return true }
  let balance_info = balance_info.iter()
                                          .map(|(addr, b)| 
                                            format!("('{addr}',{},{})", b.free(), b.locked()))
                                          .collect::<Vec<String>>().join(",");

  // let query = format!(
  // r#"update balance
  //     set 
  //       free = nv.free,
  //       locked = nv.locked
  //     from 
  //       ( values 
  //         {balance_info} 
  //       ) as nv (addr, free, locked)
  //    where balance.address = nv.addr;"#);

  let query = format!(
    r#"INSERT INTO balance (address,free,locked)
      VALUES {balance_info}
      ON CONFLICT (address) 
      DO UPDATE 
      SET 
        free = EXCLUDED.free,
        locked = EXCLUDED.locked;"#
    );

  let record_affected = match db.execute(Statement::from_string(DatabaseBackend::Postgres,query.to_owned())).await {
    Ok(result) => result.rows_affected() as usize,
    Err(err) => {
      error!("update_all_account_balance_info fail {balance_info}:{err}");
      0
    },
  };

  if balance_info.len() != record_affected {
    error!("특정 계정의 잔고 업데이트가 누락되었습니다. {}개의 계정 중 성공 레코드 갯수: {record_affected}", balance_info.len());
    println!("특정 계정의 잔고 업데이트가 누락되었습니다. {}개의 계정 중 성공 레코드 갯수: {record_affected}", balance_info.len());
  }

  true
}

async fn finish_all_block_states(block_hashs: Vec<String>, db: &DatabaseTransaction) -> bool {
  if block_hashs.is_empty() { return true }
  if let Err(err) = block_state::Entity::update_many()
                                        .col_expr(block_state::Column::IsBuild, Expr::value(true))
                                        .filter(block_state::Column::Hash.is_in(block_hashs))
                                        .exec(db).await {
    error!("finish_all_block_states fail : {err}");
    // panic!("finish_all_block_states: {err}");
    return false;
  }
  true
}

fn extract_updated_nft_owners(nft_owner_info: &HashMap<String, String>, transfered_token_id: HashSet<String>) -> HashMap<String, String> {
  nft_owner_info.iter()
    .filter(|(k, _)| transfered_token_id.contains(*k))
    .map(|(k, v)| (k.clone(), v.clone()))
    .collect()
}

fn extract_updated_balance_accounts(account_balance_info: &HashMap<String, Balance>, balanced_updated_accounts: HashSet<String>) -> HashMap<String, Balance> {
  account_balance_info.iter()
    .filter(|(k, _)| balanced_updated_accounts.contains(*k))
    .map(|(addr, balance)| (addr.clone(), balance.clone()))
    .collect()
}

fn extract_token_ids(create_nft_file_opt: Option<&AdditionalEntity>) -> Vec<String> {
  create_nft_file_opt.and_then(|x| match x {
    AdditionalEntity::CreateNftFile(vec) => Some(vec.iter()
                                                                        .map(|file| file.token_id.clone().unwrap())
                                                                        .collect()),
    _ => None
  }).unwrap_or_else(Vec::new)
}

fn extract_addresses(create_account_opt: Option<&AdditionalEntity>) -> Vec<String> {
  create_account_opt.and_then(|x| match x {
    AdditionalEntity::CreateAccount(vec) => Some(vec.iter()
                                                                        .map(|account| account.address.clone().unwrap())
                                                                        .collect()),
    _ => None
  }).unwrap_or_else(Vec::new)
}

async fn summary_loop(db: DatabaseConnection, api_key: String) {
  tokio::spawn(async move {
    loop {
      let last_summary_opt = get_last_saved_summary(&db).await;
      match (
              get_last_built_block(&db).await, 
              get_lm_price(&db, api_key.clone()).await, 
              get_total_accounts(&db).await,
              get_newly_accumumlated_tx_json_size(&db, last_summary_opt).await,
              // TODO: add total balance that is sum of locked and free 
              get_total_balance(&db).await,
            ) 
      {
        (Some(last_built_block), Some(lm_price), Some(total_accounts), Some(total_tx_size), total_balance) => {
          let summary = summary::Model::from(last_built_block.number, lm_price, total_accounts, total_tx_size, total_balance);
          if let Err(err) = summary::Entity::insert(summary).exec(&db).await {
            error!("summary loop failed {}", err);
            // panic!();
          }
        },
        _ => {
          error!("summary loop is skiped.")
        }
      }
      sleep(Duration::from_secs(60 * 10)).await;
      // sleep(Duration::from_secs(10)).await;
    }
  }).await.unwrap()
}

// get total balance that is sum of locked and free 
async fn get_total_balance(db: &DatabaseConnection) -> BigDecimal {
  let balances = balance_entity::Entity::find()
      .all(db)
      .await
      .unwrap(); 

  balances.into_iter().fold(BigDecimal::from(0), |acc, balance| acc + balance.free + balance.locked)
}

async fn save_diff_state_proc(mut curr_block_hash: String, target_hash: String, db: &DatabaseConnection) {
  println!("save_diff_state_proc started");
  let mut is_conitnue = !curr_block_hash.eq(&target_hash);

  let mut block_counter = 0;
  let mut block_states = vec![];
  let mut txs = vec![];
  
  while is_conitnue {
    let block = ApiService::get_block_always(&curr_block_hash.to_owned()).await;
    println!("block number: {}, hash: {}", block.header.number, curr_block_hash);
    
    let block_state = block_state::Model::from(curr_block_hash.as_str(), &block);
    block_states.push(block_state);
    
    if block.header.number != 1468 {
      for tx_hash in &block.transaction_hashes {
        let (tx_result, json) = ApiService::get_tx_with_json_always(tx_hash).await;
        let tx_state = tx_state::Model::from(tx_hash.as_str(), curr_block_hash.as_str(), &tx_result, json);
        txs.push(tx_state);
      }  
    }
    
    block_counter += 1;
    curr_block_hash = block.header.parent_hash.clone();
    is_conitnue = !curr_block_hash.eq(&target_hash);

    if !is_conitnue || block_counter == DOWNLOAD_BATCH_UNIT {
      let txn = db.begin().await.unwrap();
      save_all_block_states(block_states.to_vec(), &txn).await;
      save_all_tx_states(txs.to_vec(), &txn).await;

      block_counter = 0;
      block_states.clear();
      txs.clear();
      txn.commit().await.unwrap();
    }
  }
  println!("save_diff_state_proc ended");
}


async fn build_saved_state_proc 
(
  db: &DatabaseConnection, 
  mut prev_balance_info: HashMap<String, Balance>,
  nft_owner_info: &mut HashMap<String, String>
) 
  -> HashMap<String, Balance> 
{
  println!("build_saved_state_proc started");
  while let Some(block_states) = get_block_states_not_built_order_by_number_asc_limit(db).await  {
    let mut curr_balance_info = prev_balance_info.clone();
    let mut tx_entities = vec![];
    let mut block_entities = vec![];
    let mut additional_entity_store = HashMap::new();
    let mut balance_updated_accounts = HashSet::new();
    let mut transfered_nft_token_ids = HashSet::new();

    let block_hashs = block_states.iter().map(|b|b.hash.clone()).collect::<Vec<String>>();
    let mut txs_in_block: HashMap<String, Vec<(tx_state::Model, TransactionWithResult)>> = 
      get_tx_states_in_block_hashs(block_hashs.clone(), db).await
        .into_iter()
        .map(|(block_hash, tx_states)| 
          (
            block_hash,
            tx_states.into_iter()
                      .map(|state| {
                        let json = state.json.clone();
                        (state, parse_from_json_str::<TransactionWithResult>(&json))
                      })
                      .sorted_by_key(|(_, tx_res)| tx_res.signed_tx.value.created_at())
                      .collect()
          )
        )
        .collect();
    
    let curr_free_tx_signers: HashSet<String> = 
      txs_in_block
        .clone()
        .into_iter()
        .flat_map(|(_, v)| 
            v.into_iter()
             .flat_map(|(_, tx_res)| 
               vec![tx_res.signed_tx.sig.account.clone()]
             )
        )
        .collect();
    
    FreeBalanceStore::temporary_snapshots_of(&curr_free_tx_signers);
    // LockedBalanceStore::snapshots_of(&curr_free_tx_signers);
    
    let block_iter = 
      block_states.into_iter()
                  .map(|state| (parse_from_json_str::<Block>(state.json.as_str()), state));

    for (block, block_state) in block_iter {
      block_entities.push(block_entity::Model::from(&block, block_state.hash.clone()));
      let snapshot_stage = ((block.header.number + 49) / 50) * 50;

      if let Some(tx_states_in_block) = txs_in_block.remove(&block_state.hash) {
        for (tx_state, tx_res) in tx_states_in_block {
          let signer = tx_res.signed_tx.sig.account.clone();
          let spent_txs = FreeBalanceStore::spent_hashs(&signer);

          balance_updated_accounts.extend(tx_res.update_account_balance_info(&mut curr_balance_info, &spent_txs).await);
          transfered_nft_token_ids.extend(tx_res.update_nft_owner_info(nft_owner_info));

          FreeBalanceStore::merge(snapshot_stage, signer, spent_txs, tx_res.input_hashs());
          
          let tx = &tx_res.signed_tx.value;
          let tx_entity = tx.from(tx_state.hash.clone(), tx_res.signed_tx.sig.account.clone(), 
                                              tx_state.block_hash.clone(), block_state.number.clone(), 
                                              tx_state.json.clone(), tx_res.result.clone());
          tx.extract_additional_entity(&tx_entity, &mut additional_entity_store).await;
          tx_entities.push(tx_entity);
        }
      }

      // if snapshot_stage == block.header.number {
      //   FreeBalanceStore::snapshots_of();
      // }

    }

    let updated_nft_owners = extract_updated_nft_owners(&nft_owner_info, transfered_nft_token_ids);
    let updated_balance_accounts = extract_updated_balance_accounts(&curr_balance_info, balance_updated_accounts);

    let save_res = &db.transaction::<_, (), DbErr>(|txn| {
      Box::pin(async move {
        if 
          !save_all_blocks(block_entities, txn).await ||
          !save_all_txs(tx_entities, txn).await ||
          !save_all_nft_txs(additional_entity_store.remove(&AdditionalEntityKey::NftTx), txn).await ||
          !save_all_nft_files(additional_entity_store.remove(&AdditionalEntityKey::CreateNftFile), txn).await ||
          !save_all_accounts(additional_entity_store.remove(&AdditionalEntityKey::CreateAccount), txn).await ||
          !update_all_nft_owner(updated_nft_owners, txn).await ||
          !update_all_balance_info(updated_balance_accounts, txn).await ||
          !finish_all_block_states(block_hashs, txn).await ||
          !FreeBalanceStore::flush() ||
          !LockedBalanceStore::flush()
        {
          return Err(DbErr::Query(RuntimeErr::Internal("Force Rollback!".to_owned())))
        }    
        Ok(())
      })
    })
    .await;

    if let Err(err) = save_res {
      // panic!("save transaction process err: {err}");
      error!("save transaction process err: {err}");
      break;
      // TODO: break 하면 해당 block 다시 처리하는지 확인해야됨!
    } else {
      prev_balance_info = curr_balance_info;
    }
  } 
  prev_balance_info
}


async fn block_check_loop(db: DatabaseConnection) {
  tokio::spawn(async move {
    let mut balance_info = get_balance_infos(&db).await;
    let mut nft_owner_info = get_nft_owner_infos(&db).await;
    balance_info = build_saved_state_proc(&db, balance_info, &mut nft_owner_info).await;
    loop {
      println!("block_check_loop start");
      // let download_start_block = BlockState::find().order_by_asc(block_state::Column::Number).one(&db).await.unwrap().unwrap();
      // let ref node_status = ApiService::get_node_status_always().await;
      // save_diff_state_proc(node_status.best_hash.clone(), download_start_block.hash, &db).await;

      let ref node_status = ApiService::get_node_status_always().await;
      let target_hash = get_last_built_or_genesis_block_hash(node_status, &db).await;
      save_diff_state_proc("91e35be6d4b2a3c7f341566c412e32fe21b3351a55e4b44e294ce151b8eea4d3".to_owned(), target_hash, &db).await;
            
      balance_info = build_saved_state_proc(&db,balance_info, &mut nft_owner_info).await;
      sleep(Duration::from_secs(5)).await;
      println!("block_check_loop end");
      panic!()
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
  Finder::init(db.clone());

  tokio::join!(
    // summary_loop(db.clone(), coin_market_api_key),
    block_check_loop(db),
  );

}
