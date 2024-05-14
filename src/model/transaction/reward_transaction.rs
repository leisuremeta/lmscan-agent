use std::collections::{HashMap, HashSet};

use bigdecimal::BigDecimal;
use sea_orm::Set;
use serde::{Deserialize, Serialize};

use crate::{
    library::common::{as_timestamp, now},
    tx_entity::ActiveModel,
};

use super::{common::Common, TransactionWithResult};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum RewardTx {
    RecordActivity(RecordActivity),
    RegisterDao(RegisterDao),
    UpdateDao(UpdateDao),
    OfferReward(OfferReward),
    ExecuteReward(ExecuteReward),
    ExecuteOwnershipReward(ExecuteOwnershipReward),
    BuildSnapshot(BuildSnapshot),
}

impl Common for RewardTx {
    fn created_at(&self) -> i64 {
        match self {
            RewardTx::RecordActivity(t) => t.created_at(),
            RewardTx::RegisterDao(t) => t.created_at(),
            RewardTx::UpdateDao(t) => t.created_at(),
            RewardTx::OfferReward(t) => t.created_at(),
            RewardTx::ExecuteReward(t) => t.created_at(),
            RewardTx::ExecuteOwnershipReward(t) => t.created_at(),
            RewardTx::BuildSnapshot(t) => t.created_at(),
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
            RewardTx::RecordActivity(t) => t.from(hash, block_hash, block_number, tx),
            RewardTx::RegisterDao(t) => t.from(hash, block_hash, block_number, tx),
            RewardTx::UpdateDao(t) => t.from(hash, block_hash, block_number, tx),
            RewardTx::OfferReward(t) => t.from(hash, block_hash, block_number, tx),
            RewardTx::ExecuteReward(t) => t.from(hash, block_hash, block_number, tx),
            RewardTx::ExecuteOwnershipReward(t) => t.from(hash, block_hash, block_number, tx),
            RewardTx::BuildSnapshot(t) => t.from(hash, block_hash, block_number, tx),
        }
    }
}

