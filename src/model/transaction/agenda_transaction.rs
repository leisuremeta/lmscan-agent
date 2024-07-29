use std::collections::HashMap;

use sea_orm::Set;
use serde::{Deserialize, Serialize};

use crate::{
    library::common::{as_timestamp, now},
    tx_entity::ActiveModel,
};

use super::{common::Common, TransactionWithResult};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum AgendaTx {
    SuggestSimpleAgenda(SuggestSimpleAgenda),
    VoteSimpleAgenda(VoteSimpleAgenda),
}

impl Common for AgendaTx {
    fn created_at(&self) -> i64 {
        match self {
            AgendaTx::SuggestSimpleAgenda(t) => t.created_at(),
            AgendaTx::VoteSimpleAgenda(t) => t.created_at(),
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
            AgendaTx::SuggestSimpleAgenda(t) => {
                t.from(hash, block_hash, block_number, tx)
            }
            AgendaTx::VoteSimpleAgenda(t) => {
                t.from(hash, block_hash, block_number, tx)
            }
        }
    }
}

impl AgendaTx {
    pub fn get_accounts(&self) -> Vec<String> {
        // only signed account
        match self {
            AgendaTx::SuggestSimpleAgenda(_) => todo!(),
            AgendaTx::VoteSimpleAgenda(_) => todo!(),
        }
    }
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SuggestSimpleAgenda {
    pub created_at: String,
    pub title: String,
    pub voting_token: String,
    pub vote_start: String,
    pub vote_end: String,
    pub vote_options: HashMap<String, String>,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct VoteSimpleAgenda {
    pub created_at: String,
    pub agenda_tx_hash: String,
    pub selected_option: String,
}

impl Common for SuggestSimpleAgenda {
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
            token_type: Set("".to_string()),
            tx_type: Set("Agenda".to_string()),
            sub_type: Set("SuggestSimpleAgenda".to_string()),
            block_hash: Set(block_hash),
            block_number: Set(block_number),
            event_time: Set(self.created_at()),
            created_at: Set(now()),
        }
    }
}

impl Common for VoteSimpleAgenda {
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
            token_type: Set("".to_string()),
            tx_type: Set("Agenda".to_string()),
            sub_type: Set("VoteSimpleAgenda".to_string()),
            block_hash: Set(block_hash),
            block_number: Set(block_number),
            event_time: Set(self.created_at()),
            created_at: Set(now()),
        }
    }
}
