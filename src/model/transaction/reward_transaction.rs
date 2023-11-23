use std::collections::{HashMap, HashSet};

use bigdecimal::BigDecimal;
use sea_orm::Set;
use serde::{Deserialize, Serialize};

use crate::{
    library::common::{as_timestamp, as_vec, from_rawvalue_to_bigdecimal_map, now},
    tx_entity::ActiveModel,
};

use super::{common::Common, TransactionResult, TransactionWithResult};

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
    #[serde(rename = "ExecuteOwnershipReward")]
    ExecuteOwnershipReward(ExecuteOwnershipReward),
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct DaoActivity {
    pub point: i64,
    pub description: String,
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
pub struct OfferReward {
    #[serde(rename = "networkId")]
    pub network_id: i64,
    #[serde(rename = "createdAt")]
    pub created_at: String,
    #[serde(rename = "tokenDefinitionId")]
    pub token_definition_id: String,
    pub inputs: HashSet<String>,
    #[serde(deserialize_with = "from_rawvalue_to_bigdecimal_map")]
    pub outputs: HashMap<String, BigDecimal>,
    pub memo: Option<String>,
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
pub struct ExecuteOwnershipReward {
    #[serde(rename = "networkId")]
    pub network_id: i64,
    #[serde(rename = "createdAt")]
    pub created_at: String,
    #[serde(rename = "tokenDefinitionId")]
    pub definition_id: String,
    #[serde(rename = "inputs")]
    pub inputs: HashSet<String>,
    #[serde(rename = "targets")]
    pub targets: Vec<String>,
}

impl Common for RecordActivity {
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
            tx_type: Set("Reward".to_string()),
            token_type: Set("LM".to_string()),
            sub_type: Set("RegisterDao".to_string()),
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

impl Common for UpdateDao {
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
        let to_accounts: Vec<String> = self.outputs.keys().map(|s| s.to_string()).collect();
        let output_vals = self
            .outputs
            .iter()
            .map(|(k, v)| k.to_owned() + "/" + &v.to_string())
            .collect();
        ActiveModel {
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
            input_hashs: Set(Some(as_vec(self.inputs.clone()))),
            output_vals: Set(Some(output_vals)),
            json: Set(json),
        }
    }
}

impl Common for ExecuteReward {
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
        let (to_accounts, output_vals) = match tx.result {
            Option::Some(tx_res) => match tx_res {
                TransactionResult::ExecuteRewardResult(res) => (
                    res.outputs
                        .keys()
                        .into_iter()
                        .map(|to| to.to_string())
                        .collect(),
                    Some(
                        res.outputs
                            .into_iter()
                            .map(|(k, v)| k + "/" + &v.to_string())
                            .collect(),
                    ),
                ),
                _ => (vec![], None),
            },
            None => (vec![], None),
        };

        ActiveModel {
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

impl Common for ExecuteOwnershipReward {
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
        let (to_accounts, output_vals) = match tx.result {
            Option::Some(tx_res) => match tx_res {
                TransactionResult::ExecuteOwnershipRewardResult(res) => (
                    res.outputs
                        .keys()
                        .into_iter()
                        .map(|to| to.to_string())
                        .collect(),
                    Some(
                        res.outputs
                            .into_iter()
                            .map(|(k, v)| k + "/" + &v.to_string())
                            .collect(),
                    ),
                ),
                _ => (vec![], None),
            },
            None => (vec![], None),
        };

        ActiveModel {
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
            input_hashs: Set(Some(self.inputs.clone().into_iter().collect())),
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
            RewardTx::ExecuteOwnershipReward(t) => t.created_at(),
        }
    }
    fn network_id(&self) -> i64 {
        match self {
            RewardTx::RecordActivity(t) => t.network_id,
            RewardTx::RegisterDao(t) => t.network_id,
            RewardTx::UpdateDao(t) => t.network_id,
            RewardTx::OfferReward(t) => t.network_id,
            RewardTx::ExecuteReward(t) => t.network_id,
            RewardTx::ExecuteOwnershipReward(t) => t.network_id,
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
            RewardTx::RecordActivity(t) => {
                t.from(hash, from_account, block_hash, block_number, json, tx)
            }
            RewardTx::RegisterDao(t) => {
                t.from(hash, from_account, block_hash, block_number, json, tx)
            }
            RewardTx::UpdateDao(t) => {
                t.from(hash, from_account, block_hash, block_number, json, tx)
            }
            RewardTx::OfferReward(t) => {
                t.from(hash, from_account, block_hash, block_number, json, tx)
            }
            RewardTx::ExecuteReward(t) => {
                t.from(hash, from_account, block_hash, block_number, json, tx)
            }
            RewardTx::ExecuteOwnershipReward(t) => {
                t.from(hash, from_account, block_hash, block_number, json, tx)
            }
        }
    }
}
