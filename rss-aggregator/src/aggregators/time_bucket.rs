use crate::traits::{Aggregator, AggregatedOutput, AggregatorConfig};
use crate::types::{InputItem, Result};
use async_trait::async_trait;
use chrono::{DateTime, Utc, Duration};
use std::collections::HashMap;
use tracing::{info, debug};

/// Simple time-bucket aggregator that produces daily digests
pub struct TimeBucketAggregator {
    user_id: String,
    bucket_duration_hours: i64,
    last_output_time: Option<DateTime<Utc>>,
    items: Vec<InputItem>,
    max_items_per_bucket: usize,
}

impl TimeBucketAggregator {
    pub fn new(user_id: String) -> Self {
        Self {
            user_id,
            bucket_duration_hours: 24, // Default to daily buckets
            last_output_time: None,
            items: Vec::new(),
            max_items_per_bucket: 50, // Limit to prevent overwhelming digests
        }
    }
    
    pub fn new_with_duration(user_id: String, bucket_duration_hours: i64) -> Self {
        Self {
            user_id,
            bucket_duration_hours,
            last_output_time: None,
            items: Vec::new(),
            max_items_per_bucket: 50,
        }
    }
    
    /// Create a daily digest aggregator (24-hour buckets)
    pub fn daily(user_id: String) -> Self {
        Self::new_with_duration(user_id, 24)
    }
    
    /// Create an hourly digest aggregator (1-hour buckets)
    pub fn hourly(user_id: String) -> Self {
        Self::new_with_duration(user_id, 1)
    }
    
    /// Create a weekly digest aggregator (168-hour buckets)
    pub fn weekly(user_id: String) -> Self {
        Self::new_with_duration(user_id, 168)
    }
    
    fn is_bucket_ready(&self) -> bool {
        match self.last_output_time {
            None => !self.items.is_empty(), // First bucket ready when we have items
            Some(last_output) => {
                let now = Utc::now();
                let bucket_duration = Duration::hours(self.bucket_duration_hours);
                now.signed_duration_since(last_output) >= bucket_duration
            }
        }
    }
    
    fn create_summary(&self) -> String {
        if self.items.is_empty() {
            return "No items in this time bucket.".to_string();
        }
        
        let bucket_type = match self.bucket_duration_hours {
            1 => "Hourly",
            24 => "Daily",
            168 => "Weekly",
            _ => "Time Bucket",
        };
        
        format!(
            "{} Digest for {}\n\nCollected {} items:\n\n{}",
            bucket_type,
            self.user_id,
            self.items.len(),
            self.items
                .iter()
                .enumerate()
                .take(10) // Show first 10 items in summary
                .map(|(i, item)| {
                    let title = extract_title_from_item(item);
                    format!("{}. {} ({})", i + 1, title, item.uri)
                })
                .collect::<Vec<_>>()
                .join("\n")
        )
    }
}

#[async_trait]
impl Aggregator for TimeBucketAggregator {
    fn aggregator_type(&self) -> String {
        format!("time_bucket_{}h", self.bucket_duration_hours)
    }
    
    async fn add_item(&mut self, item: InputItem) -> Result<()> {
        debug!("Adding item to time bucket for user {}: {}", self.user_id, item.uri);
        
        // Add item to current bucket, respecting the maximum items limit
        if self.items.len() < self.max_items_per_bucket {
            self.items.push(item);
        } else {
            // If we're at the limit, replace the oldest item
            // This could be improved with more sophisticated prioritization
            self.items.remove(0);
            self.items.push(item);
        }
        
        Ok(())
    }
    
    fn should_produce_output(&self) -> bool {
        self.is_bucket_ready() && !self.items.is_empty()
    }
    
    async fn produce_output(&mut self) -> Result<AggregatedOutput> {
        if !self.should_produce_output() {
            return Err(crate::types::AggregatorError::General(
                "Not ready to produce output".to_string()
            ));
        }
        
        let summary = self.create_summary();
        let items = std::mem::take(&mut self.items); // Move items out, leaving empty vec
        
        let mut metadata = HashMap::new();
        metadata.insert("bucket_duration_hours".to_string(), self.bucket_duration_hours.to_string());
        metadata.insert("items_count".to_string(), items.len().to_string());
        
        let output = AggregatedOutput {
            user_id: self.user_id.clone(),
            aggregator_type: self.aggregator_type(),
            items,
            summary: Some(summary),
            created_at: Utc::now(),
            metadata,
        };
        
        // Update last output time
        self.last_output_time = Some(output.created_at);
        
        info!("Produced time bucket output for user {} with {} items", 
              self.user_id, output.items.len());
        
        Ok(output)
    }
    
    fn user_id(&self) -> String {
        self.user_id.clone()
    }
    
    async fn configure(&mut self, config: AggregatorConfig) -> Result<()> {
        // Configure bucket duration if specified
        if let Some(duration_str) = config.parameters.get("bucket_duration_hours") {
            if let Ok(duration) = duration_str.parse::<i64>() {
                self.bucket_duration_hours = duration;
                info!("Updated bucket duration to {} hours for user {}", duration, self.user_id);
            }
        }
        
        // Configure max items per bucket if specified
        if let Some(max_items_str) = config.parameters.get("max_items_per_bucket") {
            if let Ok(max_items) = max_items_str.parse::<usize>() {
                self.max_items_per_bucket = max_items;
                info!("Updated max items per bucket to {} for user {}", max_items, self.user_id);
            }
        }
        
        Ok(())
    }
}

/// Extract a title from an InputItem's text
fn extract_title_from_item(item: &InputItem) -> String {
    let text = &item.text;
    
    // Look for "Title: " prefix in RSS items
    if let Some(title_start) = text.find("Title: ") {
        let title_portion = &text[title_start + 7..];
        if let Some(title_end) = title_portion.find('\n') {
            return title_portion[..title_end].trim().to_string();
        } else if title_portion.len() <= 100 {
            return title_portion.trim().to_string();
        }
    }
    
    // Fallback: use first line or first 100 characters
    if let Some(first_line) = text.lines().next() {
        if first_line.len() <= 100 {
            first_line.trim().to_string()
        } else {
            format!("{}...", &first_line[..97].trim())
        }
    } else {
        "Untitled Item".to_string()
    }
}