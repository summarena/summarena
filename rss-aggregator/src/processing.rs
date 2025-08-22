use crate::types::{InputItem, Result, DigestPreferences, DigestModelMemory};
use async_trait::async_trait;
use interfaces::defs::InputItemReference;
use std::collections::HashMap;
use tracing::debug;

/// Trait for processing pipeline stages
#[async_trait]
pub trait ProcessingStage: Send + Sync {
    /// Process input and produce output
    async fn process(&mut self, input: ProcessingInput) -> Result<ProcessingOutput>;
    
    /// Get the name of this processing stage
    fn stage_name(&self) -> String;
}

/// Input to a processing stage
#[derive(Debug, Clone)]
pub struct ProcessingInput {
    pub items: Vec<ProcessedItem>,
    pub user_preferences: Option<DigestPreferences>,
    pub user_memory: Option<DigestModelMemory>,
    pub metadata: HashMap<String, String>,
}

/// Output from a processing stage
#[derive(Debug, Clone)]
pub struct ProcessingOutput {
    pub items: Vec<ProcessedItem>,
    pub metadata: HashMap<String, String>,
}

/// An item with processing metadata attached
#[derive(Debug, Clone)]
pub struct ProcessedItem {
    pub item: InputItem,
    pub relevance_score: Option<f64>,
    pub summary: Option<String>,
    pub tags: Vec<String>,
    pub references: Vec<InputItemReference>,
    pub processing_metadata: HashMap<String, String>,
}

impl ProcessedItem {
    pub fn new(item: InputItem) -> Self {
        Self {
            item,
            relevance_score: None,
            summary: None,
            tags: Vec::new(),
            references: Vec::new(),
            processing_metadata: HashMap::new(),
        }
    }
    
    pub fn with_relevance_score(mut self, score: f64) -> Self {
        self.relevance_score = Some(score);
        self
    }
    
    pub fn with_summary(mut self, summary: String) -> Self {
        self.summary = Some(summary);
        self
    }
    
    pub fn with_tags(mut self, tags: Vec<String>) -> Self {
        self.tags = tags;
        self
    }
}

/// Stage that computes relevance scores for items
pub struct RelevanceStage {
    compiled_preferences: Option<CompiledPreferences>,
}

impl RelevanceStage {
    pub fn new() -> Self {
        Self {
            compiled_preferences: None,
        }
    }
    
    fn compile_preferences(&mut self, preferences: &DigestPreferences, memory: &DigestModelMemory) {
        let keywords = extract_keywords(&preferences.description);
        let topics = extract_topics(&preferences.description, memory);
        
        self.compiled_preferences = Some(CompiledPreferences {
            look_out_for: preferences.description.clone(),
            keywords,
            topics,
            memory_context: memory.text.clone(),
        });
    }
}

#[async_trait]
impl ProcessingStage for RelevanceStage {
    async fn process(&mut self, input: ProcessingInput) -> Result<ProcessingOutput> {
        debug!("Processing {} items in relevance stage", input.items.len());
        
        // Compile preferences if we have them
        if let (Some(prefs), Some(memory)) = (&input.user_preferences, &input.user_memory) {
            self.compile_preferences(prefs, memory);
        }
        
        let mut processed_items = Vec::new();
        
        for mut item in input.items {
            let relevance_score = if let Some(ref compiled_prefs) = self.compiled_preferences {
                calculate_relevance_score(&item.item, compiled_prefs)
            } else {
                0.5 // Default neutral score when no preferences
            };
            
            item.relevance_score = Some(relevance_score);
            processed_items.push(item);
        }
        
        let mut metadata = input.metadata;
        metadata.insert("stage".to_string(), "relevance".to_string());
        metadata.insert("items_processed".to_string(), processed_items.len().to_string());
        
        Ok(ProcessingOutput {
            items: processed_items,
            metadata,
        })
    }
    
    fn stage_name(&self) -> String {
        "relevance_scorer".to_string()
    }
}

/// Stage that creates summaries for items
pub struct SummarizationStage {
    compiled_preferences: Option<CompiledPreferences>,
}

impl SummarizationStage {
    pub fn new() -> Self {
        Self {
            compiled_preferences: None,
        }
    }
    
