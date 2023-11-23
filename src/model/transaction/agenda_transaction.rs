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
    pub agenda_tx_hash: String,
    #[serde(rename = "selectedOption")]
    pub selected_option: String,
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
            AgendaTx::SuggestSimpleAgenda(t) => {
                t.from(hash, from_account, block_hash, block_number, json, tx)
            }
            AgendaTx::VoteSimpleAgenda(t) => {
                t.from(hash, from_account, block_hash, block_number, json, tx)
            }
        }
    }
}

impl Common for SuggestSimpleAgenda {
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