impl RewardTx {
    pub fn get_accounts(&self, signer: String) -> Vec<String> {
        let mut v = match self {
            RewardTx::RecordActivity(tx) => tx.user_activity.clone().into_keys().collect(),
            RewardTx::RegisterDao(tx) => {
                let mut v = tx.moderators.clone();
                v.push(tx.dao_account_name.clone());
                v
            }
            RewardTx::UpdateDao(tx) => tx.moderators.clone(),
            RewardTx::OfferReward(tx) => tx.outputs.clone().into_keys().collect(),
            RewardTx::ExecuteReward(tx) => match tx.dao_account.clone() {
                Some(v) => vec![v],
                None => vec![],
            },
            _ => vec![]
        };
        v.push(signer);
        v
    }
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct DaoActivity {
    pub point: i64,
    pub description: String,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RecordActivity {
    pub created_at: String,
    pub timestamp: String,
    pub user_activity: HashMap<String, Vec<DaoActivity>>,
    pub token_received: HashMap<String, Vec<DaoActivity>>,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RegisterDao {
    pub created_at: String,
    pub group_id: String,
    pub dao_account_name: String,
    pub moderators: Vec<String>,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct OfferReward {
    pub created_at: String,
    pub token_definition_id: String,
    pub inputs: HashSet<String>,
    pub outputs: HashMap<String, BigDecimal>,
    pub memo: Option<String>,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UpdateDao {
    created_at: String,
    group_id: String,
    moderators: Vec<String>,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ExecuteReward {
    pub created_at: String,
    pub dao_account: Option<String>,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ExecuteOwnershipReward {
    pub created_at: String,
    pub definition_id: String,
    pub inputs: HashSet<String>,
    pub targets: Vec<String>,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BuildSnapshot {
    pub created_at: String,
    pub timestamp: String,
    pub account_amount: BigDecimal,
    pub token_amount: BigDecimal,
    pub ownership_amount: BigDecimal,
}

impl Common for RecordActivity {
    fn created_at(&self) -> i64 {
        as_timestamp(self.created_at.as_str())
    }
    fn from(
        &self,
        hash: String,
        block_hash: String,
        block_number: i64,
        txr: TransactionWithResult,
    ) -> ActiveModel {
        ActiveModel {
            hash: Set(hash),
            signer: Set(txr.signed_tx.sig.account.clone()),
            tx_type: Set("Reward".to_string()),
            token_type: Set("LM".to_string()),
            sub_type: Set("RecordActivity".to_string()),
            block_hash: Set(block_hash),
            block_number: Set(block_number),
            event_time: Set(self.created_at()),
            created_at: Set(now()),
        }
    }
}

impl Common for RegisterDao {
    fn created_at(&self) -> i64 {
        as_timestamp(self.created_at.as_str())
    }
    fn from(
        &self,
        hash: String,
        block_hash: String,
        block_number: i64,
        txr: TransactionWithResult,
    ) -> ActiveModel {
        ActiveModel {
            hash: Set(hash),
            signer: Set(txr.signed_tx.sig.account.clone()),
            tx_type: Set("Reward".to_string()),
            token_type: Set("LM".to_string()),
            sub_type: Set("RegisterDao".to_string()),
            block_hash: Set(block_hash),
            block_number: Set(block_number),
            event_time: Set(self.created_at()),
            created_at: Set(now()),
        }
    }
}

impl Common for UpdateDao {
    fn created_at(&self) -> i64 {
        as_timestamp(self.created_at.as_str())
    }
    fn from(
        &self,
        hash: String,
        block_hash: String,
        block_number: i64,
        txr: TransactionWithResult,
    ) -> ActiveModel {
        ActiveModel {
            hash: Set(hash),
            signer: Set(txr.signed_tx.sig.account.clone()),
            tx_type: Set("Reward".to_string()),
            token_type: Set("LM".to_string()),
            sub_type: Set("UpdateDao".to_string()),
            block_hash: Set(block_hash),
            block_number: Set(block_number),
            event_time: Set(self.created_at()),
            created_at: Set(now()),
        }
    }
}

impl Common for OfferReward {
    fn created_at(&self) -> i64 {
        as_timestamp(self.created_at.as_str())
    }
    fn from(
        &self,
        hash: String,
        block_hash: String,
        block_number: i64,
        txr: TransactionWithResult,
    ) -> ActiveModel {
        ActiveModel {
            hash: Set(hash),
            signer: Set(txr.signed_tx.sig.account.clone()),
            tx_type: Set("Reward".to_string()),
            token_type: Set("LM".to_string()),
            sub_type: Set("OfferReward".to_string()),
            block_hash: Set(block_hash),
            block_number: Set(block_number),
            event_time: Set(self.created_at()),
            created_at: Set(now()),
        }
    }
}

impl Common for ExecuteReward {
    fn created_at(&self) -> i64 {
        as_timestamp(self.created_at.as_str())
    }
    fn from(
        &self,
        hash: String,
        block_hash: String,
        block_number: i64,
        txr: TransactionWithResult,
    ) -> ActiveModel {
        ActiveModel {
            hash: Set(hash),
            signer: Set(txr.signed_tx.sig.account.clone()),
            tx_type: Set("Reward".to_string()),
            token_type: Set("LM".to_string()),
            sub_type: Set("ExecuteReward".to_string()),
            block_hash: Set(block_hash),
            block_number: Set(block_number),
            event_time: Set(self.created_at()),
            created_at: Set(now()),
        }
    }
}

impl Common for ExecuteOwnershipReward {
    fn created_at(&self) -> i64 {
        as_timestamp(self.created_at.as_str())
    }

    fn from(
        &self,
        hash: String,
        block_hash: String,
        block_number: i64,
        txr: TransactionWithResult,
    ) -> ActiveModel {
        ActiveModel {
            hash: Set(hash),
            signer: Set(txr.signed_tx.sig.account.clone()),
            tx_type: Set("Reward".to_string()),
            token_type: Set("LM".to_string()),
            sub_type: Set("ExecuteOwnershipReward".to_string()),
            block_hash: Set(block_hash),
            block_number: Set(block_number),
            event_time: Set(self.created_at()),
            created_at: Set(now()),
        }
    }
}

impl Common for BuildSnapshot {
    fn created_at(&self) -> i64 {
        as_timestamp(self.created_at.as_str())
    }

    fn from(
        &self,
        hash: String,
        block_hash: String,
        block_number: i64,
        txr: TransactionWithResult,
    ) -> ActiveModel {
        ActiveModel {
            hash: Set(hash),
            signer: Set(txr.signed_tx.sig.account.clone()),
            tx_type: Set("Reward".to_string()),
            token_type: Set("LM".to_string()),
            sub_type: Set("BuildSnapshot".to_string()),
            block_hash: Set(block_hash),
            block_number: Set(block_number),
            event_time: Set(self.created_at()),
            created_at: Set(now()),
        }
    }
}
