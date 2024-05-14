use std::collections::HashMap;

use sea_orm::Set;
use serde::{Deserialize, Serialize};

use crate::{
    account_entity, library::common::{as_timestamp, now}, tx_entity::ActiveModel
};

use super::{common::Common, TransactionWithResult};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum AccountTx {
    AddPublicKeySummaries(AddPublicKeySummaries),
    CreateAccount(CreateAccount),
    UpdateAccount(UpdateAccount),
}

impl Common for AccountTx {
    fn created_at(&self) -> i64 {
        match self {
            AccountTx::AddPublicKeySummaries(t) => t.created_at(),
            AccountTx::CreateAccount(t) => t.created_at(),
            AccountTx::UpdateAccount(t) => t.created_at(),
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
            AccountTx::AddPublicKeySummaries(t) => {
                t.from(hash, block_hash, block_number, tx)
            }
            AccountTx::CreateAccount(t) => {
                t.from(hash, block_hash, block_number, tx)
            }
            AccountTx::UpdateAccount(t) => {
                t.from(hash, block_hash, block_number, tx)
            }
        }
    }
}

impl AccountTx {
    pub fn get_acc_active_model(&self) -> Option<account_entity::ActiveModel> {
        match self {
            AccountTx::CreateAccount(tx) => Some(account_entity::Model::from(tx)),
            _ => None
        }
    }

    pub fn get_accounts(&self) -> Vec<String> {
        match self {
            AccountTx::CreateAccount(tx) => vec![tx.account.clone()],
            _ => vec![],
        }
    }
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AddPublicKeySummaries {
    pub created_at: String,
    pub account: String,
    pub summaries: HashMap<String, String>,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateAccount {
    pub created_at: String,
    pub account: String,
    pub eth_address: Option<String>,
    pub guardian: Option<String>,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UpdateAccount {
    pub created_at: String,
    pub account: String,
    pub eth_address: Option<String>,
    pub guardian: Option<String>,
}

impl Common for AddPublicKeySummaries {
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
            tx_type: Set("Account".to_string()),
            token_type: Set("LM".to_string()),
            sub_type: Set("AddPublicKeySummaries".to_string()),
            block_hash: Set(block_hash),
            block_number: Set(block_number),
            event_time: Set(self.created_at()),
            created_at: Set(now()),
        }
    }
}

impl Common for CreateAccount {
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
            tx_type: Set("Account".to_string()),
            token_type: Set("LM".to_string()),
            sub_type: Set("CreateAccount".to_string()),
            block_hash: Set(block_hash),
            block_number: Set(block_number),
            event_time: Set(self.created_at()),
            created_at: Set(now()),
        }
    }
}

impl Common for UpdateAccount {
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
            tx_type: Set("Account".to_string()),
            token_type: Set("LM".to_string()),
            sub_type: Set("UpdateAccount".to_string()),
            block_hash: Set(block_hash),
            block_number: Set(block_number),
            event_time: Set(self.created_at()),
            created_at: Set(now()),
        }
    }
}
