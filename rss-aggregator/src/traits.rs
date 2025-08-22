use crate::types::{InputItem, Result};
use async_trait::async_trait;
use chrono::{DateTime, Utc};

/// Trait for pulling content from various sources (RSS feeds, APIs, etc.)
#[async_trait]
pub trait PullFeed: Send + Sync {
    /// Unique identifier for this feed source
    fn source_id(&self) -> String;
    
    /// Human-readable name for this source
    fn source_name(&self) -> String;
    
    /// Fetch new items from the source
    /// Returns items that are new since the last fetch
    async fn pull(&mut self) -> Result<Vec<InputItem>>;
    
    /// Get the recommended polling interval for this source
    fn poll_interval_ms(&self) -> u64;
    
    /// Check if the source is healthy and accessible
    async fn health_check(&self) -> Result<bool>;
    
    /// Get metadata about the source (title, description, etc.)
    async fn get_metadata(&self) -> Result<SourceMetadata>;
}

/// Metadata about a content source
#[derive(Debug, Clone)]
pub struct SourceMetadata {
    pub title: Option<String>,
    pub description: Option<String>,
    pub language: Option<String>,
    pub last_updated: Option<DateTime<Utc>>,
    pub feed_url: String,
    pub website_url: Option<String>,
    pub tags: Vec<String>,
}