use std::collections::HashMap;

use dotenvy::var;
use lmscan_agent::library::common::db_connn;
use lmscan_agent::{
    nft_file,
    service::api_service::ApiService,
    transaction::{token_transaction::TokenTx, NftMetaInfo, Transaction, TransactionWithResult},
    tx_entity,
};
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

    let nft_files = nft_file::Entity::find().all(db).await.unwrap();
    let nft_files_map: HashMap<String, nft_file::Model> = nft_files
        .into_iter()
        .filter(|file| file.nft_uri.is_empty())
        .map(|x: nft_file::Model| (x.token_id.clone(), x))
        .collect();
    for tx in mint_txs.into_iter() {
        let mint_tx: TransactionWithResult = serde_json::from_str(&tx.json).unwrap();
        if let Transaction::TokenTx(tx) = mint_tx.signed_tx.value {
            if let TokenTx::MintNft(mint_tx) = tx {
                if let Some(_) = nft_files_map.get(&mint_tx.token_id) {
                    let nft_meta_info_opt: Option<NftMetaInfo> =
                        ApiService::get_request_until(mint_tx.data_url.clone(), 6).await;

                    let nft_file = nft_file::Model::from(&mint_tx, nft_meta_info_opt);
                    nft_file::Entity::update(nft_file).exec(db).await.unwrap();
                }
            }
        }
    }
}
