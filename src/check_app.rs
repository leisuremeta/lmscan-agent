use std::collections::{HashMap, HashSet};
use std::time::Duration;
use std::vec;

use crate::{
    service::api_service::ApiService,
    store::free_balance::FreeBalanceStore,
    store::locked_balance::LockedBalanceStore,
    transaction::{
        common::Common, AdditionalEntity, ExtractEntity, Job, TransactionWithResult,
    },
    model::balance::Balance,
    block_entity::Model as BlockModel,
    block_state::Entity as BlockState,
    tx_state::Entity as TxState,
    entity::*,
    model::{block::Block, node_status::NodeStatus},
    library::common::*,
};
use sea_orm::sea_query::{Expr, OnConflict};
use sea_orm::DatabaseConnection;
use sea_orm::*;

use itertools::Itertools;

use chrono::{DateTime, Local};
use log::{error, info};
use tokio::time::sleep;

static DOWNLOAD_BATCH_UNIT: usize = 500;
static BUILD_BATCH_UNIT: u64 = 50;

async fn get_last_built_or_genesis_block_hash(
    node_status: &NodeStatus,
    db: &DatabaseConnection,
) -> String {
    BlockState::find()
        .filter(block_state::Column::IsBuild.eq(true))
        .order_by_desc(block_state::Column::Number)
        .one(db)
        .await
        .unwrap()
        .map_or_else(
            || node_status.genesis_hash.to_owned(),
            |opt| opt.hash,
        )
}

async fn get_tx_states_in_block_hashs(
    block_hashs: Vec<String>,
    db: &DatabaseConnection,
) -> HashMap<String, Vec<tx_state::Model>> {
    TxState::find()
        .filter(tx_state::Column::BlockHash.is_in(block_hashs))
        .order_by_asc(tx_state::Column::EventTime)
        .all(db)
        .await
        .unwrap()
        .into_iter()
        .fold(HashMap::new(), |mut acc, tx| {
            acc.entry(tx.block_hash.clone())
                .or_insert_with(Vec::new)
                .push(tx);
            acc
        })
}

async fn get_block_states_not_built_order_by_number_asc_limit(
    db: &DatabaseConnection,
) -> Option<Vec<block_state::Model>> {
    block_state::Entity::find()
        .filter(block_state::Column::IsBuild.eq(false))
        .order_by_asc(block_state::Column::Number)
        .paginate(db, BUILD_BATCH_UNIT)
        .fetch_and_next()
        .await
        .unwrap()
}

async fn get_balance_infos(db: &DatabaseConnection) -> HashMap<String, Balance> {
    let balances = balance_entity::Entity::find().all(db).await.unwrap();
    balances
        .into_iter()
        .map(|b| (b.address.clone(), Balance::new(b.free, b.locked)))
        .collect::<HashMap<String, Balance>>()
}

async fn get_nft_owner_infos(db: &DatabaseConnection) -> HashMap<String, String> {
    let nft_owners = nft_owner::Entity::find().all(db).await.unwrap();
    nft_owners
        .into_iter()
        .map(|nft| (nft.token_id, nft.owner))
        .collect::<HashMap<String, String>>()
}
fn parse_time(str: &String) -> i64 {
    match DateTime::parse_from_rfc3339(str) {
        Ok(x) => x.naive_utc().timestamp(),
        Err(_) => Local::now().timestamp(),
    }
}

fn extract_updated_balance_accounts(
    account_balance_info: &HashMap<String, Balance>,
    balanced_updated_accounts: HashSet<String>,
) -> HashMap<String, Balance> {
    account_balance_info
        .iter()
        .filter(|(k, _)| balanced_updated_accounts.contains(*k))
        .map(|(addr, balance)| (addr.clone(), balance.clone()))
        .collect()
}

