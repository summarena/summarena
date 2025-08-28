use anyhow::Result;
use sqlx::{PgPool, Row};
use chrono::{DateTime, Utc};
use uuid::Uuid;
use crate::types::FeedMetadata;

pub struct RssDatabase {
    pool: PgPool,
}

impl RssDatabase {
    pub async fn new(database_url: &str) -> Result<Self> {
        let pool = PgPool::connect(database_url).await?;
        Ok(Self { pool })
    }

    pub async fn setup_schema(&self) -> Result<()> {
        sqlx::query(
            r#"
            CREATE TABLE IF NOT EXISTS feed_metadata (
                id UUID PRIMARY KEY,
                url VARCHAR(2048) UNIQUE NOT NULL,
                title TEXT,
                description TEXT,
                last_fetch_time TIMESTAMP WITH TIME ZONE,
                last_successful_fetch TIMESTAMP WITH TIME ZONE,
                update_frequency_hours INTEGER,
                error_count INTEGER DEFAULT 0,
                last_error TEXT,
                is_active BOOLEAN DEFAULT true,
                created_at TIMESTAMP WITH TIME ZONE DEFAULT NOW(),
                updated_at TIMESTAMP WITH TIME ZONE DEFAULT NOW(),
                etag VARCHAR(255),
                last_modified VARCHAR(255)
            )
            "#,
        )
        .execute(&self.pool)
        .await?;
        Ok(())
    }

