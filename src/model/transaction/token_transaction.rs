use std::collections::{HashMap, HashSet};

use bigdecimal::BigDecimal;
use sea_orm::Set;
use serde::{Deserialize, Serialize};

use crate::{
    library::common::{
        as_timestamp, now,
    },
    tx_entity::{self, ActiveModel},
    nft_tx
};

use super::{common::Common, TransactionWithResult};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum TokenTx {
    #[serde(rename = "BurnNFT")]
    BurnNft(BurnNft),
    #[serde(rename = "EntrustNFT")]
    EntrustNft(EntrustNft),
    EntrustFungibleToken(EntrustFungibleToken),
    BurnFungibleToken(BurnFungibleToken),
    #[serde(rename = "TransferNFT")]
    TransferNft(TransferNft),
    TransferFungibleToken(TransferFungibleToken),
    #[serde(rename = "MintNFT")]
    MintNft(MintNft),
    MintFungibleToken(MintFungibleToken),
    DefineToken(DefineToken),
    #[serde(rename = "DisposeEntrustedNFT")]
    DisposeEntrustedNft(DisposeEntrustedNft),
    DisposeEntrustedFungibleToken(DisposeEntrustedFungibleToken),
}

impl Common for TokenTx {
    fn created_at(&self) -> i64 {
        match self {
            TokenTx::EntrustNft(t) => t.created_at(),
            TokenTx::EntrustFungibleToken(t) => t.created_at(),
            TokenTx::TransferNft(t) => t.created_at(),
            TokenTx::TransferFungibleToken(t) => t.created_at(),
            TokenTx::MintNft(t) => t.created_at(),
            TokenTx::MintFungibleToken(t) => t.created_at(),
            TokenTx::DefineToken(t) => t.created_at(),
            TokenTx::DisposeEntrustedNft(t) => t.created_at(),
            TokenTx::DisposeEntrustedFungibleToken(t) => t.created_at(),
            TokenTx::BurnNft(t) => t.created_at(),
            TokenTx::BurnFungibleToken(t) => t.created_at(),
        }
    }

    fn from(
        &self,
        hash: String,
        block_hash: String,
        block_number: i64,
        tx: TransactionWithResult,
    ) -> ActiveModel {
        match self {
            TokenTx::BurnNft(t) => t.from(hash, block_hash, block_number, tx),
            TokenTx::EntrustNft(t) => {
                t.from(hash, block_hash, block_number, tx)
            }
            TokenTx::EntrustFungibleToken(t) => {
                t.from(hash, block_hash, block_number, tx)
            }
            TokenTx::TransferNft(t) => {
                t.from(hash, block_hash, block_number, tx)
            }
            TokenTx::TransferFungibleToken(t) => {
                t.from(hash, block_hash, block_number, tx)
            }
            TokenTx::MintNft(t) => t.from(hash, block_hash, block_number, tx),
            TokenTx::MintFungibleToken(t) => {
                t.from(hash, block_hash, block_number, tx)
            }
            TokenTx::DefineToken(t) => {
                t.from(hash, block_hash, block_number, tx)
            }
            TokenTx::DisposeEntrustedNft(t) => {
                t.from(hash, block_hash, block_number, tx)
            }
            TokenTx::DisposeEntrustedFungibleToken(t) => {
                t.from(hash, block_hash, block_number, tx)
            }
            TokenTx::BurnFungibleToken(t) => {
                t.from(hash, block_hash, block_number, tx)
            }
        }
    }
}

impl TokenTx {
    pub fn get_nft_active_model(&self, tx_entity: &tx_entity::ActiveModel, from: String) -> Option<nft_tx::ActiveModel> {
        match self {
            TokenTx::EntrustNft(tx) => Some(nft_tx::Model::from(
                self, tx_entity, tx.input.clone(), tx.to.clone()
            )),
            TokenTx::TransferNft(tx) => Some(nft_tx::Model::from(
                self, tx_entity, tx.input.clone(), tx.output.clone()
            )),
            TokenTx::MintNft(tx) => Some(nft_tx::Model::from(
                self, tx_entity, from, tx.output.clone()
            )),
            TokenTx::DisposeEntrustedNft(tx) => {
                let out = tx.output.as_ref().unwrap_or(&tx.input);
                Some(nft_tx::Model::from(self, tx_entity, tx.input.clone(), out.to_string()))
            },
            _ => None
        }
    }

