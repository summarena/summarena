/// RSS-specific utility functions for the ingester

/// URL utilities for RSS feeds
pub mod url {
    use url::Url;
    
    /// Extract domain from URL
    pub fn extract_domain(url_str: &str) -> Option<String> {
        if let Ok(url) = Url::parse(url_str) {
            url.domain().map(|d| d.to_string())
        } else {
            None
        }
    }
    
    /// Check if URL is from a news domain
    pub fn is_news_url(url_str: &str) -> bool {
        let news_domains = [
            "reuters.com", "ap.org", "bbc.com", "cnn.com", "wsj.com", 
            "nytimes.com", "washingtonpost.com", "bloomberg.com", "ft.com"
        ];
        
        if let Some(domain) = extract_domain(url_str) {
            news_domains.iter().any(|&news_domain| domain.contains(news_domain))
        } else {
            false
        }
    }
    
    /// Validate RSS feed URL format
    pub fn is_valid_rss_url(url_str: &str) -> bool {
        if let Ok(url) = Url::parse(url_str) {
            url.scheme() == "http" || url.scheme() == "https"
        } else {
            false
        }
    }
}

/// Time utilities for RSS polling
pub mod time {
    use chrono::{DateTime, Utc, Duration};
    
    /// Check if enough time has passed since last update
    pub fn should_update(last_update: Option<DateTime<Utc>>, interval_seconds: i64) -> bool {
        match last_update {
            None => true,
            Some(last) => {
                let now = Utc::now();
                let duration = Duration::seconds(interval_seconds);
                now.signed_duration_since(last) >= duration
            }
        }
    }
    
    /// Calculate optimal polling interval from RSS feed item timestamps
    /// Uses outlier-removed minimum interval as suggested in PR review
    pub fn calculate_optimal_interval(timestamps: &[DateTime<Utc>]) -> Duration {
        if timestamps.len() < 2 {
            return Duration::hours(1); // Default 1 hour
        }
        
        // Calculate intervals between consecutive items
        let mut intervals = Vec::new();
        for i in 1..timestamps.len() {
            let interval = timestamps[i-1].signed_duration_since(timestamps[i]);
            if interval > Duration::zero() {
                intervals.push(interval);
            }
        }
        
        if intervals.is_empty() {
            return Duration::hours(1);
        }
        
        // Sort intervals
        intervals.sort();
        
        // Remove outliers (bottom 10% and top 10%)
        let len = intervals.len();
        let start_idx = len / 10;
        let end_idx = len - (len / 10);
        
        if start_idx >= end_idx {
            // Too few intervals, use median
            return intervals[len / 2];
        }
        
        let filtered_intervals = &intervals[start_idx..end_idx];
        
        // Use minimum of filtered intervals
        filtered_intervals.iter().min().cloned()
            .unwrap_or_else(|| Duration::hours(1))
            .max(Duration::minutes(15)) // Don't poll more than every 15 minutes
            .min(Duration::hours(24))   // Don't poll less than daily
    }
    
    /// Format duration in human-readable form
    pub fn format_duration(duration: Duration) -> String {
        let total_seconds = duration.num_seconds();
        
        if total_seconds < 60 {
            format!("{}s", total_seconds)
        } else if total_seconds < 3600 {
            format!("{}m", total_seconds / 60)
        } else if total_seconds < 86400 {
            format!("{}h", total_seconds / 3600)
        } else {
            format!("{}d", total_seconds / 86400)
        }
    }
}

/// RSS feed parsing utilities
pub mod feed {
    /// Extract clean text content from HTML
    pub fn extract_text_from_html(html: &str) -> String {
        // Simple HTML tag removal - in production you might use html2text or similar
        html.chars()
            .fold((String::new(), false), |(mut text, in_tag), c| {
                match c {
                    '<' => (text, true),
                    '>' => (text, false),
                    _ if !in_tag => {
                        text.push(c);
                        (text, in_tag)
                    },
                    _ => (text, in_tag),
                }
            })
            .0
            .split_whitespace()
            .collect::<Vec<_>>()
            .join(" ")
    }
    
    /// Estimate reading time based on content length
    pub fn estimate_reading_time_minutes(content: &str) -> u32 {
        // Average reading speed: ~200 words per minute
        let word_count = content.split_whitespace().count();
        ((word_count as f64 / 200.0).ceil() as u32).max(1)
    }
}