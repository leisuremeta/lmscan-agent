use sea_orm::Set;
use serde::{Deserialize, Serialize};

use crate::{
    library::common::{as_timestamp, now},
    tx_entity::ActiveModel,
};

use super::{common::Common, TransactionWithResult};

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

impl Common for AddAccounts {
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
            GroupTx::AddAccounts(t) => {
                t.from(hash, from_account, block_hash, block_number, json, tx)
            }
            GroupTx::CreateGroup(t) => {
                t.from(hash, from_account, block_hash, block_number, json, tx)
            }
        }
    }
}