    pub fn token_id(&self) -> String {
        match self {
            TokenTx::EntrustNft(tx) => tx.token_id.clone(),
            TokenTx::TransferNft(tx) => tx.token_id.clone(),
            TokenTx::MintNft(tx) => tx.token_id.clone(),
            TokenTx::DisposeEntrustedNft(tx) => tx.token_id.clone(),
            _ => String::from("")
        }
    }

    pub fn sub_type(&self) -> String {
        match self {
            TokenTx::EntrustNft(_) => String::from("EntrustNft"),
            TokenTx::TransferNft(_) => String::from("TransferNft"),
            TokenTx::MintNft(_) =>  String::from("MintNft"),
            TokenTx::DisposeEntrustedNft(_) => String::from("DisposeEntrustedNft"),
            _ => String::from("")
        }
    }

    pub fn get_accounts(&self, signer: String) -> Vec<String> {
        let mut v = match self {
            TokenTx::EntrustNft(tx) =>vec![tx.to.clone()],
            TokenTx::EntrustFungibleToken(tx) => vec![tx.to.clone()],
            TokenTx::TransferNft(tx) => vec![tx.output.clone()],
            TokenTx::TransferFungibleToken(tx) => tx.outputs.clone().into_keys().collect(),
            TokenTx::MintNft(tx) => vec![tx.output.clone()],
            TokenTx::MintFungibleToken(tx) => tx.outputs.clone().into_keys().collect(),
            TokenTx::DisposeEntrustedNft(tx) => match tx.output.clone() {
                Some(v) => vec![v],
                None => vec![]
            }
            TokenTx::DisposeEntrustedFungibleToken(tx) => tx.outputs.clone().into_keys().collect(),
            _ => vec![]
        };
        v.push(signer);
        v
    }
}

