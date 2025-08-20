use rss_aggregator::{FetchConfig, RssAggregator, types::*};
use sqlx::{PgPool, Row};
use std::env;
use tokio;
use tracing::info;
use tracing_subscriber;
use uuid::Uuid;

const WSJ_RSS_URL: &str = "https://feeds.content.dowjones.io/public/rss/RSSWorldNews";

#[tokio::test]
async fn test_rss_aggregator_end_to_end() -> Result<()> {
    // Initialize tracing
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::INFO)
        .init();

    // Get database URL from environment or use default
    let database_url = env::var("DATABASE_URL")
        .unwrap_or_else(|_| "postgresql://rss_user:rss_password@localhost:5432/rss_aggregator".to_string());

    info!("Starting RssAggregator end-to-end test");
    
    // Clear database and set up schema
    clear_database_and_migrate(&database_url).await?;
    
    // Create RssAggregator with proper configuration
    let fetch_config = FetchConfig {
        user_agent: "RSS-Aggregator-E2E-Test/1.0".to_string(),
        timeout_seconds: 30,
        max_retries: 2,
        retry_delay_seconds: 1,
        respect_robots_txt: false, // Disable for testing
        max_feed_size_mb: 10,
        follow_redirects: true,
        max_redirects: 5,
    };
    
    let aggregator = RssAggregator::new(&database_url, fetch_config).await?;
    
    // Add the WSJ RSS feed using the aggregator
    info!("Adding WSJ RSS feed via RssAggregator");
    let feed_id = aggregator.add_feed(WSJ_RSS_URL.to_string()).await?;
    info!("Added feed with ID: {}", feed_id);
    
    // Use the aggregator to fetch all feeds (should fetch our one feed)
    info!("Fetching all feeds via RssAggregator");
    let successful_fetches = aggregator.fetch_all_feeds().await?;
    info!("Successfully fetched {} feeds", successful_fetches);
    
    // Verify entries were stored using the aggregator's methods
    let recent_items = aggregator.get_recent_items(10).await?;
    info!("Retrieved {} recent items via RssAggregator", recent_items.len());
    
    // Get detailed database verification
    let entries_count = get_feed_entry_count(&database_url, feed_id).await?;
    info!("Successfully processed {} entries", entries_count);
    
    let stored_entries = verify_entries_in_database(&database_url, feed_id).await?;
    info!("Found {} entries in database", stored_entries.len());
    
    // Print some of the items from aggregator API
    info!("=== RECENT ITEMS VIA RSS AGGREGATOR ===");
    for (index, item) in recent_items.iter().take(5).enumerate() {
        println!("\n--- Item {} ---", index + 1);
        println!("URI: {}", item.uri);
        let lines: Vec<&str> = item.text.lines().take(3).collect();
        for line in lines {
            if !line.trim().is_empty() {
                println!("{}", line);
            }
        }
    }
    
    // Print detailed database entries
    print_feed_items_from_database(&database_url, feed_id).await?;
    
    // Get feed stats
    let stats = aggregator.get_feed_stats().await?;
    info!("Feed stats: {:?}", stats);
    
    // Comprehensive assertions
    assert_eq!(successful_fetches, 1, "Should have successfully fetched 1 feed");
    assert!(recent_items.len() > 0, "Should have retrieved recent items via aggregator API");
    assert!(stored_entries.len() > 0, "No entries were stored in the database");
    assert_eq!(entries_count as usize, stored_entries.len(), "Mismatch between processed and stored entries");
    assert!(stats.get("active_feeds").unwrap_or(&0) > &0, "Should have active feeds");
    
    info!("RssAggregator end-to-end test completed successfully!");
    Ok(())
}

async fn get_feed_entry_count(database_url: &str, feed_id: Uuid) -> Result<i64> {
    let pool = PgPool::connect(database_url).await?;
    let count = sqlx::query_scalar!(
        "SELECT COUNT(*) FROM feed_entries WHERE feed_id = $1",
        feed_id
    )
    .fetch_one(&pool)
    .await?;
    
    pool.close().await;
    Ok(count.unwrap_or(0))
}

