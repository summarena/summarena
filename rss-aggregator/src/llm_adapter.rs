use crate::types::{InputItem, Result, AggregatorError, DigestPreferences, DigestModelMemory};
use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use tracing::{info, debug};

/// Trait for LLM adapters that can process content
#[async_trait]
pub trait LlmAdapter: Send + Sync {
    /// Get the name of this LLM adapter
    fn adapter_name(&self) -> String;
    
    /// Compile user preferences into a structured format for the LLM
    async fn compile_preferences(
        &self,
        preferences: &DigestPreferences,
        memory: &DigestModelMemory,
    ) -> Result<CompiledLlmPreferences>;
    
    /// Score the relevance of an item based on user preferences
    async fn score_relevance(
        &self,
        item: &InputItem,
        compiled_preferences: &CompiledLlmPreferences,
    ) -> Result<f64>;
    
    /// Create a focused summary of an item
    async fn create_summary(
        &self,
        item: &InputItem,
        compiled_preferences: &CompiledLlmPreferences,
    ) -> Result<String>;
    
    /// Extract key topics and entities from an item
    async fn extract_entities(
        &self,
        item: &InputItem,
    ) -> Result<Vec<String>>;
    
    /// Generate a digest from multiple items
    async fn generate_digest(
        &self,
        items: &[InputItem],
        compiled_preferences: &CompiledLlmPreferences,
    ) -> Result<String>;
}

/// Compiled preferences optimized for LLM processing
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CompiledLlmPreferences {
    pub user_interests: Vec<String>,
    pub keywords: Vec<String>,
    pub topics: Vec<String>,
    pub content_types: Vec<String>,
    pub relevance_threshold: f64,
    pub summary_style: String,
    pub context: String,
}

/// Mock LLM adapter for development and testing
pub struct MockLlmAdapter {
    name: String,
    response_delay_ms: u64,
}

impl MockLlmAdapter {
    pub fn new(name: String) -> Self {
        Self {
            name,
            response_delay_ms: 100, // Simulate processing time
        }
    }
    
    pub fn with_delay(mut self, delay_ms: u64) -> Self {
        self.response_delay_ms = delay_ms;
        self
    }
    
    async fn simulate_processing(&self) {
        if self.response_delay_ms > 0 {
            tokio::time::sleep(tokio::time::Duration::from_millis(self.response_delay_ms)).await;
        }
    }
    
    fn extract_title(&self, text: &str) -> String {
        if let Some(title_start) = text.find("Title: ") {
            let title_portion = &text[title_start + 7..];
            if let Some(title_end) = title_portion.find('\n') {
                title_portion[..title_end].trim().to_string()
            } else {
                title_portion.trim().to_string()
            }
        } else {
            "Untitled Item".to_string()
        }
    }
}

#[async_trait]
impl LlmAdapter for MockLlmAdapter {
    fn adapter_name(&self) -> String {
        format!("Mock LLM Adapter ({})", self.name)
    }
    
    async fn compile_preferences(
        &self,
        preferences: &DigestPreferences,
        memory: &DigestModelMemory,
    ) -> Result<CompiledLlmPreferences> {
        self.simulate_processing().await;
        
        debug!("Compiling preferences for mock LLM adapter");
        
        // Simple keyword extraction from description
        let keywords: Vec<String> = preferences.description
            .split_whitespace()
            .filter(|word| word.len() > 3)
            .map(|word| word.to_lowercase())
            .take(10)
            .collect();
        
        // Extract topics based on common patterns
        let mut topics = Vec::new();
        let desc_lower = preferences.description.to_lowercase();
        if desc_lower.contains("tech") || desc_lower.contains("technology") {
            topics.push("technology".to_string());
        }
        if desc_lower.contains("business") || desc_lower.contains("finance") {
            topics.push("business".to_string());
        }
        if desc_lower.contains("politics") || desc_lower.contains("political") {
            topics.push("politics".to_string());
        }
        
        Ok(CompiledLlmPreferences {
            user_interests: vec![preferences.description.clone()],
            keywords,
            topics,
            content_types: vec!["news".to_string(), "articles".to_string()],
            relevance_threshold: 0.3,
            summary_style: "concise".to_string(),
            context: memory.text.clone(),
        })
    }
    
