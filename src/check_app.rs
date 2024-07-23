use std::time::Duration;
use std::vec;

use crate::{
    balance_app::BAL_VEC, block_entity::Model as BlockModel, block_state::Entity as BlockState, entity::*, library::common::*, model::{block::Block, node_status::NodeStatus}, service::api_service::ApiService, transaction::{
        common::Common, Job, TransactionWithResult
    }
};
use sea_orm::sea_query::{Expr, OnConflict};
use sea_orm::DatabaseConnection;
use sea_orm::*;

use log::{error, info};
use tokio::time::sleep;

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

async fn save_diff_state_proc(
    mut curr_block_hash: String,
    target_hash: String,
    db: &DatabaseConnection,
) {
    let mut is_conitnue = !curr_block_hash.eq(&target_hash);

    while is_conitnue {
        let mut block_states = vec![];
        let mut txs = vec![];
        let mut txss = vec![];
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
                Ok(json) => {
                    match parse_from_json_str::<TransactionWithResult>(&json) {
                        Ok(tx) => {
                            let tx_state = tx_state::Model::from(
                                tx_hash.as_str(),
                                curr_block_hash.as_str(),
                                &tx,
                                json,
                            );
                            txs.push(tx_state);
                            txss.push((tx, tx_hash.clone()));
                        }
                        Err(e) => error!("{e}")
                    }
                }
                Err(e) => error!("{e}")
            }
        }
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

        if r1.is_err() {
            panic!("block state store is fail {:?}", &curr_block_hash);
        } else if r2.is_err() {
            error!("txs state store is fail {:?}", &block.transaction_hashes);
        } else {
            txn.commit().await.unwrap();
        }

        tokio::spawn(parse_tx_and_update(db.clone(), block.clone(), txss.clone(), curr_block_hash.clone()));

        curr_block_hash = block.header.parent_hash.clone();
        is_conitnue = !curr_block_hash.eq(&target_hash);

    }
}

async fn parse_tx_and_update(
    db: DatabaseConnection,
    blc: Block,
    txs: Vec<(TransactionWithResult, String)>,
    blc_hash: String,
) {
    let mut tx_entities = vec![];
    let mut nft_tx_vec: Vec<nft_tx::ActiveModel> = vec![];
    let mut new_acc_vec: Vec<account_entity::ActiveModel> = vec![];
    let mut acc_map_vec: Vec<account_mapper::Model> = vec![];

    for (tx_res, tx_hash) in txs {

        let tx = &tx_res.signed_tx.value;
        let tx_entity = tx.from(
            tx_hash.clone(),
            blc_hash.clone(),
            blc.header.number.clone(),
            tx_res.clone(),
        );
        if let Some(nft) = tx.get_nft_active_model(&tx_entity, tx_res.signed_tx.sig.account.clone()) {
            nft_tx_vec.push(nft);
        }
        if let Some(acc) = tx.get_acc_active_model() {
            new_acc_vec.push(acc);
        }
        acc_map_vec.append(&mut tx.get_account_mapper(tx_res.signed_tx.sig.account.clone(), tx_hash.clone(), tx.created_at()));
        tx_entities.push(tx_entity);
        if tx_res.is_free_fungible() {
            unsafe {
                let mut v = BAL_VEC.lock().unwrap();
                v.push((tx_res, tx_hash));
            }
        }
    }
    let block_entity = BlockModel::from(&blc, blc_hash.clone());

    let save_res = &db
        .transaction::<_, (), DbErr>(|txn| {
            Box::pin(async move {
                Insert::one(block_entity)
                    .on_conflict(
                        OnConflict::column(block_entity::Column::Hash)
                            .do_nothing()
                            .to_owned(),
                    )
                    .do_nothing()
                    .exec(txn)
                    .await?;
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
                if !nft_tx_vec.is_empty() {
                    Insert::many(nft_tx_vec)
                        .on_conflict(
                            OnConflict::column(nft_tx::Column::TxHash)
                                .do_nothing()
                                .to_owned(),
                        )
                        .do_nothing()
                        .exec(txn)
                        .await?;
                }
                if !new_acc_vec.is_empty() {
                    Insert::many(new_acc_vec)
                        .on_conflict(
                            OnConflict::column(account_entity::Column::Address)
                                .do_nothing()
                                .to_owned(),
                        )
                        .do_nothing()
                        .exec_without_returning(txn)
                        .await?;
                }
                block_state::Entity::update_many()
                    .col_expr(block_state::Column::IsBuild, Expr::value(true))
                    .filter(block_state::Column::Hash.eq(blc_hash))
                    .exec(txn)
                    .await?;
                if !acc_map_vec.is_empty() {
                    let v = acc_map_vec.into_iter().map(|m| m.into_active_model()).collect::<Vec<account_mapper::ActiveModel>>();
                    account_mapper::Entity::insert_many(v)
                    .exec(txn)
                    .await?;
                }

                Ok(())
            })
        })
        .await;

    if let Err(err) = save_res {
        panic!("save transaction process err: {err}");
    }
}

pub async fn check_loop(db: DatabaseConnection) {
    info!("check loop start");
    tokio::spawn(async move {
        loop {
            match ApiService::get_node_status_always().await.ok() {
               Some(node_status) => {
                    let target_hash = get_last_built_or_genesis_block_hash(&node_status, &db).await;
                    save_diff_state_proc(node_status.best_hash.clone(), target_hash, &db).await;
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
