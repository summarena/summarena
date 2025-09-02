use anyhow::Result;
use sqlx::{PgPool, Row};
use chrono::{DateTime, Utc};

#[derive(Debug, Clone)]
pub struct TwitterLastSync {
    pub handle: String,
    pub last_sync_date: Option<DateTime<Utc>>,
}

pub struct TwitterDatabase {
    pool: PgPool,
}

impl TwitterDatabase {
    pub async fn new(database_url: &str) -> Result<Self> {
        let pool = PgPool::connect(database_url).await?;
        Ok(Self { pool })
    }

    pub async fn setup_schema(&self) -> Result<()> {
        sqlx::query(
            r#"
            CREATE TABLE IF NOT EXISTS twitter_last_sync (
                handle VARCHAR(255) PRIMARY KEY,
                last_sync_date TIMESTAMP WITH TIME ZONE,
                created_at TIMESTAMP WITH TIME ZONE DEFAULT NOW(),
                updated_at TIMESTAMP WITH TIME ZONE DEFAULT NOW()
            )
            "#,
        )
        .execute(&self.pool)
        .await?;
        
        Ok(())
    }


    /// Get Twitter last sync data for a given handle
    pub async fn get_last_sync(&self, handle: &str) -> Result<Option<TwitterLastSync>> {
        let row = sqlx::query(
            "SELECT handle, last_sync_date FROM twitter_last_sync WHERE handle = $1"
        )
        .bind(handle)
        .fetch_optional(&self.pool)
        .await?;

        let result = match row {
            Some(r) => {
                let sync_data = TwitterLastSync {
                    handle: r.get("handle"),
                    last_sync_date: r.get("last_sync_date"),
                };
                Some(sync_data)
            },
            None => None
        };
        
        Ok(result)
    }

    /// Update the last sync date for a given handle (creates record if it doesn't exist)
    pub async fn update_last_sync(&self, handle: &str, sync_date: DateTime<Utc>) -> Result<()> {
        sqlx::query(
            r#"
            INSERT INTO twitter_last_sync (handle, last_sync_date, updated_at)
            VALUES ($1, $2, NOW())
            ON CONFLICT (handle) 
            DO UPDATE SET 
                last_sync_date = EXCLUDED.last_sync_date,
                updated_at = NOW()
            "#,
        )
        .bind(handle)
        .bind(sync_date)
        .execute(&self.pool)
        .await?;
        
        Ok(())
    }

}
