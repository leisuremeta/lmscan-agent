use crate::library::common::now;
use crate::transaction::{Job, TransactionWithResult};
use crate::entity::*;
use bigdecimal::{BigDecimal, Zero};
use log::LevelFilter;
use sea_orm::DatabaseConnection;
use sea_orm::*;
use sea_query::OnConflict;
use tokio::time::sleep;
use std::collections::HashMap;
use std::sync::Mutex;
use std::time::Duration;
use std::vec;

pub static mut BAL_VEC: Mutex<Vec<(TransactionWithResult, String)>> = Mutex::new(Vec::new());

async fn db_connn(url: String) -> DatabaseConnection {
    let mut opt = ConnectOptions::new(url);
    opt.min_connections(4)
        .max_connections(8)
        .connect_timeout(Duration::from_secs(30))
        .acquire_timeout(Duration::from_secs(30))
        .idle_timeout(Duration::from_secs(120))
        .sqlx_logging(false)
        .sqlx_logging_level(LevelFilter::Error);

    match Database::connect(opt).await {
        Ok(conn) => conn,
        Err(err) => panic!("{err}"),
    }
}

async fn init_db(db: &DatabaseConnection) {
    let schema = Schema::new(DbBackend::Sqlite);
    let stmt = schema.create_table_from_entity(balance_tx::Entity);
    let stmt2 = schema.create_table_from_entity(balance_entity::Entity);
    let stmt3 = schema.create_table_from_entity(spend_tx::Entity);
    let _ = db.execute(db.get_database_backend().build(&stmt)).await;
    let _ = db.execute(db.get_database_backend().build(&stmt2)).await;
    let _ = db.execute(db.get_database_backend().build(&stmt3)).await;
}

