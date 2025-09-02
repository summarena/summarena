use interfaces::defs::{LiveSourceSpec, Ingester, InputItem, WatchRest};
use anyhow::Result;
use chrono::{DateTime, Utc};
use serde::Deserialize;
use reqwest;
use async_trait::async_trait;
use http::StatusCode;
use crate::database::TwitterDatabase;

// Twitter API base URL constant
const TWITTER_API_BASE_URL: &str = "https://api.twitter.com/2";

// Default maximum tweets to fetch per request
const DEFAULT_MAX_TWEETS: u32 = 10;

// Get max tweets from environment variable or use default
fn get_max_tweets() -> u32 {
    std::env::var("TWITTER_MAX_TWEETS")
        .ok()
        .and_then(|s| s.parse().ok())
        .unwrap_or(DEFAULT_MAX_TWEETS)
}

#[derive(Clone)]
pub struct TwitterIngesterConfig {
    pub twitter_handles: Vec<String>,
    pub bearer_token: String,
    pub last_sync_date: Option<DateTime<Utc>>,
}

impl TwitterIngesterConfig {
    /// Create a new TwitterIngesterConfig
    pub fn new(twitter_handles: Vec<String>, bearer_token: String) -> Self {
        Self {
            twitter_handles,
            bearer_token,
            last_sync_date: None,
        }
    }
    
    /// Create a new TwitterIngesterConfig with last sync date
    pub fn new_with_last_sync_date(
        twitter_handles: Vec<String>, 
        bearer_token: String, 
        last_sync_date: Option<DateTime<Utc>>
    ) -> Self {
        Self {
            twitter_handles,
            bearer_token,
            last_sync_date,
        }
    }
    
    /// Parse Twitter handles from URI format
    /// Expected formats: "twitter://user/handle" or "twitter://user/handle1,handle2,handle3"
    pub fn parse_handles_from_uri(uri: &str) -> Result<Vec<String>> {
        if !uri.starts_with("twitter://user/") {
            return Err(anyhow::anyhow!(
                "Invalid Twitter URI format. Expected 'twitter://user/handle' or 'twitter://user/handle1,handle2' but got: {}", 
                uri
            ));
        }
        
        let handle_part = uri.strip_prefix("twitter://user/")
            .ok_or_else(|| anyhow::anyhow!("Invalid Twitter URI format: {}", uri))?;
        
        if handle_part.trim().is_empty() {
            return Err(anyhow::anyhow!("No Twitter handle specified in URI: {}", uri));
        }
        
        // Split by comma and process each handle
        let handles: Result<Vec<String>, _> = handle_part
            .split(',')
            .map(|h| {
                let handle = h.trim().trim_start_matches('@').to_string();
                if handle.is_empty() {
                    Err(anyhow::anyhow!("Invalid Twitter handle in URI: {}", uri))
                } else {
                    Ok(handle)
                }
            })
            .collect();
        
        handles
    }
}

#[derive(Deserialize, Debug)]
struct TwitterUser {
    id: String,
    #[allow(dead_code)]
    name: String,
    #[allow(dead_code)]
    username: String,
}

#[derive(Deserialize, Debug)]
struct TwitterUserResponse {
    data: Option<TwitterUser>,
}

#[derive(Deserialize, Debug)]
struct TwitterTweet {
    id: String,
    text: String,
    created_at: Option<String>,
    #[allow(dead_code)]
    author_id: Option<String>,
    public_metrics: Option<TwitterPublicMetrics>,
}

#[derive(Deserialize, Debug)]
struct TwitterPublicMetrics {
    retweet_count: Option<u64>,
    like_count: Option<u64>,
    reply_count: Option<u64>,
    quote_count: Option<u64>,
}

#[derive(Deserialize, Debug)]
struct TwitterTweetsResponse {
    data: Option<Vec<TwitterTweet>>,
    #[allow(dead_code)]
    meta: Option<TwitterMeta>,
}

#[derive(Deserialize, Debug)]
struct TwitterMeta {
    #[allow(dead_code)]
    result_count: Option<u64>,
}

// HTTP client trait for dependency injection and testing
#[async_trait]
pub trait HttpClient: Send + Sync {
    async fn get(&self, url: &str) -> Result<HttpResponse>;
    async fn get_with_auth_and_query(
        &self,
        url: &str,
        auth_header: &str,
        query_params: &[(String, String)],
    ) -> Result<HttpResponse>;
}

