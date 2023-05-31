use std::{fs::File, path::Path, io::Write, collections::{HashMap, HashSet}, sync::Arc};

use bigdecimal::BigDecimal;
use itertools::Itertools;
use lmscan_agent::{entity::account_entity, service::api_service::ApiService, block_state, library::common::parse_from_json_str, transaction::{TransactionWithResult, Job, Common}, block_entity, tx_state, block::Block};
use dotenvy::{dotenv, var};
use lmscan_agent::library::common::db_connn;
use sea_orm::{*, sea_query::Expr};
use lmscan_agent::tx_state::{Entity as TxState};
use sled::Db;



#[tokio::test]
async fn balance_local_build() {
  dotenv().expect("Unable to load environment variables from .env file");
  let database_url = var("DATABASE_URL").expect("DATABASE_URL must be set.");
  let ref db = db_connn(database_url).await;
  let sled = 
    Arc::new(
      sled::Config::default()
        .path("/Users/jichangho/playnomm/lmscan-agent/sled/test/balance_local_build/input_tx".to_owned())
        .use_compression(true)
        .compression_factor(6)
        .flush_every_ms(None)
        .open()
        .unwrap()
    );

  let mut output_file = File::create(Path::new("prod_balance_local_build.txt"))
                                    .expect("cannot open output file");
  output_file.write(format!("address, blc_balance, scan_balance, equal, diff\n").as_bytes()).expect("write failed");
  let build_result = build_saved_state_proc(db, sled, HashMap::new(), &mut HashMap::new()).await;
  output_file.write(format!("builded account total count: {}\n", build_result.keys().count()).as_bytes()).expect("write failed");

  let scan_accounts = account_entity::Entity::find().all(db).await.unwrap();
  output_file.write(format!("scan account total count: {}\n", scan_accounts.len()).as_bytes()).expect("write failed");

  //for scan_account in scan_accounts.into_iter() {
  for (scan_addr, scan_balance) in build_result.into_iter() {
    // let scan_addr = &scan_account.address;
    // if let Some(scan_balance) = build_result.get(scan_addr) {
      // println!("{count} - scan {key} - {scan_balance}");
      // 8081
      match ApiService::get_account_balance(&scan_addr).await {
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
  // }
}

async fn build_saved_state_proc 
(
  db: &DatabaseConnection, 
  sled: Arc<Db>,
  mut account_balance_info: HashMap<String, BigDecimal>,
  nft_owner_info: &mut HashMap<String, String>
) 
  -> HashMap<String, BigDecimal> 
{
  println!("build_saved_state_proc started");
  let mut block_num = 0;
  while let Some(block_states) = get_block_states_after_order_by_asc_limit(db, block_num).await  {
    let mut cloned_account_balance_info = account_balance_info.clone();
    let mut tx_entities = vec![];
    let mut block_entities = vec![];
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
    
    let curr_tx_signers: HashSet<String> = 
      txs_in_block
        .clone()
        .into_iter()
        .flat_map(|(_, v)| 
            v.into_iter()
             .flat_map(|(_, tx_res)| {
               vec![tx_res.signed_tx.sig.account.clone()]
             })
        )
        .collect();

    let mut account_input_txs: HashMap<String, HashSet<String>> = 
      (&curr_tx_signers).into_iter().map(|account| {
        let value = sled.get(account).unwrap_or_default().unwrap_or_default();
        (
          account.clone(),
          serde_json::from_slice::<HashSet<String>>(&value).unwrap_or_else(|_| HashSet::new())
        )
      })
      .collect();
    
    // if curr_tx_signers.len() > 1 {
    //   println!("curr_tx_signers: {:?}", curr_tx_signers);
    //   println!("account_input_txs: {:?}", account_input_txs);
    // }
    
    println!("sled len: {:?}", sled.len());

    let block_iter = 
      block_states.into_iter()
                  .map(|state| (parse_from_json_str::<Block>(state.json.as_str()), state));

    for (block, block_state) in block_iter {
      block_num = block.header.number;
      block_entities.push(block_entity::Model::from(&block, block_state.hash.clone()));
      
      if let Some(tx_states_in_block) = txs_in_block.remove(&block_state.hash) {
        for (tx_state, tx_res) in tx_states_in_block {
          let signer = tx_res.signed_tx.sig.account.clone();
          let account_input_txs = account_input_txs.entry(signer.clone()).or_insert_with(HashSet::new);
          // if input_txs.contains(&tx_state.hash) { 
          //   println!("--------------");
          //   println!("{signer}\n{:?}\n", input_txs);
          //   panic!();
          //   continue; 
          // }
          balance_updated_accounts.extend(tx_res.update_account_balance_info(&mut cloned_account_balance_info, account_input_txs).await);
          transfered_nft_token_ids.extend(tx_res.update_nft_owner_info(nft_owner_info));

          account_input_txs.extend(tx_res.input_hashs());
          
          let tx = &tx_res.signed_tx.value;
          let tx_entity = tx.from(tx_state.hash.clone(), tx_res.signed_tx.sig.account, 
                                              tx_state.block_hash.clone(), block_state.number, 
                                              tx_state.json.clone(), tx_res.result);
          // tx.extract_additional_entity(&tx_entity, &mut additional_entity_store).await;
          tx_entities.push(tx_entity);
        }
      }
    }

    // let this_time_updated_nft_owners = extract_updated_nft_owners(&nft_owner_info, transfered_nft_token_ids);
    let this_time_updated_balance_accounts = extract_updated_balance_accounts(&cloned_account_balance_info, balance_updated_accounts);
    // let addresses = extract_addresses(additional_entity_store.get(&AdditionalEntityKey::CreateAccount));
    // let token_ids = extract_token_ids(additional_entity_store.get(&AdditionalEntityKey::CreateNftFile));
    
    // firstly_save_all_create_event(additional_entity_store.remove(&AdditionalEntityKey::CreateAccount),
    //                              additional_entity_store.remove(&AdditionalEntityKey::CreateNftFile),
    //                                                         &db).await;
    
    for (k, v) in account_input_txs.into_iter() {
      sled.insert(k.as_bytes(), serde_json::to_vec(&v).unwrap()).unwrap();
    }
    update_all_account_balance_info(this_time_updated_balance_accounts);

    // let save_res = &db.transaction::<_, (), DbErr>(|txn| {
    //   Box::pin(async move {
    //     if !save_all_blocks(block_entities, txn).await ||
    //        !save_all_txs(tx_entities.clone(), txn).await ||
    //        !save_all_nft_txs(additional_entity_store.remove(&AdditionalEntityKey::NftTx), txn).await ||
    //        !update_all_nft_file_owner(this_time_updated_nft_owners, txn).await ||
    //        !finish_all_block_states(block_hashs, txn).await
    //     {
    //       // let account_input_txs = 
    //       //   tx_entities.iter().map(|tx| 
    //       //     (tx.from_addr.clone().unwrap(), tx.hash.clone().unwrap())
    //       //   ).into_group_map();

    //       // account_input_txs
    //       //   .into_iter()
    //       //   .for_each(|(key, curr_input_txs)| {
    //       //     let value = sled.get(&key).unwrap_or_default().unwrap_or_default();
    //       //     let mut data = serde_json::from_slice::<HashSet<String>>(&value).unwrap_or_else(|_| HashSet::new());
    //       //     data.extend(curr_input_txs);
    //       //     let new_value = serde_json::to_vec(&data).unwrap();
    //       //     sled.insert(key, new_value).unwrap();
    //       //   });

    //       return Err(DbErr::Query(RuntimeErr::Internal("Force Rollback!".to_owned())))
    //     }    
    //     Ok(())
    //   })
    // })
    // .await;
    match cloned_account_balance_info.get("99492bc6664940e36cf21c7a33868a7bfded29e8") {
      Some(value) => {
        println!("2 - 99492bc6664940e36cf21c7a33868a7bfded29e8 - {value}");
      },
      None => (),
    };
    match cloned_account_balance_info.get("cccf6911e96ce1fa87e0757afb464a6929c8e8eb") {
      Some(value) => {
        println!("2 - cccf6911e96ce1fa87e0757afb464a6929c8e8eb - {value}");
      },
      None => (),
    };

    account_balance_info = cloned_account_balance_info;
    sled.flush_async().await.unwrap();

  } 
  account_balance_info
}


fn update_all_account_balance_info(account_balance_info: HashMap<String, BigDecimal>)  {
  if account_balance_info.is_empty() { return; }
  let balance_info = account_balance_info.iter()
                                          .map(|(address, balance)| format!("('{address}',{balance})"))
                                          .collect::<Vec<String>>().join(",");

  match account_balance_info.get("99492bc6664940e36cf21c7a33868a7bfded29e8") {
    Some(value) => {
      println!("1 - 99492bc6664940e36cf21c7a33868a7bfded29e8 - {value}");
      println!("balance_info - {balance_info}");
    },
    None => (),
  };
  match account_balance_info.get("cccf6911e96ce1fa87e0757afb464a6929c8e8eb") {
    Some(value) => {
      println!("1 - cccf6911e96ce1fa87e0757afb464a6929c8e8eb - {value}");
      println!("balance_info - {balance_info}");
    },
    None => (),
  };
}
// async fn build_saved_state_proc0 
// (
//   db: &DatabaseConnection, 
//   sled: Arc<Db>,
//   mut account_balance_info: HashMap<String, BigDecimal>,
//   nft_owner_info: &mut HashMap<String, String>
// ) 
// -> HashMap<String, BigDecimal> 
// {
//   let mut block_num = 0;
//   while let Some(block_states) = get_block_states_after_order_by_asc_limit(db, block_num).await  {
//     let mut cloned_account_balance_info = account_balance_info.clone();
//     let mut block_entities = vec![];
//     let mut balance_updated_accounts = HashSet::new();

//     let block_hashs = block_states.iter().map(|b|b.hash.clone()).collect::<Vec<String>>();
//     let mut txs_in_block = get_tx_states_in_block_hashs(block_hashs, db).await;

//     for block_state in block_states.iter() {
//       let block: Block = parse_from_json_str(block_state.json.as_str());
//       block_num = block.header.number;
//       block_entities.push(block_entity::Model::from(&block, block_state.hash.clone()));
//       println!("{}", block.header.number);
//       if let Some(tx_states_in_block) = txs_in_block.remove(&block_state.hash) {
//         let iter = tx_states_in_block.iter().map(|state| (state, parse_from_json_str::<TransactionWithResult>(state.json.as_str())))
//                                       .sorted_by_key(|(_, tx_res)| tx_res.signed_tx.value.created_at());

//         for (tx_state, tx_res) in iter {
//           balance_updated_accounts.extend(tx_res.update_account_balance_info(&mut cloned_account_balance_info).await);
//           // transfered_nft_token_ids.extend(tx_res.update_nft_owner_info(nft_owner_info));
          
//         }
//       }
//     }

//     // let this_time_updated_nft_owners = extract_updated_nft_owners(&nft_owner_info, transfered_nft_token_ids);
//     // let this_time_updated_balance_accounts = extract_updated_balance_accounts(&cloned_account_balance_info, balance_updated_accounts);
//     // let addresses = extract_addresses(additional_entity_store.get(&AdditionalEntityKey::CreateAccount));
//     // let token_ids = extract_token_ids(additional_entity_store.get(&AdditionalEntityKey::CreateNftFile));
//     account_balance_info = cloned_account_balance_info;
//     // match finish_all_block_states(block_states, db).await {
//     //   true => account_balance_info = cloned_account_balance_info,
//     //   false => panic!("finish_all_block_states error"),
//     // }
//   } 
//   account_balance_info
// }

fn extract_updated_balance_accounts(account_balance_info: &HashMap<String, BigDecimal>, balanced_updated_accounts: HashSet<String>) -> HashMap<String, BigDecimal> {
  account_balance_info.iter()
    .filter(|(k, _)| balanced_updated_accounts.contains(*k))
    .map(|(k, v)| (k.clone(), v.clone()))
    .collect()
}


async fn get_block_states_after_order_by_asc_limit(db: &DatabaseConnection, num: i64) -> Option<Vec<block_state::Model>> {
  block_state::Entity::find()
                      .filter(block_state::Column::Number.gt(num))
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
    println!("finish_all_block_states fail : {err}");
    return false;
  }
  true
}
