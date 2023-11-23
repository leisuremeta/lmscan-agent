use std::{collections::HashMap, fs::File, io::Write, path::Path};

use dotenvy::var;
use lmscan_agent::{
    balance_entity, library::common::db_connn, model::balance_info::BalanceInfo,
    service::api_service::ApiService,
};
use sea_orm::{EntityTrait, QueryOrder, QuerySelect};

// 잔고 상위 300개 계정 order by desc, blc balance json
#[tokio::test]
async fn balance_json() {
    let database_url = var("DATABASE_URL").expect("DATABASE_URL must be set.");
    let ref db = db_connn(database_url).await;

    let mut output_file =
        File::create(Path::new(&format!("balance_json.txt"))).expect("cannot open output file");

    // let query = format!(
    //   r#"select * from account order by balance desc limit 300;"#);

    // let accounts = account_entity::Entity::find().from_raw_sql(
    //                               Statement::from_sql_and_values(DbBackend::Postgres, &query, [])
    //                             )
    //                             .all(db)
    //                             .await.unwrap();

    let accounts = balance_entity::Entity::find()
        .order_by_desc(balance_entity::Column::Free)
        .limit(300)
        .all(db)
        .await
        .unwrap();

    for account in accounts {
        let address = account.address;
        let response = ApiService::get_as_text_always(format!(
            "http://lmc.leisuremeta.io/balance/{address}?movable=free"
        ))
        .await;

        let balance_res: HashMap<String, BalanceInfo> = serde_json::from_str(&response).unwrap();
        let lm_balance = balance_res.get("LM").unwrap();
        let tot_bal = lm_balance.total_amount.clone();
        output_file
            .write(format!("{address},{tot_bal}\n").as_bytes())
            .unwrap();
    }
}