    async fn score_relevance(
        &self,
        item: &InputItem,
        compiled_preferences: &CompiledLlmPreferences,
    ) -> Result<f64> {
        self.simulate_processing().await;
        
        let text = item.text.to_lowercase();
        let mut score: f64 = 0.0;
        
        // Score based on keyword matches
        for keyword in &compiled_preferences.keywords {
            if text.contains(keyword) {
                score += 0.2;
            }
        }
        
        // Score based on topic matches
        for topic in &compiled_preferences.topics {
            if text.contains(topic) {
                score += 0.3;
            }
        }
        
        // Score based on user interests
        for interest in &compiled_preferences.user_interests {
            if text.contains(&interest.to_lowercase()) {
                score += 0.4;
            }
        }
        
        // Normalize to 0-1 range
        Ok(score.min(1.0))
    }
    
    async fn create_summary(
        &self,
        item: &InputItem,
        _compiled_preferences: &CompiledLlmPreferences,
    ) -> Result<String> {
        self.simulate_processing().await;
        
        let title = self.extract_title(&item.text);
        
        // Create a simple extractive summary
        let sentences: Vec<&str> = item.text.split('.').take(2).collect();
        let summary = if sentences.len() > 1 {
            format!("{}: {}.{}", title, sentences[0], sentences.get(1).unwrap_or(&""))
        } else {
            format!("{}: {}", title, sentences.first().unwrap_or(&"No content"))
        };
        
        Ok(summary.trim().to_string())
    }
    
    async fn extract_entities(
        &self,
        item: &InputItem,
    ) -> Result<Vec<String>> {
        self.simulate_processing().await;
        
        // Simple entity extraction (in practice, you'd use NLP libraries)
        let text = &item.text;
        let mut entities = Vec::new();
        
        // Look for capitalized words as potential entities
        for word in text.split_whitespace() {
            if word.len() > 2 && word.chars().next().unwrap().is_uppercase() {
                let clean_word = word.trim_matches(|c: char| !c.is_alphabetic());
                if !clean_word.is_empty() && clean_word.len() > 2 {
                    entities.push(clean_word.to_string());
                }
            }
        }
        
        // Remove duplicates and limit count
        entities.sort();
        entities.dedup();
        entities.truncate(10);
        
        Ok(entities)
    }
    
    async fn generate_digest(
        &self,
        items: &[InputItem],
        compiled_preferences: &CompiledLlmPreferences,
    ) -> Result<String> {
        self.simulate_processing().await;
        
        if items.is_empty() {
            return Ok("No items available for digest generation.".to_string());
        }
        
        info!("Generating digest for {} items with mock LLM", items.len());
        
        let mut digest = String::new();
        digest.push_str("📰 AI-Generated Digest\n\n");
        
        // Add a summary of user interests
        if !compiled_preferences.user_interests.is_empty() {
            digest.push_str("Based on your interests: ");
            digest.push_str(&compiled_preferences.user_interests.join(", "));
            digest.push_str("\n\n");
        }
        
        // Score and sort items
        let mut scored_items = Vec::new();
        for item in items {
            if let Ok(score) = self.score_relevance(item, compiled_preferences).await {
                if score >= compiled_preferences.relevance_threshold {
                    scored_items.push((item, score));
                }
            }
        }
        
        // Sort by relevance score (descending)
        scored_items.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));
        
        // Take top items for digest
        let top_items: Vec<_> = scored_items.into_iter().take(5).collect();
        
        if top_items.is_empty() {
            digest.push_str("No items met your relevance criteria.\n");
        } else {
            digest.push_str(&format!("📝 Top {} relevant items:\n\n", top_items.len()));
            
            for (i, (item, score)) in top_items.iter().enumerate() {
                if let Ok(summary) = self.create_summary(item, compiled_preferences).await {
                    digest.push_str(&format!(
                        "{}. {} (Relevance: {:.1}%)\n   🔗 {}\n\n",
                        i + 1,
                        summary,
                        score * 100.0,
                        item.uri
                    ));
                }
            }
        }
        
        Ok(digest)
    }
}

