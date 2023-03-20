use std::time::Duration;

use log::{info, LevelFilter};
use reqwest::Client;
use sea_orm::{DatabaseConnection, ConnectOptions, Database};
use tokio::time::sleep;
use std::fmt::Debug;
use std::time::{SystemTime, UNIX_EPOCH};
use chrono::NaiveDateTime;

use crate::transaction::TransactionWithResult;
use lazy_static::lazy_static;

lazy_static! {
  static ref CLIENT: Client = reqwest::Client::new();
}

pub async fn db_connn(database_url: String) -> DatabaseConnection {
  let mut opt = ConnectOptions::new(database_url.to_string());
  opt.min_connections(8)
     .max_connections(12)
     .connect_timeout(Duration::from_secs(30))
     .acquire_timeout(Duration::from_secs(30))
     .idle_timeout(Duration::from_secs(120))
     .set_schema_search_path("public".into())
     .sqlx_logging(true)
     .sqlx_logging_level(LevelFilter::Debug);

  match Database::connect(opt).await {
    Ok(conn) => conn,
    Err(err) =>  panic!("{err}"),
  }
}

pub async fn get_tx_request_always(url: String) -> TransactionWithResult {
  println!("get_request_always : {:?}", url.as_str());
  loop {
    match CLIENT.get(url.as_str()).send().await {
      Ok(res) => match res.json::<TransactionWithResult>().await  {
        Ok(payload) => return payload,
        Err(err) => println!("get_tx_request_always parse err '{err}' - {:?}", url.as_str()),
      }
      Err(err) => println!("get_request_always err '{err}' - {:?}", url.as_str()),
    }
    sleep(Duration::from_millis(500)).await;
  }
}

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

pub async fn get_request_always<T: reqwest::IntoUrl, S: serde::de::DeserializeOwned + Debug>(url: T) -> S {
  info!("get_request_always : {:?}", url.as_str());
  // let x = match reqwest::get(url.as_str()).await {
  //     Ok(res) => match res.json::<NftMetaInfo>().await  {
  //       Ok(payload) => println!("payload: {:?}", payload),
  //       Err(err) => {
  //         println!("1 - {:?}\n{:?}", err, url.as_str());
  //       },
  //     }
  //     Err(err) => {println!("2 - {:?}", err);},
  //   };
  loop {
    match CLIENT.get(url.as_str()).send().await {
      Ok(res) => match res.json::<S>().await  {
        Ok(payload) => return payload,
        Err(err) => println!("get_request_always parse err '{err}' - {:?}", url.as_str()),
      }
      Err(err) => println!("get_request_always err '{err}' - {:?}", url.as_str()),
    }
    sleep(Duration::from_millis(500)).await;
  }
}

pub async fn get_request_until<T: reqwest::IntoUrl, S: serde::de::DeserializeOwned + Debug>(url: T, count: u8) -> Option<S> {
  info!("get_request_until {count} : {:?}", url.as_str());
  for _ in 0..count {
    match CLIENT.get(url.as_str()).send().await {
      Ok(res) => match res.json::<S>().await  {
        Ok(payload) => return Some(payload),
        Err(err) => println!("get_request_always parse err '{err}' - {:?}", url.as_str()),
      }
      Err(err) => println!("get_request_always err '{err}' - {:?}", url.as_str()),
    }
  }
  None
} 

pub fn now() -> i64 {
  SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs() as i64
}

pub fn as_timestamp(str_date: &str) -> i64 {
  match NaiveDateTime::parse_from_str(str_date.clone(), "%Y-%m-%dT%H:%M:%S%.3fZ") {
    Ok(v) => v.timestamp(),
    Err(err) => panic!("timestamp parse err '{str_date}' - {err}"),
  }
}

pub fn parse_from_json_str<'a, T: serde::Deserialize<'a>>(json: &'a str) -> T {
  serde_json::from_str::<T>(json).unwrap()
}



