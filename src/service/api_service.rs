use std::{collections::HashMap, time::Duration};

use crate::{
    block::Block,
    library::common::is_not_found_err,
    model::{
        account_info::AccountInfo, balance_info::BalanceInfo, nft_balance_info::NftBalanceInfo,
        nft_state::NftState, node_status::NodeStatus,
    },
    transaction::TransactionWithResult,
};
use lazy_static::lazy_static;
use log::info;
use std::fmt::Debug;
use tokio::time::sleep;

lazy_static! {
    static ref CLIENT: reqwest::Client = reqwest::Client::new();
}

// const BASE_URI: &str = "http://lmc.leisuremeta.io";
const BASE_URI: &str = "http://test.chain.leisuremeta.io";

pub struct ApiService;

impl ApiService {
    pub async fn get_request_header_always<
        T: reqwest::IntoUrl,
        S: serde::de::DeserializeOwned + Debug,
    >(
        url: T,
        api_key: &str,
    ) -> S {
        loop {
            match CLIENT
                .get(url.as_str())
                .header("X-CMC_PRO_API_KEY", api_key)
                .send()
                .await
            {
                Ok(res) => match res.json::<S>().await {
                    Ok(payload) => return payload,
                    Err(err) => {
                        println!("get_request_always parse err '{err}' - {:?}", url.as_str())
                    }
                },
                Err(err) => println!("get_request_always err '{err}' - {:?}", url.as_str()),
            }
            sleep(Duration::from_millis(500)).await;
        }
    }

    async fn get_request_always<T: reqwest::IntoUrl, S: serde::de::DeserializeOwned + Debug>(
        url: T,
    ) -> S {
        info!("get_request_always : {:?}", url.as_str());
        loop {
            match CLIENT.get(url.as_str()).send().await {
                Ok(res) => match res.json::<S>().await {
                    Ok(payload) => return payload,
                    Err(err) => {
                        println!("get_request_always parse err '{err}' - {:?}", url.as_str());
                        println!(
                            "{:?}",
                            CLIENT
                                .get(url.as_str())
                                .send()
                                .await
                                .ok()
                                .unwrap()
                                .text()
                                .await
                        );
                    }
                },
                Err(err) => println!("get_request_always err '{err}' - {:?}", url.as_str()),
            }
            sleep(Duration::from_millis(500)).await;
        }
    }

    async fn get_request_with_json_always<
        T: reqwest::IntoUrl,
        S: serde::ser::Serialize + serde::de::DeserializeOwned + Debug,
    >(
        url: T,
    ) -> (S, String) {
        loop {
            match CLIENT.get(url.as_str()).send().await {
                Ok(res) => match res.text().await {
                    Ok(payload) => match serde_json::from_str(&payload) {
                        Ok(res) => return (res, payload),
                        Err(err) => {
                            info!("get_request_with_json_always parse error: {err}");
                            continue;
                        }
                    },
                    Err(err) => {
                        println!(
                            "--- {}",
                            &CLIENT
                                .get(url.as_str())
                                .send()
                                .await
                                .unwrap()
                                .text()
                                .await
                                .unwrap()
                        );
                        panic!("{}", err.to_string());
                    }
                },
                Err(err) => println!(
                    "get_request_with_json_always err '{err}' - {:?}",
                    url.as_str()
                ),
            }
            sleep(Duration::from_millis(500)).await;
        }
    }

    pub async fn get_request<T: reqwest::IntoUrl, S: serde::de::DeserializeOwned + Debug>(
        url: T,
    ) -> Result<S, String> {
        match CLIENT.get(url.as_str()).send().await {
            Ok(res) => match res.json::<S>().await {
                Ok(payload) => Ok(payload),
                Err(err) => Err(err.to_string()),
            },
            Err(err) => {
                println!(
                    "get_request '{:?}' http communication err occured: '{err}'",
                    url.as_str()
                );
                Err(format!("3: {}", err.to_string()))
            }
        }
    }