    fn compile_preferences(&mut self, preferences: &DigestPreferences, memory: &DigestModelMemory) {
        let keywords = extract_keywords(&preferences.description);
        let topics = extract_topics(&preferences.description, memory);
        
        self.compiled_preferences = Some(CompiledPreferences {
            look_out_for: preferences.description.clone(),
            keywords,
            topics,
            memory_context: memory.text.clone(),
        });
    }
}

#[async_trait]
impl ProcessingStage for SummarizationStage {
    async fn process(&mut self, input: ProcessingInput) -> Result<ProcessingOutput> {
        debug!("Processing {} items in summarization stage", input.items.len());
        
        // Compile preferences if we have them
        if let (Some(prefs), Some(memory)) = (&input.user_preferences, &input.user_memory) {
            self.compile_preferences(prefs, memory);
        }
        
        let mut processed_items = Vec::new();
        
        for mut processed_item in input.items {
            let summary = if let Some(ref compiled_prefs) = self.compiled_preferences {
                create_item_summary(&processed_item.item, compiled_prefs).summary_text
            } else {
                create_basic_summary(&processed_item.item)
            };
            
            processed_item.summary = Some(summary);
            processed_items.push(processed_item);
        }
        
        let mut metadata = input.metadata;
        metadata.insert("stage".to_string(), "summarization".to_string());
        metadata.insert("items_summarized".to_string(), processed_items.len().to_string());
        
        Ok(ProcessingOutput {
            items: processed_items,
            metadata,
        })
    }
    
    fn stage_name(&self) -> String {
        "summarizer".to_string()
    }
}

/// Stage that filters items based on relevance scores
pub struct FilterStage {
    min_relevance_score: f64,
    max_items: Option<usize>,
}

impl FilterStage {
    pub fn new(min_relevance_score: f64) -> Self {
        Self {
            min_relevance_score,
            max_items: None,
        }
    }
    
    pub fn with_max_items(mut self, max_items: usize) -> Self {
        self.max_items = Some(max_items);
        self
    }
}

#[async_trait]
impl ProcessingStage for FilterStage {
    async fn process(&mut self, input: ProcessingInput) -> Result<ProcessingOutput> {
        debug!("Filtering {} items with min relevance score {}", input.items.len(), self.min_relevance_score);
        
        let mut filtered_items: Vec<_> = input.items
            .into_iter()
            .filter(|item| {
                item.relevance_score.unwrap_or(0.0) >= self.min_relevance_score
            })
            .collect();
        
        // Sort by relevance score (descending)
        filtered_items.sort_by(|a, b| {
            let score_a = a.relevance_score.unwrap_or(0.0);
            let score_b = b.relevance_score.unwrap_or(0.0);
            score_b.partial_cmp(&score_a).unwrap_or(std::cmp::Ordering::Equal)
        });
        
        // Limit number of items if specified
        if let Some(max_items) = self.max_items {
            filtered_items.truncate(max_items);
        }
        
        let mut metadata = input.metadata;
        metadata.insert("stage".to_string(), "filter".to_string());
        metadata.insert("items_filtered".to_string(), filtered_items.len().to_string());
        metadata.insert("min_relevance_score".to_string(), self.min_relevance_score.to_string());
        
        Ok(ProcessingOutput {
            items: filtered_items,
            metadata,
        })
    }
    
    fn stage_name(&self) -> String {
        "filter".to_string()
    }
}

// Import types and functions from other modules
use crate::digest::{CompiledPreferences, calculate_relevance_score, create_item_summary};
use crate::utils::{extract_keywords, extract_topics, extract_title};

/// Create a basic summary without preferences
fn create_basic_summary(input_item: &InputItem) -> String {
    let text = &input_item.text;
    let title = extract_title(text);
    
    if text.len() > 150 {
        let excerpt = &text[..150];
        if let Some(last_sentence) = excerpt.rfind('.') {
            format!("{}: {}", title, &excerpt[..last_sentence + 1])
        } else {
            format!("{}: {}...", title, excerpt)
        }
    } else {
        format!("{}: {}", title, text)
    }
}

impl Default for RelevanceStage {
    fn default() -> Self {
        Self::new()
    }
}

impl Default for SummarizationStage {
    fn default() -> Self {
        Self::new()
    }
}