#[derive(Clone, Debug)]
pub enum BalanceOp {
    AddFree { hash: String, address: String, free: BigDecimal, token: String, },
    AddLock { hash: String, address: String, free: BigDecimal, lock: BigDecimal, token: String, },
    SpendFree { hash: String, address: String, token: String, },
    SpendLock { hash: String, token: String, },
    ToOwner { new_hash: String, hash: String,  token: String, },
}
fn find_bal_tx(hash: String, address: String) -> Select<balance_tx::Entity> {
    balance_tx::Entity::find()
        .filter(balance_tx::Column::Hash.eq(hash.clone()))
        .filter(balance_tx::Column::Address.eq(address.clone()))
}
fn find_lock_tx(hash: String) -> Select<balance_tx::Entity> {
    balance_tx::Entity::find()
        .filter(balance_tx::Column::Hash.eq(hash.clone()))
}
async fn balance_check_and_update(
    remote_db: &DatabaseConnection,
    local_db: &DatabaseConnection,
) {
    let txs;
    unsafe {
        let mut v = BAL_VEC.lock().unwrap();
        txs = v.clone();
        v.clear();
    }
    let mut bal_map: HashMap<String, balance_entity::Model> = HashMap::new();
    let mut next: Vec<spend_tx::Model> = vec![];
    let mut v_bal_op : Vec<BalanceOp> = vec![];
    let mut v_bal_spend: Vec<(String, BigDecimal, BigDecimal)> = vec![];
    let mut m_to_owner: HashMap<String, (BigDecimal, String, String)> = HashMap::new();
    let mut v_add = vec![];

    for (tx_res, hash) in txs {
        v_bal_op.append(&mut tx_res.update_balance(hash.clone()).await);
    }

    let spends = spend_tx::Entity::find().all(local_db).await.unwrap();
    for spend_tx::Model { target, hash, token, t } in spends {
        match t {
            0 => match find_bal_tx(hash.clone(), target.clone()).one(local_db).await {
                Ok(Some(mut m)) => {
                    v_bal_spend.push((m.address.clone(), -m.free.clone(), BigDecimal::zero()));
                    m.spend = true;
                    let _ = balance_tx::Entity::update(m.into_active_model())
                        .exec(local_db).await;
                }
                _ => {
                    next.push(spend_tx::Model {
                        target: target.clone(),
                        hash: hash,
                        token: token,
                        t: 0,
                    });
                }
            }
            1 => match find_lock_tx(hash.clone()).one(local_db).await {
                Ok(Some(mut m)) => {
                    v_bal_spend.push((m.address.clone(), BigDecimal::zero(), -m.lock.clone()));
                    m.spend = true;
                    let _ = balance_tx::Entity::update(m.into_active_model())
                        .exec(local_db).await;
                }
                _ => {
                    next.push(spend_tx::Model {
                        target: "-".to_string(),
                        hash: hash,
                        token: token,
                        t: 1,
                    });
                }
            }
            2 => match find_lock_tx(hash.clone()).one(local_db).await {
                Ok(Some(mut m)) => {
                    match m_to_owner.get(&hash) {
                        Some((prev, a, t)) => m_to_owner.insert(hash.clone(),  (prev + m.lock.clone(), a.to_owned(), t.to_owned())),
                        None => m_to_owner.insert(hash.clone(),  (m.lock.clone(), m.address.clone(), token.clone())),
                    };
                    v_bal_spend.push((m.address.clone(), BigDecimal::zero(), -m.lock.clone()));
                    m.spend = true;
                    let _ = balance_tx::Entity::update(m.into_active_model())
                        .exec(local_db).await;
                } 
                _ => {
                    next.push(spend_tx::Model {
                        target: target.clone(),
                        hash: hash,
                        token: token,
                        t: 2,
                    });
                }
            } 
            _ => panic!("spend 't' is wrong value")
        }
    }
    spend_tx::Entity::delete_many().exec(local_db).await.unwrap();

    for bal_op in v_bal_op.clone() {
        match bal_op {
            BalanceOp::AddFree { hash, address, free, token } => {
                v_add.push(balance_tx::Model {
                    hash,
                    address,
                    free: free,
                    lock: BigDecimal::zero(),
                    spend: false,
                    token,
                });
            }
            BalanceOp::AddLock { hash, address, free, lock, token } => {
                v_add.push(balance_tx::Model {
                    hash,
                    address,
                    free: free,
                    lock: lock,
                    spend: false,
                    token,
                });
            }
            _ => (),
        }
    }

    for bal_op in v_bal_op {
        match bal_op {
            BalanceOp::SpendFree { hash, address, token } => {
                next.push(spend_tx::Model {
                    target: address.clone(),
                    hash: hash,
                    token: token,
                    t: 0,
                });
            }
            BalanceOp::SpendLock { hash, token } => {
                next.push(spend_tx::Model {
                    target: "-".to_string(),
                    hash: hash,
                    token: token,
                    t: 1,
                });
            } 
            BalanceOp::ToOwner { new_hash, hash, token } => {
                next.push(spend_tx::Model {
                    target: new_hash,
                    hash: hash,
                    token: token,
                    t: 2,
                });
            } 
            _ => (),
        }
    }

    let v: Vec<spend_tx::ActiveModel> = next.into_iter().map(|m| m.into_active_model()).collect();
    let _ = spend_tx::Entity::insert_many(v).do_nothing().exec(local_db).await;
    
    for (hash, (free, address, token)) in m_to_owner {
        v_add.push(balance_tx::Model { hash, address, free: free, lock: BigDecimal::zero(), spend: false, token });
    }

    let _ = balance_tx::Entity::insert_many::<balance_tx::ActiveModel, Vec<balance_tx::ActiveModel>>(v_add.clone().into_iter().map(|m| m.into_active_model()).collect()).do_nothing().exec(local_db).await;

    for m in v_add {
        if m.token != "LM" { continue; }
        let opt_b = bal_map.get(&m.address).map(|x| x.clone());
        let b = match opt_b {
            Some(b) => b.add(m.free, m.lock),
            None => {
                let opt_c = balance_entity::Entity::find_by_id(m.address.clone()).one(local_db).await.unwrap_or(None);
                match opt_c {
                    Some(x) => x.add(m.free, m.lock),
                    _ => {
                        balance_entity::Model {
                            address: m.address.clone(),
                            free: m.free,
                            locked: m.lock,
                            created_at: now(),
                            updated_at: now(),
                        }
                    }
                }
            }
        };
        bal_map.insert(m.address, b);
    }

    for (address, free, lock) in v_bal_spend {
        let opt_b = bal_map.get(&address).map(|x| x.clone());
        let b = match opt_b {
            Some(b) => b.add(free, lock),
            None => {
                let opt_c = balance_entity::Entity::find_by_id(address.clone()).one(local_db).await.unwrap_or(None);
                match opt_c {
                    Some(x) => x.add(free, lock),
                    _ => {
                        balance_entity::Model {
                            address: address.clone(),
                            free: free,
                            locked: lock,
                            created_at: now(),
                            updated_at: now(),
                        }
                    }
                }
            }
        };
        bal_map.insert(address, b);
    }

    let v: Vec::<balance_entity::ActiveModel> = bal_map.clone().values().into_iter().map(|m| m.to_owned().into_active_model()).collect();
    let _ = balance_entity::Entity::insert_many(v.clone()).on_conflict(
        OnConflict::column(balance_entity::Column::Address)
            .update_columns([balance_entity::Column::Free, balance_entity::Column::Locked])    
            .value(balance_entity::Column::UpdatedAt, now())
            .to_owned()
    ).do_nothing().exec(local_db).await;
    let v: Vec::<balance_entity::ActiveModel> = bal_map.clone().values().into_iter().map(|m| m.to_owned().to_bal()).collect();
    let _ = balance_entity::Entity::insert_many(v).on_conflict(
        OnConflict::column(balance_entity::Column::Address)
            .update_columns([balance_entity::Column::Free, balance_entity::Column::Locked])    
            .value(balance_entity::Column::UpdatedAt, now())
            .to_owned()
    ).do_nothing().exec(remote_db).await;
}

pub async fn balance_loop(remote_db: DatabaseConnection, sqlite_url: String) {
    tokio::spawn(async move { 
        let local_db = db_connn(sqlite_url).await;
        init_db(&local_db).await;
        loop {
            balance_check_and_update(&remote_db, &local_db).await;
            sleep(Duration::from_secs(10)).await;
        }
    })
    .await
    .unwrap()
}
