use itertools::Itertools;
use lmscan_agent::model::nft_balance_info::{NftBalanceInfo, self};
use lmscan_agent::model::nft_state::NftState;
use lmscan_agent::{entity::account_entity, library::common::get_request_always, model::balance_info::BalanceInfo};
use dotenvy::{dotenv, var};
use lmscan_agent::library::common::db_connn;
use sea_orm::*;


use std::collections::HashMap;
use std::fs::File;
use std::io::{BufRead, BufReader};

// static BASE_URI: &str = "http://lmc.leisuremeta.io";
static BASE_URI: &str = "http://test.chain.leisuremeta.io";

async fn get_nft_balance_always(address: &str) -> HashMap<String, NftBalanceInfo> {
  get_request_always(format!("{BASE_URI}/nft-balance/{address}")).await
}

async fn get_nft_token_always(token_id: &str) -> NftState {
  get_request_always(format!("{BASE_URI}/token/{token_id}")).await
}


async fn validate_nft_owner() {
  // dotenv().expect("Unable to load environment variables from .env file");
  // let database_url = var("DATABASE_URL").expect("DATABASE_URL must be set.");
  // let ref db = db_connn(database_url).await;

  let token_id_len = "202212211000092386".len();

  let filename = "/Users/user/Downloads/nft_owner_service_202303211130.sql";
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

        println!("{address}, {token_id}");
        let nft_balance_info = get_nft_balance_always(address.as_str()).await;
        
        let is_nft_exist_in_account_from_blockchain = nft_balance_info.get(token_id.as_str()).is_some();

        if !is_nft_exist_in_account_from_blockchain {
          println!("{address} 의 NFT balance 조회 결과 '{token_id}' Token ID 의 NFT를 블록체인에서 가지고 있지 않음 ");
        }

        let nft_state = get_nft_token_always(token_id.as_str()).await;
        let current_owner = nft_state.current_owner;
        let is_same_nft_owner = current_owner == address;
        
        let result = match (is_nft_exist_in_account_from_blockchain, is_same_nft_owner) {
          (true,  true)  => format!("{address} - {token_id} 일치"),
          (true,  false) => format!("{address} : '{token_id}' 소유 O, '{token_id}'의 현재 소유주 X"),
          (false, true)  => format!("{address} : '{token_id}' 소유 X, '{token_id}'의 현재 소유주 O"),
          (false, false) => format!("{address} : '{token_id}' 소유 X, '{token_id}'의 현재 소유주 X"),
        };
        println!("{result}");
      }
    },
    Err(error) => {
      println!("Error opening file {}: {}", filename, error);
    },
  }
}