async fn save_diff_state_proc(
    mut curr_block_hash: String,
    target_hash: String,
    db: &DatabaseConnection,
) {
    let mut is_conitnue = !curr_block_hash.eq(&target_hash);
    let mut block_states = vec![];
    let mut txs = vec![];

    while is_conitnue {
        let block = ApiService::get_block_always(&curr_block_hash.to_owned())
            .await
            .ok()
            .unwrap();
        if block.header.number % 1000 == 0 {
            log::info!(
                "block number: {}, hash: {}",
                block.header.number,
                curr_block_hash
            );
        }

        let block_state = block_state::Model::from(curr_block_hash.as_str(), &block);
        block_states.push(block_state);

        for tx_hash in &block.transaction_hashes {
            let res = ApiService::get_tx_with_json_always(tx_hash).await;
            match res {
                Ok((tx_result, json)) => {
                    let tx_state = tx_state::Model::from(
                        tx_hash.as_str(),
                        curr_block_hash.as_str(),
                        &tx_result,
                        json,
                    );
                    txs.push(tx_state);
                }
                Err(e) => error!("{e}")
            }
        }

        curr_block_hash = block.header.parent_hash.clone();
        is_conitnue = !curr_block_hash.eq(&target_hash);

        if !is_conitnue || (block_states.len() + txs.len()) >= DOWNLOAD_BATCH_UNIT {
            let txn = db.begin().await.unwrap();
            let r1 = block_state::Entity::insert_many(block_states.clone())
                .on_conflict(
                    OnConflict::column(block_state::Column::Hash)
                        .do_nothing()
                        .to_owned(),
                )
                .exec(&txn)
                .await;
            let r2 = tx_state::Entity::insert_many(txs.clone())
                .on_conflict(
                    OnConflict::column(tx_state::Column::Hash)
                        .do_nothing()
                        .to_owned(),
                )
                .exec(&txn)
                .await;

            if r1.and(r2).is_ok() {
                block_states.clear();
                txs.clear();
                txn.commit().await.unwrap();
            } else {
                // !TODO fail scenario
            }
        }
    }
}

