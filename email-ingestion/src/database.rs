use anyhow::Result;
use sqlx::{PgPool, Row};
use chrono::{DateTime, Utc};

#[derive(Debug, Clone)]
pub struct EmailCredential {
    pub email_address: String,
    pub password: String,
    pub last_sync_date: Option<DateTime<Utc>>,
}

pub struct EmailDatabase {
    pool: PgPool,
}

impl EmailDatabase {
    pub async fn new(database_url: &str) -> Result<Self> {
        let pool = PgPool::connect(database_url).await?;
        Ok(Self { pool })
    }

    pub async fn setup_schema(&self) -> Result<()> {
        sqlx::query(
            r#"
            CREATE TABLE IF NOT EXISTS email_credentials (
                email_address VARCHAR(255) PRIMARY KEY,
                password VARCHAR(255) NOT NULL,
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

    /// Store or update email credentials for a given email address
    pub async fn store_credentials(&self, email_address: &str, password: &str) -> Result<()> {
        sqlx::query(
            r#"
            INSERT INTO email_credentials (email_address, password, updated_at)
            VALUES ($1, $2, NOW())
            ON CONFLICT (email_address) 
            DO UPDATE SET 
                password = EXCLUDED.password,
                updated_at = NOW()
            "#,
        )
        .bind(email_address)
        .bind(password)
        .execute(&self.pool)
        .await?;
        Ok(())
    }

    /// Get email credentials for a given email address
    pub async fn get_credentials(&self, email_address: &str) -> Result<Option<EmailCredential>> {
        let row = sqlx::query(
            "SELECT email_address, password, last_sync_date FROM email_credentials WHERE email_address = $1"
        )
        .bind(email_address)
        .fetch_optional(&self.pool)
        .await?;

        match row {
            Some(r) => Ok(Some(EmailCredential {
                email_address: r.get("email_address"),
                password: r.get("password"),
                last_sync_date: r.get("last_sync_date"),
            })),
            None => Ok(None),
        }
    }

    /// Update the last sync date for a given email address
    pub async fn update_last_sync(&self, email_address: &str, sync_date: DateTime<Utc>) -> Result<()> {
        sqlx::query(
            r#"
            UPDATE email_credentials 
            SET last_sync_date = $2, updated_at = NOW()
            WHERE email_address = $1
            "#,
        )
        .bind(email_address)
        .bind(sync_date)
        .execute(&self.pool)
        .await?;
        Ok(())
    }

    /// Check if credentials exist for an email address
    pub async fn credentials_exist(&self, email_address: &str) -> Result<bool> {
        let row = sqlx::query("SELECT COUNT(*) as count FROM email_credentials WHERE email_address = $1")
            .bind(email_address)
            .fetch_one(&self.pool)
            .await?;
        Ok(row.get::<i64, _>("count") > 0)
    }

    /// Delete credentials for an email address
    pub async fn delete_credentials(&self, email_address: &str) -> Result<()> {
        sqlx::query("DELETE FROM email_credentials WHERE email_address = $1")
            .bind(email_address)
            .execute(&self.pool)
            .await?;
        Ok(())
    }

    /// Get all stored email addresses (without passwords)
    pub async fn list_email_addresses(&self) -> Result<Vec<String>> {
        let rows = sqlx::query(
            "SELECT email_address FROM email_credentials ORDER BY email_address"
        )
        .fetch_all(&self.pool)
        .await?;

        let addresses = rows.into_iter().map(|r| r.get("email_address")).collect();
        Ok(addresses)
    }
}