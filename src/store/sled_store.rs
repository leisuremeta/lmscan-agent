use std::{path::PathBuf, collections::HashSet};


use bigdecimal::ToPrimitive;
use sled::Db;

pub trait SledStore {
  fn spent_hashs(account_addr: &str) -> HashSet<String>;
  fn insert(account_addr: String, value: HashSet<String>);
  fn flush() -> bool;
}

pub fn init(sled_path: PathBuf) -> Db {
  sled::Config::default() 
      .path(sled_path)
      .use_compression(false)
      .flush_every_ms(None)
      .open()
      .unwrap()
}

pub fn init_with_compression(sled_path: PathBuf, compression_level: usize) -> Db{
  if compression_level < 1 || compression_level > 22 {
    panic!("Unsupported compression level '{compression_level}'. Ranges from 1 up to 22.")
  }
  sled::Config::default() 
    .path(sled_path)
    .use_compression(true)
    .compression_factor(compression_level.to_i32().unwrap())
    .flush_every_ms(None)
    .open()
    .unwrap()
}