    pub async fn get_request_until<T: reqwest::IntoUrl, S: serde::de::DeserializeOwned + Debug>(
        url: T,
        count: u8,
    ) -> Option<S> {
        info!("get_request_until {count} : {:?}", url.as_str());
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

    pub async fn get_node_status_always() -> NodeStatus {
        Self::get_request_always(format!("{}/status", BASE_URI)).await
    }

    pub async fn get_block_always(hash: &str) -> Block {
        Self::get_request_always(format!("{}/block/{hash}", BASE_URI)).await
    }

    pub async fn get_tx_always(hash: &str) -> TransactionWithResult {
        Self::get_request_always(format!("{}/tx/{hash}", BASE_URI)).await
    }

    pub async fn get_tx_with_json_always(hash: &str) -> (TransactionWithResult, String) {
        Self::get_request_with_json_always(format!("{}/tx/{hash}", BASE_URI)).await
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
        loop {
            match CLIENT
                .get(format!("{}/balance/{address}?movable={movable}", BASE_URI))
                .send()
                .await
            {
                Ok(res) => match res.text().await {
                    Ok(payload) => {
                        let value: Result<HashMap<String, BalanceInfo>, serde_json::Error> =
                            serde_json::from_str(&payload);
                        match value {
                            Ok(val) => return Ok(Some(val)),
                            Err(err) => {
                                if is_not_found_err(&payload) {
                                    return Ok(None);
                                }
                                println!("get_account_balance response error: {}", err.to_string())
                            }
                        }
                    }
                    Err(err) => {
                        let error = err.to_string();
                        panic!("{error}");
                    }
                },
                Err(err) => {
                    println!("get_request '{address}' http communication err occured: '{err}'");
                    // Err(format!("3: {}",err.to_string()))
                }
            }
        }
    }

    pub async fn get_account_always(address: &str) -> AccountInfo {
        Self::get_request_always(format!("{}/account/{address}", BASE_URI)).await
    }

    pub async fn get_eth_address(eth_address: &str) -> Option<String> {
        Self::get_request(format!("{}/eth/{eth_address}", BASE_URI))
            .await
            .ok()
    }

    pub async fn get_nft_token_always(token_id: &str) -> NftState {
        Self::get_request_always(format!("{}/token/{token_id}", BASE_URI)).await
    }

    pub async fn get_as_text_always(uri: String) -> String {
        loop {
            match CLIENT.get(&uri).send().await {
                Ok(res) => match res.text().await {
                    Ok(payload) => return payload,
                    Err(err) => {
                        panic!("{}", err.to_string());
                    }
                },
                Err(err) => println!(
                    "get_request_with_json_always err '{err}' - {:?}",
                    uri.as_str()
                ),
            }
            sleep(Duration::from_millis(500)).await;
        }
    }

    pub async fn get_nft_balance(address: &str) -> Option<HashMap<String, NftBalanceInfo>> {
        Self::get_request(format!("{}/nft-balance/{address}", BASE_URI))
            .await
            .ok()
    }

    pub async fn get_nft_token(token_id: &str) -> Option<NftState> {
        Self::get_request(format!("{}/token/{token_id}", BASE_URI))
            .await
            .ok()
    }

    pub async fn post_txs(txs: String) -> Result<Vec<String>, String> {
        let ref url = format!("{}/tx", BASE_URI);
        match CLIENT
            .post(url.as_str())
            .header("Content-Type", "application/json")
            .body(txs)
            .send()
            .await
        {
            Ok(res) => match res.text().await {
                Ok(payload) => {
                    println!("Raw payload: {}", payload);
                    let value: Result<Vec<String>, serde_json::Error> =
                        serde_json::from_str(&payload);
                    match value {
                        Ok(val) => Ok(val),
                        Err(_) => {
                            let err_result: Result<String, _> = serde_json::from_str(&payload);
                            match err_result {
                                Ok(err_msg) => Err(format!("1: {err_msg}")),
                                Err(err) => Err(format!("2: {}", err.to_string())),
                            }
                        }
                    }
                }
                Err(err) => Err(err.to_string()),
            },
            Err(err) => {
                println!(
                    "get_request '{}' http communication err occured: '{err}'",
                    url.as_str()
                );
                Err(format!("3: {}", err.to_string()))
            }
        }
    }
}
