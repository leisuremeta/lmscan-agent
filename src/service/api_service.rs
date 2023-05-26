use std::{time::Duration, collections::HashMap, error::Error};

use crate::{transaction::TransactionWithResult, model::{blockchain_response::{Either, ResultError}, node_status::NodeStatus, balance_info::BalanceInfo, account_info::AccountInfo, nft_state::NftState, nft_balance_info::NftBalanceInfo}, block::Block};
use lazy_static::lazy_static;
use log::info;
use rayon::vec;
use serde_json::json;
use tokio::time::sleep;
use std::fmt::Debug;

lazy_static! {
  static ref CLIENT: reqwest::Client = reqwest::Client::new();
}


// static BASE_URI: &str = "http://lmc.leisuremeta.io";
// static BASE_URI: &str = "http://test.chain.leisuremeta.io";
static BASE_URI: &str = "http://localhost:8081";

pub struct ApiService;

impl ApiService {

  pub async fn get_request_header_always<T: reqwest::IntoUrl, S: serde::de::DeserializeOwned + Debug>(url: T, api_key: &str) -> S {
    loop {
      match CLIENT.get(url.as_str()).header("X-CMC_PRO_API_KEY", api_key).send().await {
        Ok(res) => match res.json::<S>().await  {
          Ok(payload) => return payload,
          Err(err) => println!("get_request_always parse err '{err}' - {:?}", url.as_str()),
        }
        Err(err) => println!("get_request_always err '{err}' - {:?}", url.as_str()),
      }
      sleep(Duration::from_millis(500)).await;
    }
  }

  async fn get_request_always<T: reqwest::IntoUrl, S: serde::de::DeserializeOwned + Debug>(url: T) -> S {
    info!("get_request_always : {:?}", url.as_str());
    loop {
      match CLIENT.get(url.as_str()).send().await {
        Ok(res) => {
          // println!("--- {}", serde_json::to_string(&CLIENT.get(url.as_str()).send().await.unwrap().text().await.unwrap()).unwrap());
          match res.json::<S>().await  {
            Ok(payload) => return payload,
            Err(err) => {
              println!("get_request_always parse err '{err}' - {:?}", url.as_str());
              println!("{:?}",CLIENT.get(url.as_str()).send().await.ok().unwrap().text().await);
            },
          }
        },
        Err(err) => println!("get_request_always err '{err}' - {:?}", url.as_str()),
      }
      sleep(Duration::from_millis(500)).await;
    }
  }

  async fn get_request_with_json_always<T: reqwest::IntoUrl, S: serde::ser::Serialize + serde::de::DeserializeOwned + Debug>(url: T) -> (S, String) {
    loop {
      match CLIENT.get(url.as_str()).send().await {
        Ok(res) => {
          match res.text().await {
          // match res.json::<S>().await  {
            Ok(payload) => {
              //return (CLIENT.get(url.as_str()).send().await.unwrap().text().await.unwrap(), payload)
              return (serde_json::from_str(&payload).unwrap(), payload)
            },
            Err(err) => {
              println!("--- {}", &CLIENT.get(url.as_str()).send().await.unwrap().text().await.unwrap());
              panic!("{}", err.to_string());
            },
          }
        },
        Err(err) => println!("get_request_with_json_always err '{err}' - {:?}", url.as_str()),
      }
      sleep(Duration::from_millis(500)).await;
    }
  }

  // async fn get_request<T: reqwest::IntoUrl, S: serde::de::DeserializeOwned + Debug>(url: T) -> Result<Option<S>, String> {
  //   // match reqwest::get(url.as_str()).await {
  //   //   Ok(res) => match res.text().await{
  //   //     Ok(payload) => println!("api response: {}\n", payload),
  //   //     Err(err) => {
  //   //       println!("error response: {}",err);
  //   //       panic!();
  //   //     },
  //   //   }
  //   //   Err(err) => panic!(),
  //   // };
  //   match CLIENT.get(url.as_str()).send().await {
  //     Ok(res) => match res.json::<Either<S, ResultError>>().await  {
  //     // Ok(res) => match res.json().await  {
  //       Ok(payload) => match payload {
  //         Either::Right(val) => Ok(Some(val)),
  //         Either::Left(err) => {
  //           if err.value.is_not_found_err() {
  //             Ok(None)
  //           } else {
  //             Err(err.value.msg)
  //           }
  //         }
  //       },
  //       // Ok(payload) => {
  //       //   println!("{payload}");

