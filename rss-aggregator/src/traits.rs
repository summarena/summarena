use crate::types::{InputItem, Result};
use async_trait::async_trait;
use chrono::{DateTime, Utc};
use std::collections::HashMap;

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

/// Trait for aggregating content on a per-user basis
#[async_trait]
pub trait Aggregator: Send + Sync {
    /// Unique identifier for this aggregator type
    fn aggregator_type(&self) -> String;
    
    /// Add an item to this user's aggregation bucket
    async fn add_item(&mut self, item: InputItem) -> Result<()>;
    
    /// Check if it's time to produce output for this user
    fn should_produce_output(&self) -> bool;
    
    /// Produce aggregated content for the user
    async fn produce_output(&mut self) -> Result<AggregatedOutput>;
    
    /// Get the user ID this aggregator serves
    fn user_id(&self) -> String;
    
    /// Configure aggregator parameters
    async fn configure(&mut self, config: AggregatorConfig) -> Result<()>;
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

/// Output from an aggregator
#[derive(Debug, Clone)]
pub struct AggregatedOutput {
    pub user_id: String,
    pub aggregator_type: String,
    pub items: Vec<InputItem>,
    pub summary: Option<String>,
    pub created_at: DateTime<Utc>,
    pub metadata: HashMap<String, String>,
}

/// Configuration for aggregators
#[derive(Debug, Clone)]
pub struct AggregatorConfig {
    pub parameters: HashMap<String, String>,
}

impl Default for AggregatorConfig {
    fn default() -> Self {
        Self {
            parameters: HashMap::new(),
        }
    }
}