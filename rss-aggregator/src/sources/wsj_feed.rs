use crate::traits::{PullFeed, SourceMetadata};
use crate::types::{InputItem, Result, FetchConfig};
use crate::sources::RssFeedSource;
use async_trait::async_trait;
use uuid::Uuid;

/// Wall Street Journal specific RSS feed implementation
pub struct WsjFeedSource {
    inner: RssFeedSource,
}

impl WsjFeedSource {
    pub fn new(feed_id: Uuid, url: String, fetch_config: FetchConfig) -> Self {
        // WSJ feeds update frequently, so use a shorter poll interval
        let poll_interval_ms = 600_000; // 10 minutes
        
        let inner = RssFeedSource::new(feed_id, url, fetch_config, Some(poll_interval_ms))
            .with_metadata(
                Some("Wall Street Journal".to_string()),
                Some("Breaking news and analysis from The Wall Street Journal".to_string()),
            );
        
        Self { inner }
    }
    
    /// Create a WSJ feed source for the main news feed
    pub fn main_feed(fetch_config: FetchConfig) -> Self {
        let feed_id = Uuid::new_v4();
        let url = "https://feeds.a.dj.com/rss/RSSWorldNews.xml".to_string();
        Self::new(feed_id, url, fetch_config)
    }
    
    /// Create a WSJ feed source for the business news feed
    pub fn business_feed(fetch_config: FetchConfig) -> Self {
        let feed_id = Uuid::new_v4();
        let url = "https://feeds.a.dj.com/rss/WSJcomUSBusiness.xml".to_string();
        Self::new(feed_id, url, fetch_config)
    }
    
    /// Create a WSJ feed source for the technology news feed
    pub fn tech_feed(fetch_config: FetchConfig) -> Self {
        let feed_id = Uuid::new_v4();
        let url = "https://feeds.a.dj.com/rss/RSSWSJD.xml".to_string();
        Self::new(feed_id, url, fetch_config)
    }
    
    /// Create a WSJ feed source for the markets feed
    pub fn markets_feed(fetch_config: FetchConfig) -> Self {
        let feed_id = Uuid::new_v4();
        let url = "https://feeds.a.dj.com/rss/RSSMarketsMain.xml".to_string();
        Self::new(feed_id, url, fetch_config)
    }
}

#[async_trait]
impl PullFeed for WsjFeedSource {
    fn source_id(&self) -> String {
        format!("wsj_{}", self.inner.feed_id)
    }
    
    fn source_name(&self) -> String {
        format!("WSJ: {}", self.inner.source_name())
    }
    
    async fn pull(&mut self) -> Result<Vec<InputItem>> {
        // Delegate to the inner RSS feed source
        self.inner.pull().await
    }
    
    fn poll_interval_ms(&self) -> u64 {
        self.inner.poll_interval_ms()
    }
    
    async fn health_check(&self) -> Result<bool> {
        self.inner.health_check().await
    }
    
    async fn get_metadata(&self) -> Result<SourceMetadata> {
        let mut metadata = self.inner.get_metadata().await?;
        
        // Add WSJ-specific metadata
        metadata.tags.extend_from_slice(&[
            "news".to_string(),
            "business".to_string(),
            "finance".to_string(),
            "wsj".to_string(),
        ]);
        
        // Set the website URL
        metadata.website_url = Some("https://www.wsj.com".to_string());
        metadata.language = Some("en".to_string());
        
        Ok(metadata)
    }
}