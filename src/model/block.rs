use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
#[serde_with::serde_as]
pub struct Block {
    pub header: Header,
    pub transaction_hashes: Vec<String>,
    pub votes: Vec<Votes>,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Header {
    pub number: i64,
    pub parent_hash: String,
    pub state_root: StateRoot,
    pub transactions_root: Option<String>,
    pub timestamp: String,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct StateRoot {
    // pub main: Option<String>,
    pub account: AccountStateRoot,
    pub group: GroupStateRoot,
    pub token: TokenStateRoot,
    pub reward: RewardStateRoot,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AccountStateRoot {
    pub names_root: Option<String>,
    pub key_root: Option<String>,
    pub eth_root: Option<String>,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GroupStateRoot {
    pub group_root: Option<String>,
    pub group_account_root: Option<String>,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TokenStateRoot {
    pub token_definition_root: Option<String>,
    pub fungible_balance_root: Option<String>,
    pub nft_balance_root: Option<String>,
    pub nft_root: Option<String>,
    pub rarity_root: Option<String>,
    pub entrust_fungible_balance_root: Option<String>,
    pub entrust_nft_balance_root: Option<String>,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RewardStateRoot {
    pub dao: Option<String>,
    pub user_activity: Option<String>,
    pub token_received: Option<String>,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Votes {
    pub v: i64,
    pub r: String,
    pub s: String,
}
