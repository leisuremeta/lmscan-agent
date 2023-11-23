use std::{fs::File, io::Write, path::Path};

use dotenvy::var;
use itertools::Itertools;
use lmscan_agent::library::common::db_connn;
use lmscan_agent::{
    service::api_service::ApiService,
    transaction::{
        common::Common, reward_transaction::RewardTx, token_transaction::TokenTx, Job, Transaction,
        TransactionWithResult,
    },
    tx_state,
};
use sea_orm::*;

#[tokio::test]
async fn fungible_tx_tracking() {
    let database_url = var("DATABASE_URL").expect("DATABASE_URL must be set.");
    let ref db = db_connn(database_url).await;

    let mut output_file = File::create(Path::new("fungible_and_input_txs.txt"))
        // .append(true)
        // .open("")
        .expect("cannot open output file");

    let account_address = "8e39dcc13ebdb7e8eb3da92090e4058c44ec9ca7";

    output_file
        .write(format!("{account_address}_fungible_history\n\n").as_bytes())
        .unwrap();

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
      and json like '%{account_address}%';"#
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

    let tx_results = tx_states
        .map(|(hash, state)| {
            (
                hash,
                serde_json::from_str::<TransactionWithResult>(&state).unwrap(),
            )
        })
        .sorted_by_key(|(_, tx)| tx.signed_tx.value.created_at());

    for (hash, tx_res) in tx_results.clone() {
        let inputs = tx_res.input_hashs();

        let json = serde_json::to_string_pretty(&tx_res).unwrap();
        output_file
            .write(format!("LATEST TX\n").as_bytes())
            .unwrap();
        output_file.write(format!("{hash}\n").as_bytes()).unwrap();
        output_file.write(format!("{json}\n\n").as_bytes()).unwrap();

        let len = inputs.len();
        output_file
            .write(format!("INPUT TX LIST [{len}]\n").as_bytes())
            .unwrap();
        for input_hash in inputs {
            let input_tx =
                serde_json::to_string_pretty(&ApiService::get_tx_always(&input_hash).await)
                    .unwrap();
            output_file
                .write(format!("{input_hash}\n{input_tx}\n").as_bytes())
                .unwrap();
        }
        output_file.write(format!("--------------------------------------------------------------------------------------\n\n").as_bytes()).unwrap();
    }
}