async fn clear_database_and_migrate(database_url: &str) -> Result<()> {
    info!("Clearing database and running migrations");
    
    let pool = PgPool::connect(database_url).await?;
    
    // Drop all tables
    sqlx::query("DROP TABLE IF EXISTS feed_entries CASCADE")
        .execute(&pool)
        .await?;
    
    sqlx::query("DROP TABLE IF EXISTS feeds CASCADE")
        .execute(&pool)
        .await?;
    
    info!("Dropped existing tables");
    
    // Create schema manually
    // Create feeds table
    sqlx::query(r#"
        CREATE TABLE IF NOT EXISTS feeds (
            id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
            url TEXT NOT NULL UNIQUE,
            title TEXT,
            description TEXT,
            last_fetch_time TIMESTAMPTZ,
            last_successful_fetch TIMESTAMPTZ,
            update_frequency_hours INTEGER,
            error_count INTEGER NOT NULL DEFAULT 0,
            last_error TEXT,
            is_active BOOLEAN NOT NULL DEFAULT true,
            created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
            updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
            etag TEXT,
            last_modified TEXT
        )
    "#)
    .execute(&pool)
    .await?;
    
    // Create feed_entries table
    sqlx::query(r#"
        CREATE TABLE IF NOT EXISTS feed_entries (
            id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
            feed_id UUID NOT NULL,
            guid TEXT,
            url TEXT NOT NULL,
            title TEXT NOT NULL,
            description TEXT,
            content TEXT,
            author TEXT,
            published_at TIMESTAMPTZ,
            updated_at TIMESTAMPTZ,
            tags JSONB,
            created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
            last_processed TIMESTAMPTZ,
            FOREIGN KEY (feed_id) REFERENCES feeds (id) ON DELETE CASCADE
        )
    "#)
    .execute(&pool)
    .await?;
    
    // Create indexes
    sqlx::query("CREATE INDEX IF NOT EXISTS idx_feeds_active ON feeds (is_active)")
        .execute(&pool)
        .await?;
    sqlx::query("CREATE INDEX IF NOT EXISTS idx_feeds_last_fetch ON feeds (last_fetch_time)")
        .execute(&pool)
        .await?;
    sqlx::query("CREATE INDEX IF NOT EXISTS idx_feeds_url ON feeds (url)")
        .execute(&pool)
        .await?;
    sqlx::query("CREATE INDEX IF NOT EXISTS idx_entries_feed_id ON feed_entries (feed_id)")
        .execute(&pool)
        .await?;
    sqlx::query("CREATE INDEX IF NOT EXISTS idx_entries_guid ON feed_entries (guid)")
        .execute(&pool)
        .await?;
    sqlx::query("CREATE INDEX IF NOT EXISTS idx_entries_url ON feed_entries (url)")
        .execute(&pool)
        .await?;
    sqlx::query("CREATE INDEX IF NOT EXISTS idx_entries_published ON feed_entries (published_at)")
        .execute(&pool)
        .await?;
    sqlx::query("CREATE INDEX IF NOT EXISTS idx_entries_processed ON feed_entries (last_processed)")
        .execute(&pool)
        .await?;
    sqlx::query("CREATE UNIQUE INDEX IF NOT EXISTS idx_entries_unique_guid_feed ON feed_entries (feed_id, guid) WHERE guid IS NOT NULL")
        .execute(&pool)
        .await?;
    sqlx::query("CREATE UNIQUE INDEX IF NOT EXISTS idx_entries_unique_url_feed ON feed_entries (feed_id, url)")
        .execute(&pool)
        .await?;
    
    info!("Schema creation completed");
    
    pool.close().await;
    Ok(())
}


async fn verify_entries_in_database(database_url: &str, feed_id: Uuid) -> Result<Vec<FeedEntry>> {
    let pool = PgPool::connect(database_url).await?;
    
    let rows = sqlx::query(
        "SELECT * FROM feed_entries WHERE feed_id = $1 ORDER BY created_at DESC"
    )
    .bind(feed_id)
    .fetch_all(&pool)
    .await?;
    
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
    
    pool.close().await;
    Ok(entries)
}

async fn print_feed_items_from_database(database_url: &str, feed_id: Uuid) -> Result<()> {
    info!("=== RSS FEED ITEMS FROM DATABASE ===");
    
    let pool = PgPool::connect(database_url).await?;
    
    let rows = sqlx::query(
        r#"
        SELECT title, url, description, author, published_at, tags
        FROM feed_entries 
        WHERE feed_id = $1 
        ORDER BY published_at DESC NULLS LAST, created_at DESC
        LIMIT 10
        "#
    )
    .bind(feed_id)
    .fetch_all(&pool)
    .await?;
    
    info!("Found {} entries (showing first 10):", rows.len());
    
    for (index, row) in rows.iter().enumerate() {
        println!("\n--- Entry {} ---", index + 1);
        println!("Title: {}", row.try_get::<String, _>("title").unwrap_or_default());
        println!("URL: {}", row.try_get::<String, _>("url").unwrap_or_default());
        
        if let Ok(Some(description)) = row.try_get::<Option<String>, _>("description") {
            let desc = if description.len() > 200 {
                format!("{}...", &description[..200])
            } else {
                description
            };
            println!("Description: {}", desc);
        }
        
        if let Ok(Some(author)) = row.try_get::<Option<String>, _>("author") {
            println!("Author: {}", author);
        }
        
        if let Ok(Some(published_at)) = row.try_get::<Option<chrono::DateTime<chrono::Utc>>, _>("published_at") {
            println!("Published: {}", published_at.format("%Y-%m-%d %H:%M:%S UTC"));
        }
        
        if let Ok(tags_json) = row.try_get::<serde_json::Value, _>("tags") {
            let tags: Vec<String> = serde_json::from_value(tags_json).unwrap_or_default();
            if !tags.is_empty() {
                println!("Tags: {}", tags.join(", "));
            }
        }
    }
    
    pool.close().await;
    Ok(())
}