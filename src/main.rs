use lmscan_agent::service::finder_service::Finder;

use lmscan_agent::library::common::*;
use lmscan_agent::{summary_app, check_app};

extern crate dotenvy;
use dotenvy::{dotenv, var};

#[tokio::main]
async fn main() {
    dotenv().expect("Unable to load environment variables from .env file");
    log4rs::init_file(var("LOG_CONFIG_FILE_PATH").unwrap(), Default::default()).unwrap();

    let database_url = var("DATABASE_URL").expect("DATABASE_URL must be set.");
    let coin_market_api_key = var("COIN_MARKET_API_KEY").expect("COIN_MARKET_API_KEY must be set.");

    let db = db_connn(database_url).await;
    Finder::init(db.clone());
    // TODO: 몇번 블럭부터 빌드다시 시작할지 받을수 있는 설정 파일 만들기.
    tokio::join!(
        summary_app::summary_loop(db.clone(), coin_market_api_key),
        check_app::check_loop(db),
    );
}
