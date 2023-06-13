use std::collections::HashSet;

use bigdecimal::BigDecimal;
use chrono::{FixedOffset, Utc, Duration};
use lmscan_agent::{balance_entity, account_entity, model::balance::Balance};
use dotenvy::{dotenv, var};
use lmscan_agent::library::common::db_connn;
use sea_orm::*;


#[tokio::test]
async fn daily_snapshot() {
  let now = Utc::now();

  let start_of_day = (now - Duration::days(1)).date_naive().and_hms_opt(15, 0, 0).unwrap().timestamp();
  let end_of_day = now.date_naive().and_hms_opt(14, 59, 59).unwrap().timestamp();

  dotenv().expect("Unable to load environment variables from .env file");
  let database_url = var("DATABASE_URL").expect("DATABASE_URL must be set.");
  let ref db = db_connn(database_url).await;

  let balances = balance_entity::Entity::find()
                                                    .filter(Condition::all().add(balance_entity::Column::UpdatedAt.between(start_of_day, end_of_day)))
                                                    .all(db).await.unwrap();
  let balances: Vec<(String, Balance)> = balances.into_iter().map(|b| (b.address.clone(), Balance::from(b.clone()))).collect();
  for (address, balance) in balances.into_iter() {

  }
  


}
