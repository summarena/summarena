use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;
// Use the interfaces crate for core types
pub use interfaces::defs::{InputItem, LiveSourceSpec, Ingester, WatchRest};
pub use interfaces::defs::{DigestModel, DigestModelSpec, DigestModelMemory, DigestPreferences, DigestOutput, DigestSelectedItem};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FeedMetadata {
    pub id: Uuid,
    pub url: String,
    pub title: Option<String>,
    pub description: Option<String>,
    pub last_fetch_time: Option<DateTime<Utc>>,
    pub last_successful_fetch: Option<DateTime<Utc>>,
    pub update_frequency_hours: Option<u32>,
    pub error_count: u32,
    pub last_error: Option<String>,
    pub is_active: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub etag: Option<String>,
    pub last_modified: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FeedEntry {
    pub id: Uuid,
    pub feed_id: Uuid,
    pub guid: Option<String>,
    pub url: String,
    pub title: String,
    pub description: Option<String>,
    pub content: Option<String>,
    pub author: Option<String>,
    pub published_at: Option<DateTime<Utc>>,
    pub updated_at: Option<DateTime<Utc>>,
    pub tags: Vec<String>,
    pub created_at: DateTime<Utc>,
    pub last_processed: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FetchResult {
    pub feed_id: Uuid,
    pub success: bool,
    pub entries_found: usize,
    pub new_entries: usize,
    pub error: Option<String>,
    pub fetch_time: DateTime<Utc>,
    pub response_time_ms: u64,
    pub http_status: Option<u16>,
    pub etag: Option<String>,
    pub last_modified: Option<String>,
    pub content: Option<String>, // RSS/XML content
}

#[derive(Debug, Clone)]
pub struct FetchConfig {
    pub user_agent: String,
    pub timeout_seconds: u64,
    pub max_retries: u32,
    pub retry_delay_seconds: u64,
    pub respect_robots_txt: bool,
    pub max_feed_size_mb: usize,
    pub follow_redirects: bool,
    pub max_redirects: usize,
}

impl Default for FetchConfig {
    fn default() -> Self {
        Self {
            user_agent: "RSS-Aggregator/1.0".to_string(),
            timeout_seconds: 30,
            max_retries: 3,
            retry_delay_seconds: 5,
            respect_robots_txt: true,
            max_feed_size_mb: 10,
            follow_redirects: true,
            max_redirects: 5,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScheduleInfo {
    pub feed_id: Uuid,
    pub next_fetch_time: DateTime<Utc>,
    pub priority: u8, // 0-255, higher = more priority
}

#[derive(Debug)]
pub struct ParsedFeed {
    pub title: Option<String>,
    pub description: Option<String>,
    pub entries: Vec<ParsedEntry>,
    pub etag: Option<String>,
    pub last_modified: Option<String>,
}

#[derive(Debug)]
pub struct ParsedEntry {
    pub guid: Option<String>,
    pub url: String,
    pub title: String,
    pub description: Option<String>,
    pub content: Option<String>,
    pub author: Option<String>,
    pub published_at: Option<DateTime<Utc>>,
    pub updated_at: Option<DateTime<Utc>>,
    pub tags: Vec<String>,
}

// Integration with the interfaces crate
impl From<&FeedEntry> for InputItem {
    fn from(entry: &FeedEntry) -> Self {
        let text = format!(
            "Title: {}\n\nDescription: {}\n\nContent: {}",
            entry.title,
            entry.description.as_deref().unwrap_or(""),
            entry.content.as_deref().unwrap_or("")
        );
        
        Self {
            uri: entry.url.clone(),
            text,
            vision: Vec::new(), // RSS entries typically don't have image data
        }
    }
}

impl From<&FeedMetadata> for LiveSourceSpec {
    fn from(feed: &FeedMetadata) -> Self {
        Self {
            uri: feed.url.clone(),
        }
    }
}

#[derive(Debug, thiserror::Error)]
pub enum AggregatorError {
    #[error("HTTP error: {0}")]
    Http(#[from] reqwest::Error),
    
    #[error("Feed parse error: {0}")]
    Parse(String),
    
    #[error("Database error: {0}")]
    Database(#[from] sqlx::Error),
    
    #[error("Invalid URL: {0}")]
    InvalidUrl(#[from] url::ParseError),
    
    #[error("Feed not found: {id}")]
    FeedNotFound { id: Uuid },
    
    #[error("Rate limited for {seconds} seconds")]
    RateLimited { seconds: u64 },
    
    #[error("Robots.txt disallows access to {url}")]
    RobotsDisallowed { url: String },
    
    #[error("Feed size exceeds limit: {size_mb}MB")]
    FeedTooLarge { size_mb: usize },
    
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
    
    #[error("Serialization error: {0}")]
    Serialization(#[from] serde_json::Error),
    
    #[error("General error: {0}")]
    General(String),
}

pub type Result<T> = std::result::Result<T, AggregatorError>;