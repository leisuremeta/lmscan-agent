use chrono::NaiveDateTime;
use log::LevelFilter;
use sea_orm::{ConnectOptions, Database, DatabaseConnection};
use sled::IVec;
use std::time::{SystemTime, UNIX_EPOCH};
use std::{
    collections::HashSet,
    path::PathBuf,
    time::Duration,
};

pub async fn db_connn(database_url: String) -> DatabaseConnection {
    let mut opt = ConnectOptions::new(database_url.to_string());
    opt.min_connections(4)
        .max_connections(8)
        .connect_timeout(Duration::from_secs(30))
        .acquire_timeout(Duration::from_secs(30))
        .idle_timeout(Duration::from_secs(120))
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
    match NaiveDateTime::parse_from_str(str_date, "%Y-%m-%dT%H:%M:%S%.3fZ") {
        Ok(v) => v.and_utc().timestamp(),
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

pub fn parse_from_json_str<'a, T: serde::Deserialize<'a>>(json: &'a str) -> Result<T, serde_json::Error> {
    serde_json::from_str::<T>(json)
}

pub fn as_path_buf(sled_path: &str) -> PathBuf {
    let mut path = std::env::current_dir().unwrap();
    path.push(sled_path);
    path
}
