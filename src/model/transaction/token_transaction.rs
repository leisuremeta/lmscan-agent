use std::collections::{HashMap, HashSet};

use bigdecimal::BigDecimal;
use itertools::Itertools;
use sea_orm::Set;
use serde::{Deserialize, Serialize};

use crate::{
    library::common::{
        as_timestamp, as_vec, now,
    },
    tx_entity::ActiveModel,
};

use super::{common::Common, Transaction, TransactionWithResult};

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

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BurnNft {
    pub network_id: i64,
    pub created_at: String,
    pub definition_id: String,
    pub input: String,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct EntrustNft {
    pub network_id: i64,
    pub created_at: String,
    pub definition_id: String,
    pub token_id: String,
    pub input: String,
    pub to: String,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct EntrustFungibleToken {
    pub network_id: i64,
    pub created_at: String,
    pub definition_id: String,
    pub amount: BigDecimal,
    pub inputs: HashSet<String>,
    pub to: String,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BurnFungibleToken {
    pub network_id: i64,
    pub created_at: String,
    pub definition_id: String,
    pub amount: BigDecimal,
    pub inputs: HashSet<String>,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TransferNft {
    pub network_id: i64,
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
    pub network_id: i64,
    pub created_at: String,
    pub token_definition_id: String,
    pub inputs: HashSet<String>,
    pub outputs: HashMap<String, BigDecimal>,
    pub memo: Option<String>,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize, Eq, Hash)]
#[serde(rename_all = "camelCase")]
pub struct MintNft {
    pub network_id: i64,
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
    pub network_id: i64,
    pub created_at: String,
    pub definition_id: String,
    pub outputs: HashMap<String, BigDecimal>,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DefineToken {
    pub network_id: i64,
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
    pub rarity: HashMap<String, BigDecimal>,
    pub data_url: String,
    pub content_hash: String,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DisposeEntrustedNft {
    pub network_id: i64,
    pub created_at: String,
    pub definition_id: String,
    pub token_id: String,
    pub input: String,
    pub output: Option<String>,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DisposeEntrustedFungibleToken {
    pub network_id: i64,
    pub created_at: String,
    pub definition_id: String,
    pub inputs: HashSet<String>,
    pub outputs: HashMap<String, BigDecimal>,
}

impl Common for EntrustNft {
    fn created_at(&self) -> i64 {
        as_timestamp(self.created_at.as_str())
    }
    fn network_id(&self) -> i64 {
        self.network_id
    }
    fn from(
        &self,
        hash: String,
        from_account: String,
        block_hash: String,
        block_number: i64,
        json: String,
        _: TransactionWithResult,
    ) -> ActiveModel {
        ActiveModel {
            hash: Set(hash),
            tx_type: Set("Token".to_string()),
            token_type: Set("LM".to_string()),
            sub_type: Set("EntrustNft".to_string()),
            from_addr: Set(from_account),
            to_addr: Set(vec![self.to.to_owned()]),
            block_hash: Set(block_hash),
            block_number: Set(block_number),
            event_time: Set(self.created_at()),
            created_at: Set(now()),
            input_hashs: Set(Some(vec![self.input.clone()])),
            output_vals: Set(Some(vec![self.to.to_owned() + "/" + &self.token_id])),
            json: Set(json),
        }
    }
}

impl Common for EntrustFungibleToken {
    fn created_at(&self) -> i64 {
        as_timestamp(self.created_at.as_str())
    }
    fn network_id(&self) -> i64 {
        self.network_id
    }
    fn from(
        &self,
        hash: String,
        from_account: String,
        block_hash: String,
        block_number: i64,
        json: String,
        _: TransactionWithResult,
    ) -> ActiveModel {
        ActiveModel {
            hash: Set(hash),
            tx_type: Set("Token".to_string()),
            token_type: Set("LM".to_string()),
            sub_type: Set("EntrustFungibleToken".to_string()),
            from_addr: Set(from_account),
            to_addr: Set(vec![self.to.to_owned()]),
            block_hash: Set(block_hash),
            block_number: Set(block_number),
            event_time: Set(self.created_at()),
            created_at: Set(now()),
            input_hashs: Set(Some(as_vec(self.inputs.clone()))),
            output_vals: Set(Some(vec![
                self.to.to_owned() + "/" + &self.amount.to_string(),
            ])),
            json: Set(json),
        }
    }
}

impl Common for BurnFungibleToken {
    fn created_at(&self) -> i64 {
        as_timestamp(self.created_at.as_str())
    }
    fn network_id(&self) -> i64 {
        self.network_id
    }
    fn from(
        &self,
        hash: String,
        from_account: String,
        block_hash: String,
        block_number: i64,
        json: String,
        _: TransactionWithResult,
    ) -> ActiveModel {
        ActiveModel {
            hash: Set(hash),
            tx_type: Set("Token".to_string()),
            token_type: Set("LM".to_string()),
            sub_type: Set("BurnFungibleToken".to_string()),
            from_addr: Set(from_account.clone()),
            to_addr: Set(vec![from_account.clone()]),
            block_hash: Set(block_hash),
            block_number: Set(block_number),
            event_time: Set(self.created_at()),
            created_at: Set(now()),
            input_hashs: Set(Some(as_vec(self.inputs.clone()))),
            output_vals: Set(Some(vec![from_account + "/" + &self.amount.to_string()])),
            json: Set(json),
        }
    }
}

impl Common for TransferNft {
    fn created_at(&self) -> i64 {
        as_timestamp(self.created_at.as_str())
    }
    fn network_id(&self) -> i64 {
        self.network_id
    }
    fn from(
        &self,
        hash: String,
        from_account: String,
        block_hash: String,
        block_number: i64,
        json: String,
        _: TransactionWithResult,
    ) -> ActiveModel {
        ActiveModel {
            hash: Set(hash),
            tx_type: Set("Token".to_string()),
            token_type: Set("NFT".to_string()),
            sub_type: Set("TransferNft".to_string()),
            from_addr: Set(from_account),
            to_addr: Set(vec![self.output.clone()]),
            block_hash: Set(block_hash),
            block_number: Set(block_number),
            event_time: Set(self.created_at()),
            created_at: Set(now()),
            input_hashs: Set(Some(vec![self.input.clone()])),
            output_vals: Set(Some(vec![self.output.clone() + "/" + &self.token_id])),
            json: Set(json),
        }
    }
}

impl Common for TransferFungibleToken {
    fn created_at(&self) -> i64 {
        as_timestamp(self.created_at.as_str())
    }
    fn network_id(&self) -> i64 {
        self.network_id
    }
    fn from(
        &self,
        hash: String,
        from_account: String,
        block_hash: String,
        block_number: i64,
        json: String,
        tx: TransactionWithResult,
    ) -> ActiveModel {
        let mut to_accounts: Vec<String> = vec![];
        let output_vals: Option<Vec<String>> = match tx.signed_tx.value {
            Transaction::TokenTx(t) => match t {
                TokenTx::TransferFungibleToken(v) => {
                    to_accounts = v.clone().outputs.into_iter().map(|(x, _)| x).collect_vec();
                    Some(
                        v.outputs
                            .into_iter()
                            .map(|(k, v)| k + "/" + &v.to_string())
                            .collect(),
                    )
                }
                _ => None,
            },
            _ => None,
        };

        ActiveModel {
            hash: Set(hash),
            tx_type: Set("Token".to_string()),
            token_type: Set("LM".to_string()),
            sub_type: Set("TransferFungibleToken".to_string()),
            from_addr: Set(from_account),
            to_addr: Set(to_accounts),
            block_hash: Set(block_hash),
            block_number: Set(block_number),
            event_time: Set(self.created_at()),
            created_at: Set(now()),
            input_hashs: Set(Some(as_vec(self.inputs.clone()))),
            output_vals: Set(output_vals),
            json: Set(json),
        }
    }
}

impl Common for MintNft {
    fn created_at(&self) -> i64 {
        as_timestamp(self.created_at.as_str())
    }
    fn network_id(&self) -> i64 {
        self.network_id
    }
    fn from(
        &self,
        hash: String,
        from_account: String,
        block_hash: String,
        block_number: i64,
        json: String,
        _: TransactionWithResult,
    ) -> ActiveModel {
        let to_addr = self.output.to_owned();
        ActiveModel {
            hash: Set(hash),
            tx_type: Set("Token".to_string()),
            token_type: Set("NFT".to_string()),
            sub_type: Set("MintNft".to_string()),
            from_addr: Set(from_account),
            to_addr: Set(vec![to_addr.clone()]),
            block_hash: Set(block_hash),
            block_number: Set(block_number),
            event_time: Set(self.created_at()),
            created_at: Set(now()),
            input_hashs: Set(None),
            output_vals: Set(Some(vec![to_addr + "/" + &self.token_id])),
            json: Set(json),
        }
    }
}

impl Common for MintFungibleToken {
    fn created_at(&self) -> i64 {
        as_timestamp(self.created_at.as_str())
    }
    fn network_id(&self) -> i64 {
        self.network_id
    }
    fn from(
        &self,
        hash: String,
        from_account: String,
        block_hash: String,
        block_number: i64,
        json: String,
        _: TransactionWithResult,
    ) -> ActiveModel {
        ActiveModel {
            hash: Set(hash),
            tx_type: Set("Token".to_string()),
            token_type: Set("LM".to_string()),
            sub_type: Set("MintFungibleToken".to_string()),
            from_addr: Set(from_account),
            to_addr: Set(vec![]),
            block_hash: Set(block_hash),
            block_number: Set(block_number),
            event_time: Set(self.created_at()),
            created_at: Set(now()),
            input_hashs: Set(None),
            output_vals: Set(None),
            json: Set(json),
        }
    }
}

impl Common for DefineToken {
    fn created_at(&self) -> i64 {
        as_timestamp(self.created_at.as_str())
    }
    fn network_id(&self) -> i64 {
        self.network_id
    }
    fn from(
        &self,
        hash: String,
        from_account: String,
        block_hash: String,
        block_number: i64,
        json: String,
        _: TransactionWithResult,
    ) -> ActiveModel {
        ActiveModel {
            hash: Set(hash),
            tx_type: Set("Token".to_string()),
            token_type: Set(self.definition_id.clone()),
            sub_type: Set("DefineToken".to_string()),
            from_addr: Set(from_account),
            to_addr: Set(vec![]),
            block_hash: Set(block_hash),
            block_number: Set(block_number),
            event_time: Set(self.created_at()),
            created_at: Set(now()),
            input_hashs: Set(None),
            output_vals: Set(None),
            json: Set(json),
        }
    }
}

impl Common for DisposeEntrustedNft {
    fn created_at(&self) -> i64 {
        as_timestamp(self.created_at.as_str())
    }
    fn network_id(&self) -> i64 {
        self.network_id
    }
    fn from(
        &self,
        hash: String,
        from_account: String,
        block_hash: String,
        block_number: i64,
        json: String,
        _: TransactionWithResult,
    ) -> ActiveModel {
        let to_account = match &self.output {
            Option::Some(value) => value.to_owned(),
            None => String::from(""),
        };

        ActiveModel {
            hash: Set(hash),
            tx_type: Set("Token".to_string()),
            token_type: Set("NFT".to_string()),
            sub_type: Set("DisposeEntrustedNft".to_string()),
            from_addr: Set(from_account),
            to_addr: Set(vec![to_account.clone()]),
            block_hash: Set(block_hash),
            block_number: Set(block_number),
            event_time: Set(self.created_at()),
            created_at: Set(now()),
            input_hashs: Set(Some(vec![self.input.clone()])),
            output_vals: Set(Some(vec![to_account + "/" + &self.token_id])),
            json: Set(json),
        }
    }
}

impl Common for DisposeEntrustedFungibleToken {
    fn created_at(&self) -> i64 {
        as_timestamp(self.created_at.as_str())
    }
    fn network_id(&self) -> i64 {
        self.network_id
    }
    fn from(
        &self,
        hash: String,
        from_account: String,
        block_hash: String,
        block_number: i64,
        json: String,
        _: TransactionWithResult,
    ) -> ActiveModel {
        let to_accounts = (&self.outputs).keys().map(|addr| addr.to_owned()).collect();
        let output_vals: Vec<String> = (&self.outputs)
            .into_iter()
            .map(|(k, v)| k.to_owned() + "/" + &v.to_string())
            .collect();
        ActiveModel {
            hash: Set(hash),
            tx_type: Set("Token".to_string()),
            token_type: Set("LM".to_string()),
            sub_type: Set("DisposeEntrustedFungibleToken".to_string()),
            from_addr: Set(from_account),
            to_addr: Set(vec![to_accounts]),
            block_hash: Set(block_hash),
            block_number: Set(block_number),
            event_time: Set(self.created_at()),
            created_at: Set(now()),
            input_hashs: Set(Some(as_vec(self.inputs.clone()))),
            output_vals: Set(Some(output_vals)),
            json: Set(json),
        }
    }
}

impl Common for BurnNft {
    fn created_at(&self) -> i64 {
        as_timestamp(self.created_at.as_str())
    }
    fn network_id(&self) -> i64 {
        self.network_id
    }
    fn from(
        &self,
        hash: String,
        from_account: String,
        block_hash: String,
        block_number: i64,
        json: String,
        _: TransactionWithResult,
    ) -> ActiveModel {
        ActiveModel {
            hash: Set(hash),
            tx_type: Set("Token".to_string()),
            token_type: Set("NFT".to_string()),
            sub_type: Set("BurnNft".to_string()),
            from_addr: Set(from_account),
            to_addr: Set(vec![]),
            block_hash: Set(block_hash),
            block_number: Set(block_number),
            event_time: Set(self.created_at()),
            created_at: Set(now()),
            input_hashs: Set(Some(vec![self.input.clone()])),
            output_vals: Set(Some(vec![format!("/")])),
            json: Set(json),
        }
    }
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
    fn network_id(&self) -> i64 {
        match self {
            TokenTx::EntrustNft(t) => t.network_id,
            TokenTx::EntrustFungibleToken(t) => t.network_id,
            TokenTx::TransferNft(t) => t.network_id,
            TokenTx::TransferFungibleToken(t) => t.network_id,
            TokenTx::MintNft(t) => t.network_id,
            TokenTx::MintFungibleToken(t) => t.network_id,
            TokenTx::DefineToken(t) => t.network_id,
            TokenTx::DisposeEntrustedNft(t) => t.network_id,
            TokenTx::DisposeEntrustedFungibleToken(t) => t.network_id,
            TokenTx::BurnNft(t) => t.network_id,
            TokenTx::BurnFungibleToken(t) => t.network_id,
        }
    }

    fn from(
        &self,
        hash: String,
        from_account: String,
        block_hash: String,
        block_number: i64,
        json: String,
        tx: TransactionWithResult,
    ) -> ActiveModel {
        match self {
            TokenTx::BurnNft(t) => t.from(hash, from_account, block_hash, block_number, json, tx),
            TokenTx::EntrustNft(t) => {
                t.from(hash, from_account, block_hash, block_number, json, tx)
            }
            TokenTx::EntrustFungibleToken(t) => {
                t.from(hash, from_account, block_hash, block_number, json, tx)
            }
            TokenTx::TransferNft(t) => {
                t.from(hash, from_account, block_hash, block_number, json, tx)
            }
            TokenTx::TransferFungibleToken(t) => {
                t.from(hash, from_account, block_hash, block_number, json, tx)
            }
            TokenTx::MintNft(t) => t.from(hash, from_account, block_hash, block_number, json, tx),
            TokenTx::MintFungibleToken(t) => {
                t.from(hash, from_account, block_hash, block_number, json, tx)
            }
            TokenTx::DefineToken(t) => {
                t.from(hash, from_account, block_hash, block_number, json, tx)
            }
            TokenTx::DisposeEntrustedNft(t) => {
                t.from(hash, from_account, block_hash, block_number, json, tx)
            }
            TokenTx::DisposeEntrustedFungibleToken(t) => {
                t.from(hash, from_account, block_hash, block_number, json, tx)
            }
            TokenTx::BurnFungibleToken(t) => {
                t.from(hash, from_account, block_hash, block_number, json, tx)
            }
        }
    }
}