// HTTP response wrapper
pub struct HttpResponse {
    pub status: u16,
    pub body: String,
}

// Real HTTP client implementation using reqwest
pub struct ReqwestClient {
    client: reqwest::Client,
}

impl Default for ReqwestClient {
    fn default() -> Self {
        Self::new()
    }
}

impl ReqwestClient {
    pub fn new() -> Self {
        Self {
            client: reqwest::Client::new(),
        }
    }
}

#[async_trait]
impl HttpClient for ReqwestClient {
    async fn get(&self, url: &str) -> Result<HttpResponse> {
        let response = self.client.get(url).send().await?;
        let status = response.status().as_u16();
        let body = response.text().await?;
        
        Ok(HttpResponse { status, body })
    }

    async fn get_with_auth_and_query(
        &self,
        url: &str,
        auth_header: &str,
        query_params: &[(String, String)],
    ) -> Result<HttpResponse> {
        let query_pairs: Vec<(&str, &str)> = query_params
            .iter()
            .map(|(k, v)| (k.as_str(), v.as_str()))
            .collect();
        
        let response = self.client
            .get(url)
            .header("Authorization", auth_header)
            .query(&query_pairs)
            .send()
            .await?;
        
        let status = response.status().as_u16();
        let body = response.text().await?;
        
        Ok(HttpResponse { status, body })
    }
}

pub struct TwitterIngester<T: HttpClient = ReqwestClient> {
    config: Option<TwitterIngesterConfig>,
    client: T,
    database: Option<TwitterDatabase>,
}

impl Default for TwitterIngester<ReqwestClient> {
    fn default() -> Self {
        Self::new()
    }
}

impl TwitterIngester<ReqwestClient> {
    pub fn new() -> Self {
        Self {
            config: None,
            client: ReqwestClient::new(),
            database: None,
        }
    }
}

impl<T: HttpClient> TwitterIngester<T> {
    /// Create a new TwitterIngester with a custom HTTP client (for testing)
    pub fn new_with_client(client: T) -> Self {
        Self {
            config: None,
            client,
            database: None,
        }
    }
    
    /// Get a reference to the configuration (for testing)
    pub fn config(&self) -> Option<&TwitterIngesterConfig> {
        self.config.as_ref()
    }

    pub fn with_config(config: TwitterIngesterConfig) -> TwitterIngester<ReqwestClient> {
        TwitterIngester {
            config: Some(config),
            client: ReqwestClient::new(),
            database: None,
        }
    }
    
    /// Create with config and custom client (for testing)
    pub fn with_config_and_client(config: TwitterIngesterConfig, client: T) -> Self {
        Self {
            config: Some(config),
            client,
            database: None,
        }
    }

    pub fn with_config_and_last_sync(mut config: TwitterIngesterConfig, last_sync_date: DateTime<Utc>) -> TwitterIngester<ReqwestClient> {
        // Update last sync date in the config
        config.last_sync_date = Some(last_sync_date);
        TwitterIngester {
            config: Some(config),
            client: ReqwestClient::new(),
            database: None,
        }
    }

    pub async fn fetch_tweets(&self, source: &LiveSourceSpec) -> Result<Vec<InputItem>> {
        let config = match &self.config {
            Some(cfg) => cfg,
            None => {
                return Err(anyhow::anyhow!("No Twitter configuration provided. Use TwitterIngester::with_config()."));
            }
        };

        let tweets = self.fetch_from_twitter_api(config, source).await?;

        // Update database with current sync time if database is available
        if let Some(database) = &self.database {
            let current_time = Utc::now();
            for handle in &config.twitter_handles {
                if let Err(e) = database.update_last_sync(handle, current_time).await {
                    eprintln!("Failed to update last sync time in database for @{}: {}", handle, e);
                }
            }
        }
        Ok(tweets)
    }

    async fn fetch_from_twitter_api(&self, config: &TwitterIngesterConfig, source: &LiveSourceSpec) -> Result<Vec<InputItem>> {
        let mut all_tweets = Vec::new();
        
        // Fetch tweets for each handle
        for handle in &config.twitter_handles {
            match self.fetch_user_tweets(handle, config, source).await {
                Ok(tweets) => {
                    all_tweets.extend(tweets);
                },
                Err(e) => {
                    eprintln!("Failed to fetch tweets for @{}: {}", handle, e);
                    // Continue with other handles instead of failing completely
                }
            }
        }
        Ok(all_tweets)
    }

