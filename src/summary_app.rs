use crate::{entity::*, model::lm_price::LmPrice, service::api_service::ApiService};
use bigdecimal::BigDecimal;
use log::error;
use reqwest::Url;
use rust_decimal::prelude::FromPrimitive;
use rust_decimal::Decimal;
use rust_decimal_macros::dec;
use sea_orm::DatabaseConnection;
use sea_orm::*;
use serde::{Deserialize, Serialize};
use std::str::FromStr;
use std::time::Duration;
use tokio::time::sleep;

extern crate dotenvy;
use dotenvy::var;

pub async fn summary_loop(db: DatabaseConnection, api_key: String) {
    tokio::spawn(async move {
        loop {
            match (
                get_last_built_block(&db).await,
                get_lm_price(&db, api_key.clone()).await,
                get_total_accounts(&db).await,
                get_tx_size(&db).await,
                get_total_balance().await,
            ) {
                (
                    Some(last_built_block),
                    Some(lm_price),
                    Some(total_accounts),
                    Some(total_tx_size),
                    total_balance,
                ) => {
                    let summary = summary::Model::from(
                        last_built_block.number,
                        lm_price,
                        total_accounts as i64,
                        total_tx_size as i64,
                        total_balance.unwrap(),
                    );
                    if let Err(err) = summary::Entity::insert(summary).exec(&db).await {
                        error!("summary loop failed {}", err);
                    }
                }
                _ => {
                    error!("summary loop is skiped.")
                }
            }
            sleep(Duration::from_secs(60 * 10)).await;
        }
    })
    .await
    .unwrap()
}

async fn get_total_accounts(db: &DatabaseConnection) -> Option<u64> {
    account_entity::Entity::find().count(db).await.ok()
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TokenBalance {
    pub status: String,
    pub message: String,
    pub result: String,
}

async fn get_total_balance() -> Option<BigDecimal> {
    let api_key = var("SCAN_API_KEY").expect("SCAN_API_KEY must be set");
    let lm = var("LM_ADDR").expect("LM_ADDR must be set");
    let addrs: Vec<Url> = var("BAL_ADDR").expect("BAL_ADDR must be set")
        .split(",")
        .into_iter()
        .map(|addr| Url::parse(format!("https://api.etherscan.io/api?module=account&action=tokenbalance&contractaddress={lm}&address={addr}&tag=latest&apikey={api_key}"
        ).as_str()).unwrap())
        .collect();
    futures::future::join_all(addrs.into_iter().map(|url| ApiService::get_request(url)))
        .await
        .into_iter()
        .fold(
            BigDecimal::from_i32(0),
            |acc, res: Result<TokenBalance, String>| match (acc, res.ok()) {
                (Some(x), Some(tb)) => Some(x + BigDecimal::from_str(&tb.result).unwrap()),
                _ => BigDecimal::from_i32(0),
            },
        )
}

async fn get_lm_price(db: &DatabaseConnection, api_key: String) -> Option<Decimal> {
    let lm_token_id = 20315;
    match ApiService::get_request_header_always::<LmPrice>(
    Url::parse(format!(
            "https://pro-api.coinmarketcap.com/v2/cryptocurrency/quotes/latest?id={lm_token_id}"
        ).as_str()).unwrap(),
        &api_key,
    ).await
    .map(|lm| 
        lm.data.get(&lm_token_id)
        .and_then(|d| Decimal::from_f32(d.quote.usd.price))
        .or(Some(dec!(0.0))))
    .ok() {
        Some(x) => x,
        _ => get_last_saved_lm_price(db).await
    }
}

async fn get_last_saved_lm_price(db: &DatabaseConnection) -> Option<Decimal> {
    summary::Entity::find()
        .order_by_desc(summary::Column::BlockNumber)
        .one(db)
        .await
        .map(|opt| opt.map(|summary| summary.lm_price))
        .unwrap_or(Some(dec!(0.0)))
}

async fn get_tx_size(db: &DatabaseConnection) -> Option<u64> {
    tx_entity::Entity::find().count(db).await.ok()
}

async fn get_last_built_block(db: &DatabaseConnection) -> Option<block_state::Model> {
    block_state::Entity::find()
        .filter(block_state::Column::IsBuild.eq(true))
        .order_by_desc(block_state::Column::Number)
        .one(db)
        .await
        .unwrap()
}