    /// Save or update feed metadata
    pub async fn save_feed_metadata(&self, metadata: &FeedMetadata) -> Result<()> {
        let update_frequency_hours = metadata.update_frequency_hours.map(|h| h as i32);
        
        sqlx::query(
            r#"
            INSERT INTO feed_metadata (
                id, url, title, description, last_fetch_time, last_successful_fetch, 
                update_frequency_hours, error_count, last_error, is_active, 
                created_at, updated_at, etag, last_modified
            )
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13, $14)
            ON CONFLICT (url) 
            DO UPDATE SET 
                title = EXCLUDED.title,
                description = EXCLUDED.description,
                last_fetch_time = EXCLUDED.last_fetch_time,
                last_successful_fetch = EXCLUDED.last_successful_fetch,
                update_frequency_hours = EXCLUDED.update_frequency_hours,
                error_count = EXCLUDED.error_count,
                last_error = EXCLUDED.last_error,
                is_active = EXCLUDED.is_active,
                updated_at = EXCLUDED.updated_at,
                etag = EXCLUDED.etag,
                last_modified = EXCLUDED.last_modified
            "#,
        )
        .bind(&metadata.id)
        .bind(&metadata.url)
        .bind(&metadata.title)
        .bind(&metadata.description)
        .bind(&metadata.last_fetch_time)
        .bind(&metadata.last_successful_fetch)
        .bind(update_frequency_hours)
        .bind(metadata.error_count as i32)
        .bind(&metadata.last_error)
        .bind(&metadata.is_active)
        .bind(&metadata.created_at)
        .bind(&metadata.updated_at)
        .bind(&metadata.etag)
        .bind(&metadata.last_modified)
        .execute(&self.pool)
        .await?;
        Ok(())
    }

    /// Get feed metadata by URL
    pub async fn get_feed_by_url(&self, url: &str) -> Result<Option<FeedMetadata>> {
        let row = sqlx::query(
            r#"
            SELECT id, url, title, description, last_fetch_time, last_successful_fetch,
                   update_frequency_hours, error_count, last_error, is_active,
                   created_at, updated_at, etag, last_modified
            FROM feed_metadata 
            WHERE url = $1
            "#
        )
        .bind(url)
        .fetch_optional(&self.pool)
        .await?;

        match row {
            Some(r) => Ok(Some(FeedMetadata {
                id: r.get("id"),
                url: r.get("url"),
                title: r.get("title"),
                description: r.get("description"),
                last_fetch_time: r.get("last_fetch_time"),
                last_successful_fetch: r.get("last_successful_fetch"),
                update_frequency_hours: r.get::<Option<i32>, _>("update_frequency_hours").map(|v| v as u32),
                error_count: r.get::<i32, _>("error_count") as u32,
                last_error: r.get("last_error"),
                is_active: r.get("is_active"),
                created_at: r.get("created_at"),
                updated_at: r.get("updated_at"),
                etag: r.get("etag"),
                last_modified: r.get("last_modified"),
            })),
            None => Ok(None),
        }
    }

    /// Get feed metadata by ID
    pub async fn get_feed_by_id(&self, id: Uuid) -> Result<Option<FeedMetadata>> {
        let row = sqlx::query(
            r#"
            SELECT id, url, title, description, last_fetch_time, last_successful_fetch,
                   update_frequency_hours, error_count, last_error, is_active,
                   created_at, updated_at, etag, last_modified
            FROM feed_metadata 
            WHERE id = $1
            "#
        )
        .bind(id)
        .fetch_optional(&self.pool)
        .await?;

        match row {
            Some(r) => Ok(Some(FeedMetadata {
                id: r.get("id"),
                url: r.get("url"),
                title: r.get("title"),
                description: r.get("description"),
                last_fetch_time: r.get("last_fetch_time"),
                last_successful_fetch: r.get("last_successful_fetch"),
                update_frequency_hours: r.get::<Option<i32>, _>("update_frequency_hours").map(|v| v as u32),
                error_count: r.get::<i32, _>("error_count") as u32,
                last_error: r.get("last_error"),
                is_active: r.get("is_active"),
                created_at: r.get("created_at"),
                updated_at: r.get("updated_at"),
                etag: r.get("etag"),
                last_modified: r.get("last_modified"),
            })),
            None => Ok(None),
        }
    }

    /// Update the last successful fetch time for a feed
    pub async fn update_last_successful_fetch(&self, feed_id: Uuid, fetch_time: DateTime<Utc>) -> Result<()> {
        sqlx::query(
            r#"
            UPDATE feed_metadata 
            SET last_successful_fetch = $2, updated_at = NOW()
            WHERE id = $1
            "#,
        )
        .bind(feed_id)
        .bind(fetch_time)
        .execute(&self.pool)
        .await?;
        Ok(())
    }

    /// Get all active feeds
    pub async fn get_active_feeds(&self) -> Result<Vec<FeedMetadata>> {
        let rows = sqlx::query(
            r#"
            SELECT id, url, title, description, last_fetch_time, last_successful_fetch,
                   update_frequency_hours, error_count, last_error, is_active,
                   created_at, updated_at, etag, last_modified
            FROM feed_metadata 
            WHERE is_active = true
            ORDER BY created_at
            "#
        )
        .fetch_all(&self.pool)
        .await?;

        let feeds = rows.into_iter().map(|r| FeedMetadata {
            id: r.get("id"),
            url: r.get("url"),
            title: r.get("title"),
            description: r.get("description"),
            last_fetch_time: r.get("last_fetch_time"),
            last_successful_fetch: r.get("last_successful_fetch"),
            update_frequency_hours: r.get::<Option<i32>, _>("update_frequency_hours").map(|v| v as u32),
            error_count: r.get::<i32, _>("error_count") as u32,
            last_error: r.get("last_error"),
            is_active: r.get("is_active"),
            created_at: r.get("created_at"),
            updated_at: r.get("updated_at"),
            etag: r.get("etag"),
            last_modified: r.get("last_modified"),
        }).collect();
        
        Ok(feeds)
    }

    /// Delete a feed by URL
    pub async fn delete_feed_by_url(&self, url: &str) -> Result<()> {
        sqlx::query("DELETE FROM feed_metadata WHERE url = $1")
            .bind(url)
            .execute(&self.pool)
            .await?;
        Ok(())
    }

    /// Deactivate a feed (soft delete)
    pub async fn deactivate_feed(&self, feed_id: Uuid) -> Result<()> {
        sqlx::query(
            r#"
            UPDATE feed_metadata 
            SET is_active = false, updated_at = NOW()
            WHERE id = $1
            "#,
        )
        .bind(feed_id)
        .execute(&self.pool)
        .await?;
        Ok(())
    }
}