// TokenTx::MintNft(tx) => 
//     let nft_meta_info_opt =
//         ApiService::get_request_until(tx.data_url.clone(), 1).await;
//     let nft_file = nft_file::Model::from(tx, nft_meta_info_opt, tx.data_url.clone());

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BurnNft {
    pub created_at: String,
    pub definition_id: String,
    pub input: String,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct EntrustNft {
    pub created_at: String,
    pub definition_id: String,
    pub token_id: String,
    pub input: String,
    pub to: String,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct EntrustFungibleToken {
    pub created_at: String,
    pub definition_id: String,
    pub amount: BigDecimal,
    pub inputs: HashSet<String>,
    pub to: String,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BurnFungibleToken {
    pub created_at: String,
    pub definition_id: String,
    pub amount: BigDecimal,
    pub inputs: HashSet<String>,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TransferNft {
    pub created_at: String,
    pub definition_id: String,
    pub token_id: String,
    pub input: String,
    pub output: String,
    pub memo: Option<String>,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TransferFungibleToken {
    pub created_at: String,
    pub token_definition_id: String,
    pub inputs: HashSet<String>,
    pub outputs: HashMap<String, BigDecimal>,
    pub memo: Option<String>,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize, Eq, Hash)]
#[serde(rename_all = "camelCase")]
pub struct MintNft {
    pub created_at: String,
    pub token_definition_id: String,
    pub token_id: String,
    pub rarity: String,
    pub data_url: String,
    pub content_hash: String,
    pub output: String,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MintFungibleToken {
    pub created_at: String,
    pub definition_id: String,
    pub outputs: HashMap<String, BigDecimal>,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DefineToken {
    pub created_at: String,
    pub definition_id: String,
    pub name: String,
    pub symbol: Option<String>,
    pub minter_group: Option<String>,
    pub nft_info: Option<NftInfo>,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct NftInfo {
    #[serde(rename= "Some")]
    pub some: Inner,
}
#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Inner {
    pub value: Value,
}
#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Value {
    pub minter: String,
    pub rarity: HashMap<String, Option<BigDecimal>>,
    pub data_url: String,
    pub content_hash: String,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DisposeEntrustedNft {
    pub created_at: String,
    pub definition_id: String,
    pub token_id: String,
    pub input: String,
    pub output: Option<String>,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DisposeEntrustedFungibleToken {
    pub created_at: String,
    pub definition_id: String,
    pub inputs: HashSet<String>,
    pub outputs: HashMap<String, BigDecimal>,
}

impl Common for EntrustNft {
    fn created_at(&self) -> i64 {
        as_timestamp(self.created_at.as_str())
    }
    fn from(
        &self,
        hash: String,
        block_hash: String,
        block_number: i64,
        _: TransactionWithResult,
    ) -> ActiveModel {
        ActiveModel {
            hash: Set(hash),
            tx_type: Set("Token".to_string()),
            token_type: Set("LM".to_string()),
            sub_type: Set("EntrustNft".to_string()),
            block_hash: Set(block_hash),
            block_number: Set(block_number),
            event_time: Set(self.created_at()),
            created_at: Set(now()),
        }
    }
}

impl Common for EntrustFungibleToken {
    fn created_at(&self) -> i64 {
        as_timestamp(self.created_at.as_str())
    }
    fn from(
        &self,
        hash: String,
        block_hash: String,
        block_number: i64,
        _: TransactionWithResult,
    ) -> ActiveModel {
        ActiveModel {
            hash: Set(hash),
            tx_type: Set("Token".to_string()),
            token_type: Set("LM".to_string()),
            sub_type: Set("EntrustFungibleToken".to_string()),
            block_hash: Set(block_hash),
            block_number: Set(block_number),
            event_time: Set(self.created_at()),
            created_at: Set(now()),
        }
    }
}

impl Common for BurnFungibleToken {
    fn created_at(&self) -> i64 {
        as_timestamp(self.created_at.as_str())
    }
    fn from(
        &self,
        hash: String,
        block_hash: String,
        block_number: i64,
        _: TransactionWithResult,
    ) -> ActiveModel {
        ActiveModel {
            hash: Set(hash),
            tx_type: Set("Token".to_string()),
            token_type: Set("LM".to_string()),
            sub_type: Set("BurnFungibleToken".to_string()),
            block_hash: Set(block_hash),
            block_number: Set(block_number),
            event_time: Set(self.created_at()),
            created_at: Set(now()),
        }
    }
}

impl Common for TransferNft {
    fn created_at(&self) -> i64 {
        as_timestamp(self.created_at.as_str())
    }
    fn from(
        &self,
        hash: String,
        block_hash: String,
        block_number: i64,
        _: TransactionWithResult,
    ) -> ActiveModel {
        ActiveModel {
            hash: Set(hash),
            tx_type: Set("Token".to_string()),
            token_type: Set("NFT".to_string()),
            sub_type: Set("TransferNft".to_string()),
            block_hash: Set(block_hash),
            block_number: Set(block_number),
            event_time: Set(self.created_at()),
            created_at: Set(now()),
        }
    }
}

impl Common for TransferFungibleToken {
    fn created_at(&self) -> i64 {
        as_timestamp(self.created_at.as_str())
    }
    fn from(
        &self,
        hash: String,
        block_hash: String,
        block_number: i64,
        _: TransactionWithResult,
    ) -> ActiveModel {
        ActiveModel {
            hash: Set(hash),
            tx_type: Set("Token".to_string()),
            token_type: Set("LM".to_string()),
            sub_type: Set("TransferFungibleToken".to_string()),
            block_hash: Set(block_hash),
            block_number: Set(block_number),
            event_time: Set(self.created_at()),
            created_at: Set(now()),
        }
    }
}

impl Common for MintNft {
    fn created_at(&self) -> i64 {
        as_timestamp(self.created_at.as_str())
    }
    fn from(
        &self,
        hash: String,
        block_hash: String,
        block_number: i64,
        _: TransactionWithResult,
    ) -> ActiveModel {
        ActiveModel {
            hash: Set(hash),
            tx_type: Set("Token".to_string()),
            token_type: Set("NFT".to_string()),
            sub_type: Set("MintNft".to_string()),
            block_hash: Set(block_hash),
            block_number: Set(block_number),
            event_time: Set(self.created_at()),
            created_at: Set(now()),
        }
    }
}

impl Common for MintFungibleToken {
    fn created_at(&self) -> i64 {
        as_timestamp(self.created_at.as_str())
    }
    fn from(
        &self,
        hash: String,
        block_hash: String,
        block_number: i64,
        _: TransactionWithResult,
    ) -> ActiveModel {
        ActiveModel {
            hash: Set(hash),
            tx_type: Set("Token".to_string()),
            token_type: Set("LM".to_string()),
            sub_type: Set("MintFungibleToken".to_string()),
            block_hash: Set(block_hash),
            block_number: Set(block_number),
            event_time: Set(self.created_at()),
            created_at: Set(now()),
        }
    }
}

impl Common for DefineToken {
    fn created_at(&self) -> i64 {
        as_timestamp(self.created_at.as_str())
    }
    fn from(
        &self,
        hash: String,
        block_hash: String,
        block_number: i64,
        _: TransactionWithResult,
    ) -> ActiveModel {
        ActiveModel {
            hash: Set(hash),
            tx_type: Set("Token".to_string()),
            token_type: Set(self.definition_id.clone()),
            sub_type: Set("DefineToken".to_string()),
            block_hash: Set(block_hash),
            block_number: Set(block_number),
            event_time: Set(self.created_at()),
            created_at: Set(now()),
        }
    }
}

impl Common for DisposeEntrustedNft {
    fn created_at(&self) -> i64 {
        as_timestamp(self.created_at.as_str())
    }
    fn from(
        &self,
        hash: String,
        block_hash: String,
        block_number: i64,
        _: TransactionWithResult,
    ) -> ActiveModel {
        ActiveModel {
            hash: Set(hash),
            tx_type: Set("Token".to_string()),
            token_type: Set("NFT".to_string()),
            sub_type: Set("DisposeEntrustedNft".to_string()),
            block_hash: Set(block_hash),
            block_number: Set(block_number),
            event_time: Set(self.created_at()),
            created_at: Set(now()),
        }
    }
}

impl Common for DisposeEntrustedFungibleToken {
    fn created_at(&self) -> i64 {
        as_timestamp(self.created_at.as_str())
    }
    fn from(
        &self,
        hash: String,
        block_hash: String,
        block_number: i64,
        _: TransactionWithResult,
    ) -> ActiveModel {
        ActiveModel {
            hash: Set(hash),
            tx_type: Set("Token".to_string()),
            token_type: Set("LM".to_string()),
            sub_type: Set("DisposeEntrustedFungibleToken".to_string()),
            block_hash: Set(block_hash),
            block_number: Set(block_number),
            event_time: Set(self.created_at()),
            created_at: Set(now()),
        }
    }
}

impl Common for BurnNft {
    fn created_at(&self) -> i64 {
        as_timestamp(self.created_at.as_str())
    }
    fn from(
        &self,
        hash: String,
        block_hash: String,
        block_number: i64,
        _: TransactionWithResult,
    ) -> ActiveModel {
        ActiveModel {
            hash: Set(hash),
            tx_type: Set("Token".to_string()),
            token_type: Set("NFT".to_string()),
            sub_type: Set("BurnNft".to_string()),
            block_hash: Set(block_hash),
            block_number: Set(block_number),
            event_time: Set(self.created_at()),
            created_at: Set(now()),
        }
    }
}
