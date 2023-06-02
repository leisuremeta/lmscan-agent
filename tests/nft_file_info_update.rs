use std::{collections::HashMap};


use lmscan_agent::{service::api_service::ApiService, transaction::{TransactionWithResult, Transaction, TokenTx, NftMetaInfo}, nft_file, tx_entity, library::common::as_timestamp};
use dotenvy::{var};
use lmscan_agent::library::common::db_connn;
use sea_orm::*;

#[tokio::test]
async fn nft_file_info_update() {
  let database_url = var("DATABASE_URL").expect("DATABASE_URL must be set.");
  let ref db = db_connn(database_url).await;

  let mint_txs = tx_entity::Entity::find()
                                    .filter(tx_entity::Column::SubType.eq("MintNft"))
                                    .all(db)
                                    .await
                                    .unwrap();

  let mut nft_file_map: HashMap<String, nft_file::Model> =
    nft_file::Entity::find().all(db).await.unwrap()
                    .into_iter().map(|nft_file| (nft_file.clone().token_id, nft_file))
                    .collect();

  for tx in mint_txs.into_iter() {
    let mint_tx: TransactionWithResult = serde_json::from_str(&tx.json).unwrap();
    if let Transaction::TokenTx(tx) = mint_tx.signed_tx.value {
      if let TokenTx::MintNft(mint_tx) = tx {
        let nft_meta_info_opt: Option<NftMetaInfo> = ApiService::get_request_until(mint_tx.data_url.clone(), 6).await;
        let nft_file_opt = nft_file_map.get_mut(&mint_tx.token_id);
        match (nft_meta_info_opt, nft_file_opt) {
          (Some(info), Some(nft_file)) =>  {
            nft_file.collection_name = info.collection_name;
            nft_file.creator = info.creator;
            nft_file.creator_description = info.collection_description;
            nft_file.data_url = mint_tx.data_url;
            nft_file.rarity = info.rarity;
            nft_file.event_time = as_timestamp(&mint_tx.created_at);
            nft_file.owner = mint_tx.output;
            nft_file.nft_uri = info.nft_uri;
            nft_file.nft_name = info.nft_name;

            let nft_file_model = nft_file.clone().into_active_model();

            nft_file::Entity::update(nft_file_model).exec(db).await.unwrap();
          },
          _ => {
            println!("mint_tx error: {:?}", mint_tx);
          }
        }
      }
    }
  };
    
  
}