    async fn fetch_user_tweets(
        &self,
        handle: &str,
        config: &TwitterIngesterConfig,
        source: &LiveSourceSpec,
    ) -> Result<Vec<InputItem>> {
        // First, get the user ID from the username
        let user_url = format!("{}/users/by/username/{}", TWITTER_API_BASE_URL, handle);
        
        let user_response = self.client
            .get_with_auth_and_query(
                &user_url,
                &format!("Bearer {}", config.bearer_token),
                &[],
            )
            .await
            .map_err(|e| anyhow::anyhow!("Failed to fetch user info for @{}: {}", handle, e))?;

        if user_response.status != StatusCode::OK.as_u16() {
            return Err(anyhow::anyhow!("Twitter API error for user @{}: {}", handle, user_response.status));
        }
        
        let user_data: TwitterUserResponse = serde_json::from_str(&user_response.body)
            .map_err(|e| anyhow::anyhow!("Failed to parse user response for @{}: {}", handle, e))?;
        
        let user = user_data.data
            .ok_or_else(|| anyhow::anyhow!("No user data found for @{}", handle))?;
        
        // Sleep for 1 second between user lookup and tweets fetch
        tokio::time::sleep(tokio::time::Duration::from_secs(1)).await;
        
        // Now fetch tweets for this user
        let tweets_url = format!("{}/users/{}/tweets", TWITTER_API_BASE_URL, user.id);
        
        // Build query parameters
        let mut query_params = vec![
            ("max_results".to_string(), get_max_tweets().to_string()),
            ("tweet.fields".to_string(), "created_at,author_id,public_metrics".to_string()),
        ];
        
        // Determine which last sync date to use based on new logic
        let effective_last_sync = if let Some(ref db) = self.database {
            // Get last sync date from database for this handle
            match db.get_last_sync(handle).await {
                Ok(Some(sync_data)) => {
                    if let Some(db_sync_date) = sync_data.last_sync_date {
                        // Use the later of db date or config date
                        match config.last_sync_date {
                            Some(config_sync_date) => Some(db_sync_date.max(config_sync_date)),
                            None => Some(db_sync_date),
                        }
                    } else {
                        // No date in db, use config date
                        config.last_sync_date
                    }
                }
                _ => {
                    // Database error or no entry, use config date
                    config.last_sync_date
                }
            }
        } else {
            // No database, use config date
            config.last_sync_date
        };
        
        // Add start_time filter if we have an effective last sync date
        if let Some(last_sync) = effective_last_sync {
            // Twitter API expects RFC3339 format (ISO 8601)
            let start_time = last_sync.to_rfc3339();
            query_params.push(("start_time".to_string(), start_time));
        }
        
        // Retry logic for rate limiting (429 errors)
        let max_retries = 3;
        let retry_delay_secs = 300; // 5 minutes
        
        let tweets_response = {
            let mut attempts = 0;
            loop {
                let response = self.client
                    .get_with_auth_and_query(
                        &tweets_url,
                        &format!("Bearer {}", config.bearer_token),
                        &query_params,
                    )
                    .await
                    .map_err(|e| anyhow::anyhow!("Failed to fetch tweets for @{}: {}", handle, e))?;
                
                if response.status == 429 && attempts < max_retries {
                    attempts += 1;
                    tokio::time::sleep(tokio::time::Duration::from_secs(retry_delay_secs)).await;
                    continue;
                }
                
                break response;
            }
        };

        if tweets_response.status != StatusCode::OK.as_u16() {
            return Err(anyhow::anyhow!("Twitter API error fetching tweets for @{}: {}", handle, tweets_response.status));
        }

        let tweets_data: TwitterTweetsResponse = serde_json::from_str(&tweets_response.body)
            .map_err(|e| anyhow::anyhow!("Failed to parse tweets response for @{}: {}", handle, e))?;
        
        let mut input_items = Vec::new();
        
        // Process the tweets
        if let Some(tweets) = tweets_data.data {
            for tweet in tweets {
                let tweet_id = tweet.id.clone();
                let tweet_text = tweet.text.clone();
                let created_at = tweet.created_at
                    .unwrap_or_else(|| "Unknown".to_string());
                
                // Get engagement metrics if available
                let metrics_text = if let Some(metrics) = &tweet.public_metrics {
                    format!(
                        "\nLikes: {}, Retweets: {}, Replies: {}, Quotes: {}",
                        metrics.like_count.unwrap_or(0),
                        metrics.retweet_count.unwrap_or(0),
                        metrics.reply_count.unwrap_or(0),
                        metrics.quote_count.unwrap_or(0)
                    )
                } else {
                    String::new()
                };
                
                input_items.push(InputItem {
                    uri: format!("twitter://tweet/{}", tweet_id),
                    live_source_uri: source.uri.clone(),
                    text: format!(
                        "@{} - {}\n{}\nhttps://twitter.com/{}/status/{}{}",
                        handle,
                        created_at,
                        tweet_text,
                        handle,
                        tweet_id,
                        metrics_text
                    ),
                    vision: None,
                });
            }
        }
        
        // Sleep for 1 second after tweets fetch
        tokio::time::sleep(tokio::time::Duration::from_secs(1)).await;
        
        Ok(input_items)
    }
}

