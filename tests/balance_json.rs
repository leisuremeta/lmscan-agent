use std::{fs::{File, self}, path::Path, io::Write, collections::{HashMap, HashSet}, sync::Arc};

use bigdecimal::BigDecimal;
use dotenvy::var;
use itertools::Itertools;
use lmscan_agent::{library::common::db_connn, tx_state, transaction::{TransactionWithResult, Job, Transaction, RewardTx, TransactionResult, TokenTx}, service::api_service::ApiService, account_entity};
use sea_orm::{Statement, DbBackend, EntityTrait, DatabaseConnection, sea_query, QueryOrder, QuerySelect};
use lmscan_agent::transaction::Common;

use std::hash::{Hash, Hasher};

// 잔고 상위 300개 계정 order by desc, blc balance json
#[tokio::test]
async fn balance_json() {
  let database_url = var("DATABASE_URL").expect("DATABASE_URL must be set.");
  let ref db = db_connn(database_url).await;



  let mut output_file = File::create(Path::new(&format!("balance_json.txt")))
                                    .expect("cannot open output file");

  // let query = format!(
  //   r#"select * from account order by balance desc limit 300;"#);
    
  // let accounts = account_entity::Entity::find().from_raw_sql(
  //                               Statement::from_sql_and_values(DbBackend::Postgres, &query, [])
  //                             )
  //                             .all(db)
  //                             .await.unwrap();

  let accounts = account_entity::Entity::find()
                                .order_by_desc(account_entity::Column::Balance)
                                .limit(300)
                                .all(db)
                                .await
                                .unwrap();

  for account in accounts {
    let address = account.address;

    let response = ApiService::get_as_text_always(format!("http://lmc.leisuremeta.io/balance/{address}?movable=free")).await;
    output_file.write(format!("{address},{response}\n").as_bytes()).unwrap();
  }

}
