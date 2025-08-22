use crate::traits::{PullFeed, SourceMetadata};
use crate::types::{InputItem, Result, AggregatorError, FetchConfig};
use crate::{Fetcher, FeedParser};
use async_trait::async_trait;
use chrono::{DateTime, Utc};
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::{info, warn, error};
use uuid::Uuid;

/// Generic RSS feed source implementation
pub struct RssFeedSource {
    pub feed_id: Uuid,
    pub url: String,
    pub title: Option<String>,
    pub description: Option<String>,
    fetcher: Arc<RwLock<Fetcher>>,
    parser: Arc<RwLock<FeedParser>>,
    last_fetch: Option<DateTime<Utc>>,
    last_etag: Option<String>,
    last_modified: Option<String>,
    poll_interval_ms: u64,
}

impl RssFeedSource {
    pub fn new(
        feed_id: Uuid,
        url: String,
        fetch_config: FetchConfig,
        poll_interval_ms: Option<u64>,
    ) -> Self {
        let fetcher = Arc::new(RwLock::new(Fetcher::new(fetch_config)));
        let parser = Arc::new(RwLock::new(FeedParser::new()));
        
        // Default poll interval based on URL characteristics
        let default_interval = if url.contains("news") || url.contains("breaking") {
            900_000  // 15 minutes for news feeds
        } else if url.contains("blog") || url.contains("post") {
            3_600_000 // 1 hour for blog feeds  
        } else {
            1_800_000 // 30 minutes default
        };
        
        Self {
            feed_id,
            url,
            title: None,
            description: None,
            fetcher,
            parser,
            last_fetch: None,
            last_etag: None,
            last_modified: None,
            poll_interval_ms: poll_interval_ms.unwrap_or(default_interval),
        }
    }
    
    pub fn with_metadata(mut self, title: Option<String>, description: Option<String>) -> Self {
        self.title = title;
        self.description = description;
        self
    }
}

#[async_trait]
impl PullFeed for RssFeedSource {
    fn source_id(&self) -> String {
        format!("rss_{}", self.feed_id)
    }
    
    fn source_name(&self) -> String {
        self.title.clone().unwrap_or_else(|| {
            // Extract domain name from URL as fallback
            if let Ok(parsed) = url::Url::parse(&self.url) {
                if let Some(domain) = parsed.domain() {
                    return format!("RSS Feed ({})", domain);
                }
            }
            "RSS Feed".to_string()
        })
    }
    
    async fn pull(&mut self) -> Result<Vec<InputItem>> {
        info!("Pulling RSS feed: {}", self.url);
        
        let fetcher = self.fetcher.read().await;
        let fetch_result = fetcher.fetch_feed(
            self.feed_id,
            &self.url,
            self.last_etag.as_deref(),
            self.last_modified.as_deref(),
        ).await?;
        
        if !fetch_result.success {
            let error_msg = fetch_result.error.unwrap_or_else(|| "Fetch failed".to_string());
            error!("Failed to fetch RSS feed {}: {}", self.url, error_msg);
            return Err(AggregatorError::General(error_msg));
        }
        
        // Update last fetch metadata
        self.last_fetch = Some(fetch_result.fetch_time);
        self.last_etag = fetch_result.etag.clone();
        self.last_modified = fetch_result.last_modified.clone();
        
        // Get content from fetch result
        let content = match fetch_result.content {
            Some(content) => content,
            None => {
                warn!("No content returned for RSS feed {}", self.url);
                return Ok(Vec::new());
            }
        };
        
        // Parse the RSS content
        let mut parser = self.parser.write().await;
        let parsed_feed = parser.parse_feed(&content)?;
        
        // Update metadata if we don't have it
        if self.title.is_none() {
            self.title = parsed_feed.title.clone();
        }
        if self.description.is_none() {
            self.description = parsed_feed.description.clone();
        }
        
        // Convert entries to InputItems
        let feed_entries = parser.convert_to_feed_entries(&parsed_feed, self.feed_id);
        let input_items: Vec<InputItem> = feed_entries.iter().map(|entry| entry.into()).collect();
        
        info!("Successfully pulled {} items from RSS feed {}", input_items.len(), self.url);
        Ok(input_items)
    }
    
    fn poll_interval_ms(&self) -> u64 {
        self.poll_interval_ms
    }
    
    async fn health_check(&self) -> Result<bool> {
        let fetcher = self.fetcher.read().await;
        let fetch_result = fetcher.fetch_feed(
            self.feed_id,
            &self.url,
            None, // No conditional headers for health check
            None,
        ).await?;
        
        Ok(fetch_result.success)
    }
    
    async fn get_metadata(&self) -> Result<SourceMetadata> {
        Ok(SourceMetadata {
            title: self.title.clone(),
            description: self.description.clone(),
            language: None, // Could be extracted from RSS feed
            last_updated: self.last_fetch,
            feed_url: self.url.clone(),
            website_url: None, // Could be extracted from RSS feed
            tags: Vec::new(), // Could be extracted from RSS categories
        })
    }
}