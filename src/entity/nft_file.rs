use sea_orm::entity::prelude::*;
use sea_orm::*;

use crate::{
    library::common::{as_timestamp, now},
    transaction::{token_transaction::MintNft, NftMetaInfo},
};

#[derive(Clone, Debug, PartialEq, Eq, DeriveEntityModel)]
#[sea_orm(table_name = "nft_file")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub token_id: String,
    pub token_def_id: String,
    pub collection_name: String,
    pub nft_name: String,
    pub nft_uri: String,
    pub creator_description: String,
    pub data_url: String,
    pub rarity: String,
    pub creator: String,
    pub event_time: i64,
    pub created_at: i64,
    // pub owner: String,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}
impl ActiveModelBehavior for ActiveModel {}

impl Model {
    pub fn from(tx: &MintNft, info_opt: Option<NftMetaInfo>) -> ActiveModel {
        let info = info_opt.unwrap_or_default();
        ActiveModel {
            token_id: Set(tx.token_id.clone()),
            token_def_id: Set(tx.token_definition_id.clone()),
            collection_name: Set(info.collection_name),
            nft_name: Set(info.nft_name),
            nft_uri: Set(info.nft_uri.clone()),
            creator_description: Set(info.creator_description),
            data_url: Set(info.nft_uri.clone()),
            rarity: Set(info.rarity),
            creator: Set(info.creator),
            event_time: Set(as_timestamp(&tx.created_at)),
            created_at: Set(now()),
            // owner: Set(tx.output.to_owned()),
        }
    }
}