impl Ingester for TwitterIngester<ReqwestClient> {
    async fn watch(source: &LiveSourceSpec) -> Result<WatchRest> {
        // Get Bearer token from environment variable
        let bearer_token = match std::env::var("TWITTER_BEARER_TOKEN") {
            Ok(token) => {
                if token.trim().is_empty() {
                    eprintln!("TWITTER_BEARER_TOKEN is empty");
                    return Ok(WatchRest {
                        wait_at_least_ms: 300000, // Wait 5 minutes
                    });
                }
                token
            },
            Err(_) => {
                eprintln!("TWITTER_BEARER_TOKEN environment variable not set");
                return Ok(WatchRest {
                    wait_at_least_ms: 300000, // Wait 5 minutes
                });
            }
        };

        // Parse Twitter handles from source URI
        let twitter_handles = match TwitterIngesterConfig::parse_handles_from_uri(&source.uri) {
            Ok(handles) => handles,
            Err(e) => {
                eprintln!("Failed to parse Twitter handles from URI '{}': {}", source.uri, e);
                return Ok(WatchRest {
                    wait_at_least_ms: 300000, // Wait 5 minutes on parse error
                });
            }
        };

        // Check for database URL and setup database if available
        let database = match std::env::var("DATABASE_URL") {
            Ok(database_url) => {
                match TwitterDatabase::new(&database_url).await {
                    Ok(db) => {
                        // Setup schema
                        if let Err(e) = db.setup_schema().await {
                            eprintln!("Failed to setup database schema: {}", e);
                            None
                        } else {
                            Some(db)
                        }
                    }
                    Err(e) => {
                        eprintln!("Failed to connect to database: {}", e);
                        None
                    }
                }
            }
            Err(_) => None
        };

        // Get last sync date from environment variable (config)
        let config_last_sync_date = std::env::var("TWITTER_LAST_SYNC_DATE")
            .ok()
            .and_then(|date_str| DateTime::parse_from_rfc3339(&date_str).ok())
            .map(|dt| dt.with_timezone(&Utc));

        // Create configuration with config last sync date
        let config = TwitterIngesterConfig::new_with_last_sync_date(
            twitter_handles.clone(), 
            bearer_token.clone(), 
            config_last_sync_date
        );

        // Create ingester - database setup will be handled internally during fetch_tweets
        let handles_for_logging = config.twitter_handles.clone();
        let mut ingester = TwitterIngester::<ReqwestClient>::with_config(config);
        ingester.database = database;
        
        match ingester.fetch_tweets(source).await {
            Ok(tweets) => {
                let tweet_count = tweets.len();
                
                // Process each tweet through the state system
                for tweet in tweets {
                    if let Err(e) = interfaces::state::ingest(&tweet).await {
                        eprintln!("Failed to ingest tweet {}: {}", tweet.uri, e);
                    }
                }
                
                println!("Successfully ingested {} tweets for handles: {:?}", tweet_count, handles_for_logging);
                
                Ok(WatchRest {
                    wait_at_least_ms: 300000, // Check again in 5 minutes (Twitter API rate limits)
                })
            }
            Err(e) => {
                eprintln!("Failed to fetch tweets for handles {:?}: {}", handles_for_logging, e);
                Ok(WatchRest {
                    wait_at_least_ms: 900000, // Wait longer on error (15 minutes)
                })
            }
        }
    }
}


