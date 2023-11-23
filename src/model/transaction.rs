pub mod account_transaction;
mod agenda_transaction;
pub mod common;
mod group_transaction;
pub mod reward_transaction;
pub mod token_transaction;

use bigdecimal::BigDecimal;
use core::panic;
use sea_orm::prelude::async_trait::async_trait;
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};
use std::fmt::Debug;
extern crate chrono;
use crate::library::common::{from_rawvalue_to_bigdecimal, from_rawvalue_to_bigdecimal_map};
use crate::service::api_service::ApiService;
use crate::service::finder_service::Finder;
use crate::store::free_balance::FreeBalanceStore;
use crate::store::locked_balance::LockedBalanceStore;
use crate::store::sled_store::SledStore;
use crate::store::wal::State;
use crate::tx_entity::{self, ActiveModel};
use crate::{account_entity, nft_file, nft_tx};

use self::account_transaction::*;
use self::agenda_transaction::*;
use self::common::Common;
use self::group_transaction::*;
use self::reward_transaction::*;
use self::token_transaction::*;

use super::balance::Balance;

impl TransactionWithResult {
    pub fn from(json: &str) -> Option<TransactionWithResult> {
        match serde_json::from_str::<TransactionWithResult>(json) {
            Ok(tx_res) => Some(tx_res),
            Err(err) => panic!("TransactionWithResult encode err: {err}"),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct TransactionWithResult {
    #[serde(rename = "signedTx")]
    pub signed_tx: SignedTx,
    pub result: Option<TransactionResult>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum TransactionResult {
    #[serde(rename = "AddPublicKeySummariesResult")]
    AddPublicKeySummariesResult(AddPublicKeySummariesResult),
    #[serde(rename = "BurnFungibleTokenResult")]
    BurnFungibleTokenResult(BurnFungibleTokenResult),
    #[serde(rename = "EntrustFungibleTokenResult")]
    EntrustFungibleTokenResult(EntrustFungibleTokenResult),
    #[serde(rename = "ExecuteRewardResult")]
    ExecuteRewardResult(ExecuteRewardResult),
    #[serde(rename = "ExecuteOwnershipRewardResult")]
    ExecuteOwnershipRewardResult(ExecuteOwnershipRewardResult),
    #[serde(rename = "VoteSimpleAgendaResult")]
    VoteSimpleAgendaResult(VoteSimpleAgendaResult),
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct AddPublicKeySummariesResult {
    pub removed: HashMap<String, String>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct BurnFungibleTokenResult {
    #[serde(
        rename = "outputAmount",
        deserialize_with = "from_rawvalue_to_bigdecimal"
    )]
    pub output_amount: BigDecimal,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct EntrustFungibleTokenResult {
    #[serde(deserialize_with = "from_rawvalue_to_bigdecimal")]
    pub remainder: BigDecimal,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ExecuteRewardResult {
    #[serde(deserialize_with = "from_rawvalue_to_bigdecimal_map")]
    pub outputs: HashMap<String, BigDecimal>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ExecuteOwnershipRewardResult {
    #[serde(deserialize_with = "from_rawvalue_to_bigdecimal_map")]
    pub outputs: HashMap<String, BigDecimal>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct VoteSimpleAgendaResult {
    #[serde(
        rename = "votingAmount",
        deserialize_with = "from_rawvalue_to_bigdecimal"
    )]
    pub voting_amount: BigDecimal,
}
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SignedTx {
    pub sig: AccountSignature,
    pub value: Transaction,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct AccountSignature {
    pub sig: Signature,
    pub account: String,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Signature {
    pub v: i64,
    pub r: String,
    pub s: String,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum Transaction {
    #[serde(rename = "RewardTx")]
    RewardTx(RewardTx),
    #[serde(rename = "TokenTx")]
    TokenTx(TokenTx),
    #[serde(rename = "AccountTx")]
    AccountTx(AccountTx),
    #[serde(rename = "GroupTx")]
    GroupTx(GroupTx),
    #[serde(rename = "AgendaTx")]
    AgendaTx(AgendaTx),
}

impl Common for Transaction {
    fn created_at(&self) -> i64 {
        match self {
            Transaction::RewardTx(t) => t.created_at(),
            Transaction::TokenTx(t) => t.created_at(),
            Transaction::AccountTx(t) => t.created_at(),
            Transaction::GroupTx(t) => t.created_at(),
            Transaction::AgendaTx(t) => t.created_at(),
        }
    }

    fn network_id(&self) -> i64 {
        match self {
            Transaction::RewardTx(t) => t.network_id(),
            Transaction::TokenTx(t) => t.network_id(),
            Transaction::AccountTx(t) => t.network_id(),
            Transaction::GroupTx(t) => t.network_id(),
            Transaction::AgendaTx(t) => t.network_id(),
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
        let from_account = from_account.to_owned();
        match self {
            Transaction::RewardTx(t) => {
                t.from(hash, from_account, block_hash, block_number, json, tx)
            }
            Transaction::TokenTx(t) => {
                t.from(hash, from_account, block_hash, block_number, json, tx)
            }
            Transaction::AccountTx(t) => {
                t.from(hash, from_account, block_hash, block_number, json, tx)
            }
            Transaction::GroupTx(t) => {
                t.from(hash, from_account, block_hash, block_number, json, tx)
            }
            Transaction::AgendaTx(t) => {
                t.from(hash, from_account, block_hash, block_number, json, tx)
            }
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct NftMetaInfo {
    #[serde(rename = "Creator_description")]
    pub creator_description: String,
    #[serde(rename = "Collection_description")]
    pub collection_description: String,
    #[serde(rename = "Rarity")]
    pub rarity: String,
    #[serde(rename = "NFT_checksum")]
    pub nft_checksum: String,
    #[serde(rename = "Collection_name")]
    pub collection_name: String,
    #[serde(rename = "Creator")]
    pub creator: String,
    #[serde(rename = "NFT_name")]
    pub nft_name: String,
    #[serde(rename = "NFT_URI")]
    pub nft_uri: String,
}

pub trait NftTx {
    fn token_id(&self) -> String;
    fn sub_type(&self) -> String;
}

impl NftTx for MintNft {
    fn token_id(&self) -> String {
        self.token_id.clone()
    }
    fn sub_type(&self) -> String {
        String::from("MintNft")
    }
}

impl NftTx for TransferNft {
    fn token_id(&self) -> String {
        self.token_id.clone()
    }
    fn sub_type(&self) -> String {
        String::from("TransferNft")
    }
}

impl NftTx for EntrustNft {
    fn token_id(&self) -> String {
        self.token_id.clone()
    }
    fn sub_type(&self) -> String {
        String::from("EntrustNft")
    }
}

impl NftTx for DisposeEntrustedNft {
    fn token_id(&self) -> String {
        self.token_id.clone()
    }
    fn sub_type(&self) -> String {
        String::from("DisposeEntrustedNft")
    }
}

#[async_trait]
pub trait Job {
    async fn update_free_balance(
        &self,
        info: &mut HashMap<String, Balance>,
        state_info: &mut HashMap<String, State>,
    ) -> HashSet<String>;
    async fn update_locked_balance(
        &self,
        info: &mut HashMap<String, Balance>,
        state_info: &mut HashMap<String, State>,
    ) -> HashSet<String>;
    fn update_nft_owner_info(&self) -> Option<(String, String, String)>;
    fn input_hashs(&self) -> HashSet<String>;
    fn is_free_fungible(&self) -> bool;
    fn is_locked_fungible(&self) -> bool;
}

#[async_trait]
impl Job for TransactionWithResult {
    fn is_free_fungible(&self) -> bool {
        match &self.signed_tx.value {
            Transaction::RewardTx(tx) => matches!(
                tx,
                RewardTx::OfferReward(_)
                    | RewardTx::ExecuteOwnershipReward(_)
                    | RewardTx::ExecuteReward(_)
            ),
            Transaction::TokenTx(tx) => matches!(
                tx,
                TokenTx::TransferFungibleToken(_)
                    | TokenTx::MintFungibleToken(_)
                    | TokenTx::DisposeEntrustedFungibleToken(_)
                    | TokenTx::EntrustFungibleToken(_)
                    | TokenTx::BurnFungibleToken(_)
            ),
            _ => false,
        }
    }

    fn is_locked_fungible(&self) -> bool {
        match &self.signed_tx.value {
            Transaction::TokenTx(tx) => matches!(
                tx,
                TokenTx::EntrustFungibleToken(_) | TokenTx::DisposeEntrustedFungibleToken(_)
            ),
            _ => false,
        }
    }

    fn update_nft_owner_info(&self) -> Option<(String, String, String)> {
        if let Transaction::TokenTx(tx) = &self.signed_tx.value {
            match tx {
                TokenTx::MintNft(tx) => Some((
                    tx.token_id.clone(),
                    tx.output.clone(),
                    tx.created_at.clone(),
                )),
                TokenTx::TransferNft(tx) => Some((
                    tx.token_id.clone(),
                    tx.output.clone(),
                    tx.created_at.clone(),
                )),
                _ => None,
            }
        } else {
            None
        }
    }

    fn input_hashs(&self) -> HashSet<String> {
        match self.signed_tx.value.clone() {
            Transaction::RewardTx(tx) => match tx {
                RewardTx::OfferReward(t) => t.inputs,
                RewardTx::ExecuteOwnershipReward(t) => t.inputs,
                RewardTx::ExecuteReward(_) => HashSet::new(),
                _ => HashSet::new(),
            },
            Transaction::TokenTx(tx) => match tx {
                TokenTx::TransferFungibleToken(t) => t.inputs,
                TokenTx::DisposeEntrustedFungibleToken(t) => t.inputs,
                TokenTx::EntrustFungibleToken(t) => t.inputs,
                TokenTx::BurnFungibleToken(t) => t.inputs,
                _ => HashSet::new(),
            },
            _ => HashSet::new(),
        }
        .into_iter()
        .collect()
    }

    async fn update_locked_balance(
        &self,
        info: &mut HashMap<String, Balance>,
        state_info: &mut HashMap<String, State>,
    ) -> HashSet<String> {
        let mut updated_accounts = HashSet::new();
        match &self.signed_tx.value {
            Transaction::TokenTx(tx) => match tx {
                TokenTx::EntrustFungibleToken(t) => {
                    let from_account = &self.signed_tx.sig.account;
                    info.get_mut(from_account).map(|b| b.add_locked(&t.amount));
                    let entry = info
                        .get_key_value(from_account)
                        .map(|(k, v)| (k.clone(), v.locked()))
                        .unwrap();
                    LockedBalanceStore::insert0(state_info, entry);
                    updated_accounts.insert(from_account.clone());
                }
                TokenTx::DisposeEntrustedFungibleToken(t) => {
                    // Dispose locked balance
                    let unspent_inputs = t
                        .inputs
                        .iter()
                        .filter(|input_hash| !LockedBalanceStore::contains(input_hash));
                    for input_hash in unspent_inputs {
                        let input_tx = Finder::transaction_with_result(&input_hash).await;
                        if let Transaction::TokenTx(TokenTx::EntrustFungibleToken(entrust)) =
                            input_tx.signed_tx.value
                        {
                            let input_signer = input_tx.signed_tx.sig.account;
                            info.get_mut(&input_signer)
                                .map(|b| b.sub_locked(&entrust.amount));
                            let entry = info
                                .get_key_value(&input_signer)
                                .map(|(k, v)| (k.clone(), v.locked()))
                                .unwrap();
                            LockedBalanceStore::insert(state_info, entry, input_hash.clone());
                            updated_accounts.insert(input_signer);
                        }
                    }
                }
                _ => (),
            },
            _ => (),
        }
        updated_accounts
    }

    async fn update_free_balance(
        &self,
        info: &mut HashMap<String, Balance>,
        state_info: &mut HashMap<String, State>,
    ) -> HashSet<String> {
        // BurnFungibleToken 의 경우에는 해당이 안됨.
        fn deposit_to_accounts(
            outputs: &HashMap<String, BigDecimal>,
            info: &mut HashMap<String, Balance>,
        ) {
            outputs.iter().for_each(|(to_account, amount)| {
                let balance = info.entry(to_account.clone()).or_insert(Balance::default());
                balance.add_free(amount);
            })
        }

        fn extract_outputs_from_input_tx_for_withdraw(
            input_tx_with_res: TransactionWithResult,
            from_account: &String,
        ) -> HashMap<String, BigDecimal> {
            // withdraw from account
            // b: account's balance
            // d: deposit amount
            match input_tx_with_res.signed_tx.value {
                Transaction::RewardTx(rw) => match rw {
                    RewardTx::OfferReward(t) => t.outputs,
                    RewardTx::ExecuteReward(_) => match input_tx_with_res.result.unwrap() {
                        TransactionResult::ExecuteRewardResult(res) => res.outputs,
                        _ => panic!("invalid ExecuteRewardResult"),
                    },
                    RewardTx::ExecuteOwnershipReward(_) => {
                        match input_tx_with_res.result.unwrap() {
                            TransactionResult::ExecuteOwnershipRewardResult(res) => res.outputs,
                            _ => panic!("invalid ExecuteOwnershipRewardResult"),
                        }
                    }
                    _ => panic!(),
                },
                Transaction::TokenTx(tk) => match tk {
                    TokenTx::MintFungibleToken(t) => t.outputs,
                    TokenTx::TransferFungibleToken(t) => t.outputs,
                    TokenTx::DisposeEntrustedFungibleToken(t) => t.outputs,
                    TokenTx::BurnFungibleToken(_) => {
                        let output_amount = match input_tx_with_res.result.unwrap() {
                            TransactionResult::BurnFungibleTokenResult(res) => res.output_amount,
                            _ => panic!("invalid ExecuteOwnershipRewardResult"),
                        };
                        HashMap::from([(from_account.clone(), output_amount)])
                    }
                    TokenTx::EntrustFungibleToken(_) => {
                        let remainder = match input_tx_with_res.result.unwrap() {
                            TransactionResult::EntrustFungibleTokenResult(res) => res.remainder,
                            _ => panic!("invalid EntrustFungibleTokenResult"),
                        };
                        HashMap::from([(from_account.clone(), remainder)])
                    }
                    _ => panic!(),
                },
                _ => panic!(),
            }
        }

        let from_account = &self.signed_tx.sig.account;
        // extract transaction outputs and input hashs
        let latest_fungible_tx_opt = match self.signed_tx.value.clone() {
            Transaction::RewardTx(tx) => match tx {
                RewardTx::OfferReward(t) =>
                //
                {
                    Some((Some(t.outputs), t.inputs))
                }
                RewardTx::ExecuteOwnershipReward(t) => match self.result.clone().unwrap() {
                    TransactionResult::ExecuteOwnershipRewardResult(res) => {
                        Some((Some(res.outputs), t.inputs))
                    }
                    _ => None,
                },
                RewardTx::ExecuteReward(_) => match self.result.clone().unwrap() {
                    TransactionResult::ExecuteRewardResult(res) => {
                        Some((Some(res.outputs), HashSet::new()))
                    }
                    _ => None,
                },
                _ => None,
            },
            Transaction::TokenTx(tx) => match tx {
                TokenTx::TransferFungibleToken(t) => Some((Some(t.outputs), t.inputs)),
                TokenTx::MintFungibleToken(t) => Some((Some(t.outputs), HashSet::new())),
                TokenTx::DisposeEntrustedFungibleToken(t) => {
                    Some((Some(t.outputs), HashSet::new()))
                }
                TokenTx::EntrustFungibleToken(t) => {
                    let remainder = match (&self.result).as_ref().unwrap() {
                        TransactionResult::EntrustFungibleTokenResult(res) => &res.remainder,
                        _ => panic!("invalid BurnFungibleTokenResult"),
                    };
                    info.get_mut(from_account).map(|b| b.add_free(remainder));
                    Some((None, t.inputs))
                }
                TokenTx::BurnFungibleToken(t) => {
                    let output_amount = match (&self.result).as_ref().unwrap() {
                        TransactionResult::BurnFungibleTokenResult(res) => &res.output_amount,
                        _ => panic!("invalid BurnFungibleTokenResult"),
                    };
                    info.get_mut(from_account)
                        .map(|b| b.add_free(output_amount));
                    Some((None, t.inputs))
                }
                _ => None,
            },
            _ => None,
        };

        let mut updated_accounts = HashSet::new();
        if let Some((outputs_in_latest_opt, inputs_txs)) = latest_fungible_tx_opt {
            updated_accounts.insert(from_account.clone());

            // deposits to latest txs's outputs
            outputs_in_latest_opt.map(|outputs_in_latest| {
                deposit_to_accounts(&outputs_in_latest, info);
                for (to_account, _) in outputs_in_latest.iter() {
                    FreeBalanceStore::merge(
                        state_info,
                        info.get_key_value(to_account)
                            .map(|(k, v)| (k.clone(), v.free()))
                            .unwrap(),
                    );
                }
                updated_accounts.extend(
                    outputs_in_latest
                        .keys()
                        .cloned()
                        .collect::<HashSet<String>>(),
                );
            });

            let spent_txs = FreeBalanceStore::spent_hashs(&from_account);
            let unspent_txs = inputs_txs
                .iter()
                .filter(|input_tx| !spent_txs.contains(*input_tx));
            let mut withdraw_occured = false;
            for utxo_hash in unspent_txs {
                let input_tx_res = Finder::transaction_with_result(utxo_hash).await;
                let outputs_in_input_tx =
                    extract_outputs_from_input_tx_for_withdraw(input_tx_res, from_account);

                // withdraw from outputs
                match outputs_in_input_tx
            .get(from_account)
            .ok_or_else(|| println!("'{from_account}'가 input tx의 outputs에 존재하지 않습니다. Latest_tx - {:?}", 
                                      serde_json::to_string(&self).unwrap().replace("\\", "").replace("\n", "")))
            .and_then(|withdraw_val| {
              info.get_mut(from_account)
                  .ok_or_else(|| println!("'{from_account}'의 기존 balance 가 존재하지 않습니다. Latest_tx - {:?}", 
                                            serde_json::to_string(self).unwrap().replace("\\", "").replace("\n", "")))
                  .map(|b| {
                    b.sub_free(withdraw_val);
                  })
            }) {
              Ok(_) => withdraw_occured = true,
              Err(_) => (),
            }
            }

            if withdraw_occured {
                FreeBalanceStore::merge_with_inputs(
                    state_info,
                    info.get_key_value(from_account)
                        .map(|(k, v)| (k.clone(), v.free()))
                        .unwrap(),
                    spent_txs,
                    inputs_txs,
                );
            }
        };
        updated_accounts
    }
}

#[derive(Debug, Clone)]
pub enum AdditionalEntity {
    CreateAccount(Vec<account_entity::ActiveModel>),
    CreateNftFile(Vec<nft_file::ActiveModel>),
    NftTx(Vec<nft_tx::ActiveModel>),
}

#[derive(Hash, Eq, PartialEq)]
pub enum AdditionalEntityKey {
    CreateNftFile,
    NftTx,
    CreateAccount,
}

pub trait Extract: Send + Debug {}

#[async_trait]
pub trait ExtractEntity {
    async fn extract_additional_entity(
        &self,
        tx_entity: &tx_entity::ActiveModel,
        store: &mut HashMap<AdditionalEntityKey, AdditionalEntity>,
    );
    fn is_locked_fungible_tx(&self) -> bool;
}

#[async_trait]
impl ExtractEntity for Transaction {
    fn is_locked_fungible_tx(&self) -> bool {
        if let Transaction::TokenTx(
            TokenTx::EntrustFungibleToken(_) | TokenTx::DisposeEntrustedFungibleToken(_),
        ) = self
        {
            return true;
        }
        false
    }

    async fn extract_additional_entity(
        &self,
        tx_entity: &tx_entity::ActiveModel,
        store: &mut HashMap<AdditionalEntityKey, AdditionalEntity>,
    ) {
        match self {
            Transaction::AccountTx(tx) => match tx {
                AccountTx::CreateAccount(tx) => {
                    let account = account_entity::Model::from(tx);
                    match store.get_mut(&AdditionalEntityKey::CreateAccount) {
                        Some(v) => match v {
                            AdditionalEntity::CreateAccount(vec) => vec.push(account.clone()),
                            _ => {}
                        },
                        None => {
                            store.insert(
                                AdditionalEntityKey::CreateAccount,
                                AdditionalEntity::CreateAccount(vec![account]),
                            );
                        }
                    }
                }
                _ => {}
            },
            Transaction::TokenTx(tx) => match tx {
                TokenTx::MintNft(tx) => {
                    let nft_meta_info_opt =
                        ApiService::get_request_until(tx.data_url.clone(), 5).await;
                    let nft_file = nft_file::Model::from(tx, nft_meta_info_opt);
                    match store.get_mut(&AdditionalEntityKey::CreateNftFile) {
                        Some(v) => match v {
                            AdditionalEntity::CreateNftFile(vec) => vec.push(nft_file.clone()),
                            _ => (),
                        },
                        None => {
                            store.insert(
                                AdditionalEntityKey::CreateNftFile,
                                AdditionalEntity::CreateNftFile(vec![nft_file]),
                            );
                        }
                    };

                    let nft_tx = nft_tx::Model::from(tx, tx_entity);
                    match store.get_mut(&AdditionalEntityKey::NftTx) {
                        Some(v) => match v {
                            AdditionalEntity::NftTx(vec) => vec.push(nft_tx.clone()),
                            _ => (),
                        },
                        None => {
                            store.insert(
                                AdditionalEntityKey::NftTx,
                                AdditionalEntity::NftTx(vec![nft_tx]),
                            );
                        }
                    };
                }
                TokenTx::TransferNft(tx) => {
                    let nft_tx = nft_tx::Model::from(tx, tx_entity);
                    match store.get_mut(&AdditionalEntityKey::NftTx) {
                        Some(v) => match v {
                            AdditionalEntity::NftTx(vec) => vec.push(nft_tx.clone()),
                            _ => (),
                        },
                        None => {
                            store.insert(
                                AdditionalEntityKey::NftTx,
                                AdditionalEntity::NftTx(vec![nft_tx.clone()]),
                            );
                        }
                    };
                }
                TokenTx::EntrustNft(tx) => {
                    let nft_tx = nft_tx::Model::from(tx, tx_entity);
                    match store.get_mut(&AdditionalEntityKey::NftTx) {
                        Some(v) => match v {
                            AdditionalEntity::NftTx(vec) => vec.push(nft_tx.clone()),
                            _ => (),
                        },
                        None => {
                            store.insert(
                                AdditionalEntityKey::NftTx,
                                AdditionalEntity::NftTx(vec![nft_tx]),
                            );
                        }
                    };
                }
                TokenTx::DisposeEntrustedNft(tx) => {
                    let nft_tx = nft_tx::Model::from(tx, tx_entity);
                    match store.get_mut(&AdditionalEntityKey::NftTx) {
                        Some(v) => match v {
                            AdditionalEntity::NftTx(vec) => vec.push(nft_tx.clone()),
                            _ => (),
                        },
                        None => {
                            store.insert(
                                AdditionalEntityKey::NftTx,
                                AdditionalEntity::NftTx(vec![nft_tx]),
                            );
                        }
                    };
                }
                TokenTx::BurnNft(_) => {
                    // TODO: BurnNft 트랜잭션에서 token_id 추가 되어야 될듯.
                    // let nft_tx = nft_tx::Model::from(tx, tx_entity);
                    // match store.get_mut(&AdditionalEntity::NftTx) {
                    //   Some(v) => v.push(Box::new(nft_tx)),
                    //   None => { store.insert(AdditionalEntity::NftTx, vec![Box::new(nft_tx)]); },
                    // };
                }
                _ => {}
            },
            _ => {}
        }
    }
}
