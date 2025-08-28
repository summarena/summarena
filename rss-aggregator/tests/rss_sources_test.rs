use rss_aggregator::{
    FetchConfig, WsjFeedSource, RssFeedSource, PullFeed,
    rss_utils::{url, time}
};
use std::sync::Once;
use tokio;
use tracing::info;
use tracing_subscriber;
use uuid::Uuid;

static INIT: Once = Once::new();

fn init_tracing() {
    INIT.call_once(|| {
        tracing_subscriber::fmt()
            .with_max_level(tracing::Level::INFO)
            .try_init()
            .ok();
    });
}

#[tokio::test]
async fn test_wsj_feed_source_creation() {
    init_tracing();

    info!("Testing WSJ feed source creation");
    
    // Create fetch configuration
    let fetch_config = FetchConfig {
        user_agent: "RSS-Test/1.0".to_string(),
        timeout_seconds: 30,
        max_retries: 2,
        retry_delay_seconds: 1,
        respect_robots_txt: false,
        max_feed_size_mb: 10,
        follow_redirects: true,
        max_redirects: 5,
    };
    
    // Create WSJ feed source
    let wsj_source = WsjFeedSource::main_feed(fetch_config);
    
    // Test basic properties
    assert!(!wsj_source.source_id().is_empty());
    assert_eq!(wsj_source.source_name(), "WSJ: Wall Street Journal");
    assert!(wsj_source.poll_interval_ms() > 0);
    
    // Test metadata retrieval
    let metadata_result = wsj_source.get_metadata().await;
    assert!(metadata_result.is_ok());
    
    let metadata = metadata_result.unwrap();
    assert!(metadata.feed_url.contains("dj.com") || metadata.feed_url.contains("wsj") || metadata.feed_url.contains("dowjones"));
    
    info!("WSJ feed source test completed successfully!");
}

#[tokio::test]
async fn test_rss_feed_source_creation() {
    init_tracing();

    info!("Testing generic RSS feed source creation");
    
    // Create fetch configuration
    let fetch_config = FetchConfig {
        user_agent: "RSS-Test/1.0".to_string(),
        timeout_seconds: 30,
        max_retries: 2,
        retry_delay_seconds: 1,
        respect_robots_txt: false,
        max_feed_size_mb: 10,
        follow_redirects: true,
        max_redirects: 5,
    };
    
    // Create generic RSS feed source with a test URL
    let test_url = "https://feeds.content.dowjones.io/public/rss/RSSWorldNews";
    let mut rss_source = RssFeedSource::new(
        Uuid::new_v4(),
        test_url.to_string(),
        fetch_config,
        Some(1800000), // 30 minutes
    );
    
    // Set title and description manually since constructor doesn't take them
    rss_source.title = Some("Test RSS Feed".to_string());
    rss_source.description = Some("Test description".to_string());
    
    // Test basic properties
    assert!(!rss_source.source_id().is_empty());
    assert_eq!(rss_source.source_name(), "Test RSS Feed");
    assert!(rss_source.poll_interval_ms() > 0);
    
    // Test metadata retrieval
    let metadata_result = rss_source.get_metadata().await;
    assert!(metadata_result.is_ok());
    
    let metadata = metadata_result.unwrap();
    assert_eq!(metadata.feed_url, test_url);
    
    info!("Generic RSS feed source test completed successfully!");
}

#[test]
fn test_rss_url_utilities() {
    // Test URL validation
    assert!(url::is_valid_rss_url("https://example.com/feed.xml"));
    assert!(url::is_valid_rss_url("http://example.com/rss"));
    assert!(!url::is_valid_rss_url("ftp://example.com/feed"));
    assert!(!url::is_valid_rss_url("invalid-url"));
    
    // Test domain extraction
    assert_eq!(url::extract_domain("https://www.wsj.com/feed"), Some("www.wsj.com".to_string()));
    assert_eq!(url::extract_domain("http://example.com/path"), Some("example.com".to_string()));
    assert_eq!(url::extract_domain("invalid-url"), None);
    
    // Test news URL detection
    assert!(url::is_news_url("https://www.wsj.com/feed"));
    assert!(url::is_news_url("https://www.reuters.com/rss"));
    assert!(!url::is_news_url("https://example.com/feed"));
}

#[test]
fn test_time_utilities() {
    use chrono::{Utc, Duration};
    
    // Test should_update logic
    assert!(time::should_update(None, 3600)); // No previous update
    
    let one_hour_ago = Utc::now() - Duration::hours(1);
    assert!(time::should_update(Some(one_hour_ago), 3600)); // Should update after 1 hour
    assert!(!time::should_update(Some(one_hour_ago), 7200)); // Should not update within 2 hours
    
    // Test interval calculation
    let now = Utc::now();
    let timestamps = vec![
        now,
        now - Duration::hours(1),
        now - Duration::hours(2),
        now - Duration::hours(3),
    ];
    
    let optimal_interval = time::calculate_optimal_interval(&timestamps);
    assert!(optimal_interval >= Duration::minutes(15));
    assert!(optimal_interval <= Duration::hours(24));
}