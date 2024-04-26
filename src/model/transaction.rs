pub mod account_transaction;
mod agenda_transaction;
pub mod common;
mod group_transaction;
pub mod reward_transaction;
pub mod token_transaction;

use bigdecimal::BigDecimal;
use itertools::Itertools;
use core::panic;
use sea_orm::prelude::async_trait::async_trait;
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};
use std::fmt::Debug;
extern crate chrono;
use crate::service::finder_service::Finder;
use crate::store::free_balance::FreeBalanceStore;
use crate::store::locked_balance::LockedBalanceStore;
use crate::store::sled_store::SledStore;
use crate::store::wal::State;
use crate::tx_entity::{self, ActiveModel};
use crate::{account_entity, account_mapper, nft_tx};

use self::account_transaction::*;
use self::agenda_transaction::*;
use self::common::Common;
use self::group_transaction::*;
use self::reward_transaction::*;
use self::token_transaction::*;

use super::balance::Balance;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct TransactionWithResult {
    #[serde(rename = "signedTx")]
    pub signed_tx: SignedTx,
    pub result: Option<TransactionResult>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum TransactionResult {
    AddPublicKeySummariesResult { removed :HashMap<String, String> },
    BurnFungibleTokenResult {
        #[serde(rename = "outputAmount")]
        output_amount: BigDecimal,
    },
    EntrustFungibleTokenResult { remainder: BigDecimal },
    ExecuteRewardResult { outputs: HashMap<String, BigDecimal> },
    ExecuteOwnershipRewardResult { outputs: HashMap<String, BigDecimal> },
    VoteSimpleAgendaResult {
        #[serde(rename = "votingAmount")]
        voting_amount: BigDecimal,
    },
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
    RewardTx(RewardTx),
    TokenTx(TokenTx),
    AccountTx(AccountTx),
    GroupTx(GroupTx),
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

    fn from(
        &self,
        hash: String,
        block_hash: String,
        block_number: i64,
        tx: TransactionWithResult,
    ) -> ActiveModel {
        match self {
            Transaction::RewardTx(t) => {
                t.from(hash, block_hash, block_number, tx)
            }
            Transaction::TokenTx(t) => {
                t.from(hash, block_hash, block_number, tx)
            }
            Transaction::AccountTx(t) => {
                t.from(hash, block_hash, block_number, tx)
            }
            Transaction::GroupTx(t) => {
                t.from(hash, block_hash, block_number, tx)
            }
            Transaction::AgendaTx(t) => {
                t.from(hash, block_hash, block_number, tx)
            }
        }
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
                TokenTx::DisposeEntrustedNft(tx) => Some((
                    tx.token_id.clone(),
                    tx.output.clone().unwrap_or(String::new()),
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
                    match info
                        .get_key_value(from_account)
                        .map(|(k, v)| (k.clone(), v.locked())) {
                            Some(entry) => {
                                LockedBalanceStore::insert0(state_info, entry);
                                updated_accounts.insert(from_account.clone());
                            },
                            _ => (),
                        }
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
                            match info
                                .get_key_value(&input_signer)
                                .map(|(k, v)| (k.clone(), v.locked())) {
                                    Some(entry) => {
                                        LockedBalanceStore::insert(state_info, entry, input_hash.clone());
                                        updated_accounts.insert(input_signer);
                                    },
                                    _ => (),
                                }
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
                        TransactionResult::ExecuteRewardResult {outputs} => outputs,
                        _ => panic!("invalid ExecuteRewardResult"),
                    },
                    RewardTx::ExecuteOwnershipReward(_) => {
                        match input_tx_with_res.result.unwrap() {
                            TransactionResult::ExecuteOwnershipRewardResult{outputs} => outputs,
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
                            TransactionResult::BurnFungibleTokenResult { output_amount } => output_amount,
                            _ => panic!("invalid ExecuteOwnershipRewardResult"),
                        };
                        HashMap::from([(from_account.clone(), output_amount)])
                    }
                    TokenTx::EntrustFungibleToken(_) => {
                        let remainder = match input_tx_with_res.result.unwrap() {
                            TransactionResult::EntrustFungibleTokenResult { remainder } => remainder,
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
                    TransactionResult::ExecuteOwnershipRewardResult { outputs } => {
                        Some((Some(outputs), t.inputs))
                    }
                    _ => None,
                },
                RewardTx::ExecuteReward(_) => match self.result.clone().unwrap() {
                    TransactionResult::ExecuteRewardResult { outputs } => {
                        Some((Some(outputs), HashSet::new()))
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
                        TransactionResult::EntrustFungibleTokenResult { remainder } => remainder,
                        _ => panic!("invalid BurnFungibleTokenResult"),
                    };
                    info.get_mut(from_account).map(|b| b.add_free(remainder));
                    Some((None, t.inputs))
                }
                TokenTx::BurnFungibleToken(t) => {
                    let output_amount = match (&self.result).as_ref().unwrap() {
                        TransactionResult::BurnFungibleTokenResult { output_amount } => output_amount,
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

impl Transaction {
    pub fn get_nft_active_model(&self, tx_entity: &tx_entity::ActiveModel, from: String) -> Option<nft_tx::ActiveModel> {
        match self {
            Transaction::TokenTx(tx) => tx.get_nft_active_model(tx_entity, from),
            _ => None
        }
    }
    pub fn get_acc_active_model(&self) -> Option<account_entity::ActiveModel> {
        match self {
            Transaction::AccountTx(tx) => tx.get_acc_active_model(),
            _ => None
        }
    }
    pub fn get_account_mapper(&self, signer: String, hash: String, event_time: i64) -> Vec<account_mapper::Model> {
        let v = match self {
            Transaction::RewardTx(tx) => tx.get_accounts(signer.clone()),
            Transaction::TokenTx(tx) => tx.get_accounts(signer.clone()),
            Transaction::AccountTx(tx) => tx.get_accounts(),
            Transaction::GroupTx(tx) => tx.get_accounts(signer.clone()),
            Transaction::AgendaTx(_) => vec![signer],
        };
        v.into_iter().map(|account| account_mapper::Model {
            address: account,
            hash: hash.clone(),
            event_time: event_time,
        }).collect_vec()
    }
}
