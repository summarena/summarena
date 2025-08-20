use rss_aggregator::{RssAggregator, FetchConfig};
use std::env;
use tracing::{info, error};
use tracing_subscriber;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    tracing_subscriber::fmt::init();
    
    info!("Starting RSS Aggregator (PostgreSQL mode)");
    
    // Get database URL from environment or use default PostgreSQL connection
    let database_url = env::var("DATABASE_URL")
        .unwrap_or_else(|_| "postgresql://rss_user:rss_password@localhost:5432/rss_aggregator".to_string());
    
    info!("Connecting to database: {}", 
          database_url.replace("rss_password", "***")); // Hide password in logs
    
    // Create fetch configuration
    let fetch_config = FetchConfig::default();
    
    // Initialize the aggregator
    let aggregator = RssAggregator::new(&database_url, fetch_config).await
        .map_err(|e| {
            error!("Failed to connect to database. Make sure PostgreSQL is running:");
            error!("  Run: make postgres");
            error!("  Or check connection string: {}", database_url.replace("rss_password", "***"));
            Box::new(e) as Box<dyn std::error::Error>
        })?;
    
    info!("Successfully connected to PostgreSQL database");
    
    // Example usage: Add some sample feeds
    let sample_feeds = vec![
        "https://feeds.bbci.co.uk/news/rss.xml",
        "https://rss.cnn.com/rss/edition.rss",
        "https://feeds.npr.org/1001/rss.xml",
    ];
    
    for feed_url in sample_feeds {
        match aggregator.add_feed(feed_url.to_string()).await {
            Ok(feed_id) => info!("Added feed: {} (ID: {})", feed_url, feed_id),
            Err(e) => error!("Failed to add feed {}: {}", feed_url, e),
        }
    }
    
    // Fetch all feeds once
    match aggregator.fetch_all_feeds().await {
        Ok(count) => info!("Successfully fetched {} feeds", count),
        Err(e) => error!("Failed to fetch feeds: {}", e),
    }
    
    // Display stats
    match aggregator.get_feed_stats().await {
        Ok(stats) => {
            info!("Feed statistics:");
            for (key, value) in stats {
                info!("  {}: {}", key, value);
            }
        }
        Err(e) => error!("Failed to get stats: {}", e),
    }
    
    info!("RSS Aggregator finished");
    Ok(())
}
