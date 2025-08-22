use crate::types::DigestModelMemory;

/// Extract important keywords from text description
pub fn extract_keywords(description: &str) -> Vec<String> {
    // Simple keyword extraction - in production, you might use NLP libraries
    let words: Vec<String> = description
        .to_lowercase()
        .split_whitespace()
        .filter(|word| word.len() > 3)
        .filter(|word| !is_stop_word(word))
        .map(|word| word.trim_matches(|c: char| !c.is_alphanumeric()).to_string())
        .filter(|word| !word.is_empty())
        .collect();
    
    // Remove duplicates
    let mut unique_words = words;
    unique_words.sort();
    unique_words.dedup();
    
    unique_words
}

/// Extract topics from preferences and memory
pub fn extract_topics(description: &str, memory: &DigestModelMemory) -> Vec<String> {
    let mut topics = Vec::new();
    
    // Look for explicit topics in preferences
    if description.contains("technology") || description.contains("tech") {
        topics.push("technology".to_string());
    }
    if description.contains("politics") || description.contains("political") {
        topics.push("politics".to_string());
    }
    if description.contains("business") || description.contains("economy") {
        topics.push("business".to_string());
    }
    if description.contains("science") || description.contains("research") {
        topics.push("science".to_string());
    }
    if description.contains("sports") || description.contains("games") {
        topics.push("sports".to_string());
    }
    
    // Extract topics from memory context
    if memory.text.contains("AI") || memory.text.contains("artificial intelligence") {
        topics.push("artificial-intelligence".to_string());
    }
    
    topics
}

/// Extract title from RSS item text
pub fn extract_title(text: &str) -> String {
    if let Some(title_start) = text.find("Title: ") {
        let title_portion = &text[title_start + 7..];
        if let Some(title_end) = title_portion.find('\n') {
            title_portion[..title_end].trim().to_string()
        } else {
            title_portion.trim().to_string()
        }
    } else {
        "RSS Item".to_string()
    }
}

/// Check if a word is a common stop word
pub fn is_stop_word(word: &str) -> bool {
    matches!(
        word,
        "the" | "and" | "or" | "but" | "in" | "on" | "at" | "to" | "for" | "of" | "with" | "by" | 
        "a" | "an" | "is" | "are" | "was" | "were" | "be" | "been" | "have" | "has" | "had" | 
        "do" | "does" | "did" | "will" | "would" | "could" | "should" | "may" | "might" | "must" | 
        "can" | "this" | "that" | "these" | "those"
    )
}

/// Text processing utilities
pub mod text {
    /// Truncate text to a maximum length, trying to break at sentence boundaries
    pub fn smart_truncate(text: &str, max_length: usize) -> String {
        if text.len() <= max_length {
            return text.to_string();
        }
        
        let truncated = &text[..max_length];
        if let Some(last_sentence) = truncated.rfind('.') {
            format!("{}", &truncated[..last_sentence + 1])
        } else if let Some(last_space) = truncated.rfind(' ') {
            format!("{}...", &truncated[..last_space])
        } else {
            format!("{}...", truncated)
        }
    }
    
    /// Extract the first N sentences from text
    pub fn extract_sentences(text: &str, count: usize) -> String {
        let sentences: Vec<&str> = text.split('.').take(count).collect();
        sentences.join(".") + if sentences.len() == count { "." } else { "" }
    }
    
    /// Clean and normalize text for processing
    pub fn normalize_text(text: &str) -> String {
        text.chars()
            .filter(|c| c.is_alphanumeric() || c.is_whitespace() || ".,!?;:-".contains(*c))
            .collect::<String>()
            .split_whitespace()
            .collect::<Vec<_>>()
            .join(" ")
    }
}

/// URL utilities
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

/// Time utilities
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