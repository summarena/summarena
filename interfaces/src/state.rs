use anyhow::Result;
use sqlx::Connection;
use sqlx::SqliteConnection;
use sqlx::migrate::MigrateDatabase;

use crate::defs::InputItem;
use crate::defs::LiveSourceSpec;

async fn get_db_connection() -> Result<SqliteConnection> {
    Ok(SqliteConnection::connect("sqlite:omni.db").await?)
}

pub async fn migrate() -> Result<()> {
    sqlx::Sqlite::create_database("sqlite:omni.db").await?;
    let mut conn = get_db_connection().await?;
    sqlx::migrate!("./migrations").run(&mut conn).await?;
    Ok(())
}

pub async fn create_live_source_spec(live_source_spec: &LiveSourceSpec) -> Result<()> {
    let mut conn = get_db_connection().await?;
    sqlx::query("
        INSERT OR IGNORE INTO live_source_specs
            (uri)
        VALUES
            (?1)
    ")
    .bind(&live_source_spec.uri)
    .execute(&mut conn)
    .await?;
    Ok(())
}

pub async fn ingest(input_item: &InputItem) -> Result<()> {
    let mut conn = get_db_connection().await?;
    sqlx::query("
        INSERT OR IGNORE INTO input_items
            (uri, live_source_uri, text, vision)
        VALUES
            (?1, ?2, ?3, ?4)
    ")
    .bind(&input_item.uri)
    .bind(&input_item.live_source_uri)
    .bind(&input_item.text)
    .bind(&input_item.vision)
    .execute(&mut conn)
    .await?;
    Ok(())
}
