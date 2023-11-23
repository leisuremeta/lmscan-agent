use chrono::NaiveDateTime;
use log::LevelFilter;
use sea_orm::{ConnectOptions, Database, DatabaseConnection};
use serde_json::value::RawValue;
use sled::IVec;
use std::time::{SystemTime, UNIX_EPOCH};
use std::{
    collections::{HashMap, HashSet},
    path::PathBuf,
    time::Duration,
};

use bigdecimal::BigDecimal;
use serde::{Deserialize, Deserializer};
use std::str::FromStr;

pub async fn db_connn(database_url: String) -> DatabaseConnection {
    let mut opt = ConnectOptions::new(database_url.to_string());
    opt.min_connections(4)
        .max_connections(8)
        .connect_timeout(Duration::from_secs(30))
        .acquire_timeout(Duration::from_secs(30))
        .idle_timeout(Duration::from_secs(120))
        .set_schema_search_path("public".into())
        .sqlx_logging(true)
        .sqlx_logging_level(LevelFilter::Debug);

    match Database::connect(opt).await {
        Ok(conn) => conn,
        Err(err) => panic!("{err}"),
    }
}

pub fn now() -> i64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_secs() as i64
}

pub fn as_timestamp(str_date: &str) -> i64 {
    match NaiveDateTime::parse_from_str(str_date.clone(), "%Y-%m-%dT%H:%M:%S%.3fZ") {
        Ok(v) => v.timestamp(),
        Err(err) => panic!("timestamp parse err '{str_date}' - {err}"),
    }
}

pub fn as_vec<T: std::hash::Hash + std::cmp::Eq>(set: HashSet<T>) -> Vec<T> {
    set.into_iter().collect()
}

pub fn as_json_byte_vec<T: serde::Serialize>(value: &T) -> Vec<u8> {
    serde_json::to_vec(value).unwrap()
}

pub fn from_ivec<T: for<'a> serde::Deserialize<'a> + Default>(bytes: &IVec) -> T {
    if bytes.is_empty() {
        return T::default();
    }

    // println!("bytes: {:?}", bytes);
    match bincode::deserialize(bytes) {
        Ok(deserialized_val) => deserialized_val,
        Err(err) => {
            eprintln!("Failed to deserialize data: {:?}. Make sure the serialized data is in the correct format and matches the expected type.", err);
            panic!()
        }
    }
}

pub fn into_byte_vec<T: serde::Serialize>(value: &T) -> Vec<u8> {
    bincode::serialize(value).unwrap()
}

pub fn parse_from_json_str<'a, T: serde::Deserialize<'a>>(json: &'a str) -> T {
    // serde_json::from_str::<T>(json).unwrap()
    match serde_json::from_str::<T>(json) {
        Ok(result) => result,
        Err(err) => {
            println!("{json}");
            panic!("{err}");
        }
    }
}

pub fn from_rawvalue_to_bigdecimal<'de, D>(deserializer: D) -> Result<BigDecimal, D::Error>
where
    D: Deserializer<'de>,
{
    let raw_value: &RawValue = Deserialize::deserialize(deserializer)?;
    let value_str = String::from_utf8_lossy(raw_value.get().as_bytes()).to_string();
    BigDecimal::from_str(&value_str).map_err(serde::de::Error::custom)
}

pub fn from_rawvalue_to_bigdecimal_map<'de, D, K>(
    deserializer: D,
) -> Result<HashMap<K, BigDecimal>, D::Error>
where
    D: Deserializer<'de>,
    K: Deserialize<'de> + std::hash::Hash + Eq,
{
    let map = HashMap::<K, &RawValue>::deserialize(deserializer)?;

    map.into_iter()
        .map(|(k, v)| {
            let value_str = String::from_utf8_lossy(v.get().as_bytes()).to_string();
            let value = BigDecimal::from_str(&value_str).map_err(serde::de::Error::custom)?;
            Ok((k, value))
        })
        .collect()
}

pub fn is_not_found_err(msg: &str) -> bool {
    msg.contains("not found")
}

pub fn as_path_buf(sled_path: &str) -> PathBuf {
    let mut path = std::env::current_dir().unwrap();
    path.push(sled_path);
    path
}
