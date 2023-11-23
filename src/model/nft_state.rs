use bigdecimal::BigDecimal;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Default)]
pub struct NftState {
    #[serde(rename = "tokenId")]
    pub token_id: String,
    #[serde(rename = "tokenDefinitionId")]
    pub token_def_id: String,
    #[serde(rename = "rarity")]
    pub rarity: String,
    pub weight: BigDecimal,
    #[serde(rename = "currentOwner")]
    pub current_owner: String,
}
