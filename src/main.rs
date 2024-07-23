use lmscan_agent::service::finder_service::Finder;

use lmscan_agent::library::common::*;
use lmscan_agent::{check_app, nft_app, summary_app, balance_app};

extern crate dotenvy;
use dotenvy::{dotenv, var};

#[tokio::main]
async fn main() {
    dotenv().expect("Unable to load environment variables from .env file");
    log4rs::init_file(var("LOG_CONFIG_FILE_PATH").unwrap(), Default::default()).unwrap();

    let database_url = var("DATABASE_URL").expect("DATABASE_URL must be set.");
    let coin_market_api_key = var("COIN_MARKET_API_KEY").expect("COIN_MARKET_API_KEY must be set.");
    let sqlite_url = var("SQLITE_URL").expect("SQLITE_URL must be set");

    let db = db_connn(database_url).await;
    Finder::init(db.clone());
    tokio::join!(
        summary_app::summary_loop(db.clone(), coin_market_api_key),
        check_app::check_loop(db.clone()),
        nft_app::nft_loop(db.clone()),
        balance_app::balance_loop(db.clone(), sqlite_url),
    );
}
