use std::collections::HashMap;
use crate::{
    block::Block,
    model::{
        balance_info::BalanceInfo,
        node_status::NodeStatus,
    },
    transaction::TransactionWithResult,
};
use futures_util::TryFutureExt;
use lazy_static::lazy_static;
use std::fmt::Debug;
use reqwest::Url;

lazy_static! {
    static ref CLIENT: reqwest::Client = reqwest::Client::new();
}

// const BASE_URI: &str = "http://lmc.leisuremeta.io";
const BASE_URI: &str = "http://test.chain.leisuremeta.io";

pub struct ApiService;

impl ApiService {
    pub async fn get_request_header_always<
        S: serde::de::DeserializeOwned + Debug,
    >(
        url: Url,
        api_key: &str,
    ) -> Result<S, String> {
        CLIENT.get(url)
            .header("X-CMC_PRO_API_KEY", api_key)
            .send().and_then(|res| res.json()).map_err(|e| e.to_string()).await
    }

    pub async fn get_request_t(
        url: Url,
    ) -> Result<String, String> {
        CLIENT.get(url).send().and_then(|res| res.text()).map_err(|e| e.to_string()).await
    }
    pub async fn get_request<S: serde::de::DeserializeOwned + Debug>(
        url: Url,
    ) -> Result<S, String> {
        CLIENT.get(url).send().and_then(|res| res.json()).map_err(|e| e.to_string()).await
    }

    pub async fn get_request_until<T: reqwest::IntoUrl, S: serde::de::DeserializeOwned + Debug>(
        url: T,
        count: u8,
    ) -> Option<S> {
        for _ in 0..count {
            match CLIENT.get(url.as_str()).send().await {
                Ok(res) => match res.json::<S>().await {
                    Ok(payload) => return Some(payload),
                    Err(err) => {
                        println!("get_request_until parse err '{err}' - {:?}", url.as_str())
                    }
                },
                Err(err) => println!("get_request_until err '{err}' - {:?}", url.as_str()),
            }
        }
        None
    }

    pub async fn get_node_status_always() -> Result<NodeStatus, String> {
        Self::get_request(Url::parse(format!("{}/status", BASE_URI).as_str()).unwrap()).await
    }

    pub async fn get_block_always(hash: &str) -> Result<Block, String> {
        Self::get_request(Url::parse(format!("{}/block/{hash}", BASE_URI).as_str()).unwrap()).await
    }

    pub async fn get_tx_always(hash: &str) -> Result<TransactionWithResult, String> {
        Self::get_request(Url::parse(format!("{}/tx/{hash}", BASE_URI).as_str()).unwrap()).await
    }

    pub async fn get_tx_with_json_always(hash: &str) -> (TransactionWithResult, String) {
        Self::get_request::<TransactionWithResult>(
            Url::parse(format!("{}/tx/{hash}", BASE_URI).as_str()).unwrap())
        .await
        .and_then(|result| 
            serde_json::to_string(&result)
            .map(|txt| (result, txt))
            .map_err(|err| err.to_string())
        )
        .unwrap()
    }

    pub async fn get_free_balance(
        address: &str,
    ) -> Result<Option<HashMap<String, BalanceInfo>>, String> {
        Self::get_balance(address, "free").await
    }

    pub async fn get_locked_balance(
        address: &str,
    ) -> Result<Option<HashMap<String, BalanceInfo>>, String> {
        Self::get_balance(address, "locked").await
    }

    pub async fn get_balance(
        address: &str,
        movable: &str,
    ) -> Result<Option<HashMap<String, BalanceInfo>>, String> {
        Self::get_request_t(Url::parse(format!("{}/balance/{address}?movable={movable}", BASE_URI).as_str()).unwrap())
        .await
        .and_then(|txt| 
            serde_json::from_str::<HashMap<String, BalanceInfo>>(&txt)
            .map_or_else(|err| if txt.contains("not found") {
                Ok(None)
            } else {
                Err(err.to_string())
            }, |v| Ok(Some(v))))
    }
}
