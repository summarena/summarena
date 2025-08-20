use crate::types::{AggregatorError, FeedEntry, FeedMetadata, Result, ScheduleInfo};
use chrono::{Duration, Utc};
use sqlx::{Pool, Postgres, PgPool, Row};
use std::collections::HashMap;
use tracing::{debug, info};
use uuid::Uuid;

pub struct FeedManager {
    db: Pool<Postgres>,
}

impl FeedManager {
    pub async fn new(database_url: &str) -> Result<Self> {
        let db = PgPool::connect(database_url).await?;
        
        // Note: Database schema should be initialized with migrations before running
        // In production, run: sqlx migrate run
        
        Ok(Self { db })
    }

    pub async fn add_feed(&self, url: String, title: Option<String>, description: Option<String>) -> Result<Uuid> {
        let feed_id = Uuid::new_v4();
        let now = Utc::now();
        
        sqlx::query(
            r#"
            INSERT INTO feeds (id, url, title, description, is_active, created_at, updated_at, error_count)
            VALUES ($1, $2, $3, $4, true, $5, $6, 0)
            "#,
        )
        .bind(feed_id)
        .bind(&url)
        .bind(title)
        .bind(description)
        .bind(now)
        .bind(now)
        .execute(&self.db)
        .await?;
        
        info!("Added new feed: {} with ID: {}", url, feed_id);
        Ok(feed_id)
    }

    pub async fn get_feed(&self, feed_id: Uuid) -> Result<FeedMetadata> {
        let row = sqlx::query!(
            "SELECT * FROM feeds WHERE id = $1",
            feed_id
        )
        .fetch_optional(&self.db)
        .await?;

        match row {
            Some(row) => Ok(FeedMetadata {
                id: row.id,
                url: row.url,
                title: row.title,
                description: row.description,
                last_fetch_time: row.last_fetch_time.map(|t| t.with_timezone(&Utc)),
                last_successful_fetch: row.last_successful_fetch.map(|t| t.with_timezone(&Utc)),
                update_frequency_hours: row.update_frequency_hours.map(|h| h as u32),
                error_count: row.error_count as u32,
                last_error: row.last_error,
                is_active: row.is_active,
                created_at: row.created_at.with_timezone(&Utc),
                updated_at: row.updated_at.with_timezone(&Utc),
                etag: row.etag,
                last_modified: row.last_modified,
            }),
            None => Err(AggregatorError::FeedNotFound { id: feed_id }),
        }
    }

    pub async fn list_active_feeds(&self) -> Result<Vec<FeedMetadata>> {
        let rows = sqlx::query!(
            "SELECT * FROM feeds WHERE is_active = true ORDER BY created_at"
        )
        .fetch_all(&self.db)
        .await?;

        let mut feeds = Vec::new();
        for row in rows {
            feeds.push(FeedMetadata {
                id: row.id,
                url: row.url,
                title: row.title,
                description: row.description,
                last_fetch_time: row.last_fetch_time.map(|t| t.with_timezone(&Utc)),
                last_successful_fetch: row.last_successful_fetch.map(|t| t.with_timezone(&Utc)),
                update_frequency_hours: row.update_frequency_hours.map(|h| h as u32),
                error_count: row.error_count as u32,
                last_error: row.last_error,
                is_active: row.is_active,
                created_at: row.created_at.with_timezone(&Utc),
                updated_at: row.updated_at.with_timezone(&Utc),
                etag: row.etag,
                last_modified: row.last_modified,
            });
        }

        Ok(feeds)
    }

    pub async fn update_fetch_result(&self, feed_id: Uuid, success: bool, error: Option<String>, etag: Option<String>, last_modified: Option<String>) -> Result<()> {
        let now = Utc::now();
        
        if success {
            sqlx::query!(
                r#"
                UPDATE feeds 
                SET last_fetch_time = $1, last_successful_fetch = $2, error_count = 0, last_error = NULL, 
                    etag = $3, last_modified = $4, updated_at = $5
                WHERE id = $6
                "#,
                now,
                now,
                etag,
                last_modified,
                now,
                feed_id
            )
            .execute(&self.db)
            .await?;
        } else {
            sqlx::query!(
                r#"
                UPDATE feeds 
                SET last_fetch_time = $1, error_count = error_count + 1, last_error = $2, updated_at = $3
                WHERE id = $4
                "#,
                now,
                error,
                now,
                feed_id
            )
            .execute(&self.db)
            .await?;
        }

        Ok(())
    }