async fn build_saved_state_proc(
    db: &DatabaseConnection,
    mut prev_balance_info: HashMap<String, Balance>,
    _: &mut HashMap<String, String>,
) -> HashMap<String, Balance> {
    while let Some(block_states) = get_block_states_not_built_order_by_number_asc_limit(db).await {
        let mut curr_balance_info = prev_balance_info.clone();
        let mut tx_entities = vec![];
        let mut additional_entity_store = HashMap::new();
        let mut balance_updated_accounts = HashSet::new();
        let mut nft_owner_vec: Vec<(String, String, String)> = vec![];

        let block_hashs = block_states
            .iter()
            .map(|b| b.hash.clone())
            .collect::<Vec<String>>();
        let mut txs_in_block: HashMap<String, Vec<(tx_state::Model, TransactionWithResult)>> = HashMap::new();
        for (block_hash, tx_states) in get_tx_states_in_block_hashs(block_hashs.clone(), db).await {
            let mut vec: Vec<(tx_state::Model, TransactionWithResult)> = vec![]; 
            for state in tx_states {
                match parse_from_json_str::<TransactionWithResult>(&state.json) {
                    Ok(txr) => vec.push((state, txr)),
                    Err(e) => error!("{}: {e}", state.json)
                }
            }
            vec.sort_by_key(|(_, res)| res.signed_tx.value.created_at());
            txs_in_block.insert(block_hash, vec);
        }

        let curr_free_tx_signers: HashSet<String> = txs_in_block
            .clone()
            .into_iter()
            .flat_map(|(_, v)| {
                v.into_iter()
                    .flat_map(|(_, tx_res)| vec![tx_res.signed_tx.sig.account.clone()])
            })
            .collect();

        let snapshot_stage = block_states.first().unwrap().number as u64;
        FreeBalanceStore::temporary_snapshot_of(&curr_free_tx_signers);
        LockedBalanceStore::temporary_snapshot_of();

        let mut block_entities: Vec<block_entity::ActiveModel> = vec![];
        for state in block_states {
            match parse_from_json_str::<Block>(state.json.as_str()) {
                Ok(block) => block_entities.push(BlockModel::from(&block, state.hash.clone())),
                Err(e) => error!("{}: {e}", state.json)
            }
        }

        let mut free_state = FreeBalanceStore::log_of_snapshot_stage(snapshot_stage);
        let mut locked_state = LockedBalanceStore::log_of_snapshot_stage(snapshot_stage);

        for (number, txs) in block_entities
            .iter()
            .map(|b| (b.hash.clone().unwrap(), b.number.clone().unwrap()))
            .filter_map(|(hash, number)| txs_in_block.remove(&hash).map(|txs| (number, txs)))
        {
            // Scan tx entity process
            for (tx_state, tx_res) in txs.iter() {
                let tx = &tx_res.signed_tx.value;
                let tx_entity = tx.from(
                    tx_state.hash.clone(),
                    tx_res.signed_tx.sig.account.clone(),
                    tx_state.block_hash.clone(),
                    number,
                    tx_state.json.clone(),
                    tx_res.clone(),
                );
                tx.extract_additional_entity(&tx_entity, &mut additional_entity_store)
                    .await;
                tx_entities.push(tx_entity);
            }

            let tx_res_vec: Vec<TransactionWithResult> =
                txs.into_iter().map(|(_, tx_res)| tx_res).collect();
            // Free balance fungible txs
            for free_tx in tx_res_vec.iter().filter(|tx_res| tx_res.is_free_fungible()) {
                balance_updated_accounts.extend(
                    free_tx
                        .update_free_balance(&mut curr_balance_info, &mut free_state)
                        .await,
                );
            }

            // Locked balance fungible txs
            for locked_tx in tx_res_vec
                .iter()
                .filter(|tx_res| tx_res.is_locked_fungible())
            {
                balance_updated_accounts.extend(
                    locked_tx
                        .update_locked_balance(&mut curr_balance_info, &mut locked_state)
                        .await,
                );
            }

            // Nft owner transfer txs
            nft_owner_vec = tx_res_vec
                .iter()
                .filter_map(|tx| tx.update_nft_owner_info())
                .collect_vec();
        }

        let updated_balance_accounts =
            extract_updated_balance_accounts(&curr_balance_info, balance_updated_accounts);

        let save_res = &db
            .transaction::<_, (), DbErr>(|txn| {
                Box::pin(async move {
                    if !block_entities.is_empty() {
                        Insert::many(block_entities)
                            .on_conflict(
                                OnConflict::column(block_entity::Column::Hash)
                                    .do_nothing()
                                    .to_owned(),
                            )
                            .do_nothing()
                            .exec(txn)
                            .await?;
                    }
                    if !tx_entities.is_empty() {
                        Insert::many(tx_entities)
                            .on_conflict(
                                OnConflict::column(tx_entity::Column::Hash)
                                    .do_nothing()
                                    .to_owned(),
                            )
                            .do_nothing()
                            .exec(txn)
                            .await?;
                    }
                    for (_, entities) in additional_entity_store {
                        match entities {
                            AdditionalEntity::NftTx(v) => {
                                Insert::many(v)
                                    .on_conflict(
                                        OnConflict::column(nft_tx::Column::TxHash)
                                            .do_nothing()
                                            .to_owned(),
                                    )
                                    .do_nothing()
                                    .exec(txn)
                                    .await?;
                            }
                            AdditionalEntity::CreateAccount(v) => {
                                Insert::many(v)
                                    .on_conflict(
                                        OnConflict::column(account_entity::Column::Address)
                                            .do_nothing()
                                            .to_owned(),
                                    )
                                    .do_nothing()
                                    .exec_without_returning(txn)
                                    .await?;
                            }
                            AdditionalEntity::CreateNftFile(v) => {
                                // why chunks size 10?
                                Insert::many(v)
                                    .on_conflict(
                                        OnConflict::column(nft_file::Column::TokenId)
                                            .do_nothing()
                                            .to_owned(),
                                    )
                                    .do_nothing()
                                    .exec(txn)
                                    .await?;
                            }
                        }
                    }
                    if !nft_owner_vec.is_empty() {
                        let mut m: HashMap<String, &(String, String, String)> = HashMap::new();
                        for tuple in nft_owner_vec.iter() {
                            match m.get(&tuple.0) {
                                Some(other) => {
                                    if tuple.2.cmp(&other.2).is_le() {
                                        continue;
                                    } else {
                                        m.insert(tuple.0.clone(), tuple);
                                    }
                                }
                                None => { m.insert(tuple.0.clone(), tuple); }
                            }
                        }
                        let owners = m.into_values().map(
                            |(ti, ow, et)| nft_owner::ActiveModel {
                                token_id: Set(ti.to_owned()),
                                owner: Set(ow.to_owned()),
                                event_time: Set(parse_time(et)),
                                created_at: NotSet,
                            },
                        );
                        nft_owner::Entity::insert_many(owners)
                            .on_conflict(
                                OnConflict::column(nft_owner::Column::TokenId)
                                    .update_columns([nft_owner::Column::Owner, nft_owner::Column::EventTime])
                                    .to_owned(),
                            )
                            .do_nothing()
                            .exec(txn)
                            .await?;
                    }
                    if !updated_balance_accounts.is_empty() {
                        let bals = updated_balance_accounts.iter().map(|(addr, bal)| {
                            balance_entity::ActiveModel {
                                address: Unchanged(addr.to_owned()),
                                free: Set(bal.free.to_owned()),
                                locked: Set(bal.locked.to_owned()),
                                created_at: NotSet,
                                updated_at: Set(now()),
                            }
                        });
                        balance_entity::Entity::insert_many(bals)
                            .on_conflict(
                                OnConflict::column(balance_entity::Column::Address)
                                    .update_columns([
                                        balance_entity::Column::Free,
                                        balance_entity::Column::Locked,
                                        balance_entity::Column::UpdatedAt,
                                    ])
                                    .to_owned(),
                            )
                            .do_nothing()
                            .exec(txn)
                            .await?;
                    }
                    if !block_hashs.is_empty() {
                        block_state::Entity::update_many()
                            .col_expr(block_state::Column::IsBuild, Expr::value(true))
                            .filter(block_state::Column::Hash.is_in(block_hashs))
                            .exec(txn)
                            .await?;
                    }

                    if !FreeBalanceStore::flush(snapshot_stage, free_state)
                        || !LockedBalanceStore::flush(snapshot_stage, locked_state)
                    {
                        return Err(DbErr::Query(RuntimeErr::Internal(
                            "Force Rollback!".to_owned(),
                        )));
                    }
                    Ok(())
                })
            })
            .await;

        if let Err(err) = save_res {
            // TODO: break 하면 해당 block 다시 처리하는지 확인해야됨!
            FreeBalanceStore::rollback(snapshot_stage);
            LockedBalanceStore::rollback();
            error!("save transaction process err: {err}");
            panic!("save transaction process err: {err}");
        } else {
            prev_balance_info = curr_balance_info;
        }
    }
    prev_balance_info
}

pub async fn check_loop(db: DatabaseConnection) {
    info!("check loop start");
    tokio::spawn(async move {
        let mut balance_info = get_balance_infos(&db).await;
        let mut nft_owner_info = get_nft_owner_infos(&db).await;
        balance_info = build_saved_state_proc(&db, balance_info, &mut nft_owner_info).await;
        loop {
            match ApiService::get_node_status_always().await.ok() {
               Some(node_status) => {
                    let target_hash = get_last_built_or_genesis_block_hash(&node_status, &db).await;
                    save_diff_state_proc(node_status.best_hash.clone(), target_hash, &db).await;

                    balance_info = build_saved_state_proc(&db, balance_info, &mut nft_owner_info).await;
               }
               _ =>  error!("can't load status")
            }
            sleep(Duration::from_secs(3)).await;
        }
    })
    .await
    .map_err(|err| error!("{err}"))
    .unwrap()
}
