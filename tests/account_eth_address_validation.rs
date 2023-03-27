use std::{fs::{OpenOptions, File}, io::{BufReader, BufRead}, collections::HashMap};

use lmscan_agent::{model::{account_info::AccountInfo, nft_state::NftState}, library::common::get_request_always};


static BASE_URI: &str = "http://lmc.leisuremeta.io";
// static BASE_URI: &str = "http://test.chain.leisuremeta.io";

async fn get_account_always(address: &str) -> AccountInfo {
  get_request_always(format!("{BASE_URI}/account/{address}")).await
}

async fn get_nft_token_always(token_id: &str) -> NftState {
  get_request_always(format!("{BASE_URI}/token/{token_id}")).await
}

async fn get_nft_balance(address: &str) -> Option<HashMap<String, NftBalanceInfo>> {
  let res: Result<Option<HashMap<String, NftBalanceInfo>>, String> = get_request(format!("{BASE_URI}/nft-balance/{address}")).await;
  res.unwrap_or_default()
}

async fn get_nft_token(token_id: &str) -> Option<NftState> {
  let res: Result<Option<NftState>, String> = get_request(format!("{BASE_URI}/token/{token_id}")).await;
  res.unwrap_or_default()
}

#[tokio::test]
async fn validate_nft_owner() {
  let read_file = "/app/playnomm_scan/deploy/test/lmscan-agent/data.txt";
  // let filename = "/app/playnomm_scan/deploy/test/lmscan-agent/nft_owner_service_202303211130.sql";
  let write_file = "/app/playnomm_scan/deploy/test/lmscan-agent/output.csv";

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
    let address  = items.next().unwrap().trim();
    let token_id = items.next().unwrap().trim();

    // let nft = get_nft_token_always(token_id).await.token_id;
    // println!("{nft}");

    // let account = get_account_always(address).await;
    // println!("{:?}, {:?}", address, token_id);

    let nft_balance_info = get_nft_balance(address).await.unwrap_or_default();

    let is_nft_exist_in_account_from_blc = nft_balance_info.contains_key(token_id);

    let nft_opt = get_nft_token(token_id).await;
    let current_owner = if nft_opt.is_none() { "NFT 데이터 블록체인에 존재 X".to_string() } else { nft_opt.as_ref().unwrap().current_owner.clone() };
    let is_same_nft_owner = current_owner == address;
    
    let result = match nft_opt {
      Some(_) => match (is_nft_exist_in_account_from_blc, is_same_nft_owner) {
        (true,  true)  => 1, // Address가 Token 소유하며 Token의 현재 소유주와 일치.
        (true,  false) => 2, // Address가 Token 소유하나, 현재 토큰의 소유주는 다른 계정.
        (false, true)  => 3, // Address가 Token 소유하지 않으나, 현재 토큰의 소유주로 되있음.
        (false, false) => 4, // Address가 Token 소유하지 않으며, 현재 토큰의 소유주도 다른 사람.
      }
      None => 5, // NFT 데이터가 블록체인에 존재하지 않음.
    };       
    let token_ids = nft_balance_info.into_keys().collect::<Vec<String>>().join(",");
    let output = format!("'{address}'\t'{token_id}'\t'{result}'\t'{token_ids}'");
    println!("{output}");
 
    output_file
        .write(output.as_bytes())
        .expect("write failed");
  }


}