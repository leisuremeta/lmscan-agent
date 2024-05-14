use sea_orm::Set;
use serde::{Deserialize, Serialize};

use crate::{
    library::common::{as_timestamp, now},
    tx_entity::ActiveModel,
};

use super::{common::Common, TransactionWithResult};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum GroupTx {
    AddAccounts(AddAccounts),
    CreateGroup(CreateGroup),
}

impl Common for GroupTx {
    fn created_at(&self) -> i64 {
        match self {
            GroupTx::AddAccounts(t) => t.created_at(),
            GroupTx::CreateGroup(t) => t.created_at(),
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
            GroupTx::AddAccounts(t) => {
                t.from(hash, block_hash, block_number, tx)
            }
            GroupTx::CreateGroup(t) => {
                t.from(hash, block_hash, block_number, tx)
            }
        }
    }
}

impl GroupTx {
    pub fn get_accounts(&self, signer: String) -> Vec<String> {
        match self {
            GroupTx::AddAccounts(tx) => {
                let mut v = tx.accounts.clone();
                v.push(signer);
                v
            }
            GroupTx::CreateGroup(tx) => vec![tx.coordinator.clone()],
        }
    }
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AddAccounts {
    pub created_at: String,
    pub group_id: String,
    pub accounts: Vec<String>,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateGroup {
    pub created_at: String,
    pub group_id: String,
    pub name: String,
    pub coordinator: String,
}

impl Common for AddAccounts {
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
            tx_type: Set("Group".to_string()),
            token_type: Set("LM".to_string()),
            sub_type: Set("AddAccounts".to_string()),
            block_hash: Set(block_hash),
            block_number: Set(block_number),
            event_time: Set(self.created_at()),
            created_at: Set(now()),
        }
    }
}

impl Common for CreateGroup {
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
            token_type: Set("LM".to_string()),
            tx_type: Set("Group".to_string()),
            sub_type: Set("CreateGroup".to_string()),
            block_hash: Set(block_hash),
            block_number: Set(block_number),
            event_time: Set(self.created_at()),
            created_at: Set(now()),
        }
    }
}
