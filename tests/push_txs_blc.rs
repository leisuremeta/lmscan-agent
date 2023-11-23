use std::{collections::HashMap, thread::sleep, time::Duration};

use dotenvy::var;
use itertools::Itertools;
use lmscan_agent::library::common::db_connn;
use lmscan_agent::{
    block_entity, service::api_service::ApiService, transaction::TransactionWithResult, tx_entity,
};
use sea_orm::*;

#[tokio::test]
async fn push_txs_blc() {
    let database_url = var("DATABASE_URL").expect("DATABASE_URL must be set.");
    let ref db = db_connn(database_url).await;

    let blocks: Vec<(String, i64)> = block_entity::Entity::find()
        .select_only()
        .columns([block_entity::Column::Hash, block_entity::Column::Number])
        .filter(block_entity::Column::Number.lte(1400000))
        .order_by_asc(block_entity::Column::Number)
        .into_tuple()
        .all(db)
        .await
        .unwrap();

    let block_hashs: Vec<String> = blocks
        .into_iter()
        .filter(|(_, number)| number != &1468)
        .map(|(hash, _)| hash)
        .collect();

    let block_txs_map: HashMap<String, Vec<String>> = tx_entity::Entity::find()
        .select_only()
        .columns([tx_entity::Column::BlockHash, tx_entity::Column::Json])
        .order_by_asc(tx_entity::Column::EventTime)
        .into_tuple()
        .all(db)
        .await
        .unwrap()
        .into_iter()
        .into_group_map();

    for (i, block_hash) in block_hashs.into_iter().enumerate() {
        println!("block_hash: {block_hash}");
        if let Some(txs) = block_txs_map.get(&block_hash) {
            let tx_res_vec: Vec<TransactionWithResult> = txs
                .into_iter()
                .map(|tx| TransactionWithResult::from(tx).unwrap())
                .collect();
            let txs: Vec<String> = tx_res_vec
                .iter()
                .map(|tx_res| serde_json::to_string(&tx_res.signed_tx).unwrap())
                .collect();
            let body_str = format!("[{}]", txs.join(","));
            println!("body: {body_str}");
            match ApiService::post_txs(body_str).await {
                Ok(result_hashs) => {
                    if result_hashs.is_empty() {
                        panic!("post 실패");
                    }
                }
                Err(err) => println!("err: {:?}", err),
            }
        }
        if i % 100000 == 0 {
            println!("sleep");
            sleep(Duration::from_secs(10));
        }
    }
}
