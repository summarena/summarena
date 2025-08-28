use interfaces::defs::{LiveSourceSpec, Ingester, InputItem, WatchRest};
use anyhow::Result;
use chrono::{DateTime, Utc};
use url::Url;
use crate::types::{FetchConfig, FeedMetadata};
use crate::sources::RssFeedSource;
use crate::traits::PullFeed;
use crate::database::RssDatabase;
use uuid::Uuid;
use tracing::{info, error};

#[derive(Clone)]
pub struct RssIngesterConfig {
    pub url: String,
    pub user_agent: String,
    pub timeout_seconds: u64,
    pub max_retries: u32,
    pub poll_interval_seconds: u64,
    pub respect_robots_txt: bool,
    pub last_sync_date: Option<DateTime<Utc>>,
}

impl RssIngesterConfig {
    /// Parse RSS configuration from URI and database state
    /// Expected URI format: rss://domain.com/feed.xml?poll_interval=3600&user_agent=MyBot
    pub async fn from_uri_and_metadata(uri: &str, metadata: Option<&FeedMetadata>) -> Result<Self> {
        let parsed_uri = Url::parse(uri)
            .map_err(|e| anyhow::anyhow!("Invalid RSS URI '{}': {}", uri, e))?;
        
        // Convert rss:// scheme to https:// for actual fetching
        let actual_url = if parsed_uri.scheme() == "rss" {
            let mut url = parsed_uri.clone();
            url.set_scheme("https").map_err(|_| anyhow::anyhow!("Failed to set https scheme"))?;
            url.to_string()
        } else if parsed_uri.scheme() == "http" || parsed_uri.scheme() == "https" {
            uri.to_string()
        } else {
            return Err(anyhow::anyhow!("URI must use 'rss://', 'http://', or 'https://' scheme, got: {}", parsed_uri.scheme()));
        };
        
        // Extract configuration from query parameters
        let poll_interval_seconds = parsed_uri.query_pairs()
            .find(|(key, _)| key == "poll_interval")
            .map(|(_, value)| value.parse().unwrap_or(3600))
            .unwrap_or(3600); // Default to 1 hour
        
        let user_agent = parsed_uri.query_pairs()
            .find(|(key, _)| key == "user_agent")
            .map(|(_, value)| value.to_string())
            .unwrap_or_else(|| "RSS-Ingester/1.0".to_string());
        
        let timeout_seconds = parsed_uri.query_pairs()
            .find(|(key, _)| key == "timeout")
            .map(|(_, value)| value.parse().unwrap_or(30))
            .unwrap_or(30);
        
        let max_retries = parsed_uri.query_pairs()
            .find(|(key, _)| key == "max_retries")
            .map(|(_, value)| value.parse().unwrap_or(3))
            .unwrap_or(3);
        
        let respect_robots_txt = parsed_uri.query_pairs()
            .find(|(key, _)| key == "respect_robots_txt")
            .map(|(_, value)| value.parse().unwrap_or(true))
            .unwrap_or(true);
        
        Ok(Self {
            url: actual_url,
            user_agent,
            timeout_seconds,
            max_retries,
            poll_interval_seconds,
            respect_robots_txt,
            last_sync_date: metadata.and_then(|m| m.last_successful_fetch),
        })
    }
}

pub struct RssIngester {
    config: Option<RssIngesterConfig>,
}

impl RssIngester {
    pub fn new() -> Self {
        Self {
            config: None,
        }
    }

    pub fn with_config(config: RssIngesterConfig) -> Self {
        Self {
            config: Some(config),
        }
    }

    pub fn with_config_and_last_sync(mut config: RssIngesterConfig, last_sync_date: DateTime<Utc>) -> Self {
        config.last_sync_date = Some(last_sync_date);
        Self {
            config: Some(config),
        }
    }

    pub async fn fetch_rss_items(&self, source: &LiveSourceSpec) -> Result<Vec<InputItem>> {
        let config = match &self.config {
            Some(cfg) => cfg,
            None => return Err(anyhow::anyhow!("No RSS configuration provided. Use RssIngester::with_config() or configure via URI parameters.")),
        };

        self.fetch_from_rss_feed(config, source).await
    }