#[cfg(test)]
mod tests {
    use super::*;
    use chrono::{DateTime, Utc};

    #[test]
    fn test_parse_handles_from_uri_multiple() {
        let handles = TwitterIngesterConfig::parse_handles_from_uri("twitter://user/handle1,handle2,handle3").unwrap();
        assert_eq!(handles, vec!["handle1", "handle2", "handle3"]);
    }

    #[test]
    fn test_parse_handles_from_uri_multiple_with_spaces() {
        let handles = TwitterIngesterConfig::parse_handles_from_uri("twitter://user/handle1, handle2 , @handle3").unwrap();
        assert_eq!(handles, vec!["handle1", "handle2", "handle3"]);
    }

    #[test]
    fn test_twitter_ingester_config_new_multiple_handles() {
        let handles = vec!["handle1".to_string(), "handle2".to_string()];
        let bearer_token = "test_token".to_string();
        
        let config = TwitterIngesterConfig::new(handles.clone(), bearer_token.clone());
        
        assert_eq!(config.twitter_handles, handles);
        assert_eq!(config.bearer_token, bearer_token);
        assert!(config.last_sync_date.is_none());
    }

    #[test]
    fn test_twitter_ingester_config_new() {
        let handles = vec!["dawnsongtweets".to_string()];
        let bearer_token = "test_token".to_string();
        
        let config = TwitterIngesterConfig::new(handles.clone(), bearer_token.clone());
        
        assert_eq!(config.twitter_handles, handles);
        assert_eq!(config.bearer_token, bearer_token);
        assert!(config.last_sync_date.is_none());
    }

    #[test]
    fn test_twitter_ingester_config_new_with_last_sync_date_some() {
        let handles = vec!["dawnsongtweets".to_string()];
        let bearer_token = "test_token".to_string();
        let last_sync_date = DateTime::parse_from_rfc3339("2024-01-15T10:30:45.000Z")
            .unwrap()
            .with_timezone(&Utc);
        
        let config = TwitterIngesterConfig::new_with_last_sync_date(
            handles.clone(), 
            bearer_token.clone(), 
            Some(last_sync_date)
        );
        
        assert_eq!(config.twitter_handles, handles);
        assert_eq!(config.bearer_token, bearer_token);
        assert_eq!(config.last_sync_date, Some(last_sync_date));
    }

    #[test]
    fn test_twitter_ingester_config_new_with_last_sync_date_none() {
        let handles = vec!["dawnsongtweets".to_string()];
        let bearer_token = "test_token".to_string();
        
        let config = TwitterIngesterConfig::new_with_last_sync_date(
            handles.clone(), 
            bearer_token.clone(), 
            None
        );
        
        assert_eq!(config.twitter_handles, handles);
        assert_eq!(config.bearer_token, bearer_token);
        assert_eq!(config.last_sync_date, None);
    }

    #[test]
    fn test_twitter_ingester_new() {
        let ingester = TwitterIngester::new();
        assert!(ingester.config().is_none());
    }

    #[test]
    fn test_twitter_ingester_with_config() {
        let handles = vec!["dawnsongtweets".to_string()];
        let bearer_token = "test_token".to_string();
        let config = TwitterIngesterConfig::new(handles, bearer_token);
        
        let ingester = TwitterIngester::<ReqwestClient>::with_config(config.clone());
        assert!(ingester.config().is_some());
        assert_eq!(ingester.config().unwrap().twitter_handles, config.twitter_handles);
    }

    #[test]
    fn test_twitter_ingester_with_config_and_last_sync() {
        let handles = vec!["dawnsongtweets".to_string()];
        let bearer_token = "test_token".to_string();
        let config = TwitterIngesterConfig::new(handles.clone(), bearer_token);
        let last_sync_date = DateTime::parse_from_rfc3339("2024-01-15T10:30:45.000Z")
            .unwrap()
            .with_timezone(&Utc);
        
        let ingester = TwitterIngester::<ReqwestClient>::with_config_and_last_sync(config, last_sync_date);
        assert!(ingester.config().is_some());
        assert_eq!(ingester.config().unwrap().last_sync_date, Some(last_sync_date));
    }
}
