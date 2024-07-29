use std::str::FromStr;
use crate::{
    block::Block, model::node_status::NodeStatus, transaction::TransactionWithResult
};
use futures_util::TryFutureExt;
use lazy_static::lazy_static;
use log::error;
use std::fmt::Debug;
use reqwest::Url;

extern crate dotenvy;
use dotenvy::var;

lazy_static! {
    static ref CLIENT: reqwest::Client = reqwest::Client::new();
    static ref BASE: String = var("BASE_URL").expect("URL must be set");
}

pub struct ApiService;

impl ApiService {
    fn make_url(param: &str) -> Url {
        let mut url = Url::from_str(BASE.as_str()).unwrap();
        url.set_path(param);
        url
    }
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

    pub async fn get_request<S: serde::de::DeserializeOwned + Debug>(
        url: Url,
    ) -> Result<S, String> {
        CLIENT.get(url.as_str()).send().and_then(|res| res.json()).map_err(|e| {
            error!("{}", url);
            e.to_string()
        }).await
    }

    pub async fn get_request_until<T: reqwest::IntoUrl, S: serde::de::DeserializeOwned + Debug>(
        url: T,
        count: u8,
    ) -> Option<S> {
        for _ in 0..count {
            match CLIENT.get(url.as_str()).send().await {
                Ok(res) => match res.json::<S>().await {
                    Ok(payload) => return Some(payload),
                    Err(err) => error!("get_request_until parse err '{err}' - {:?}", url.as_str())
                },
                Err(err) => error!("get_request_until err '{err}' - {:?}", url.as_str()),
            }
        }
        None
    }

    pub async fn get_node_status_always() -> Result<NodeStatus, String> {
        Self::get_request(Self::make_url("/status")).await
    }

    pub async fn get_block_always(hash: &str) -> Result<Block, String> {
        Self::get_request(Self::make_url(&("/block/".to_owned() + hash))).await
    }

    pub async fn get_tx_always(hash: &str) -> Result<TransactionWithResult, String> {
        Self::get_request(Self::make_url(&("/tx/".to_owned() + hash))).await
    }

    pub async fn get_tx_with_json_always(hash: &str) -> Result<String, reqwest::Error> {
        CLIENT.get(Self::make_url(&("/tx/".to_owned() + hash))).send()
            .and_then(|res| res.text()).await
    }
}
