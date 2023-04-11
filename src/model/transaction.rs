use bigdecimal::BigDecimal;
use bigdecimal::num_bigint::BigInt;
use sea_orm::Set;
use sea_orm::prelude::async_trait::async_trait;
use serde::{Deserialize, Serialize};
use std::collections::{HashSet, HashMap};
use std::fmt::Debug;
extern crate chrono;


use crate::library::common::{as_timestamp, now, get_request_until};
use crate::{nft_file, nft_tx, account_entity};
use crate::tx_entity::{ActiveModel as TxModel, self};


impl TransactionWithResult {
  pub fn from(json: &str) -> Option<TransactionWithResult>{
    match serde_json::from_str::<TransactionWithResult>(json) {
      Ok(tx_res) => Some(tx_res),
      Err(err) => {println!("{err}"); panic!()},
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
  #[serde(rename = "VoteSimpleAgendaResult")]
  VoteSimpleAgendaResult(VoteSimpleAgendaResult),
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct AddPublicKeySummariesResult {
  pub removed: HashMap<String, String>
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct BurnFungibleTokenResult {
  #[serde(rename = "outputAmount")]
  pub output_amount: BigDecimal
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct EntrustFungibleTokenResult {
  #[serde(rename = "remainder")]
  pub remainder: BigDecimal
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ExecuteRewardResult {
  #[serde(rename = "outputs")]
  pub outputs: HashMap<String, BigDecimal>
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct VoteSimpleAgendaResult {
  #[serde(rename = "votingAmount")]
  pub voting_amount: BigDecimal
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
  AgendaTx(AgendaTx)
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum RewardTx {
  #[serde(rename = "RecordActivity")]
  RecordActivity(RecordActivity),
  #[serde(rename = "RegisterDao")]
  RegisterDao(RegisterDao),
  #[serde(rename = "UpdateDao")]
  UpdateDao(UpdateDao),
  #[serde(rename = "OfferReward")]
  OfferReward(OfferReward),
  #[serde(rename = "ExecuteReward")]
  ExecuteReward(ExecuteReward),
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct RecordActivity {
  #[serde(rename = "networkId")]
  pub network_id: i64,
  #[serde(rename = "createdAt")]
  pub created_at: String,
  pub timestamp: String,
  #[serde(rename = "userActivity")]
  pub user_activity: HashMap<String, Vec<DaoActivity>>,
  #[serde(rename = "tokenReceived")]
  pub token_received: HashMap<String, Vec<DaoActivity>>,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct DaoActivity {
    pub point: i64,
    pub description: String,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ExecuteReward {
  #[serde(rename = "networkId")]
  pub network_id: i64,
  #[serde(rename = "createdAt")]
  pub created_at: String,
  #[serde(rename = "daoAccount")]
  pub dao_account: Option<String>,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct OfferReward {
  #[serde(rename = "networkId")]
  pub network_id: i64,
  #[serde(rename = "createdAt")]
  pub created_at: String,
  #[serde(rename = "tokenDefinitionId")]
  pub token_definition_id: String,
  pub inputs: Vec<String>,
  pub outputs: HashMap<String, BigDecimal>,
  pub memo: Option<String>,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct RegisterDao {
  #[serde(rename = "networkId")]
  pub network_id: i64,
  #[serde(rename = "createdAt")]
  pub created_at: String,
  #[serde(rename = "groupId")]
  pub group_id: String,
  #[serde(rename = "daoAccountName")]
  pub dao_account_name: String,
  pub moderators: Vec<String>,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct UpdateDao {
  #[serde(rename = "networkId")]
  network_id: i64,
  #[serde(rename = "createdAt")]
  created_at: String,
  #[serde(rename = "groupId")]
  group_id: String,
  moderators: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum TokenTx {
  #[serde(rename = "BurnNFT")]
  BurnNft(BurnNft),
  #[serde(rename = "EntrustNFT")]
  EntrustNft(EntrustNft),
  #[serde(rename = "EntrustFungibleToken")]
  EntrustFungibleToken(EntrustFungibleToken),
  #[serde(rename = "BurnFungibleToken")]
  BurnFungibleToken(BurnFungibleToken),
  #[serde(rename = "TransferNFT")]
  TransferNft(TransferNft),
  #[serde(rename = "TransferFungibleToken")]
  TransferFungibleToken(TransferFungibleToken),
  #[serde(rename = "MintNFT")]
  MintNft(MintNft),
  #[serde(rename = "MintFungibleToken")]
  MintFungibleToken(MintFungibleToken),
  #[serde(rename = "DefineToken")]
  DefineToken(DefineToken),
  #[serde(rename = "DisposeEntrustedNFT")]
  DisposeEntrustedNft(DisposeEntrustedNft),
  #[serde(rename = "DisposeEntrustedFungibleToken")]
  DisposeEntrustedFungibleToken(DisposeEntrustedFungibleToken),
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct BurnNft {
  #[serde(rename = "networkId")]
  pub network_id: i64,
  #[serde(rename = "createdAt")]
  pub created_at: String,
  #[serde(rename = "definitionId")]
  pub definition_id: String,
  pub input: String,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct EntrustNft {
  #[serde(rename = "networkId")]
  pub network_id: i64,
  #[serde(rename = "createdAt")]
  pub created_at: String,
  #[serde(rename = "definitionId")]
  pub definition_id: String,
  #[serde(rename = "tokenId")]
  pub token_id: String,
  pub input: String,
  pub to: String,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct EntrustFungibleToken {
  #[serde(rename = "networkId")]
  pub network_id: i64,
  #[serde(rename = "createdAt")]
  pub created_at: String,
  #[serde(rename = "definitionId")]
  pub definition_id: String,
  pub amount: BigDecimal,
  pub inputs: Vec<String>,
  pub to: String,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct BurnFungibleToken {
  #[serde(rename = "networkId")]
  pub network_id: i64,
  #[serde(rename = "createdAt")]
  pub created_at: String,
  #[serde(rename = "definitionId")]
  pub definition_id: String,
  pub amount: BigDecimal,
  pub inputs: Vec<String>,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct TransferNft {
  #[serde(rename = "networkId")]
  pub network_id: i64,
  #[serde(rename = "createdAt")]
  pub created_at: String,
  #[serde(rename = "definitionId")]
  pub definition_id: String,
  #[serde(rename = "tokenId")]
  pub token_id: String,
  pub input: String,
  pub output: String,
  pub memo: Option<String>,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct TransferFungibleToken {
  #[serde(rename = "networkId")]
  pub network_id: i64,
  #[serde(rename = "createdAt")]
  pub created_at: String,
  #[serde(rename = "tokenDefinitionId")]
  pub token_definition_id: String,
  pub inputs: Vec<String>,
  pub outputs: HashMap<String, BigDecimal>,
  pub memo: Option<String>,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize, Eq, Hash)]
pub struct MintNft {
  #[serde(rename = "networkId")]
  pub network_id: i64,
  #[serde(rename = "createdAt")]
  pub created_at: String,
  #[serde(rename = "tokenDefinitionId")]
  pub token_definition_id: String,
  #[serde(rename = "tokenId")]
  pub token_id: String,
  pub rarity: String,
  #[serde(rename = "dataUrl")]
  pub data_url: String,
  #[serde(rename = "contentHash")]
  pub content_hash: String,
  pub output: String,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct MintFungibleToken {
  #[serde(rename = "networkId")]
  pub network_id: i64,
  #[serde(rename = "createdAt")]
  pub created_at: String,
  #[serde(rename = "definitionId")]
  pub definition_id: String,
  pub outputs: HashMap<String, BigDecimal>,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct DefineToken {
  #[serde(rename = "networkId")]
  pub network_id: i64,
  #[serde(rename = "createdAt")]
  pub created_at: String,
  #[serde(rename = "definitionId")]
  pub definition_id: String,
  pub name: String,
  pub symbol: Option<String>,
  #[serde(rename = "minterGroup")]
  pub minter_group: Option<String>,
  #[serde(rename = "nftInfo")]
  pub nft_info: Option<NftInfo>,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct NftInfo {
  pub some: Option<Some>,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct Some {
  #[serde(rename = "value")]
  pub value: Value,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct Value {
  #[serde(rename = "minter")]
  pub minter: String,
  #[serde(rename = "rarity")]
  pub rarity: Rarity,
  #[serde(rename = "dataUrl")]
  pub data_url: String,
  #[serde(rename = "contentHash")]
  pub content_hash: String,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Rarity {
  #[serde(rename = "LGDY")]
  pub lgdy: Option<i64>,
  #[serde(rename = "UNIQ")]
  pub uniq: Option<i64>,
  #[serde(rename = "EPIC")]
  pub epic: Option<i64>,
  #[serde(rename = "RARE")]
  pub rare: Option<i64>,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct DisposeEntrustedNft {
  #[serde(rename = "networkId")]
  pub network_id: i64,
  #[serde(rename = "createdAt")]
  pub created_at: String,
  #[serde(rename = "definitionId")]
  pub definition_id: String,
  #[serde(rename = "tokenId")]
  pub token_id: String,
  pub input: String,
  pub output: Option<String>,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct DisposeEntrustedFungibleToken {
  #[serde(rename = "networkId")]
  pub network_id: i64,
  #[serde(rename = "createdAt")]
  pub created_at: String,
  #[serde(rename = "definitionId")]
  pub definition_id: String,
  pub inputs: Vec<String>,
  pub outputs: HashMap<String, BigDecimal>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum AccountTx {
  #[serde(rename = "AddPublicKeySummaries")]
  AddPublicKeySummaries(AddPublicKeySummaries),
  #[serde(rename = "CreateAccount")]
  CreateAccount(CreateAccount),
  #[serde(rename = "UpdateAccount")]
  UpdateAccount(UpdateAccount),
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct AddPublicKeySummaries {
  #[serde(rename = "networkId")]
  pub network_id: i64,
  #[serde(rename = "createdAt")]
  pub created_at: String,
  pub account: String,
  pub summaries: HashMap<String, String>,
}


#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CreateAccount {
  #[serde(rename = "networkId")]
  pub network_id: i64,
  #[serde(rename = "createdAt")]
  pub created_at: String,
  pub account: String,
  #[serde(rename = "ethAddress")]
  pub eth_address: Option<String>,
  pub guardian: Option<String>,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct UpdateAccount {
  #[serde(rename = "networkId")]
  pub network_id: i64,
  #[serde(rename = "createdAt")]
  pub created_at: String,
  pub account: String,
  #[serde(rename = "ethAddress")]
  pub eth_address: Option<String>,
  pub guardian: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum GroupTx {
  #[serde(rename = "AddAccounts")]
  AddAccounts(AddAccounts),
  #[serde(rename = "CreateGroup")]
  CreateGroup(CreateGroup),
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct AddAccounts {
  #[serde(rename = "networkId")]
  pub network_id: i64,
  #[serde(rename = "createdAt")]
  pub created_at: String,
  #[serde(rename = "groupId")]
  pub group_id: String,
  pub accounts: Vec<String>,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CreateGroup {
  #[serde(rename = "networkId")]
  pub network_id: i64,
  #[serde(rename = "createdAt")]
  pub created_at: String,
  #[serde(rename = "groupId")]
  pub group_id: String,
  pub name: String,
  pub coordinator: String,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum AgendaTx {
  #[serde(rename = "SuggestSimpleAgenda")]
  SuggestSimpleAgenda(SuggestSimpleAgenda),
  #[serde(rename = "VoteSimpleAgenda")]
  VoteSimpleAgenda(VoteSimpleAgenda),
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SuggestSimpleAgenda {
  #[serde(rename = "networkId")]
  pub network_id: i64,
  #[serde(rename = "createdAt")]
  pub created_at: String,
  pub title: String,
  #[serde(rename = "votingToken")]
  pub voting_token: String,
  #[serde(rename = "voteStart")]
  pub vote_start: String,
  #[serde(rename = "voteEnd")]
  pub vote_end: String,
  #[serde(rename = "voteOptions")]
  pub vote_options: HashMap<String, String>,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct VoteSimpleAgenda {
  #[serde(rename = "networkId")]
  pub network_id: i64,
  #[serde(rename = "createdAt")]
  pub created_at: String,
  #[serde(rename = "agendaTxHash")]
  agenda_tx_hash: String,
  #[serde(rename = "selectedOption")]
  selected_option: String,
}

pub trait Common {
  fn created_at(&self) -> i64;
  fn network_id(&self) -> i64;
  fn from(&self, hash: String, from_account: String, block_hash: String, block_number: i64, json: String, tx_res_opt: Option<TransactionResult>) -> TxModel;
}


impl Common for RecordActivity {
  fn created_at(&self) -> i64 { as_timestamp(self.created_at.as_str()) }
  fn network_id(&self) -> i64 { self.network_id }
  fn from(&self, hash: String, from_account: String, block_hash: String, block_number: i64, json: String, tx_res_opt: Option<TransactionResult>) -> TxModel {
    TxModel {
      hash: Set(hash),
      tx_type: Set("Reward".to_string()),
      token_type: Set("LM".to_string()),
      sub_type: Set("RecordActivity".to_string()),
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

impl Common for RegisterDao {
  fn created_at(&self) -> i64 { as_timestamp(self.created_at.as_str()) }
  fn network_id(&self) -> i64 { self.network_id }
  fn from(&self, hash: String, fromAccount: String, block_hash: String, block_number: i64, json: String, tx_res_opt: Option<TransactionResult>) -> TxModel {
    TxModel {
      hash: Set(hash),
      tx_type: Set("Reward".to_string()),
      token_type: Set("LM".to_string()),
      sub_type: Set("RegisterDao".to_string()),
      from_addr: Set(fromAccount),
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

impl Common for UpdateDao {
  fn created_at(&self) -> i64 { as_timestamp(self.created_at.as_str()) }
  fn network_id(&self) -> i64 { self.network_id }
  fn from(&self, hash: String, from_account: String, block_hash: String, block_number: i64, json: String, tx_res_opt: Option<TransactionResult>) -> TxModel {
    TxModel {
      hash: Set(hash),
      tx_type: Set("Reward".to_string()),
      token_type: Set("LM".to_string()),
      sub_type: Set("UpdateDao".to_string()),
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

impl Common for OfferReward {
  fn created_at(&self) -> i64 { as_timestamp(self.created_at.as_str()) }
  fn network_id(&self) -> i64 { self.network_id }
  fn from(&self, hash: String, from_account: String, block_hash: String, block_number: i64, json: String, tx_res_opt: Option<TransactionResult>) -> TxModel {
    let to_accounts: Vec<String> = self.outputs.keys().cloned().collect();
    let output_vals = self.outputs.iter().map(|(k, v)| k.to_owned() + "/" + &v.to_string()).collect();
    TxModel {
      hash: Set(hash),
      tx_type: Set("Reward".to_string()),
      token_type: Set("LM".to_string()),
      sub_type: Set("OfferReward".to_string()),
      from_addr: Set(from_account),
      to_addr: Set(to_accounts),
      block_hash: Set(block_hash),
      block_number: Set(block_number),
      event_time: Set(self.created_at()),
      created_at: Set(now()),
      input_hashs: Set(None),
      output_vals: Set(Some(output_vals)),
      json: Set(json),
    }
  }
}

impl Common for ExecuteReward {
  fn created_at(&self) -> i64 { as_timestamp(self.created_at.as_str()) }
  fn network_id(&self) -> i64 { self.network_id }
  fn from(&self, hash: String, from_account: String, block_hash: String, block_number: i64, json: String, tx_res_opt: Option<TransactionResult>) -> TxModel {

    let mut to_accounts = vec![];
    let output_vals: Option<Vec<String>> = match tx_res_opt {
        Option::Some(tx_res) => 
          match tx_res {
            TransactionResult::ExecuteRewardResult(res) => {
              to_accounts = res.outputs.keys().into_iter().cloned().collect();
              Some(res.outputs.into_iter().map(|(k, v)| {k + "/" + &v.to_string()}).collect())
            }
            _ => None,
        },
        None => None,
    };

    TxModel {
      hash: Set(hash),
      tx_type: Set("Reward".to_string()),
      token_type: Set("LM".to_string()),
      sub_type: Set("ExecuteReward".to_string()),
      from_addr: Set(from_account),
      to_addr: Set(to_accounts),
      block_hash: Set(block_hash),
      block_number: Set(block_number),
      event_time: Set(self.created_at()),
      created_at: Set(now()),
      input_hashs: Set(None),
      output_vals: Set(output_vals),
      json: Set(json),
    }
  }
}

impl Common for RewardTx {
  fn created_at(&self) -> i64 {
    match self {
      RewardTx::RecordActivity(t) => t.created_at(),
      RewardTx::RegisterDao(t) => t.created_at(),
      RewardTx::UpdateDao(t) => t.created_at(),
      RewardTx::OfferReward(t) => t.created_at(),
      RewardTx::ExecuteReward(t) => t.created_at(),
    }
  }
  fn network_id(&self) -> i64 {
    match self {
      RewardTx::RecordActivity(t) => t.network_id,
      RewardTx::RegisterDao(t) => t.network_id,
      RewardTx::UpdateDao(t) => t.network_id,
      RewardTx::OfferReward(t) => t.network_id,
      RewardTx::ExecuteReward(t) => t.network_id,
    }
  }
  fn from(&self, hash: String, from_account: String, block_hash: String, block_number: i64, json: String, tx_res_opt: Option<TransactionResult>) -> TxModel {
    match self {
      RewardTx::RecordActivity(t) => t.from(hash, from_account, block_hash, block_number, json, tx_res_opt),
      RewardTx::RegisterDao(t) => t.from(hash, from_account, block_hash, block_number, json, tx_res_opt),
      RewardTx::UpdateDao(t) => t.from(hash, from_account, block_hash, block_number, json, tx_res_opt),
      RewardTx::OfferReward(t) => t.from(hash, from_account, block_hash, block_number, json, tx_res_opt),
      RewardTx::ExecuteReward(t) => t.from(hash, from_account, block_hash, block_number, json, tx_res_opt),
    }
  }
}


impl Common for EntrustNft {
  fn created_at(&self) -> i64 { as_timestamp(self.created_at.as_str()) }
  fn network_id(&self) -> i64 { self.network_id }
  fn from(&self, hash: String, from_account: String, block_hash: String, block_number: i64, json: String, tx_res_opt: Option<TransactionResult>) -> TxModel {
    TxModel {
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
  fn created_at(&self) -> i64 { as_timestamp(self.created_at.as_str()) }
  fn network_id(&self) -> i64 { self.network_id }
  fn from(&self, hash: String, from_account: String, block_hash: String, block_number: i64, json: String, tx_res_opt: Option<TransactionResult>) -> TxModel {
    TxModel {
      hash: Set(hash),
      tx_type: Set("Token".to_string()),
      token_type: Set("LM".to_string()),
      sub_type: Set("EntrustFungibleToken".to_string()),
      from_addr: Set(from_account),
      to_addr: Set(vec![self.to.clone()]),
      block_hash: Set(block_hash),
      block_number: Set(block_number),
      event_time: Set(self.created_at()),
      created_at: Set(now()),
      input_hashs: Set(Some(self.inputs.clone())),
      output_vals: Set(Some(vec![self.to.to_owned() + "/" + &self.amount.to_string()])),
      json: Set(json),
    }
  }
}

impl Common for BurnFungibleToken {
  fn created_at(&self) -> i64 { as_timestamp(self.created_at.as_str()) }
  fn network_id(&self) -> i64 { self.network_id }
  fn from(&self, hash: String, from_account: String, block_hash: String, block_number: i64, json: String, tx_res_opt: Option<TransactionResult>) -> TxModel {
    TxModel {
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
      input_hashs: Set(Some(self.inputs.clone())),
      output_vals: Set(Some(vec![from_account + "/" + &self.amount.to_string()])),
      json: Set(json),
    }
  }
}

impl Common for TransferNft {
  fn created_at(&self) -> i64 { as_timestamp(self.created_at.as_str()) }
  fn network_id(&self) -> i64 { self.network_id }
  fn from(&self, hash: String, from_account: String, block_hash: String, block_number: i64, json: String, tx_res_opt: Option<TransactionResult>) -> TxModel {
    TxModel {
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
  fn created_at(&self) -> i64 { as_timestamp(self.created_at.as_str()) }
  fn network_id(&self) -> i64 { self.network_id }
  fn from(&self, hash: String, from_account: String, block_hash: String, block_number: i64, json: String, tx_res_opt: Option<TransactionResult>) -> TxModel {

    let mut to_accounts = vec![];
    let output_vals: Option<Vec<String>> = match tx_res_opt {
        Option::Some(tx_res) => 
          match tx_res {
            TransactionResult::ExecuteRewardResult(res) => {
              to_accounts = res.outputs.keys().into_iter().cloned().collect();
              Some(res.outputs.into_iter().map(|(k, v)| {k + "/" + &v.to_string()}).collect())
            }
            _ => None,
        },
        None => None,
    };

    TxModel {
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
      input_hashs: Set(Some(self.inputs.clone())),
      output_vals: Set(output_vals),
      json: Set(json),
    }
  }
}

impl Common for MintNft {
  fn created_at(&self) -> i64 { as_timestamp(self.created_at.as_str()) }
  fn network_id(&self) -> i64 { self.network_id }
  fn from(&self, hash: String, from_account: String, block_hash: String, block_number: i64, json: String, tx_res_opt: Option<TransactionResult>) -> TxModel {
    let to_addr = self.output.clone();
    TxModel {
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
      output_vals: Set(Some(vec![to_addr+"/"+&self.token_id])),
      json: Set(json),
    }
  }
}

impl Common for MintFungibleToken {
  fn created_at(&self) -> i64 { as_timestamp(self.created_at.as_str()) }
  fn network_id(&self) -> i64 { self.network_id }
  fn from(&self, hash: String, from_account: String, block_hash: String, block_number: i64, json: String, tx_res_opt: Option<TransactionResult>) -> TxModel {
    TxModel {
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
  fn created_at(&self) -> i64 { as_timestamp(self.created_at.as_str()) }
  fn network_id(&self) -> i64 { self.network_id }
  fn from(&self, hash: String, from_account: String, block_hash: String, block_number: i64, json: String, tx_res_opt: Option<TransactionResult>) -> TxModel {
    TxModel {
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
  fn created_at(&self) -> i64 { as_timestamp(self.created_at.as_str()) }
  fn network_id(&self) -> i64 { self.network_id }
  fn from(&self, hash: String, from_account: String, block_hash: String, block_number: i64, json: String, tx_res_opt: Option<TransactionResult>) -> TxModel {
    let to_account = match &self.output {
        Option::Some(value) => value.clone(),
        None => String::from(""),
    };

    TxModel {
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
      input_hashs: Set(None),
      output_vals: Set(Some(vec![to_account+"/"+&self.token_id])),
      json: Set(json),
    }
  }
}

impl Common for DisposeEntrustedFungibleToken {
  fn created_at(&self) -> i64 { as_timestamp(self.created_at.as_str()) }
  fn network_id(&self) -> i64 { self.network_id }
  fn from(&self, hash: String, from_account: String, block_hash: String, block_number: i64, json: String, tx_res_opt: Option<TransactionResult>) -> TxModel {
    let to_accounts = (&self.outputs).keys().cloned().collect();
    let output_vals: Vec<String> = (&self.outputs).into_iter().map(|(k, v)| k.to_owned() + "/" + &v.to_string()).collect();
    TxModel {
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
      input_hashs: Set(Some(self.inputs.clone())),
      output_vals: Set(Some(output_vals)),
      json: Set(json),
    }
  }
}

impl Common for BurnNft {
  fn created_at(&self) -> i64 { as_timestamp(self.created_at.as_str()) }
  fn network_id(&self) -> i64 { self.network_id }
  fn from(&self, hash: String, from_account: String, block_hash: String, block_number: i64, json: String, tx_res_opt: Option<TransactionResult>) -> TxModel {
    TxModel {
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

  fn from(&self, hash: String, from_account: String, block_hash: String, block_number: i64, json: String, tx_res_opt: Option<TransactionResult>) -> TxModel {
    match self {
      TokenTx::BurnNft(t) => t.from(hash, from_account, block_hash, block_number, json, tx_res_opt),
      TokenTx::EntrustNft(t) => t.from(hash, from_account, block_hash, block_number, json, tx_res_opt),
      TokenTx::EntrustFungibleToken(t) => t.from(hash, from_account, block_hash, block_number, json, tx_res_opt),
      TokenTx::TransferNft(t) => t.from(hash, from_account, block_hash, block_number, json, tx_res_opt),
      TokenTx::TransferFungibleToken(t) => t.from(hash, from_account, block_hash, block_number, json, tx_res_opt),
      TokenTx::MintNft(t) => t.from(hash, from_account, block_hash, block_number, json, tx_res_opt),
      TokenTx::MintFungibleToken(t) => t.from(hash, from_account, block_hash, block_number, json, tx_res_opt),
      TokenTx::DefineToken(t) => t.from(hash, from_account, block_hash, block_number, json, tx_res_opt),
      TokenTx::DisposeEntrustedNft(t) => t.from(hash, from_account, block_hash, block_number, json, tx_res_opt),
      TokenTx::DisposeEntrustedFungibleToken(t) => t.from(hash, from_account, block_hash, block_number, json, tx_res_opt),
      TokenTx::BurnFungibleToken(t) => t.from(hash, from_account, block_hash, block_number, json, tx_res_opt),
    }
  }
}

impl Common for AddPublicKeySummaries {
  fn created_at(&self) -> i64 { as_timestamp(self.created_at.as_str()) }
  fn network_id(&self) -> i64 { self.network_id }

  fn from(&self, hash: String, from_account: String, block_hash: String, block_number: i64, json: String, tx_res_opt: Option<TransactionResult>) -> TxModel {
    TxModel {
      hash: Set(hash),
      tx_type: Set("Account".to_string()),
      token_type: Set("LM".to_string()),
      sub_type: Set("AddPublicKeySummaries".to_string()),
      from_addr: Set(from_account),
      to_addr: Set(vec![self.account.clone()]),
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

impl Common for CreateAccount {
  fn created_at(&self) -> i64 { as_timestamp(self.created_at.as_str()) }
  fn network_id(&self) -> i64 { self.network_id }
  fn from(&self, hash: String, from_account: String, block_hash: String, block_number: i64, json: String, tx_res_opt: Option<TransactionResult>) -> TxModel {
    TxModel {
      hash: Set(hash),
      tx_type: Set("Account".to_string()),
      token_type: Set("LM".to_string()),
      sub_type: Set("CreateAccount".to_string()),
      from_addr: Set(from_account),
      to_addr: Set(vec![self.account.clone()]),
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

impl Common for UpdateAccount {
  fn created_at(&self) -> i64 { as_timestamp(self.created_at.as_str()) }
  fn network_id(&self) -> i64 { self.network_id }
  fn from(&self, hash: String, from_account: String, block_hash: String, block_number: i64, json: String, tx_res_opt: Option<TransactionResult>) -> TxModel {
    TxModel {
      hash: Set(hash),
      tx_type: Set("Account".to_string()),
      token_type: Set("LM".to_string()),
      from_addr: Set(from_account),
      sub_type: Set("UpdateAccount".to_string()),
      to_addr: Set(vec![self.account.clone()]),
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

impl Common for AccountTx {
  fn created_at(&self) -> i64 {
    match self {
      AccountTx::AddPublicKeySummaries(t) => t.created_at(),
      AccountTx::CreateAccount(t) => t.created_at(),
      AccountTx::UpdateAccount(t) => t.created_at(),
    }
  }

  fn network_id(&self) -> i64 {
    match self {
      AccountTx::AddPublicKeySummaries(t) => t.network_id,
      AccountTx::CreateAccount(t) => t.network_id,
      AccountTx::UpdateAccount(t) => t.network_id,
    }
  }

  fn from(&self, hash: String, from_account: String, block_hash: String, block_number: i64, json: String, tx_res_opt: Option<TransactionResult>) -> TxModel {
    match self {
      AccountTx::AddPublicKeySummaries(t) => t.from(hash, from_account, block_hash, block_number, json, tx_res_opt),
      AccountTx::CreateAccount(t) => t.from(hash, from_account, block_hash, block_number, json, tx_res_opt),
      AccountTx::UpdateAccount(t) => t.from(hash, from_account, block_hash, block_number, json, tx_res_opt),
    }
  }
}

impl Common for AddAccounts {
  fn created_at(&self) -> i64 { as_timestamp(self.created_at.as_str()) }
  fn network_id(&self) -> i64 { self.network_id }

  fn from(&self, hash: String, from_account: String, block_hash: String, block_number: i64, json: String, tx_res_opt: Option<TransactionResult>) -> TxModel {
    TxModel {
      hash: Set(hash),
      tx_type: Set("Group".to_string()),
      token_type: Set("LM".to_string()),
      sub_type: Set("AddAccounts".to_string()),
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

impl Common for CreateGroup {
  fn created_at(&self) -> i64 { as_timestamp(self.created_at.as_str()) }
  fn network_id(&self) -> i64 { self.network_id }
  fn from(&self, hash: String, from_account: String, block_hash: String, block_number: i64, json: String, tx_res_opt: Option<TransactionResult>) -> TxModel {
    TxModel {
      hash: Set(hash),
      token_type: Set("LM".to_string()),
      tx_type: Set("Group".to_string()),
      sub_type: Set("CreateGroup".to_string()),
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

impl Common for SuggestSimpleAgenda {
  fn created_at(&self) -> i64 { as_timestamp(self.created_at.as_str()) }
  fn network_id(&self) -> i64 { self.network_id }
  fn from(&self, hash: String, from_account: String, block_hash: String, block_number: i64, json: String, tx_res_opt: Option<TransactionResult>) -> TxModel {
    TxModel {
      hash: Set(hash),
      token_type: Set("".to_string()),
      tx_type: Set("Agenda".to_string()),
      sub_type: Set("SuggestSimpleAgenda".to_string()),
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


impl Common for VoteSimpleAgenda {
  fn created_at(&self) -> i64 { as_timestamp(self.created_at.as_str()) }
  fn network_id(&self) -> i64 { self.network_id }
  fn from(&self, hash: String, from_account: String, block_hash: String, block_number: i64, json: String, tx_res_opt: Option<TransactionResult>) -> TxModel {
    TxModel {
      hash: Set(hash),
      token_type: Set("".to_string()),
      tx_type: Set("Agenda".to_string()),
      sub_type: Set("VoteSimpleAgenda".to_string()),
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


impl Common for GroupTx {
  fn created_at(&self) -> i64 {
    match self {
      GroupTx::AddAccounts(t) => t.created_at(),
      GroupTx::CreateGroup(t) => t.created_at(),
    }
  }

  fn network_id(&self) -> i64 {
    match self {
      GroupTx::AddAccounts(t) => t.network_id(),
      GroupTx::CreateGroup(t) => t.network_id(),
    }
  }

  fn from(&self, hash: String, from_account: String, block_hash: String, block_number: i64, json: String, tx_res_opt: Option<TransactionResult>) -> TxModel {
    match self {
      GroupTx::AddAccounts(t) => t.from(hash, from_account, block_hash, block_number, json, tx_res_opt),
      GroupTx::CreateGroup(t) => t.from(hash, from_account, block_hash, block_number, json, tx_res_opt),
    }
  }
}

impl Common for AgendaTx {
  fn created_at(&self) -> i64 {
    match self {
      AgendaTx::SuggestSimpleAgenda(t) => t.created_at(),
      AgendaTx::VoteSimpleAgenda(t) => t.created_at(),
    }
  }

  fn network_id(&self) -> i64 {
    match self {
      AgendaTx::SuggestSimpleAgenda(t) => t.network_id(),
      AgendaTx::VoteSimpleAgenda(t) => t.network_id(),
    }
  }

  fn from(&self, hash: String, from_account: String, block_hash: String, block_number: i64, json: String, tx_res_opt: Option<TransactionResult>) -> TxModel {
    match self {
      AgendaTx::SuggestSimpleAgenda(t) => t.from(hash, from_account, block_hash, block_number, json, tx_res_opt),
      AgendaTx::VoteSimpleAgenda(t) => t.from(hash, from_account, block_hash, block_number, json, tx_res_opt),
    }
  }
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

  fn from(&self, hash: String, from_account: String, block_hash: String, block_number: i64, json: String, tx_res_opt: Option<TransactionResult>) -> TxModel {
    match self {
      Transaction::RewardTx(t) => t.from(hash, from_account, block_hash, block_number, json, tx_res_opt),
      Transaction::TokenTx(t) => t.from(hash, from_account, block_hash, block_number, json, tx_res_opt),
      Transaction::AccountTx(t) => t.from(hash, from_account, block_hash, block_number, json, tx_res_opt),
      Transaction::GroupTx(t) => t.from(hash, from_account, block_hash, block_number, json, tx_res_opt),
      Transaction::AgendaTx(t) => t.from(hash, from_account, block_hash, block_number, json, tx_res_opt),
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
  pub nft_uri: String
}


pub trait Job {
  fn update_account_balance_info(&self, info: &mut HashMap<String, BigDecimal>) -> HashSet<String>;
  fn update_nft_owner_info(&self, nft_owner_info: &mut HashMap<String, String>) -> HashSet<String>;
}

impl Job for TransactionWithResult {
  fn update_nft_owner_info(&self, info: &mut HashMap<String, String>) -> HashSet<String> {
    let mut updated_accouts = HashSet::new();
    match &self.signed_tx.value {
      Transaction::TokenTx(tx) => match tx {
        TokenTx::TransferNft(tx) => {
          info.insert(tx.token_id.clone(), tx.output.clone());
          updated_accouts.insert(tx.token_id.clone());
        },
        _ => ()
      },
      _ => ()
    }
    updated_accouts
  }

  fn update_account_balance_info(&self, info: &mut HashMap<String, BigDecimal>) -> HashSet<String> {
    let mut updated_accouts = HashSet::new();
    let from_account = &self.signed_tx.sig.account;
    match &self.signed_tx.value {
        Transaction::RewardTx(tx) => match tx {
          RewardTx::OfferReward(t) => {
            // withdrawl from_account
            let sum: BigDecimal = t.outputs.values().sum();
            match info.get_mut(from_account) {
              Option::Some(value) => *value -= sum,
              None => {
                info.insert(from_account.to_owned(), -sum);
              },
            }
            updated_accouts.insert(from_account.clone());

            // deposit to_account
            for (to_account, new_value) in t.outputs.iter() {
              match info.get_mut(to_account) {
                Option::Some(value) => *value += new_value,
                None => {
                  info.insert(to_account.to_owned(), new_value.clone());
                },
              };
              updated_accouts.insert(to_account.clone());
            };
          },
        RewardTx::ExecuteReward(t) => {
          self.result.as_ref().map(|res| match res {
            TransactionResult::ExecuteRewardResult(res) => {
              // withdrawl from_account
              let sum: BigDecimal = res.outputs.values().sum();
              match info.get_mut(from_account) {
                Option::Some(value) => *value -= sum,
                None => {
                  info.insert(from_account.to_owned(), -sum);
                },
              }
              updated_accouts.insert(from_account.clone());

              // deposit to_account
              for (to_account, new_value) in res.outputs.iter() {
                match info.get_mut(to_account) {
                  Option::Some(value) => *value += new_value,
                  None => {
                    info.insert(to_account.to_owned(), new_value.clone());
                  },
                };
                updated_accouts.insert(to_account.clone());
              };
            },
            _ => (),
          });
        },
        _ => (),
      },
      Transaction::TokenTx(tx) => match tx {
        TokenTx::EntrustFungibleToken(t) => {
          // withdrawl from_account
          let sum: BigDecimal = t.amount.clone();
          match info.get_mut(from_account) {
            Option::Some(value) => *value -= sum,
            None => {
              info.insert(from_account.clone(), -sum);
            },
          }
          updated_accouts.insert(from_account.clone());

          // deposit to_account
          match info.get_mut(t.to.as_str()) {
            Option::Some(value) => *value += t.amount.clone(),
            None => {
              info.insert(t.to.clone(), t.amount.clone());
            },
          };
          updated_accouts.insert(t.to.clone());
        },
        TokenTx::TransferFungibleToken(t) => {
          // withdrawl from_account
          let sum: BigDecimal = t.outputs.values().sum();
          match info.get_mut(from_account) {
            Option::Some(value) => *value -= sum,
            None => {
              info.insert(from_account.to_owned(), -sum);
            },
          }
          updated_accouts.insert(from_account.clone());

          // deposit to_account
          for (to_account, new_value) in t.outputs.iter() {
            match info.get_mut(to_account) {
              Option::Some(value) => *value += new_value,
              None => {
                info.insert(to_account.to_owned(), new_value.clone());
              },
            };
            updated_accouts.insert(to_account.clone());
          };
        },
        TokenTx::MintFungibleToken(t) => {
          // withdrawl from_account
          let sum: BigDecimal = t.outputs.values().sum();
          match info.get_mut(from_account) {
            Option::Some(value) => *value -= sum,
            None => {
              info.insert(from_account.to_owned(), -sum);
            },
          }
          updated_accouts.insert(from_account.clone());

          // deposit to_account
          for (to_account, new_value) in t.outputs.iter() {
            match info.get_mut(to_account) {
              Option::Some(value) => *value += new_value,
              None => {
                info.insert(to_account.to_owned(), new_value.clone());
              },
            };
            updated_accouts.insert(to_account.clone());
          };
        },  
        TokenTx::DisposeEntrustedFungibleToken(t) => {
          // withdrawl from_account
          let sum: BigDecimal = t.outputs.values().sum();
          match info.get_mut(from_account) {
            Option::Some(value) => *value -= sum,
            None => {
              info.insert(from_account.to_owned(), -sum);
            },
          }
          updated_accouts.insert(from_account.clone());

          // deposit to_account
          for (to_account, new_value) in t.outputs.iter() {
            match info.get_mut(to_account) {
              Option::Some(value) => *value += new_value,
              None => {
                info.insert(to_account.to_owned(), new_value.clone());
              },
            };
            updated_accouts.insert(to_account.clone());
          };
        },  
        TokenTx::BurnFungibleToken(t) => {
          match info.get_mut(from_account) {
            Option::Some(value) => *value -= t.amount.clone(),
            None => { 
              info.insert(from_account.clone(), BigDecimal::from(0));
            },
          }
        },
        _ => (),
      },
      _ => ()
    };
    updated_accouts
  }

 
}




#[derive(Debug, Clone)]
pub enum AdditionalEntity {
  CreateAccount(Vec<account_entity::ActiveModel>),
  CreateNftFile(Vec<nft_file::ActiveModel>),
  UpdateNftFile(Vec<UpdateNftFile>),
  NftTx(Vec<nft_tx::ActiveModel>),
}

#[derive(Hash, Eq, PartialEq)]
pub enum AdditionalEntityKey {
  CreateNftFile, 
  UpdateNftFile,
  NftTx,
  CreateAccount,
}


pub trait Extract: Send + Debug  {}

#[async_trait]
pub trait ExtractEntity {
  async fn extract_additional_entity(&self, tx_entity: &tx_entity::ActiveModel, store: &mut HashMap<AdditionalEntityKey, AdditionalEntity>);
}

#[derive(Debug, Clone)]
pub struct UpdateNftFile {
  pub token_id: String,
  pub owner: String,
}

#[async_trait]
impl ExtractEntity for Transaction {
  async fn extract_additional_entity(&self, tx_entity: &tx_entity::ActiveModel, store: &mut HashMap<AdditionalEntityKey, AdditionalEntity>){
    match self {
      Transaction::AccountTx(tx) => match tx {
        AccountTx::CreateAccount(tx) => {
          let account = account_entity::Model::from(tx);
          match store.get_mut(&AdditionalEntityKey::CreateAccount) {
            Some(v) => match v {
              AdditionalEntity::CreateAccount(vec) => vec.push(account.clone()),
              _ => {},
            },
            None => { store.insert(AdditionalEntityKey::CreateAccount, AdditionalEntity::CreateAccount(vec![account])); }
          }
        },
        _ => {},
      },
      Transaction::TokenTx(tx) => match tx {
        TokenTx::MintNft(tx) => {
          let nft_meta_info_opt = get_request_until(tx.data_url.clone(), 5).await;
          let nft_file = nft_file::Model::from(tx, nft_meta_info_opt);
          match store.get_mut(&AdditionalEntityKey::CreateNftFile) {
            Some(v) => match v { 
              AdditionalEntity::CreateNftFile(vec) => vec.push(nft_file.clone()),
              _ => (),
            },
            None => { store.insert(AdditionalEntityKey::CreateNftFile, AdditionalEntity::CreateNftFile(vec![nft_file])); },
          };

          let nft_tx = nft_tx::Model::from(tx, tx_entity);
          match store.get_mut(&AdditionalEntityKey::NftTx) {
            Some(v) => match v {
              AdditionalEntity::NftTx(vec) => vec.push(nft_tx.clone()),
              _ => (),
            },
            None => { store.insert(AdditionalEntityKey::NftTx, AdditionalEntity::NftTx(vec![nft_tx])); },
          };
        },
        TokenTx::TransferNft(tx) => {
          let nft_tx = nft_tx::Model::from(tx, tx_entity);
          match store.get_mut(&AdditionalEntityKey::NftTx) {
            Some(v) => match v {
              AdditionalEntity::NftTx(vec) => vec.push(nft_tx.clone()),
              _ => (),
            },
            None => { store.insert(AdditionalEntityKey::NftTx, AdditionalEntity::NftTx(vec![nft_tx.clone()])); },
          };

          let upd_nft_file = UpdateNftFile { token_id: tx.token_id.clone(), owner: nft_tx.to_addr.clone().unwrap() };
          match store.get_mut(&AdditionalEntityKey::UpdateNftFile) {
            Some(v) => match v {
              AdditionalEntity::UpdateNftFile(vec) => vec.push(upd_nft_file.clone()),
              _ => (),
            },
            None => { store.insert(AdditionalEntityKey::UpdateNftFile, AdditionalEntity::UpdateNftFile(vec![upd_nft_file])); },
          };
        },
        TokenTx::EntrustNft(tx) => {
          let nft_tx = nft_tx::Model::from(tx, tx_entity);
          match store.get_mut(&AdditionalEntityKey::NftTx) {
            Some(v) => match v {
              AdditionalEntity::NftTx(vec) => vec.push(nft_tx.clone()),
              _ => (),
            },
            None => { store.insert(AdditionalEntityKey::NftTx, AdditionalEntity::NftTx(vec![nft_tx])); },
          };
        },
        TokenTx::DisposeEntrustedNft(tx) => {
          let nft_tx = nft_tx::Model::from(tx, tx_entity);
          match store.get_mut(&AdditionalEntityKey::NftTx) {
            Some(v) => match v {
              AdditionalEntity::NftTx(vec) => vec.push(nft_tx.clone()),
              _ => (),
            },
            None => { store.insert(AdditionalEntityKey::NftTx, AdditionalEntity::NftTx(vec![nft_tx])); },
          };
        },
        TokenTx::BurnNft(tx) => {
          // TODO: BurnNft 트랜잭션에서 token_id 추가 되어야 될듯.
          // let nft_tx = nft_tx::Model::from(tx, tx_entity);
          // match store.get_mut(&AdditionalEntity::NftTx) {
          //   Some(v) => v.push(Box::new(nft_tx)),
          //   None => { store.insert(AdditionalEntity::NftTx, vec![Box::new(nft_tx)]); },
          // };
        },
        _ => {},
      },
      _ => {}
    }
  }
}



pub fn test() {
  let json_err = r#"
    {
      "signedTx": {
        "sig": {
          "sig": {
            "v": 28,
            "r": "549091536c3628a6c7a7c95988cdd863a85bb2a8d42de28016bfb4523359252d",
            "s": "67750967063706d08b53f7079134207ded2b2f7039b01d50b96d551bf0a51bae"
          },
          "account": "playnomm"
        },
        "value": {
          "TokenTx": {
            "TransferNFT": {
              "networkId": 1000,
              "createdAt": "2023-02-24T05:10:42Z",
              "definitionId": "202302061200440725",
              "tokenId": "2023020612004400000000087",
              "input": "64c203e4019667ef8c26909c7d717f0b25bac405e8b44e57e5adf38853b0b67d",
              "output": "f9ff65d52bccb9c60f581f7bf5a61c364848b717",
              "memo": "Random Box Reveal"
            }
          }
        }
      },
      "result": null
    }
  "#;
  let json = r#"
  [
    {
      "sig" : {
        "sig" : {
          "v" : 28,
          "r" : "22c14ac6fbdce52c256640f1e36851ef901ea1b5cfebc3a430283a89df99bc11",
          "s" : "3474ebcc861c2d31a60d363356c4c89c196d450432b33bedadfb94d66edf2ffd"
        },
        "account" : "alice"
      },
      "value" : {
        "AccountTx" : {
          "UpdateAccount" : {
            "networkId" : 1000,
            "createdAt" : "2020-05-22T09:00:00Z",
            "account" : "alice",
            "ethAddress" : "0xefD277f6da7ac53e709392044AE98220Df142753",
            "guardian" : null
          }
        }
      }
    },
    {
      "sig" : {
        "sig" : {
          "v" : 28,
          "r" : "495c3bcc143eea328c11b7ec55069dd4fb16c26463999f9dbc085094c3b59423",
          "s" : "707a75e433abd208cfb76d4e0cdbc04b1ce2389e3a1f866348ef2e3ea5785e93"
        },
        "account" : "alice"
      },
      "value" : {
        "AccountTx" : {
          "CreateAccount" : {
            "networkId" : 1000,
            "createdAt" : "2020-05-22T09:00:00Z",
            "account" : "alice",
            "ethAddress" : null,
            "guardian" : null
          }
        }
      }
    },
    {
      "sig": {
        "sig": {
          "v": 27,
          "r": "816df20e4ff581fd2056689b48be73cca29e4f81977e5c42754e598757434c51",
          "s": "4e43aef8d836e79380067365cd7a4a452df5f52b73ec78463bdc7cdea2e11ca0"
        },
        "account": "alice"
      },
      "value": {
        "AccountTx": {
          "AddPublicKeySummaries": {
            "networkId": 1000,
            "createdAt": "2020-05-22T09:00:00Z",
            "account": "alice",
            "summaries": {
              "5b6ed47b96cd913eb938b81ee3ea9e7dc9affbff": "another key"
            }
          }
        }
      }
    },
    {
      "sig" : {
        "sig" : {
          "v" : 28,
          "r" : "aab6f7ccc108b8e75601c726d43270c1a60f38f830136dfe293a2633dc86a0dd",
          "s" : "3cc1b610df7a421f9ae560853d5f07005a20c6ad225a00861a76e5e91aa183c0"
        },
        "account" : "alice"
      },
      "value" : {
        "GroupTx" : {
          "CreateGroup" : {
            "networkId" : 1000,
            "createdAt" : "2022-06-08T09:00:00Z",
            "groupId" : "mint-group",
            "name" : "mint group",
            "coordinator" : "alice"
          }
        }
      }
    },
    {
      "sig" : {
        "sig" : {
          "v" : 28,
          "r" : "2dd00a2ebf07ff2d09d6e9bcd889ddc775c17989827e3e19b5e8d1744c021466",
          "s" : "05bd60fef3d45463e22e5c157c814a7cbd1681410b67b0233c97ce7116d60729"
        },
        "account" : "alice"
      },
      "value" : {
        "GroupTx" : {
          "AddAccounts" : {
            "networkId" : 1000,
            "createdAt" : "2022-06-08T09:00:00Z",
            "groupId" : "mint-group",
            "accounts" : [
              "alice",
              "bob"
            ]
          }
        }
      }
    },
    {
      "sig" : {
        "sig" : {
          "v" : 28,
          "r" : "ce2b48b7da96eef22a2b92170fb81865adb99cbcae99a2b81bb7ce9b4ba990b6",
          "s" : "35a708c9ffc1b7ef4e88389255f883c96e551a404afc4627e3f6ca32a617bae6"
        },
        "account" : "alice"
      },
      "value" : {
        "TokenTx" : {
          "DefineToken" : {
            "networkId" : 1000,
            "createdAt" : "2020-05-22T09:01:00Z",
            "definitionId" : "test-token",
            "name" : "test-token",
            "symbol" : "TT",
            "minterGroup" : "mint-group",
            "nftInfo" : {
              "Some" : {
                "value" : {
                  "minter" : "alice",
                  "rarity" : {
                    "LGDY" : 8,
                    "UNIQ" : 4,
                    "EPIC" : 2,
                    "RARE" : 1
                  },
                  "dataUrl" : "https://www.playnomm.com/data/test-token.json",
                  "contentHash" : "0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef"
                }
              }
            }
          }
        }
      }
    },
    {
      "sig" : {
        "sig" : {
          "v" : 28,
          "r" : "76fb1b3be81101638c9ce070628db035ad7d86d3363d664da0c5afe254494e90",
          "s" : "7ffb1c751fe4f5341c75341e4a51373139a7f730a56a08078ac89b6e1a77fc76"
        },
        "account" : "alice"
      },
      "value" : {
        "TokenTx" : {
          "MintFungibleToken" : {
            "networkId" : 1000,
            "createdAt" : "2020-05-22T09:01:00Z",
            "definitionId" : "test-token",
            "outputs" : {
              "alice" : 100
            }
          }
        }
      }
    },
    {
      "sig" : {
        "sig" : {
          "v" : 27,
          "r" : "0a914259cc0e8513512ea6356fc3056efe104e84756cf23a6c1c1aff7a580613",
          "s" : "71a15b331b9e7337a018b442ee978a15f0d86e71ca53d2f54a9a8ccb92646cf9"
        },
        "account" : "alice"
      },
      "value" : {
        "TokenTx" : {
          "MintNFT" : {
            "networkId" : 1000,
            "createdAt" : "2022-06-08T09:00:00Z",
            "tokenDefinitionId" : "test-token",
            "tokenId" : "2022061710000513118",
            "rarity" : "EPIC",
            "dataUrl" : "https://d3j8b1jkcxmuqq.cloudfront.net/temp/collections/TEST_NOMM4/NFT_ITEM/F7A92FB1-B29F-4E6F-BEF1-47C6A1376D68.jpg",
            "contentHash" : "0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef",
            "output" : "alice"
          }
        }
      }
    }
    ,
    {
      "sig" : {
        "sig" : {
          "v" : 28,
          "r" : "09a5f46d29bd8598f04cb6db32627aadd562e30e181135c2898594080db6aa79",
          "s" : "340abd1b6618d3bbf4b586294a4f902942f597672330563a43591a14be0a6504"
        },
        "account" : "alice"
      },
      "value" : {
        "TokenTx" : {
          "TransferFungibleToken" : {
            "networkId" : 1000,
            "createdAt" : "2022-06-09T09:00:00Z",
            "tokenDefinitionId" : "test-token",
            "inputs" : [
              "a3f35adb3d5d08692a7350e61aaa28da992a4280ad8e558953898ef96a0051ca"
            ],
            "outputs" : {
              "bob" : 10,
              "alice" : 90
            },
            "memo" : "transfer from alice to bob"
          }
        }
      }
    },
    {
      "sig" : {
        "sig" : {
          "v" : 27,
          "r" : "c443ed5eda3d484bcda7bf77f030d3f6c20e4130d9bc4e03ca75df3074b40239",
          "s" : "2e7a19f1baee2099ccbef500e7ceb03c5053957a55085ef52b21c022c43242d9"
        },
        "account" : "alice"
      },
      "value" : {
        "TokenTx" : {
          "TransferNFT" : {
            "networkId" : 1000,
            "createdAt" : "2022-06-09T09:00:00Z",
            "definitionId" : "test-token",
            "tokenId" : "2022061710000513118",
            "input" : "6040003b0020245ce82f352bed95dee2636442efee4e5a15ee3911c67910b657",
            "output" : "bob",
            "memo" : null
          }
        }
      }
    },
    {
      "sig" : {
        "sig" : {
          "v" : 27,
          "r" : "8d438670820bb788f0ef7106aa55c5fa2fa9c898eaded4d92f29d3c21a99c127",
          "s" : "1545783ca442a5ae2fdd347c79286a1c62256cd91ac76cb392f28dc190ac9c8a"
        },
        "account" : "alice"
      },
      "value" : {
        "TokenTx" : {
          "EntrustFungibleToken" : {
            "networkId" : 1000,
            "createdAt" : "2022-06-09T09:00:00Z",
            "definitionId" : "test-token",
            "amount" : 1000,
            "inputs" : [
              "a3f35adb3d5d08692a7350e61aaa28da992a4280ad8e558953898ef96a0051ca"
            ],
            "to" : "alice"
          }
        }
      }
    },
    {
      "sig" : {
        "sig" : {
          "v" : 27,
          "r" : "05705f380f7a7fbad853094f69ff1527703476be30d2ac19f90a24a7900100c0",
          "s" : "37fac4695829b188ebe3d8238259a212ba52588c4593a51ef81631ab9ab90581"
        },
        "account" : "alice"
      },
      "value" : {
        "TokenTx" : {
          "EntrustNFT" : {
            "networkId" : 1000,
            "createdAt" : "2020-06-09T09:00:00Z",
            "definitionId" : "test-token",
            "tokenId" : "2022061710000513118",
            "input" : "6040003b0020245ce82f352bed95dee2636442efee4e5a15ee3911c67910b657",
            "to" : "alice"
          }
        }
      }
    },
    {
      "sig" : {
        "sig" : {
          "v" : 28,
          "r" : "fb6c99c0e26da04e8dc0855ea629708a17a8deabfabb5a488ba9faa001c4a31f",
          "s" : "7de70d3fd15176451e46856af2dbedf05e58d7cfc0bfb0e0fac1b6d06550f5d3"
        },
        "account" : "alice"
      },
      "value" : {
        "TokenTx" : {
          "DisposeEntrustedFungibleToken" : {
            "networkId" : 1000,
            "createdAt" : "2020-06-10T09:00:00Z",
            "definitionId" : "test-token",
            "inputs" : [
              "45df6a88e74ea44f2d759251fed5a3c319e7cf9c37fafa7471418fec7b26acce"
            ],
            "outputs" : {
              "bob" : 1000
            }
          }
        }
      }
    },
    {
      "sig" : {
        "sig" : {
          "v" : 28,
          "r" : "a03080b98925010e241783482e83a5fdfc25343406564a4e3fc4e6b2535657d3",
          "s" : "1de0ede5ebeba4aea455094ac1b58fc24ad943f0a5422a93f60a4f2b8b59b982"
        },
        "account" : "alice"
      },
      "value" : {
        "TokenTx" : {
          "DisposeEntrustedNFT" : {
            "networkId" : 1000,
            "createdAt" : "2020-06-10T09:00:00Z",
            "definitionId" : "test-token",
            "tokenId" : "2022061710000513118",
            "input" : "10cb0802f3dfc85abb502bad260120a424fc583016db84d384904c1c0a580955",
            "output" : "bob"
          }
        }
      }
    },
    {
      "sig" : {
        "sig" : {
          "v" : 27,
          "r" : "d4b2d1cfe009e0e5b6dea67779fd898a7f1718e7b1869b5b36b6daacc68e88f6",
          "s" : "42d8c69e964109ceab5996abdbc59d53661904e6b56337599e9c5beebe665d51"
        },
        "account" : "alice"
      },
      "value" : {
        "RewardTx" : {
          "RegisterDao" : {
            "networkId" : 1000,
            "createdAt" : "2020-06-09T09:00:00Z",
            "groupId" : "sample-dao-group-id",
            "daoAccountName" : "sample-dao-group-account",
            "moderators" : [
              "alice"
            ]
          }
        }
      }
    },
    {
      "sig" : {
        "sig" : {
          "v" : 27,
          "r" : "95aff6586d03fa7c66165d9bb49f2a2fd54650f2776c728401c664622d5e2d4c",
          "s" : "2cff82c55822d3266add84ea5853dbc86cf47f24e5787080b76e58681477ba09"
        },
        "account" : "alice"
      },
      "value" : {
        "RewardTx" : {
          "RecordActivity" : {
            "networkId" : 2021,
            "createdAt" : "2023-01-10T18:01:00Z",
            "timestamp" : "2023-01-09T09:00:00Z",
            "userActivity" : {
              "bob" : [
                {
                  "point" : 3,
                  "description" : "like"
                }
              ],
              "carol" : [
                {
                  "point" : 3,
                  "description" : "like"
                }
              ]
            },
            "tokenReceived" : {
              "text-20230109-0000" : [
                {
                  "point" : 2,
                  "description" : "like"
                }
              ],
              "text-20230109-0001" : [
                {
                  "point" : 2,
                  "description" : "like"
                }
              ],
              "text-20230109-0002" : [
                {
                  "point" : 2,
                  "description" : "like"
                }
              ]
            }
          }
        }
      }
    }
  ]
  "#;

  // let profiles: Vec<TransactionWithResult> =  match serde_json::from_str::<Vec<TransactionWithResult>>(json) {
  //   Ok(seq) => {
  //     for x in seq.iter() {
  //       println!("{}", serde_json::to_string(x).unwrap());
  //     }
  //     seq
  //   },
  //   Err(e) => {
  //     println!("{}", e);
  //     todo!()
  //   },
  // };
  let profiles: TransactionWithResult =  match serde_json::from_str::<TransactionWithResult>(json_err) {
    Ok(res) => {
      println!("success: {}", serde_json::to_string(&res).unwrap());
      res
    },
    Err(e) => {
      println!("{}", e);
      todo!()
    },
  };
}