/// LLM adapter registry for managing multiple adapters
pub struct LlmAdapterRegistry {
    adapters: HashMap<String, Box<dyn LlmAdapter>>,
    default_adapter: Option<String>,
}

impl LlmAdapterRegistry {
    pub fn new() -> Self {
        Self {
            adapters: HashMap::new(),
            default_adapter: None,
        }
    }
    
    /// Register a new LLM adapter
    pub fn register_adapter(&mut self, adapter: Box<dyn LlmAdapter>) {
        let name = adapter.adapter_name();
        info!("Registering LLM adapter: {}", name);
        
        // Set as default if it's the first adapter
        if self.default_adapter.is_none() {
            self.default_adapter = Some(name.clone());
        }
        
        self.adapters.insert(name, adapter);
    }
    
    /// Get an adapter by name
    pub fn get_adapter(&self, name: &str) -> Option<&dyn LlmAdapter> {
        self.adapters.get(name).map(|adapter| adapter.as_ref())
    }
    
    /// Get the default adapter
    pub fn get_default_adapter(&self) -> Option<&dyn LlmAdapter> {
        self.default_adapter.as_ref()
            .and_then(|name| self.get_adapter(name))
    }
    
    /// Set the default adapter
    pub fn set_default_adapter(&mut self, name: &str) -> Result<()> {
        if self.adapters.contains_key(name) {
            self.default_adapter = Some(name.to_string());
            info!("Set default LLM adapter to: {}", name);
            Ok(())
        } else {
            Err(AggregatorError::General(format!("Adapter '{}' not found", name)))
        }
    }
    
    /// List all registered adapters
    pub fn list_adapters(&self) -> Vec<String> {
        self.adapters.keys().cloned().collect()
    }
    
    /// Remove an adapter
    pub fn remove_adapter(&mut self, name: &str) -> bool {
        let removed = self.adapters.remove(name).is_some();
        
        // Update default adapter if it was removed
        if self.default_adapter.as_deref() == Some(name) {
            self.default_adapter = self.adapters.keys().next().cloned();
        }
        
        removed
    }
}

impl Default for LlmAdapterRegistry {
    fn default() -> Self {
        let mut registry = Self::new();
        
        // Register a default mock adapter
        let mock_adapter = Box::new(MockLlmAdapter::new("default".to_string()));
        registry.register_adapter(mock_adapter);
        
        registry
    }
}

/// Builder for creating LLM adapter configurations
pub struct LlmAdapterBuilder {
    registry: LlmAdapterRegistry,
}

impl LlmAdapterBuilder {
    pub fn new() -> Self {
        Self {
            registry: LlmAdapterRegistry::new(),
        }
    }
    
    pub fn add_mock_adapter(mut self, name: String, delay_ms: Option<u64>) -> Self {
        let adapter = if let Some(delay) = delay_ms {
            MockLlmAdapter::new(name).with_delay(delay)
        } else {
            MockLlmAdapter::new(name)
        };
        
        self.registry.register_adapter(Box::new(adapter));
        self
    }
    
    pub fn set_default(mut self, name: &str) -> Result<Self> {
        self.registry.set_default_adapter(name)?;
        Ok(self)
    }
    
    pub fn build(self) -> LlmAdapterRegistry {
        self.registry
    }
}

impl Default for LlmAdapterBuilder {
    fn default() -> Self {
        Self::new()
    }
}