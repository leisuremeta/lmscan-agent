use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
#[serde_with::serde_as]
pub struct Block {
    pub header: Header,
    #[serde(rename = "transactionHashes")]
    pub transaction_hashes: Vec<String>,
    pub votes: Vec<Votes>,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Header {
    pub number: i64,
    #[serde(rename = "parentHash")]
    pub parent_hash: String,
    #[serde(rename = "stateRoot")]
    pub state_root: StateRoot,
    #[serde(rename = "transactionsRoot")]
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
pub struct AccountStateRoot {
    #[serde(rename = "namesRoot")]
    pub names_root: Option<String>,
    #[serde(rename = "keyRoot")]
    pub key_root: Option<String>,
    #[serde(rename = "ethRoot")]
    pub eth_root: Option<String>,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct GroupStateRoot {
    #[serde(rename = "groupRoot")]
    pub group_root: Option<String>,
    #[serde(rename = "groupAccountRoot")]
    pub group_account_root: Option<String>,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct TokenStateRoot {
    #[serde(rename = "tokenDefinitionRoot")]
    pub token_definition_root: Option<String>,
    #[serde(rename = "fungibleBalanceRoot")]
    pub fungible_balance_root: Option<String>,
    #[serde(rename = "nftBalanceRoot")]
    pub nft_balance_root: Option<String>,
    #[serde(rename = "nftRoot")]
    pub nft_root: Option<String>,
    #[serde(rename = "rarityRoot")]
    pub rarity_root: Option<String>,
    #[serde(rename = "entrustFungibleBalanceRoot")]
    pub entrust_fungible_balance_root: Option<String>,
    #[serde(rename = "entrustNftBalanceRoot")]
    pub entrust_nft_balance_root: Option<String>,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct RewardStateRoot {
    pub dao: Option<String>,
    #[serde(rename = "userActivity")]
    pub user_activity: Option<String>,
    #[serde(rename = "tokenReceived")]
    pub token_received: Option<String>,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Votes {
    pub v: i64,
    pub r: String,
    pub s: String,
}
