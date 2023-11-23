use std::{
    collections::{HashMap, HashSet},
    fs::{self, File},
    io::Write,
    path::Path,
    sync::Arc,
};

use bigdecimal::BigDecimal;
use dotenvy::var;
use itertools::Itertools;
use lmscan_agent::transaction::common::Common;
use lmscan_agent::{
    account_entity,
    library::common::{as_json_byte_vec, db_connn},
    service::finder_service::Finder,
    transaction::{
        reward_transaction::RewardTx, token_transaction::TokenTx, Job, Transaction,
        TransactionResult, TransactionWithResult,
    },
    tx_state,
};
use sea_orm::{DatabaseConnection, DbBackend, EntityTrait, Statement};

// TODO:
// case
//  [1] input output 불일지
//  [2] input tx 재사용
//  [3] 다른 사람의 utxo 사용

#[tokio::test]
async fn sum_diff() {
    let database_url = var("DATABASE_URL").expect("DATABASE_URL must be set.");
    let ref db = db_connn(database_url).await;

    let mut sled_path = std::env::current_dir().unwrap();
    sled_path.push("test");
    sled_path.push("sled");
    sled_path.push("input_tx");

    // This will remove the directory and all its contents.
    if let Err(err) = fs::remove_dir_all(&sled_path) {
        println!("Failed to remove old Sled DB directory. - {err}");
    }

    let sled = Arc::new(
        sled::Config::default()
            .path(sled_path)
            .use_compression(true)
            .compression_factor(6)
            .flush_every_ms(None)
            .open()
            .unwrap(),
    );

    let mut output_file = File::create(Path::new(&format!("output_input_diff.txt")))
        .expect("cannot open output file");

    // -- curr_balance, result_balance, inequality_sign, amount
    // ++ 타겟 계정 자신에게 남은 잔고 보내는 양 (amount)
    // double spanding utxo 의 amount * duplicated count
    output_file
        .write(
            format!("signer, hash, sub_type, output_sum, inequality_sign, input_sum, diff\n")
                .as_bytes(),
        )
        .unwrap();

    let query = format!(
        r#"select * from tx_state where 
        (
          json like '%OfferReward%' or
          json like '%ExecuteOwnershipReward%' or
          json like '%ExecuteReward%' or
          json like '%EntrustFungibleToken%' or 
          json like '%TransferFungibleToken%'
        );"#
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

    for (hash, tx_res) in tx_results.into_iter() {
        let signer = &tx_res.signed_tx.sig.account;
        let value = sled.get(signer).unwrap_or_default().unwrap_or_default();
        let mut spent_txs =
            serde_json::from_slice::<HashSet<String>>(&value).unwrap_or_else(|_| HashSet::new());

        let sub_type = extract_subtype(&tx_res);
        println!("tx hash: {hash}");

        let output_sum = output_sum_in_latest_tx(&tx_res).abs();
        let input_sum = input_sum_in_latest_tx(&tx_res, &db).await.abs();
        if output_sum == input_sum {
            continue;
        }

        let inequality_sign = if output_sum < input_sum { "<" } else { ">" };
        // let input_txs = input_txs(&tx_res.signed_tx.value).join(",");

        let diff = if output_sum > input_sum {
            output_sum.clone() - input_sum.clone()
        } else {
            input_sum.clone() - output_sum.clone()
        };
        output_file.write(format!("{signer}, {hash}, {sub_type}, {output_sum}, {inequality_sign}, {input_sum}, {diff}\n").as_bytes()).unwrap();

        spent_txs.extend(tx_res.input_hashs());
        sled.insert(signer.as_bytes(), as_json_byte_vec(&spent_txs))
            .unwrap();
    }
}

#[tokio::test]
async fn filter_double_spend_other_tx() {
    let database_url = var("DATABASE_URL").expect("DATABASE_URL must be set.");
    let ref db = db_connn(database_url).await;

    let mut output_file = File::create(Path::new(&format!("double_spend_other_tx.txt")))
        .expect("cannot open output file");
    let query = format!(
        r#"select * from tx_state where 
        (
          json like '%OfferReward%' or
          json like '%ExecuteOwnershipReward%' or
          json like '%ExecuteReward%' or
          json like '%EntrustFungibleToken%' or 
          json like '%TransferFungibleToken%'
        );"#
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

    for (hash, tx_res) in tx_results {
        let latest_signer = tx_res.clone().signed_tx.sig.account;
        let tx_res = tx_res.clone();
        let sub_type = extract_subtype(&tx_res);

        let inputs = tx_res.input_hashs();

        for input_tx_hash in inputs {
            // let input_tx = ApiService::get_tx_always(&input_tx_hash).await;
            // let input_tx_state = tx_state::Entity::find_by_id(input_tx_hash.clone()).one(db).await.unwrap().unwrap();
            // let input_tx: TransactionWithResult = serde_json::from_str(&input_tx_state.json).unwrap();
            let input_tx = Finder::transaction_with_result(&input_tx_hash).await;
            let input_signer = input_tx.signed_tx.sig.account;
            let input_tx_outputs = match input_tx.signed_tx.value.clone() {
                Transaction::RewardTx(tx) => match tx {
                    RewardTx::OfferReward(t) => Some(t.outputs),
                    RewardTx::ExecuteOwnershipReward(t) => match input_tx.result.clone().unwrap() {
                        TransactionResult::ExecuteOwnershipRewardResult(res) => Some(res.outputs),
                        _ => None,
                    },
                    RewardTx::ExecuteReward(t) => match input_tx.result.clone().unwrap() {
                        TransactionResult::ExecuteRewardResult(res) => Some(res.outputs),
                        _ => None,
                    },
                    _ => None,
                },
                Transaction::TokenTx(tx) => match tx {
                    TokenTx::TransferFungibleToken(t) => Some(t.outputs),
                    TokenTx::MintFungibleToken(t) => Some(t.outputs),
                    TokenTx::DisposeEntrustedFungibleToken(t) => Some(t.outputs),
                    TokenTx::EntrustFungibleToken(t) => {
                        let remainder = match (&input_tx.result).clone().unwrap() {
                            TransactionResult::EntrustFungibleTokenResult(res) => {
                                res.remainder.clone()
                            }
                            _ => panic!("invalid BurnFungibleTokenResult"),
                        };
                        if latest_signer == input_signer {
                            Some(HashMap::from([(input_signer, remainder)]))
                        } else {
                            None
                        }
                    }
                    TokenTx::BurnFungibleToken(t) => {
                        let output_amount = match (&input_tx.result).clone().unwrap() {
                            TransactionResult::BurnFungibleTokenResult(res) => {
                                res.output_amount.clone()
                            }
                            _ => panic!("invalid BurnFungibleTokenResult"),
                        };
                        if latest_signer == input_signer {
                            Some(HashMap::from([(input_signer, output_amount)]))
                        } else {
                            None
                        }
                    }
                    _ => None,
                },
                _ => None,
            };

            match input_tx_outputs {
                Some(outputs) => match outputs.get(&latest_signer) {
                    None => {
                        output_file
                            .write(
                                format!("{latest_signer},{hash},{sub_type},{input_tx_hash}")
                                    .as_bytes(),
                            )
                            .unwrap();
                    }
                    _ => (),
                },
                _ => (),
            };
        }
    }
}

#[tokio::test]
async fn balance_build_history() {
    let database_url = var("DATABASE_URL").expect("DATABASE_URL must be set.");
    let ref db = db_connn(database_url).await;

    let mut sled_path = std::env::current_dir().unwrap();
    sled_path.push("test");
    sled_path.push("sled");
    sled_path.push("input_tx");

    // This will remove the directory and all its contents.
    if let Err(err) = fs::remove_dir_all(&sled_path) {
        println!("Failed to remove old Sled DB directory. - {err}");
    }

    let sled = Arc::new(
        sled::Config::default()
            .path(sled_path)
            .use_compression(true)
            .compression_factor(6)
            .flush_every_ms(None)
            .open()
            .unwrap(),
    );

    let mut output_file = File::create(Path::new(&format!("output_input_diff.txt")))
        // .append(true)
        // .open("")
        .expect("cannot open output file");

    // -- curr_balance, result_balance, inequality_sign, amount
    // ++ 타겟 계정 자신에게 남은 잔고 보내는 양 (amount)
    // double spanding utxo 의 amount * duplicated count
    output_file
        .write(
            format!("signer, hash, sub_type, output_sum, inequality_sign, input_sum, diff\n")
                .as_bytes(),
        )
        .unwrap();

    let accounts = account_entity::Entity::find().all(db).await.unwrap();
    for account in accounts {
        let signer = account.address;

        if signer.is_empty() {
            panic!()
        }

        let query = format!(
            r#"select * from tx_state where 
          (
            json like '%OfferReward%' or
            json like '%ExecuteOwnershipReward%' or
            json like '%ExecuteReward%' or
            json like '%EntrustFungibleToken%' or 
            json like '%TransferFungibleToken%'
          ) 
        and json like '%account":"{signer}%';"#
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

        for (hash, tx_res) in tx_results.into_iter() {
            let value = sled.get(&signer).unwrap_or_default().unwrap_or_default();
            let mut spent_txs = serde_json::from_slice::<HashSet<String>>(&value)
                .unwrap_or_else(|_| HashSet::new());

            let sub_type = extract_subtype(&tx_res);
            println!("tx hash: {hash}");

            let output_sum = output_sum_in_latest_tx(&tx_res).abs();
            let input_sum = input_sum_in_latest_tx(&tx_res, db).await.abs();
            if output_sum == input_sum {
                continue;
            }

            let inequality_sign = if output_sum < input_sum { "<" } else { ">" };
            // let input_txs = input_txs(&tx_res.signed_tx.value).join(",");

            let diff = if output_sum > input_sum {
                output_sum.clone() - input_sum.clone()
            } else {
                input_sum.clone() - output_sum.clone()
            };
            output_file.write(format!("{signer},{hash},{sub_type},{output_sum},{inequality_sign},{input_sum},{diff}\n").as_bytes()).unwrap();

            spent_txs.extend(tx_res.input_hashs());
            sled.insert(signer.as_bytes(), as_json_byte_vec(&spent_txs))
                .unwrap();
        }
    }
}

fn outputs(tx_res: &TransactionWithResult) -> Option<HashMap<String, BigDecimal>> {
    let from_account = tx_res.signed_tx.sig.account.clone();
    match tx_res.signed_tx.value.clone() {
        Transaction::RewardTx(tx) => match tx {
            RewardTx::OfferReward(t) => Some(t.outputs),
            RewardTx::ExecuteOwnershipReward(t) => match tx_res.result.clone().unwrap() {
                TransactionResult::ExecuteOwnershipRewardResult(res) => Some(res.outputs),
                _ => None,
            },
            RewardTx::ExecuteReward(t) => match tx_res.result.clone().unwrap() {
                TransactionResult::ExecuteRewardResult(res) => Some(res.outputs),
                _ => None,
            },
            _ => None,
        },
        Transaction::TokenTx(tx) => match tx {
            TokenTx::TransferFungibleToken(t) => Some(t.outputs),
            TokenTx::MintFungibleToken(t) => Some(t.outputs),
            TokenTx::DisposeEntrustedFungibleToken(t) => Some(t.outputs),
            TokenTx::EntrustFungibleToken(t) => {
                let remainder = match (&tx_res.result).as_ref().unwrap() {
                    TransactionResult::EntrustFungibleTokenResult(res) => res.remainder.clone(),
                    _ => panic!("invalid BurnFungibleTokenResult"),
                };
                Some(HashMap::from([(from_account, remainder + t.amount)]))
            }
            TokenTx::BurnFungibleToken(t) => {
                let output_amount = match (&tx_res.result).as_ref().unwrap() {
                    TransactionResult::BurnFungibleTokenResult(res) => res.output_amount.clone(),
                    _ => panic!("invalid BurnFungibleTokenResult"),
                };
                Some(HashMap::from([(from_account, output_amount)]))
            }
            _ => None,
        },
        _ => None,
    }
}

fn remainder(tx_res: &TransactionWithResult) -> Option<BigDecimal> {
    match &tx_res.signed_tx.value {
        Transaction::TokenTx(tx) => match tx {
            TokenTx::EntrustFungibleToken(t) => match tx_res.result.as_ref().unwrap() {
                TransactionResult::EntrustFungibleTokenResult(res) => Some(res.remainder.clone()),
                _ => panic!("invalid ExecuteRewardResult"),
            },
            _ => None,
        },
        _ => None,
    }
}

fn output_sum_in_latest_tx(tx_res: &TransactionWithResult) -> BigDecimal {
    outputs(tx_res)
        .map(|outputs| outputs.values().into_iter().sum())
        .unwrap_or(BigDecimal::from(0))
}

async fn input_sum_in_latest_tx(
    tx_res: &TransactionWithResult,
    db: &DatabaseConnection,
) -> BigDecimal {
    let from_account = &tx_res.signed_tx.sig.account;
    let inputs = tx_res.input_hashs();

    let mut output_sum: BigDecimal = BigDecimal::from(0);
    for input_tx_hash in inputs {
        // let input_tx = ApiService::get_tx_always(&input_tx_hash).await;
        let input_tx_state = tx_state::Entity::find_by_id(input_tx_hash)
            .one(db)
            .await
            .unwrap()
            .unwrap();
        let input_tx = serde_json::from_str(&input_tx_state.json).unwrap();

        let outputs_in_input_tx =
            extract_outputs_from_input_tx_for_withdraw(input_tx, from_account);

        output_sum += outputs_in_input_tx
            .get(from_account)
            .unwrap_or(&BigDecimal::from(0));
    }

    output_sum
}

fn extract_subtype(input_tx_with_res: &TransactionWithResult) -> &str {
    match &input_tx_with_res.signed_tx.value {
        Transaction::RewardTx(rw) => match rw {
            RewardTx::OfferReward(_) => "OfferReward",
            RewardTx::ExecuteReward(_) => "ExecuteReward",
            RewardTx::ExecuteOwnershipReward(_) => "ExecuteOwnershipReward",
            _ => panic!(),
        },
        Transaction::TokenTx(tk) => match tk {
            TokenTx::MintFungibleToken(_) => "MintFungibleToken",
            TokenTx::TransferFungibleToken(_) => "TransferFungibleToken",
            TokenTx::DisposeEntrustedFungibleToken(_) => "DisposeEntrustedFungibleToken",
            TokenTx::BurnFungibleToken(_) => "BurnFungibleToken",
            TokenTx::EntrustFungibleToken(_) => "EntrustFungibleToken",
            _ => panic!(),
        },
        _ => panic!(),
    }
}

fn extract_updated_balance_accounts(
    account_balance_info: &HashMap<String, BigDecimal>,
    balanced_updated_accounts: HashSet<String>,
) -> HashMap<String, BigDecimal> {
    account_balance_info
        .iter()
        .filter(|(k, _)| balanced_updated_accounts.contains(*k))
        .map(|(k, v)| (k.clone(), v.clone()))
        .collect()
}

fn extract_outputs_from_input_tx_for_withdraw(
    input_tx_with_res: TransactionWithResult,
    from_account: &String,
) -> HashMap<String, BigDecimal> {
    // withdraw from_account
    // b: account's balance
    // d: deposit amount
    match input_tx_with_res.signed_tx.value {
        Transaction::RewardTx(rw) => match rw {
            RewardTx::OfferReward(t) => t.outputs,
            RewardTx::ExecuteReward(_) => match input_tx_with_res.result.unwrap() {
                TransactionResult::ExecuteRewardResult(res) => res.outputs,
                _ => panic!("invalid ExecuteRewardResult"),
            },
            RewardTx::ExecuteOwnershipReward(_) => match input_tx_with_res.result.unwrap() {
                TransactionResult::ExecuteOwnershipRewardResult(res) => res.outputs,
                _ => panic!("invalid ExecuteOwnershipRewardResult"),
            },
            _ => panic!(),
        },
        Transaction::TokenTx(tk) => match tk {
            TokenTx::MintFungibleToken(t) => t.outputs,
            TokenTx::TransferFungibleToken(t) => t.outputs,
            TokenTx::DisposeEntrustedFungibleToken(t) => t.outputs,
            TokenTx::BurnFungibleToken(_) => {
                let output_amount = match input_tx_with_res.result.unwrap() {
                    TransactionResult::BurnFungibleTokenResult(res) => res.output_amount,
                    _ => panic!("invalid ExecuteOwnershipRewardResult"),
                };
                HashMap::from([(from_account.clone(), output_amount)])
            }
            TokenTx::EntrustFungibleToken(_) => {
                let remainder = match input_tx_with_res.result.unwrap() {
                    TransactionResult::EntrustFungibleTokenResult(res) => res.remainder,
                    _ => panic!("invalid EntrustFungibleTokenResult"),
                };
                HashMap::from([(from_account.clone(), remainder)])
            }
            _ => panic!(),
        },
        _ => panic!(),
    }
}