  //       //   let payload: Result<Either<S, ResultError>, serde_json::Error> = serde_json::from_str(&payload);
  //       //   match payload {
  //       //     Ok(either) => match either {
  //       //       Either::Right(val) => Ok(Some(val)),
  //       //       Either::Left(err) => {
  //       //         if err.value.is_not_found_err() {
  //       //           Ok(None)
  //       //         } else {
  //       //           Err(err.value.msg)
  //       //         }
  //       //       },
  //       //     },
  //       //     Err(err) => Err(format!("1: {}",err.to_string()))
  //       //   }
  //       // },
  //       Err(err) => {
  //         // println!("get_request parse err '{err}' - {:?}", url.as_str()); 
  //         Err(format!("2: {}",err.to_string()))
  //       },
  //     }
  //     Err(err) => {
  //       println!("get_request '{:?}' http communication err occured: '{err}'", url.as_str()); 
  //       Err(format!("3: {}",err.to_string()))
  //     },
  //   }
  // }
  
  async fn get_request<T: reqwest::IntoUrl, S: serde::de::DeserializeOwned + serde::de::DeserializeOwned + Debug>(url: T) -> Result<Option<S>, String> {
    match CLIENT.get(url.as_str()).send().await {
      Ok(res) => match res.text().await  {
        Ok(payload) => {
          let value: Result<S, serde_json::Error> = serde_json::from_str(&payload);
          match value {
            Ok(val) => Ok(Some(val)),
            Err(err) => {
              let err_result: Result<String, _> = serde_json::from_str(&payload);
              match err_result {
                Ok(err_msg) => Ok(None),
                Err(err) => Err(err.to_string()),
              }
            },
          }
        },
        Err(err) => Err(err.to_string()),
      },
      Err(err) => {
        println!("get_request '{:?}' http communication err occured: '{err}'", url.as_str()); 
        Err(format!("3: {}",err.to_string()))
      },
    
    }
  }


  // async fn get_request<T: reqwest::IntoUrl, S: serde::de::DeserializeOwned + Debug>(url: T) -> Result<Option<S>, String> {
  //   // match reqwest::get(url.as_str()).await {
  //   //   Ok(res) => match res.text().await{
  //   //     Ok(payload) => println!("api response: {}\n", payload),
  //   //     Err(err) => {
  //   //       println!("error response: {}",err);
  //   //       panic!();
  //   //     },
  //   //   }
  //   //   Err(err) => panic!(),
  //   // };
  //   match CLIENT.get(url.as_str()).send().await {
  //     // Ok(res) => match res.json::<Either<S, ResultError>>().await  {
  //     Ok(res) => match res.text().await  {
  //       // Ok(payload) => match payload {
  //       //   Either::Right(val) => Ok(Some(val)),
  //       //   Either::Left(err) => {
  //       //     if err.value.is_not_found_err() {
  //       //       Ok(None)
  //       //     } else {
  //       //       Err(err.value.msg)
  //       //     }
  //       //   }
  //       // },
  //       Ok(payload) => {
  //         println!("{payload}");

  //         let payload: Result<Either<S, ResultError>, serde_json::Error> = serde_json::from_str(&payload);
  //         match payload {
  //           Ok(either) => match either {
  //             Either::Right(val) => Ok(Some(val)),
  //             Either::Left(err) => {
  //               if err.value.is_not_found_err() {
  //                 Ok(None)
  //               } else {
  //                 Err(err.value.msg)
  //               }
  //             },
  //           },
  //           Err(err) => Err(format!("1: {}",err.to_string()))
  //         }
  //       },
  //       Err(err) => {
  //         // println!("get_request parse err '{err}' - {:?}", url.as_str()); 
  //         Err(format!("2: {}",err.to_string()))
  //       },
  //     }
  //     Err(err) => {
  //       println!("get_request '{:?}' http communication err occured: '{err}'", url.as_str()); 
  //       Err(format!("3: {}",err.to_string()))
  //     },
  //   }
  // }

