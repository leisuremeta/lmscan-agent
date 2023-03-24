use itertools::Itertools;
use lmscan_agent::model::account_info::AccountInfo;
use lmscan_agent::model::nft_balance_info::{NftBalanceInfo, self};
use lmscan_agent::model::nft_state::NftState;
use lmscan_agent::{entity::account_entity, library::common::get_request_always, model::balance_info::BalanceInfo};
use dotenvy::{dotenv, var};
use lmscan_agent::library::common::{db_connn, get_request};
use sea_orm::*;



use std::collections::HashMap;
use std::fs::{File, OpenOptions};
use std::io::{BufRead, BufReader, Write};

static BASE_URI: &str = "http://lmc.leisuremeta.io";
// static BASE_URI: &str = "http://test.chain.leisuremeta.io";

async fn get_account_always(address: &str) -> AccountInfo {
  get_request_always(format!("{BASE_URI}/account/{address}")).await
}

async fn get_nft_token_always(token_id: &str) -> NftState {
  get_request_always(format!("{BASE_URI}/token/{token_id}")).await
}

async fn get_nft_balance(address: &str) -> HashMap<String, NftBalanceInfo> {
  let res: Result<HashMap<String, NftBalanceInfo>, reqwest::Error> = get_request(format!("{BASE_URI}/nft-balance/{address}")).await;
  res.unwrap_or_default()
}

async fn get_nft_token(token_id: &str) -> Option<NftState> {
  let res: Result<NftState, reqwest::Error> = get_request(format!("{BASE_URI}/token/{token_id}")).await;
  res.ok()
}

// (true,  true)  => format!("{address}: '{token_id}' 일치"),
//           (true,  false) => format!("{address}: '{token_id}' 소유 O, '{token_id}'의 현재 소유주 X '{current_owner}'"),
//           (false, true)  => format!("{address}: '{token_id}' 소유 X, '{token_id}'의 현재 소유주 O"),
//           (false, false) => format!("{address}: '{token_id}' 소유 X, '{token_id}'의 현재 소유주 X '{current_owner}'"),

#[tokio::test]
async fn validate_nft_owner() {
  let token_id_len = "202212211000092386".len();


  let read_file = "/Users/user/playnomm/source_code/lmscan-agent/data.txt";
  // let filename = "/app/playnomm_scan/deploy/test/lmscan-agent/nft_owner_service_202303211130.sql";
  let write_file = "/Users/user/playnomm/source_code/lmscan-agent/output.csv";

  let mut output_file = OpenOptions::new()
                                          .append(true)
                                          .open(write_file)
                                          .expect("cannot open output file");
  let input_file = File::open(read_file)
                              .expect("cannot open input file");
  let reader = BufReader::new(input_file);
  let mut lines = reader.lines();
  lines.next();
  for line in lines {
    let line = line.unwrap();
    let mut items = line.split_whitespace();
    // println!("{line}");
    let address = items.next().unwrap().trim();
    let token_id = items.next().unwrap().trim();

    // let account = get_account_always(address).await;
    // println!("{:?}, {:?}", address, token_id);

    let nft_balance_info = get_nft_balance(address).await;
    let is_nft_exist_in_account_from_blc = nft_balance_info.contains_key(token_id);

    let nft_opt = get_nft_token(token_id).await;
    let current_owner = if nft_opt.is_none() { "NFT 데이터 블록체인에 존재 X".to_string() } else { nft_opt.as_ref().unwrap().current_owner.clone() };
    let is_same_nft_owner = current_owner == address;
    
    let result = match (is_nft_exist_in_account_from_blc, nft_opt, is_same_nft_owner) {
      (true,  None,    true)  => 5,
      (true,  None,    false) => 5,
      (false, None,    true)  => 5,
      (false, None,    false) => 5,
      (true,  Some(_), true)  => 1,
      (true,  Some(_), false) => 2,
      (false, Some(_), true)  => 3,
      (false, Some(_), false) => 4,
    };       
    let token_ids = nft_balance_info.into_keys().collect::<Vec<String>>().join(",");
    let output = format!("{address}    {token_id}    {result}    {token_ids}\n");
    // Write to a file
    output_file
        .write(output.as_bytes())
        .expect("write failed");
  }


}
