use std::{path::PathBuf, collections::HashSet};


use sled::Db;

pub trait SledStore {
  fn spent_hashs(account_addr: &str) -> HashSet<String>;
  fn insert(account_addr: String, value: HashSet<String>);
  fn flush();
}


pub fn init(sled_path: PathBuf) -> Db{
  sled::Config::default() 
    .path(sled_path)
    .use_compression(false)
    // .compression_factor(6)
    .flush_every_ms(None)
    .open()
    .unwrap()
}


