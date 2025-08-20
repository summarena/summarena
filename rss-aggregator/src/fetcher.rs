use crate::types::{AggregatorError, FetchConfig, FetchResult, Result};
use backoff::{exponential::ExponentialBackoff, backoff::Backoff};
use chrono::Utc;
use reqwest::{Client, Response};
// use robotstxt::RobotsTxt; // Simplified for now
use std::collections::HashMap;
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::RwLock;
use tracing::{debug, error, info, warn};
use url::Url;
use uuid::Uuid;

pub struct Fetcher {
    client: Client,
    config: FetchConfig,
    robots_cache: Arc<RwLock<HashMap<String, bool>>>, // Simplified: just store allow/deny
    rate_limiter: Arc<RwLock<HashMap<String, Instant>>>,
}

impl Fetcher {
    pub fn new(config: FetchConfig) -> Self {
        let client = Client::builder()
            .user_agent(&config.user_agent)
            .timeout(Duration::from_secs(config.timeout_seconds))
            .gzip(true)
            .deflate(true)
            .brotli(true)
            .redirect(reqwest::redirect::Policy::limited(config.max_redirects))
            .build()
            .expect("Failed to create HTTP client");

        Self {
            client,
            config,
            robots_cache: Arc::new(RwLock::new(HashMap::new())),
            rate_limiter: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    pub async fn fetch_feed(&self, feed_id: Uuid, url: &str, etag: Option<&str>, last_modified: Option<&str>) -> Result<FetchResult> {
        let start_time = Instant::now();
        let fetch_time = Utc::now();
        
        debug!("Fetching feed: {} (ID: {})", url, feed_id);
        
        // Check robots.txt if enabled
        if self.config.respect_robots_txt {
            if let Err(e) = self.check_robots_txt(url).await {
                warn!("Robots.txt check failed for {}: {}", url, e);
                return Ok(FetchResult {
                    feed_id,
                    success: false,
                    entries_found: 0,
                    new_entries: 0,
                    error: Some(e.to_string()),
                    fetch_time,
                    response_time_ms: start_time.elapsed().as_millis() as u64,
                    http_status: None,
                    etag: None,
                    last_modified: None,
                    content: None,
                });
            }
        }
        
        // Apply rate limiting
        self.apply_rate_limit(url).await?;
        
        let mut backoff: ExponentialBackoff<backoff::SystemClock> = ExponentialBackoff {
            current_interval: Duration::from_secs(self.config.retry_delay_seconds),
            initial_interval: Duration::from_secs(self.config.retry_delay_seconds),
            max_interval: Duration::from_secs(self.config.retry_delay_seconds * 32),
            multiplier: 2.0,
            max_elapsed_time: Some(Duration::from_secs(self.config.retry_delay_seconds * 60)),
            ..Default::default()
        };
        
        let mut last_error = None;
        
        for attempt in 0..=self.config.max_retries {
            match self.fetch_with_conditional_headers(url, etag, last_modified).await {
                Ok(response) => {
                    let response_time = start_time.elapsed().as_millis() as u64;
                    let status = response.status();
                    
                    if status == reqwest::StatusCode::NOT_MODIFIED {
                        debug!("Feed not modified: {}", url);
                        return Ok(FetchResult {
                            feed_id,
                            success: true,
                            entries_found: 0,
                            new_entries: 0,
                            error: None,
                            fetch_time,
                            response_time_ms: response_time,
                            http_status: Some(status.as_u16()),
                            etag: etag.map(|s| s.to_string()),
                            last_modified: last_modified.map(|s| s.to_string()),
                            content: None, // No content for 304 Not Modified
                        });
                    }
                    
                    if !status.is_success() {
                        last_error = Some(AggregatorError::General(format!("HTTP {}: {}", status, status.canonical_reason().unwrap_or("Unknown"))));
                        
                        if attempt < self.config.max_retries {
                            if let Some(delay) = backoff.next_backoff() {
                                warn!("Attempt {} failed for {}, retrying in {:?}", attempt + 1, url, delay);
                                tokio::time::sleep(delay).await;
                                continue;
                            }
                        }
                        break;
                    }
                    
                    // Extract headers for caching
                    let new_etag = response.headers()
                        .get("etag")
                        .and_then(|v| v.to_str().ok())
                        .map(|s| s.to_string());
                    
                    let new_last_modified = response.headers()
                        .get("last-modified")
                        .and_then(|v| v.to_str().ok())
                        .map(|s| s.to_string());
                    
                    // Check content length
                    if let Some(content_length) = response.content_length() {
                        let size_mb = content_length as usize / (1024 * 1024);
                        if size_mb > self.config.max_feed_size_mb {
                            return Ok(FetchResult {
                                feed_id,
                                success: false,
                                entries_found: 0,
                                new_entries: 0,
                                error: Some(format!("Feed too large: {}MB", size_mb)),
                                fetch_time,
                                response_time_ms: response_time,
                                http_status: Some(status.as_u16()),
                                etag: new_etag,
                                last_modified: new_last_modified,
                                content: None,
                            });
                        }
                    }
                    
                    match response.text().await {
                        Ok(content) => {
                            info!("Successfully fetched feed: {} ({} bytes)", url, content.len());
                            return Ok(FetchResult {
                                feed_id,
                                success: true,
                                entries_found: 0, // Will be set by parser
                                new_entries: 0,   // Will be set by parser
                                error: None,
                                fetch_time,
                                response_time_ms: response_time,
                                http_status: Some(status.as_u16()),
                                etag: new_etag,
                                last_modified: new_last_modified,
                                content: Some(content), // Include the RSS content!
                            });
                        }
                        Err(e) => {
                            last_error = Some(AggregatorError::Http(e));
                        }
                    }
                }
                Err(e) => {
                    last_error = Some(e);
                    
                    if attempt < self.config.max_retries {
                        if let Some(delay) = backoff.next_backoff() {
                            warn!("Attempt {} failed for {}, retrying in {:?}", attempt + 1, url, delay);
                            tokio::time::sleep(delay).await;
                            continue;
                        }
                    }
                }
            }
        }
        
        let error_msg = last_error.map(|e| e.to_string()).unwrap_or_else(|| "Unknown error".to_string());
        error!("Failed to fetch feed after {} attempts: {}", self.config.max_retries + 1, url);
        
        Ok(FetchResult {
            feed_id,
            success: false,
            entries_found: 0,
            new_entries: 0,
            error: Some(error_msg),
            fetch_time,
            response_time_ms: start_time.elapsed().as_millis() as u64,
            http_status: None,
            etag: None,
            last_modified: None,
            content: None,
        })
    }
    
    async fn fetch_with_conditional_headers(&self, url: &str, etag: Option<&str>, last_modified: Option<&str>) -> Result<Response> {
        let mut request = self.client.get(url);
        
        if let Some(etag) = etag {
            request = request.header("If-None-Match", etag);
        }
        
        if let Some(last_modified) = last_modified {
            request = request.header("If-Modified-Since", last_modified);
        }
        
        let response = request.send().await?;
        Ok(response)
    }
    
    async fn check_robots_txt(&self, url: &str) -> Result<()> {
        let parsed_url = Url::parse(url)?;
        let base_url = format!("{}://{}", parsed_url.scheme(), parsed_url.host_str().unwrap_or(""));
        
        // Check cache first
        {
            let cache = self.robots_cache.read().await;
            if let Some(&allowed) = cache.get(&base_url) {
                if allowed {
                    return Ok(());
                } else {
                    return Err(AggregatorError::RobotsDisallowed { url: url.to_string() });
                }
            }
        }
        
        // For now, just assume allowed (robots.txt parsing would require additional dependency)
        // In a production system, you'd want to implement proper robots.txt parsing
        debug!("Robots.txt check simplified - assuming allowed for {}", base_url);
        
        // Cache as allowed
        {
            let mut cache = self.robots_cache.write().await;
            cache.insert(base_url, true);
        }
        
        Ok(())
    }
    
    async fn apply_rate_limit(&self, url: &str) -> Result<()> {
        let parsed_url = Url::parse(url)?;
        let host = parsed_url.host_str().unwrap_or("").to_string();
        
        let now = Instant::now();
        let min_interval = Duration::from_secs(1); // Minimum 1 second between requests to same host
        
        {
            let mut rate_limiter = self.rate_limiter.write().await;
            
            if let Some(last_request) = rate_limiter.get(&host) {
                let elapsed = now.duration_since(*last_request);
                if elapsed < min_interval {
                    let wait_time = min_interval - elapsed;
                    debug!("Rate limiting {}: waiting {:?}", host, wait_time);
                    tokio::time::sleep(wait_time).await;
                }
            }
            
            rate_limiter.insert(host, now);
        }
        
        Ok(())
    }
    
    pub async fn fetch_full_content(&self, url: &str) -> Result<String> {
        debug!("Fetching full content from: {}", url);
        
        // Apply rate limiting
        self.apply_rate_limit(url).await?;
        
        let response = self.client.get(url).send().await?;
        
        if !response.status().is_success() {
            return Err(AggregatorError::General(format!(
                "HTTP {}: {}", 
                response.status(), 
                response.status().canonical_reason().unwrap_or("Unknown")
            )));
        }
        
        let content = response.text().await?;
        Ok(content)
    }
    
    pub fn update_config(&mut self, config: FetchConfig) {
        self.config = config;
    }
}