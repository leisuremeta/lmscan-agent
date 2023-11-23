use std::{collections::HashMap, fs::File, io::Write, path::Path};

use dotenvy::var;
use itertools::Itertools;
use lmscan_agent::transaction::common::Common;
use lmscan_agent::{
    library::common::db_connn,
    transaction::{Job, TransactionWithResult},
    tx_state,
};
use sea_orm::{DbBackend, EntityTrait, Statement};

#[tokio::test]
async fn filter_double_spanding() {
    let database_url = var("DATABASE_URL").expect("DATABASE_URL must be set.");
    let ref db = db_connn(database_url).await;

    // let mut output_file = File::create(Path::new("filter_double_spanding"))
    //                                   // .append(true)
    //                                   // .open("")
    //                                   .expect("cannot open output file");

    // let accounts = account_entity::Entity::find().all(db).await.unwrap();
    let mut counter = 0;
    // for account in accounts {
    // let account_address = account.address;
    let account_address = "reward-activity";
    let mut output_file = File::create(Path::new(&format!(
        "{account_address}_filter_double_spanding"
    )))
    // .append(true)
    // .open("")
    .expect("cannot open output file");

    let query = format!(
        r#"select * from tx where 
          (
            json like '%OfferReward%' or
            json like '%ExecuteOwnershipReward%' or
            json like '%ExecuteReward%' or
            json like '%EntrustFungibleToken%' or 
            json like '%TransferFungibleToken%' or
            json like '%MintFungibleToken%' or
            json like '%DisposeEntrustedFungibleToken%' or 
            json like '%BurnFungibleToken%'
          ) 
        and json like '%account":"{account_address}%';"#
    );

    let tx_states = tx_state::Entity::find()
        .from_raw_sql(Statement::from_sql_and_values(
            DbBackend::Postgres,
            &query,
            [],
        ))
        .all(db)
        .await
        .unwrap();

    let tx_states = tx_states
        .iter()
        .map(|state| (state.hash.clone(), state.json.clone()));

    let tx_results: std::vec::IntoIter<(String, TransactionWithResult)> = tx_states
        .map(|(hash, state)| {
            (
                hash,
                serde_json::from_str::<TransactionWithResult>(&state).unwrap(),
            )
        })
        .sorted_by_key(|(_, tx)| tx.signed_tx.value.created_at());

    // let mut duplicated_txs = Vec::new();
    let mut total_input_hashs = Vec::new();
    for (_, tx_res) in tx_results.into_iter() {
        let inputs = tx_res.input_hashs();
        total_input_hashs.extend(inputs);
    }

    let mut map = HashMap::new();
    for hash in total_input_hashs.into_iter() {
        match map.get_mut(&hash) {
            Some(count) => *count += 1,
            None => {
                map.insert(hash, 1);
            }
        }
    }

    if !map.is_empty() {
        output_file
            .write(format!("{account_address}\n").as_bytes())
            .unwrap();

        map.into_iter().filter(|(_, v)| *v > 1).for_each(|(k, v)| {
            output_file.write(format!("{k}, {v}\n").as_bytes()).unwrap();
        });

        output_file.write("\n".as_bytes()).unwrap();

        counter += 1;
    }
    // }

    output_file
        .write(format!("\n\n{counter}\n").as_bytes())
        .unwrap();
}