    pub async fn update_feed_frequency(&self, feed_id: Uuid, frequency_hours: u32) -> Result<()> {
        let now = Utc::now();
        
        sqlx::query!(
            "UPDATE feeds SET update_frequency_hours = $1, updated_at = $2 WHERE id = $3",
            frequency_hours as i64,
            now,
            feed_id
        )
        .execute(&self.db)
        .await?;

        debug!("Updated feed {} frequency to {} hours", feed_id, frequency_hours);
        Ok(())
    }

    pub async fn get_feeds_to_fetch(&self, limit: usize) -> Result<Vec<ScheduleInfo>> {
        let now = Utc::now();
        
        let rows = sqlx::query!(
            r#"
            SELECT id, last_fetch_time, update_frequency_hours, error_count 
            FROM feeds 
            WHERE is_active = true 
            ORDER BY 
                CASE 
                    WHEN last_fetch_time IS NULL THEN 0
                    WHEN error_count > 0 THEN 1
                    ELSE 2
                END,
                last_fetch_time ASC NULLS FIRST
            LIMIT $1
            "#,
            limit as i64
        )
        .fetch_all(&self.db)
        .await?;

        let mut schedule_info = Vec::new();
        
        for row in rows {
            let feed_id = row.id;
            let last_fetch = row.last_fetch_time
                .map(|t| t.with_timezone(&Utc));
            let frequency_hours = row.update_frequency_hours.unwrap_or(1) as i64;
            let error_count = row.error_count as u32;
            
            let next_fetch_time = match last_fetch {
                Some(last) => {
                    let interval = if error_count > 0 {
                        // Exponential backoff for failed feeds
                        Duration::hours(frequency_hours * (2_i64.pow(error_count.min(5))))
                    } else {
                        Duration::hours(frequency_hours)
                    };
                    last + interval
                }
                None => now, // Fetch immediately if never fetched
            };
            
            let priority = if error_count > 0 {
                50 // Lower priority for failing feeds
            } else if last_fetch.is_none() {
                255 // Highest priority for new feeds
            } else {
                150 // Normal priority
            };
            
            if next_fetch_time <= now {
                schedule_info.push(ScheduleInfo {
                    feed_id,
                    next_fetch_time,
                    priority,
                });
            }
        }
        
        // Sort by priority (highest first) then by next_fetch_time
        schedule_info.sort_by(|a, b| {
            b.priority.cmp(&a.priority)
                .then_with(|| a.next_fetch_time.cmp(&b.next_fetch_time))
        });
        
        Ok(schedule_info)
    }

    pub async fn deactivate_feed(&self, feed_id: Uuid) -> Result<()> {
        let now = Utc::now();
        
        sqlx::query!(
            "UPDATE feeds SET is_active = false, updated_at = $1 WHERE id = $2",
            now,
            feed_id
        )
        .execute(&self.db)
        .await?;

        info!("Deactivated feed: {}", feed_id);
        Ok(())
    }

    pub async fn validate_feed_url(&self, url: &str) -> Result<bool> {
        use url::Url;
        
        let parsed_url = Url::parse(url)?;
        
        // Basic validation
        if !matches!(parsed_url.scheme(), "http" | "https") {
            return Ok(false);
        }
        
        if parsed_url.host().is_none() {
            return Ok(false);
        }
        
        // Check if URL already exists
        let existing = sqlx::query!(
            "SELECT COUNT(*) as count FROM feeds WHERE url = $1",
            url
        )
        .fetch_one(&self.db)
        .await?;
        
        Ok(existing.count.unwrap_or(0) == 0)
    }

