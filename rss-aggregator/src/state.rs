use crate::types::{InputItem, LiveSourceSpec, Result, AggregatorError};
use crate::FeedManager;
use std::sync::Arc;
use uuid::Uuid;
use chrono::Utc;
use tracing::{info, error};
use sqlx::Row;

/// State manager that implements the ingest function from interfaces
pub struct RssState {
    feed_manager: Arc<FeedManager>,
}

impl RssState {
    pub fn new(feed_manager: Arc<FeedManager>) -> Self {
        Self { feed_manager }
    }
    
    /// Implementation of the ingest function from interfaces::state
    /// This saves InputItem to the database as normalized entries
    pub async fn ingest(&self, source: &LiveSourceSpec, input_item: InputItem) -> Result<()> {
        // Convert InputItem back to a FeedEntry-like structure for storage
        // In a real implementation, you might want a more direct storage mechanism
        
        // First, check if we have a feed for this source
        let feed_id = match self.get_or_create_feed_for_source(source).await? {
            Some(id) => id,
            None => {
                error!("Failed to get or create feed for source: {}", source.uri);
                return Err(AggregatorError::General("Failed to get feed for source".to_string()));
            }
        };
        
        // Store the input item as a feed entry
        let entry_id = self.store_input_item(feed_id, &input_item).await?;
        
        info!("Successfully ingested item {} for source {}", entry_id, source.uri);
        Ok(())
    }
    
    async fn get_or_create_feed_for_source(&self, source: &LiveSourceSpec) -> Result<Option<Uuid>> {
        // Try to find existing feed by URL
        match self.feed_manager.get_feed_by_url(&source.uri).await {
            Ok(feed) => Ok(Some(feed.id)),
            Err(AggregatorError::FeedNotFound { .. }) => {
                // Feed doesn't exist, create it
                match self.feed_manager.add_feed(source.uri.clone(), None, None).await {
                    Ok(feed_id) => Ok(Some(feed_id)),
                    Err(e) => {
                        error!("Failed to create feed for source {}: {}", source.uri, e);
                        Err(e)
                    }
                }
            }
            Err(e) => Err(e),
        }
    }
    
    async fn store_input_item(&self, feed_id: Uuid, input_item: &InputItem) -> Result<Uuid> {
        // Create a feed entry from the input item
        let entry_id = Uuid::new_v4();
        let now = Utc::now();
        
        // Parse the input item text to extract title and content
        // The InputItem text format is: "Title: {title}\n\nDescription: {description}\n\nContent: {content}"
        let (title, description, content) = self.parse_input_item_text(&input_item.text);
        
        // Store in the input_items table (new table for normalized InputItems)
        self.store_as_input_item_record(entry_id, feed_id, input_item, &title, &description, &content).await?;
        
        Ok(entry_id)
    }
    
    fn parse_input_item_text(&self, text: &str) -> (String, Option<String>, Option<String>) {
        let lines: Vec<&str> = text.split('\n').collect();
        let mut title = String::new();
        let mut description: Option<String> = None;
        let mut content: Option<String> = None;
        
        let mut current_section = "";
        let mut current_content = String::new();
        
        for line in lines {
            if line.starts_with("Title: ") {
                if !current_content.is_empty() && current_section == "content" {
                    content = Some(current_content.trim().to_string());
                    current_content.clear();
                }
                title = line.strip_prefix("Title: ").unwrap_or("").to_string();
                current_section = "title";
            } else if line.starts_with("Description: ") {
                if !current_content.is_empty() && current_section == "content" {
                    content = Some(current_content.trim().to_string());
                    current_content.clear();
                }
                let desc = line.strip_prefix("Description: ").unwrap_or("").to_string();
                if !desc.is_empty() {
                    description = Some(desc);
                }
                current_section = "description";
            } else if line.starts_with("Content: ") {
                let cont = line.strip_prefix("Content: ").unwrap_or("").to_string();
                if !cont.is_empty() {
                    current_content = cont;
                } else {
                    current_content.clear();
                }
                current_section = "content";
            } else if current_section == "content" && !line.trim().is_empty() {
                if !current_content.is_empty() {
                    current_content.push('\n');
                }
                current_content.push_str(line);
            } else if current_section == "description" && !line.trim().is_empty() {
                if let Some(ref mut desc) = description {
                    desc.push('\n');
                    desc.push_str(line);
                } else {
                    description = Some(line.to_string());
                }
            }
        }
        
        // Handle final content section
        if !current_content.is_empty() && current_section == "content" {
            content = Some(current_content.trim().to_string());
        }
        
        (title, description, content)
    }
    