  pub async fn get_request_until<T: reqwest::IntoUrl, S: serde::de::DeserializeOwned + Debug>(url: T, count: u8) -> Option<S> {
    info!("get_request_until {count} : {:?}", url.as_str());
    for _ in 0..count {
      match CLIENT.get(url.as_str()).send().await {
        Ok(res) => match res.json::<S>().await  {
          Ok(payload) => return Some(payload),
          Err(err) => println!("get_request_until parse err '{err}' - {:?}", url.as_str()),
        }
        Err(err) => println!("get_request_until err '{err}' - {:?}", url.as_str()),
      }
    }
    None
  } 

  pub async fn get_node_status_always() -> NodeStatus {
    Self::get_request_always(format!("{BASE_URI}/status")).await
  }
  
  pub async fn get_block_always(hash: &str) -> Block {
    Self::get_request_always(format!("{BASE_URI}/block/{hash}")).await
  }
  
  pub async fn get_tx_always(hash: &str) -> TransactionWithResult {
    Self::get_request_always(format!("{BASE_URI}/tx/{hash}")).await
  }  

  pub async fn get_tx_with_json_always(hash: &str) -> (TransactionWithResult, String) {
    Self::get_request_with_json_always(format!("{BASE_URI}/tx/{hash}")).await
  }  
  
  pub async fn get_account_balance(hash: &str) -> Result<Option<HashMap<String, BalanceInfo>>, String> {
    Self::get_request(format!("{BASE_URI}/balance/{hash}?movable=free")).await
  }
  
  pub async fn get_account_always(address: &str) -> AccountInfo {
    Self::get_request_always(format!("{BASE_URI}/account/{address}")).await
  }

  pub async fn get_eth_address(eth_address: &str) -> Option<String> {
    let res: Result<Option<String>, String> = Self::get_request(format!("{BASE_URI}/eth/{eth_address}")).await;
    res.unwrap_or_default()
  }

  pub async fn get_nft_token_always(token_id: &str) -> NftState {
    Self::get_request_always(format!("{BASE_URI}/token/{token_id}")).await
  }

  pub async fn get_nft_balance(address: &str) -> Option<HashMap<String, NftBalanceInfo>> {
    let res: Result<Option<HashMap<String, NftBalanceInfo>>, String> = Self::get_request(format!("{BASE_URI}/nft-balance/{address}")).await;
    res.unwrap_or_default()
  }

  pub async fn get_nft_token(token_id: &str) -> Option<NftState> {
    let res: Result<Option<NftState>, String> = Self::get_request(format!("{BASE_URI}/token/{token_id}")).await;
    res.unwrap_or_default()
  }

  pub async fn post_txs(txs: String) -> Result<Vec<String>, String> {
    let ref url = format!("{BASE_URI}/tx");
    match CLIENT.post(url.as_str()).header("Content-Type", "application/json").body(txs).send().await {
      Ok(res) => match res.text().await  {
        Ok(payload) => {
          println!("Raw payload: {}", payload);  
          let value: Result<Vec<String>, serde_json::Error> = serde_json::from_str(&payload);
          match value {
            Ok(val) => Ok(val),
            Err(err) => {
              // println!("0: {}", err.to_string());
              let err_result: Result<String, _> = serde_json::from_str(&payload);
              match err_result {
                Ok(err_msg) => Err(format!("1: {err_msg}")),
                Err(err) => Err(format!("2: {}", err.to_string())),
              }
            },
          }
        },
        Err(err) => Err(err.to_string()),
      },
      Err(err) => {
        println!("get_request '{:?}' http communication err occured: '{err}'", url.as_str()); 
        Err(format!("3: {}",err.to_string()))
      },
    }
  }  
}
