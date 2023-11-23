use once_cell::sync::OnceCell;
use sea_orm::{DatabaseConnection, EntityTrait};

use super::api_service::ApiService;
use crate::{transaction::TransactionWithResult, tx_state};

static DB: OnceCell<DatabaseConnection> = OnceCell::new();

pub struct Finder {}

impl Finder {
    pub fn init(db: DatabaseConnection) {
        DB.set(db).unwrap();
    }

    pub async fn transaction_with_result(hash: &String) -> TransactionWithResult {
        match tx_state::Entity::find_by_id(hash)
            .one(DB.get().unwrap())
            .await
            .unwrap()
        {
            Some(model) => serde_json::from_str(&model.json).unwrap(),
            None => ApiService::get_tx_always(hash).await,
        }
    }
}
