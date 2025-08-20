use crate::{FeedManager, Fetcher, FeedParser, FetchConfig, Result, AggregatorError};
use crate::types::{InputItem, LiveSourceSpec};

// Simplified local definitions (would normally use interfaces crate)
pub trait Ingester {
    fn watch(source: &LiveSourceSpec) -> WatchRest;
}

#[derive(Debug)]
pub struct WatchRest {
    pub wait_at_least_ms: u32,
}
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::{error, info, warn};
use uuid::Uuid;

pub struct RssAggregator {
    feed_manager: Arc<FeedManager>,
    fetcher: Arc<RwLock<Fetcher>>,
    parser: Arc<RwLock<FeedParser>>,
}

impl RssAggregator {
    pub async fn new(database_url: &str, fetch_config: FetchConfig) -> Result<Self> {
        let feed_manager = Arc::new(FeedManager::new(database_url).await?);
        let fetcher = Arc::new(RwLock::new(Fetcher::new(fetch_config)));
        let parser = Arc::new(RwLock::new(FeedParser::new()));
        
        Ok(Self {
            feed_manager,
            fetcher,
            parser,
        })
    }
    
    pub async fn add_feed(&self, url: String) -> Result<Uuid> {
        // Validate URL first
        if !self.feed_manager.validate_feed_url(&url).await? {
            return Err(AggregatorError::General("Invalid or duplicate feed URL".to_string()));
        }
        
        // Try to fetch and parse the feed to extract metadata
        let fetcher = self.fetcher.read().await;
        let feed_id = Uuid::new_v4();
        let fetch_result = fetcher.fetch_feed(feed_id, &url, None, None).await?;
        
        if !fetch_result.success {
            return Err(AggregatorError::General(
                fetch_result.error.unwrap_or_else(|| "Failed to fetch feed".to_string())
            ));
        }
        
        // Extract feed metadata by parsing
        // Note: In a complete implementation, you'd get the content from fetch_result
        // For now, we'll add the feed with basic info
        let feed_id = self.feed_manager.add_feed(url, None, None).await?;
        
        info!("Successfully added feed with ID: {}", feed_id);
        Ok(feed_id)
    }
    
    pub async fn fetch_all_feeds(&self) -> Result<usize> {
        let feeds_to_fetch = self.feed_manager.get_feeds_to_fetch(100).await?;
        let total_feeds = feeds_to_fetch.len();
        let mut successful_fetches = 0;
        
        info!("Fetching {} feeds", total_feeds);
        
        for schedule_info in feeds_to_fetch {
            match self.fetch_single_feed(schedule_info.feed_id).await {
                Ok(_) => {
                    successful_fetches += 1;
                }
                Err(e) => {
                    error!("Failed to fetch feed {}: {}", schedule_info.feed_id, e);
                }
            }
        }
        
        info!("Successfully fetched {}/{} feeds", successful_fetches, total_feeds);
        Ok(successful_fetches)
    }
    
    async fn fetch_single_feed(&self, feed_id: Uuid) -> Result<usize> {
        let feed_metadata = self.feed_manager.get_feed(feed_id).await?;
        
        let fetcher = self.fetcher.read().await;
        let fetch_result = fetcher.fetch_feed(
            feed_id,
            &feed_metadata.url,
            feed_metadata.etag.as_deref(),
            feed_metadata.last_modified.as_deref(),
        ).await?;
        
        // Update feed metadata with fetch result
        self.feed_manager.update_fetch_result(
            feed_id,
            fetch_result.success,
            fetch_result.error.clone(),
            fetch_result.etag.clone(),
            fetch_result.last_modified.clone(),
        ).await?;
        
        if !fetch_result.success {
            return Err(AggregatorError::General(
                fetch_result.error.unwrap_or_else(|| "Fetch failed".to_string())
            ));
        }
        
        // Get the RSS content from the fetch result
        let content = match fetch_result.content {
            Some(content) => content,
            None => {
                warn!("No content returned for feed {}", feed_id);
                return Ok(0);
            }
        };

        // Parse the RSS content
        let mut parser = self.parser.write().await;
        let parsed_feed = parser.parse_feed(&content)?;
        
        // Convert to feed entries
        let feed_entries = parser.convert_to_feed_entries(&parsed_feed, feed_id);
        let entries_found = feed_entries.len();
        
        // Store entries in the database
        let new_entries_count = self.feed_manager.store_feed_entries(&feed_entries).await?;
        
        info!("Feed {}: found {} entries, stored {} new entries", feed_id, entries_found, new_entries_count);
        
        Ok(new_entries_count)
    }
    
    pub async fn get_feed_stats(&self) -> Result<std::collections::HashMap<String, i64>> {
        self.feed_manager.get_feed_stats().await
    }
    
    pub async fn get_recent_items(&self, limit: usize) -> Result<Vec<InputItem>> {
        let feed_entries = self.feed_manager.get_recent_feed_entries(None, limit).await?;
        let input_items: Vec<InputItem> = feed_entries.iter().map(|entry| entry.into()).collect();
        Ok(input_items)
    }
    
    pub async fn deactivate_feed(&self, feed_id: Uuid) -> Result<()> {
        self.feed_manager.deactivate_feed(feed_id).await
    }
    
    pub async fn update_fetch_config(&self, config: FetchConfig) -> Result<()> {
        let mut fetcher = self.fetcher.write().await;
        fetcher.update_config(config);
        Ok(())
    }
}

// Implementation of the Ingester trait for integration with the interfaces
pub struct RssIngester {
    _aggregator: Arc<RssAggregator>,
}

impl RssIngester {
    pub fn new(aggregator: Arc<RssAggregator>) -> Self {
        Self { _aggregator: aggregator }
    }
}

impl Ingester for RssIngester {
    fn watch(_source: &LiveSourceSpec) -> WatchRest {
        // This would typically trigger the aggregator to start monitoring the RSS feed
        // For now, return a default wait time
        WatchRest {
            wait_at_least_ms: 3600000, // 1 hour
        }
    }
}