use dotenvy::{dotenv, var};
use lmscan_agent::library::common::db_connn;
use lmscan_agent::service::snapshot::daily_snapshot;

#[tokio::test]
async fn daily_snapshot_test() {
    dotenv().expect("Unable to load environment variables from .env file");
    let database_url = var("DATABASE_URL").expect("DATABASE_URL must be set.");
    let ref db = db_connn(database_url).await;

    daily_snapshot(db).await;
}
