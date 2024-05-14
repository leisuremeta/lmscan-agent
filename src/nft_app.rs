use std::collections::HashMap;
use std::time::Duration;
use chrono::{DateTime, Local};
use itertools::Itertools;

use crate::{nft_file, nft_owner, tx_state};
use crate::transaction::token_transaction::TokenTx;
use crate::{
    service::api_service::ApiService,
    transaction::{
        TransactionWithResult, Transaction,
    },
    library::common::*,
};
use sea_orm::sea_query::{Expr, OnConflict};
use sea_orm::DatabaseConnection;
use sea_orm::*;

use log::{error, info};
use serde::{Deserialize, Serialize};
use tokio::time::sleep;

async fn get_mint_nft_tx(
    latest_time: i64,
    db: &DatabaseConnection,
) -> Vec<tx_state::Model> {
    tx_state::Entity::find()
        .from_raw_sql(Statement::from_sql_and_values(DbBackend::Postgres, 
            r#"select ts.* from tx_state ts join (select * from nft where created_at > $1) nft on ts.hash = nft.tx_hash order by ts.event_time desc"#, 
            [latest_time.into()]))
        .all(db).await.unwrap()
}

pub fn parse_time(str: &String) -> i64 {
    match DateTime::parse_from_rfc3339(str) {
        Ok(x) => x.naive_utc().and_utc().timestamp(),
        Err(_) => Local::now().timestamp(),
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct NftMetaInfo {
    #[serde(rename = "Creator_description")]
    pub creator_description: String,
    #[serde(rename = "Collection_description")]
    pub collection_description: String,
    #[serde(rename = "Rarity")]
    pub rarity: String,
    #[serde(rename = "NFT_checksum")]
    pub nft_checksum: String,
    #[serde(rename = "Collection_name")]
    pub collection_name: String,
    #[serde(rename = "Creator")]
    pub creator: String,
    #[serde(rename = "NFT_name")]
    pub nft_name: String,
    #[serde(rename = "NFT_URI")]
    pub nft_uri: String,
}

async fn update_nft_from_tx(
    latest_time: i64,
    db: &DatabaseConnection,
) -> i64 {
    let model_vec: Vec<tx_state::Model> = get_mint_nft_tx(latest_time, db).await;
    let mut file_map: HashMap<String, nft_file::ActiveModel> = HashMap::new();
    let mut owner_map: HashMap<String, (String, i64)> = HashMap::new();
    for m in model_vec.clone() {
        let tx = match parse_from_json_str::<TransactionWithResult>(&m.json) {
            Ok(r) => r.signed_tx.value,
            Err(e) => {
                error!("{e}");
                continue;
            }
        };
        let token = match tx {
            Transaction::TokenTx(t) => t,
            _ => continue
        };
        if let TokenTx::MintNft(mint) = token.clone() {
            let d = ApiService::get_request_until::<String, NftMetaInfo>(mint.data_url.clone(), 1).await;
            let nft_file = nft_file::Model::from(&mint, d, mint.data_url.clone());
            if !file_map.contains_key(&mint.token_id) {
                file_map.insert(mint.token_id, nft_file);
            }
        }
        let (id, owner, et) = match token {
            TokenTx::MintNft(t) => (t.token_id, t.output, t.created_at),
            TokenTx::TransferNft(t) => (t.token_id, t.output, t.created_at),
            TokenTx::DisposeEntrustedNft(t) => (t.token_id, t.output.unwrap_or(t.input), t.created_at),
            _ => continue
        };
        if !owner_map.contains_key(&id) {
            owner_map.insert(id.clone(),(owner, parse_time(&et)));
        }
    }

    let res = db.transaction::<_, (), DbErr>(|dbtx| { Box::pin(async move {
        if !file_map.is_empty() {
            let values: Vec<Vec<nft_file::ActiveModel>> = file_map.into_values().collect_vec()
                .chunks(500) // 11 fileds * records < 6,000
                .map(|chunk| chunk.to_vec())
                .collect();
            for input in values {
                nft_file::Entity::insert_many(input)
                    .on_conflict(OnConflict::column(nft_file::Column::TokenId).do_nothing().to_owned())
                    .do_nothing()
                    .exec(dbtx).await?;
            }
        }
        if !owner_map.is_empty() {
            for (id, (owner, et)) in owner_map.iter() {
                nft_owner::Entity::insert(
                    nft_owner::ActiveModel {
                        token_id: Set(id.clone()),
                        owner: Set(owner.clone()),
                        event_time: Set(et.clone()),
                        created_at: Set(now()),
                    }
                )
                    .on_conflict(
                        OnConflict::column(nft_owner::Column::TokenId)
                            .target_and_where(Expr::col(nft_owner::Column::EventTime).lt::<i64>(et.to_owned()))
                        .update_columns([nft_owner::Column::Owner, nft_owner::Column::EventTime]).to_owned()
                    )
                    .do_nothing()
                    .exec(dbtx).await?;
            }
        }
        Ok(())

    }) }).await;

    if let Err(err) = res {
        error!("{err}");
    }
    match model_vec.get(0) {
        Some(o) => o.created_at,
        None => latest_time
    }
}

pub async fn nft_loop(db: DatabaseConnection) {
    info!("nft loop start");
    tokio::spawn(async move {
        let mut latest_time = 0;
        loop {
            latest_time = update_nft_from_tx(latest_time, &db).await;
            sleep(Duration::from_secs(30)).await;
        }
    })
    .await
    .map_err(|err| error!("{err}"))
    .unwrap()
}