    async fn fetch_from_rss_feed(&self, config: &RssIngesterConfig, _source: &LiveSourceSpec) -> Result<Vec<InputItem>> {
        // Create a fetch configuration
        let fetch_config = FetchConfig {
            user_agent: config.user_agent.clone(),
            timeout_seconds: config.timeout_seconds,
            max_retries: config.max_retries,
            retry_delay_seconds: 5,
            respect_robots_txt: config.respect_robots_txt,
            max_feed_size_mb: 10,
            follow_redirects: true,
            max_redirects: 5,
        };

        // Create RSS feed source
        let feed_id = Uuid::new_v4();
        let mut rss_source = RssFeedSource::new(
            feed_id,
            config.url.clone(),
            fetch_config,
            Some((config.poll_interval_seconds * 1000) as u64), // Convert to milliseconds
        );

        // Pull items from the RSS feed
        let input_items = rss_source.pull().await?;

        Ok(input_items)
    }
}

impl Ingester for RssIngester {
    async fn watch(source: &LiveSourceSpec) -> Result<WatchRest> {
        info!("Starting RSS ingester watch for source: {}", source.uri);
        
        // Connect to database and get feed metadata
        let database_url = std::env::var("DATABASE_URL")
            .unwrap_or_else(|_| "postgresql://rss_user:rss_password@localhost:5432/rss_aggregator".to_string());
        let db = RssDatabase::new(&database_url).await?;
        
        // Try to find existing feed metadata
        let metadata = db.get_feed_by_url(&source.uri).await?;
        
        // Build configuration from URI and metadata
        let config = RssIngesterConfig::from_uri_and_metadata(&source.uri, metadata.as_ref()).await?;

        let ingester = RssIngester::with_config(config.clone());
        
        match ingester.fetch_rss_items(source).await {
            Ok(items) => {
                let item_count = items.len();
                
                // Process each item through the state system
                for item in items {
                    if let Err(e) = interfaces::state::ingest(&item).await {
                        error!("Failed to ingest RSS item {}: {}", item.uri, e);
                    }
                }
                
                // Update last sync time in database if we have metadata
                let now = Utc::now();
                if let Some(feed_metadata) = metadata {
                    if let Err(e) = db.update_last_successful_fetch(feed_metadata.id, now).await {
                        error!("Failed to update last sync time for feed {}: {}", feed_metadata.id, e);
                    }
                } else {
                    // Create new feed metadata if it doesn't exist
                    let new_feed = FeedMetadata {
                        id: Uuid::new_v4(),
                        url: source.uri.clone(),
                        title: None,
                        description: None,
                        last_fetch_time: Some(now),
                        last_successful_fetch: Some(now),
                        update_frequency_hours: Some((config.poll_interval_seconds / 3600) as u32),
                        error_count: 0,
                        last_error: None,
                        is_active: true,
                        created_at: now,
                        updated_at: now,
                        etag: None,
                        last_modified: None,
                    };
                    
                    if let Err(e) = db.save_feed_metadata(&new_feed).await {
                        error!("Failed to create new feed metadata: {}", e);
                    }
                }
                
                info!("Successfully ingested {} RSS items for {}", item_count, source.uri);
                
                Ok(WatchRest {
                    wait_at_least_ms: (config.poll_interval_seconds * 1000) as u32, // Convert to milliseconds
                })
            }
            Err(e) => {
                error!("Failed to fetch RSS items for {}: {}", source.uri, e);
                
                // Update error count in database if we have metadata
                if let Some(feed_metadata) = metadata {
                    let updated_metadata = FeedMetadata {
                        error_count: feed_metadata.error_count + 1,
                        last_error: Some(e.to_string()),
                        updated_at: Utc::now(),
                        ..feed_metadata
                    };
                    
                    if let Err(db_err) = db.save_feed_metadata(&updated_metadata).await {
                        error!("Failed to update feed error count: {}", db_err);
                    }
                }
                
                // Return error to allow caller to determine retry logic
                Err(e)
            }
        }
    }
}