    pub async fn get_feed_stats(&self) -> Result<HashMap<String, i64>> {
        let mut stats = HashMap::new();
        
        let total_feeds = sqlx::query_scalar!(
            "SELECT COUNT(*) FROM feeds"
        )
        .fetch_one(&self.db)
        .await?;
        stats.insert("total_feeds".to_string(), total_feeds.unwrap_or(0));
        
        let active_feeds = sqlx::query_scalar!(
            "SELECT COUNT(*) FROM feeds WHERE is_active = true"
        )
        .fetch_one(&self.db)
        .await?;
        stats.insert("active_feeds".to_string(), active_feeds.unwrap_or(0));
        
        let failing_feeds = sqlx::query_scalar!(
            "SELECT COUNT(*) FROM feeds WHERE error_count > 0 AND is_active = true"
        )
        .fetch_one(&self.db)
        .await?;
        stats.insert("failing_feeds".to_string(), failing_feeds.unwrap_or(0));
        
        let never_fetched = sqlx::query_scalar!(
            "SELECT COUNT(*) FROM feeds WHERE last_fetch_time IS NULL AND is_active = true"
        )
        .fetch_one(&self.db)
        .await?;
        stats.insert("never_fetched".to_string(), never_fetched.unwrap_or(0));
        
        Ok(stats)
    }

    /// Store feed entries in the database, avoiding duplicates
    pub async fn store_feed_entries(&self, entries: &[FeedEntry]) -> Result<usize> {
        let mut stored_count = 0;
        
        for entry in entries {
            let result = sqlx::query(
                r#"
                INSERT INTO feed_entries (id, feed_id, guid, url, title, description, content, author, published_at, updated_at, tags, created_at)
                VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12)
                ON CONFLICT (feed_id, url) DO NOTHING
                "#,
            )
            .bind(entry.id)
            .bind(entry.feed_id)
            .bind(&entry.guid)
            .bind(&entry.url)
            .bind(&entry.title)
            .bind(&entry.description)
            .bind(&entry.content)
            .bind(&entry.author)
            .bind(entry.published_at)
            .bind(entry.updated_at)
            .bind(serde_json::to_value(&entry.tags).unwrap_or_default())
            .bind(entry.created_at)
            .execute(&self.db)
            .await?;
            
            if result.rows_affected() > 0 {
                stored_count += 1;
            }
        }
        
        info!("Stored {} new entries out of {} total entries", stored_count, entries.len());
        Ok(stored_count)
    }

    /// Get recent feed entries from the database
    pub async fn get_recent_feed_entries(&self, feed_id: Option<Uuid>, limit: usize) -> Result<Vec<FeedEntry>> {
        let rows = if let Some(feed_id) = feed_id {
            sqlx::query(
                "SELECT * FROM feed_entries WHERE feed_id = $1 ORDER BY created_at DESC LIMIT $2"
            )
            .bind(feed_id)
            .bind(limit as i64)
            .fetch_all(&self.db)
            .await?
        } else {
            sqlx::query(
                "SELECT * FROM feed_entries ORDER BY created_at DESC LIMIT $1"
            )
            .bind(limit as i64)
            .fetch_all(&self.db)
            .await?
        };
        
        let mut entries = Vec::new();
        for row in rows {
            let tags_json: serde_json::Value = row.try_get("tags").unwrap_or_default();
            let tags: Vec<String> = serde_json::from_value(tags_json).unwrap_or_default();
            
            entries.push(FeedEntry {
                id: row.try_get("id")?,
                feed_id: row.try_get("feed_id")?,
                guid: row.try_get("guid")?,
                url: row.try_get("url")?,
                title: row.try_get("title")?,
                description: row.try_get("description")?,
                content: row.try_get("content")?,
                author: row.try_get("author")?,
                published_at: row.try_get::<Option<chrono::DateTime<chrono::Utc>>, _>("published_at")?,
                updated_at: row.try_get::<Option<chrono::DateTime<chrono::Utc>>, _>("updated_at")?,
                tags,
                created_at: row.try_get::<chrono::DateTime<chrono::Utc>, _>("created_at")?,
                last_processed: row.try_get::<Option<chrono::DateTime<chrono::Utc>>, _>("last_processed")?,
            });
        }
        
        Ok(entries)
    }

    /// Get entry count for a specific feed
    pub async fn get_feed_entry_count(&self, feed_id: Uuid) -> Result<i64> {
        let count = sqlx::query_scalar!(
            "SELECT COUNT(*) FROM feed_entries WHERE feed_id = $1",
            feed_id
        )
        .fetch_one(&self.db)
        .await?;
        
        Ok(count.unwrap_or(0))
    }
}