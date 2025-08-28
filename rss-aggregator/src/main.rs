use rss_aggregator::{WsjFeedSource, PullFeed, FetchConfig};
use tracing::{info, error};
use tracing_subscriber;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    tracing_subscriber::fmt::init();
    
    info!("RSS Ingester Demo - Testing PullFeed trait");
    
    // Create fetch configuration
    let fetch_config = FetchConfig {
        user_agent: "RSS-Ingester-Demo/1.0".to_string(),
        timeout_seconds: 30,
        max_retries: 2,
        retry_delay_seconds: 1,
        respect_robots_txt: false,
        max_feed_size_mb: 10,
        follow_redirects: true,
        max_redirects: 5,
    };
    
    // Create WSJ feed source
    let mut wsj_source = WsjFeedSource::main_feed(fetch_config);
    
    info!("Created RSS source: {}", wsj_source.source_name());
    info!("Source ID: {}", wsj_source.source_id());
    info!("Poll interval: {}ms", wsj_source.poll_interval_ms());
    
    // Get metadata
    match wsj_source.get_metadata().await {
        Ok(metadata) => {
            info!("Feed URL: {}", metadata.feed_url);
            if let Some(title) = &metadata.title {
                info!("Feed Title: {}", title);
            }
            if let Some(description) = &metadata.description {
                info!("Feed Description: {}", description);
            }
        }
        Err(e) => error!("Failed to get metadata: {}", e),
    }
    
    // Test health check
    match wsj_source.health_check().await {
        Ok(healthy) => info!("Health check: {}", if healthy { "OK" } else { "Failed" }),
        Err(e) => error!("Health check error: {}", e),
    }
    
    // Try to pull some items (this will make an actual HTTP request)
    info!("Attempting to pull items from RSS feed...");
    match wsj_source.pull().await {
        Ok(items) => {
            info!("Successfully pulled {} items", items.len());
            for (i, item) in items.iter().take(3).enumerate() {
                info!("Item {}: {} ({})", i + 1, item.uri, 
                     item.text.chars().take(100).collect::<String>() + "...");
            }
        }
        Err(e) => error!("Failed to pull items: {}", e),
    }
    
    info!("RSS Ingester Demo finished");
    Ok(())
}