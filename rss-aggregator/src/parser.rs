use crate::types::{AggregatorError, FeedEntry, ParsedEntry, ParsedFeed, Result};
use chrono::Utc;
use feed_rs::parser;
use std::collections::HashSet;
use tracing::{debug, info};
use uuid::Uuid;

pub struct FeedParser {
    seen_guids: HashSet<String>,
    seen_urls: HashSet<String>,
}

impl FeedParser {
    pub fn new() -> Self {
        Self {
            seen_guids: HashSet::new(),
            seen_urls: HashSet::new(),
        }
    }

    pub fn parse_feed(&mut self, content: &str) -> Result<ParsedFeed> {
        debug!("Parsing feed content ({} bytes)", content.len());
        
        // Attempt to parse the feed
        let feed = parser::parse(content.as_bytes())
            .map_err(|e| AggregatorError::Parse(format!("Failed to parse feed: {}", e)))?;

        let title = feed.title.map(|t| t.content);
        let description = feed.description.map(|d| d.content);
        
        // Extract HTTP cache headers if available (these would typically come from the HTTP response)
        let etag = None; // Will be set by the fetcher
        let last_modified = None; // Will be set by the fetcher
        
        let mut entries = Vec::new();
        
        for entry in feed.entries {
            if let Some(parsed_entry) = self.parse_entry(entry) {
                entries.push(parsed_entry);
            }
        }
        
        info!("Parsed feed with {} entries", entries.len());
        
        Ok(ParsedFeed {
            title,
            description,
            entries,
            etag,
            last_modified,
        })
    }
    
    fn parse_entry(&mut self, entry: feed_rs::model::Entry) -> Option<ParsedEntry> {
        // Extract basic information
        let title = entry.title.map(|t| t.content).unwrap_or_else(|| "Untitled".to_string());
        
        // Get the primary link
        let url = entry.links.first()?.href.clone();
        
        // Check for duplicates based on GUID or URL
        let guid = if !entry.id.is_empty() {
            Some(entry.id.clone())
        } else {
            None
        };
        
        if let Some(ref guid) = guid {
            if self.seen_guids.contains(guid) {
                debug!("Skipping duplicate entry with GUID: {}", guid);
                return None;
            }
            self.seen_guids.insert(guid.clone());
        }
        
        if self.seen_urls.contains(&url) {
            debug!("Skipping duplicate entry with URL: {}", url);
            return None;
        }
        self.seen_urls.insert(url.clone());
        
        // Extract description
        let description = entry.summary.map(|s| s.content);
        
        // Extract content (prefer content over summary)
        let content = if let Some(content_data) = &entry.content {
            content_data.body.clone()
        } else {
            description.clone()
        };
        
        // Extract author
        let author = entry.authors.first().map(|a| a.name.clone());
        
        // Extract dates
        let published_at = entry.published.map(|dt| dt.with_timezone(&Utc));
        let updated_at = entry.updated.map(|dt| dt.with_timezone(&Utc));
        
        // Extract categories/tags
        let tags = entry.categories.into_iter()
            .map(|c| c.term)
            .collect();
        
        Some(ParsedEntry {
            guid,
            url,
            title,
            description,
            content,
            author,
            published_at,
            updated_at,
            tags,
        })
    }
    
    pub fn convert_to_feed_entries(&self, parsed_feed: &ParsedFeed, feed_id: Uuid) -> Vec<FeedEntry> {
        let now = Utc::now();
        
        parsed_feed.entries.iter().map(|entry| {
            FeedEntry {
                id: Uuid::new_v4(),
                feed_id,
                guid: entry.guid.clone(),
                url: entry.url.clone(),
                title: entry.title.clone(),
                description: entry.description.clone(),
                content: entry.content.clone(),
                author: entry.author.clone(),
                published_at: entry.published_at,
                updated_at: entry.updated_at,
                tags: entry.tags.clone(),
                created_at: now,
                last_processed: None,
            }
        }).collect()
    }
    
    pub fn is_valid_feed_content(content: &str) -> bool {
        // Basic validation to check if content might be a valid RSS/Atom feed
        let content_lower = content.to_lowercase();
        
        // Check for RSS/Atom indicators
        let has_rss_indicators = content_lower.contains("<rss") 
            || content_lower.contains("<feed") 
            || content_lower.contains("xmlns=\"http://www.w3.org/2005/atom\"")
            || content_lower.contains("xmlns:atom")
            || content_lower.contains("<channel");
        
        // Additional validation: check for XML declaration
        let has_xml_declaration = content.trim_start().starts_with("<?xml");
        
        // Must have RSS/Atom indicators and be well-formed enough to contain basic tags
        has_rss_indicators && (has_xml_declaration || content_lower.contains("<"))
    }
    
    pub fn extract_feed_info(content: &str) -> Result<(Option<String>, Option<String>)> {
        match parser::parse(content.as_bytes()) {
            Ok(feed) => {
                let title = feed.title.map(|t| t.content);
                let description = feed.description.map(|d| d.content);
                Ok((title, description))
            }
            Err(e) => Err(AggregatorError::Parse(format!("Failed to extract feed info: {}", e)))
        }
    }
    
    pub fn normalize_encoding(content: &str) -> Result<String> {
        // Basic encoding normalization
        // In a production system, you might want to use a library like encoding_rs
        // to handle various encodings properly
        
        // For now, we'll assume UTF-8 and do basic cleanup
        let normalized = content
            .replace("\r\n", "\n")  // Normalize line endings
            .replace("\r", "\n")    // Normalize line endings
            .trim()
            .to_string();
        
        // Check if content looks like valid UTF-8
        if normalized.is_empty() {
            return Err(AggregatorError::Parse("Empty content after normalization".to_string()));
        }
        
        Ok(normalized)
    }
    
    pub fn deduplicate_entries(&mut self, entries: &[FeedEntry]) -> Vec<FeedEntry> {
        let mut unique_entries = Vec::new();
        let mut seen_guids = HashSet::new();
        let mut seen_urls = HashSet::new();
        
        for entry in entries {
            let mut is_duplicate = false;
            
            // Check GUID first (most reliable)
            if let Some(ref guid) = entry.guid {
                if seen_guids.contains(guid) {
                    is_duplicate = true;
                } else {
                    seen_guids.insert(guid.clone());
                }
            }
            
            // Check URL if not already marked as duplicate
            if !is_duplicate {
                if seen_urls.contains(&entry.url) {
                    is_duplicate = true;
                } else {
                    seen_urls.insert(entry.url.clone());
                }
            }
            
            if !is_duplicate {
                unique_entries.push(entry.clone());
            } else {
                debug!("Removing duplicate entry: {} ({})", entry.title, entry.url);
            }
        }
        
        let removed_count = entries.len() - unique_entries.len();
        if removed_count > 0 {
            info!("Removed {} duplicate entries", removed_count);
        }
        
        unique_entries
    }
    
    pub fn clear_deduplication_cache(&mut self) {
        self.seen_guids.clear();
        self.seen_urls.clear();
        debug!("Cleared deduplication cache");
    }
    
    pub fn get_cache_stats(&self) -> (usize, usize) {
        (self.seen_guids.len(), self.seen_urls.len())
    }
}