    async fn store_as_input_item_record(
        &self, 
        entry_id: Uuid, 
        feed_id: Uuid, 
        input_item: &InputItem, 
        title: &str,
        description: &Option<String>,
        content: &Option<String>
    ) -> Result<()> {
        let db = self.feed_manager.get_db_pool();
        let now = Utc::now();
        
        // Store in input_items table (create if doesn't exist)
        sqlx::query(
            r#"
            INSERT INTO input_items (id, feed_id, uri, title, description, content, vision_data, text_content, created_at, updated_at)
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10)
            ON CONFLICT (uri) DO UPDATE SET 
                title = EXCLUDED.title,
                description = EXCLUDED.description,
                content = EXCLUDED.content,
                vision_data = EXCLUDED.vision_data,
                text_content = EXCLUDED.text_content,
                updated_at = EXCLUDED.updated_at
            "#,
        )
        .bind(entry_id)
        .bind(feed_id)
        .bind(&input_item.uri)
        .bind(title)
        .bind(description)
        .bind(content)
        .bind(&input_item.vision)
        .bind(&input_item.text)
        .bind(now)
        .bind(now)
        .execute(db)
        .await?;
        
        Ok(())
    }
    
    /// Get recent InputItems from storage
    pub async fn get_recent_input_items(&self, limit: usize) -> Result<Vec<InputItem>> {
        let db = self.feed_manager.get_db_pool();
        
        let rows = sqlx::query(
            r#"
            SELECT uri, text_content, vision_data 
            FROM input_items 
            ORDER BY created_at DESC 
            LIMIT $1
            "#,
        )
        .bind(limit as i64)
        .fetch_all(db)
        .await?;
        
        let input_items = rows
            .into_iter()
            .map(|row| InputItem {
                uri: row.try_get("uri").unwrap_or_default(),
                text: row.try_get("text_content").unwrap_or_default(),
                vision: row.try_get::<Option<Vec<u8>>, _>("vision_data").unwrap_or_default().unwrap_or_default(),
            })
            .collect();
            
        Ok(input_items)
    }
    
    /// Get InputItems for a specific source
    pub async fn get_input_items_for_source(&self, source: &LiveSourceSpec, limit: usize) -> Result<Vec<InputItem>> {
        let db = self.feed_manager.get_db_pool();
        
        // First get the feed ID for the source
        let feed_metadata = self.feed_manager.get_feed_by_url(&source.uri).await?;
        
        let rows = sqlx::query(
            r#"
            SELECT uri, text_content, vision_data 
            FROM input_items 
            WHERE feed_id = $1
            ORDER BY created_at DESC 
            LIMIT $2
            "#,
        )
        .bind(feed_metadata.id)
        .bind(limit as i64)
        .fetch_all(db)
        .await?;
        
        let input_items = rows
            .into_iter()
            .map(|row| InputItem {
                uri: row.try_get("uri").unwrap_or_default(),
                text: row.try_get("text_content").unwrap_or_default(),
                vision: row.try_get::<Option<Vec<u8>>, _>("vision_data").unwrap_or_default().unwrap_or_default(),
            })
            .collect();
            
        Ok(input_items)
    }
}

/// Standalone implementation of the ingest function from interfaces::state
/// This provides the global entry point that the interfaces expect
pub async fn ingest(source: &LiveSourceSpec, input_item: InputItem) -> Result<()> {
    // This is a simplified implementation. In production, you'd want to:
    // 1. Get database connection from a global pool or configuration
    // 2. Use dependency injection to provide the FeedManager
    
    // For now, we'll need the caller to use RssState directly
    // or we could store a global instance
    
    // This is a placeholder - the real implementation should be called through RssState
    info!("Global ingest called for source: {}, item: {}", source.uri, input_item.uri);
    
    // In a real implementation, you might have:
    // GLOBAL_STATE.ingest(source, input_item).await
    
    Ok(())
}