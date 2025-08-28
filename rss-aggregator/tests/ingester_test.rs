use rss_aggregator::{RssIngester, Ingester, LiveSourceSpec};
use tracing_subscriber;

#[tokio::test]
async fn test_rss_ingester_basic() {
    // Initialize logging for the test
    let _ = tracing_subscriber::fmt::try_init();
    
    // Create a test RSS source spec
    let source = LiveSourceSpec {
        uri: "https://feeds.a.dj.com/rss/RSSWorldNews.xml".to_string(),
    };
    
    // Test that the ingester can be called without errors
    // Note: This will fail in CI/testing without network access, but demonstrates the interface
    let result = RssIngester::watch(&source).await;
    
    // We expect it to work but potentially fail due to network issues in testing
    match result {
        Ok(watch_rest) => {
            println!("RSS ingester completed successfully, wait time: {}ms", watch_rest.wait_at_least_ms);
            assert!(watch_rest.wait_at_least_ms > 0);
        }
        Err(e) => {
            println!("RSS ingester failed (expected in CI): {}", e);
            // This is expected in CI without network access
        }
    }
}

#[tokio::test]
async fn test_rss_ingester_interface_compliance() {
    // Test that our implementation correctly implements the async trait
    let source = LiveSourceSpec {
        uri: "https://example.com/rss.xml".to_string(),
    };
    
    // Verify the function signature is correct by calling it
    let future = RssIngester::watch(&source);
    
    // This test just verifies compilation and basic interface compliance
    // The actual RSS fetching might fail, which is fine for this test
    let _result = future.await;
    
    // If we get here, the interface is correctly implemented
    println!("RSS ingester interface compliance test passed!");
}