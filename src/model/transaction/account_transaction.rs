use std::collections::HashMap;

use sea_orm::Set;
use serde::{Deserialize, Serialize};

use crate::{
    library::common::{as_timestamp, now},
    tx_entity::ActiveModel,
};

use super::{common::Common, TransactionWithResult};

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

impl Common for AddPublicKeySummaries {
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
            tx_type: Set("Account".to_string()),
            token_type: Set("LM".to_string()),
            sub_type: Set("AddPublicKeySummaries".to_string()),
            from_addr: Set(from_account),
            to_addr: Set(vec![self.account.to_owned()]),
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
            tx_type: Set("Account".to_string()),
            token_type: Set("LM".to_string()),
            sub_type: Set("CreateAccount".to_string()),
            from_addr: Set(from_account),
            to_addr: Set(vec![self.account.to_owned()]),
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
            tx_type: Set("Account".to_string()),
            token_type: Set("LM".to_string()),
            from_addr: Set(from_account),
            sub_type: Set("UpdateAccount".to_string()),
            to_addr: Set(vec![self.account.to_owned().clone()]),
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
            AccountTx::AddPublicKeySummaries(t) => {
                t.from(hash, from_account, block_hash, block_number, json, tx)
            }
            AccountTx::CreateAccount(t) => {
                t.from(hash, from_account, block_hash, block_number, json, tx)
            }
            AccountTx::UpdateAccount(t) => {
                t.from(hash, from_account, block_hash, block_number, json, tx)
            }
        }
    }
}
