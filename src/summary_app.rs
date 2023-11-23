use crate::{entity::*, model::lm_price::LmPrice, service::api_service::ApiService};
use bigdecimal::BigDecimal;
use log::error;
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
    let addrs: Vec<String> = var("BAL_ADDR").expect("BAL_ADDR must be set")
        .split(",")
        .into_iter()
        .map(|addr| format!("https://api.etherscan.io/api?module=account&action=tokenbalance&contractaddress={lm}&address={addr}&tag=latest&apikey={api_key}"
        ))
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
    let coin_market: LmPrice = ApiService::get_request_header_always(
        format!(
            "https://pro-api.coinmarketcap.com/v2/cryptocurrency/quotes/latest?id={lm_token_id}"
        ),
        &api_key,
    )
    .await;
    if coin_market.status.error_code == 0 {
        match coin_market.data.get(&lm_token_id) {
            Some(data) => return Some(Decimal::from_f32(data.quote.usd.price).unwrap_or_default()),
            None => {
                error!(
                    "coin market api returned response error (code: {}, message: {})",
                    coin_market.status.error_code,
                    coin_market.status.error_message.unwrap_or_default()
                );
            }
        }
    }
    match get_last_saved_lm_price(db).await {
        Some(latest_price) => Some(latest_price.lm_price),
        None => Some(dec!(0.0)),
    }
}

async fn get_last_saved_lm_price(db: &DatabaseConnection) -> Option<summary::Model> {
    summary::Entity::find()
        .order_by_desc(summary::Column::BlockNumber)
        .one(db)
        .await
        .unwrap()
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
