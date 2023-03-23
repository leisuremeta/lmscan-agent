use itertools::Itertools;
use lmscan_agent::model::nft_balance_info::{NftBalanceInfo, self};
use lmscan_agent::model::nft_state::NftState;
use lmscan_agent::{entity::account_entity, library::common::get_request_always, model::balance_info::BalanceInfo};
use dotenvy::{dotenv, var};
use lmscan_agent::library::common::{db_connn, get_request};
use sea_orm::*;



use std::collections::HashMap;
use std::fs::File;
use std::io::{BufRead, BufReader};

static BASE_URI: &str = "http://lmc.leisuremeta.io";
// static BASE_URI: &str = "http://test.chain.leisuremeta.io";

async fn get_nft_balance(address: &str) -> HashMap<String, NftBalanceInfo> {
  let res: Result<HashMap<String, NftBalanceInfo>, reqwest::Error> = get_request(format!("{BASE_URI}/nft-balance/{address}")).await;
  res.unwrap_or_default()
}

async fn get_nft_token(token_id: &str) -> Option<NftState> {
  let res: Result<NftState, reqwest::Error> = get_request(format!("{BASE_URI}/token/{token_id}")).await;
  res.ok()
}

#[tokio::test]
async fn validate_nft_owner() {
  let token_id_len = "202212211000092386".len();

  // let filename = "/Users/user/Downloads/nft_owner_service_202303211130.sql";
  let filename = "/app/playnomm_scan/deploy/test/lmscan-agent/nft_owner_service_202303211130.sql";
  match File::open(filename) {
    Ok(file) => {
      let reader = BufReader::new(file);
      let mut lines = reader.lines();
      lines.next();
      for line in lines {
        let line = line.unwrap();
        let items = line.split(",");
        
        let mut token_id = String::new();
        let mut address = String::new();
        for item in items {
          let item = item.replace("\"", "");
          if token_id_len == item.len() &&
             item.chars().all(char::is_numeric) 
          {
            token_id = item;
          }
          else if item.len() == 40 {
            address = item;
          }
        }
        
        if address.is_empty()  { address = String::from("playnomm"); }
        if token_id.is_empty() { println!("line: {line}"); panic!(); } 

        // println!("{address}, {token_id}");
        let nft_balance_info = get_nft_balance(address.as_str()).await;
        
        let is_nft_exist_in_account_from_blockchain = nft_balance_info.get(token_id.as_str()).is_some();

        let nft_state_opt = get_nft_token(token_id.as_str()).await;
        let current_owner = if nft_state_opt.is_none() { "nft 데이터 블록체인에 존재 X".to_string() } else { nft_state_opt.unwrap().current_owner };
        let is_same_nft_owner = current_owner == address;
        
        let result = match (is_nft_exist_in_account_from_blockchain, is_same_nft_owner) {
          (true,  true)  => format!("{address} - {token_id} 일치"),
          (true,  false) => format!("{address} : '{token_id}' 소유 O, '{token_id}'의 현재 소유주 X '{current_owner}'"),
          (false, true)  => format!("{address} : '{token_id}' 소유 X, '{token_id}'의 현재 소유주 O"),
          (false, false) => format!("{address} : '{token_id}' 소유 X, '{token_id}'의 현재 소유주 X '{current_owner}'"),
        };
        println!("{result}");
      }
    },
    Err(error) => {
      println!("Error opening file {}: {}", filename, error);
    },
  }
}
