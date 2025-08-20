use crate::types::{
    DigestModel, DigestModelSpec, DigestModelMemory, DigestPreferences, DigestOutput,
    InputItem, DigestSelectedItem
};
use interfaces::defs::InputItemReference;
use tracing::{info, debug};

/// RSS-specific implementation of DigestModel that creates summaries from RSS feed items
pub struct RssDigestModel;

impl RssDigestModel {
    /// Analyze preferences to determine what to look for in RSS items
    fn ponder_preferences(
        _spec: &DigestModelSpec, 
        memory: &DigestModelMemory, 
        preferences: &DigestPreferences
    ) -> PonderedPreferences {
        // Extract keywords and topics from preferences
        let keywords = extract_keywords(&preferences.description);
        let topics = extract_topics(&preferences.description, memory);
        
        PonderedPreferences {
            look_out_for: preferences.description.clone(),
            keywords,
            topics,
            memory_context: memory.text.clone(),
        }
    }
    
    /// Analyze each RSS item for relevance and create focused summary
    fn ponder_relevance_and_summarize(
        _spec: &DigestModelSpec,
        pondered_preferences: &PonderedPreferences, 
        input_item: &InputItem
    ) -> ScoredSummary {
        let relevance_score = calculate_relevance_score(input_item, pondered_preferences);
        let summary = create_item_summary(input_item, pondered_preferences);
        
        ScoredSummary {
            summary,
            relevance_score,
            input_item_uri: input_item.uri.clone(),
        }
    }
    
    /// Select the best RSS items based on relevance scores
    fn select_best(
        _spec: &DigestModelSpec,
        _pondered_preferences: &PonderedPreferences,
        scored_summaries: &[ScoredSummary]
    ) -> Vec<usize> {
        let mut indexed_summaries: Vec<(usize, &ScoredSummary)> = scored_summaries
            .iter()
            .enumerate()
            .collect();
            
        // Sort by relevance score (descending)
        indexed_summaries.sort_by(|a, b| b.1.relevance_score.partial_cmp(&a.1.relevance_score).unwrap_or(std::cmp::Ordering::Equal));
        
        // Take top items (max 10 for readability)
        let max_items = std::cmp::min(scored_summaries.len(), 10);
        indexed_summaries
            .into_iter()
            .take(max_items)
            .filter(|(_, summary)| summary.relevance_score > 0.1) // Only include somewhat relevant items
            .map(|(index, _)| index)
            .collect()
    }
    
    /// Compose final digest from selected summaries
    fn compose_digest(
        _spec: &DigestModelSpec,
        _pondered_preferences: &PonderedPreferences,
        best_summaries: &[ScoredSummary]
    ) -> String {
        if best_summaries.is_empty() {
            return "No relevant RSS items found for your preferences.".to_string();
        }
        
        let mut digest = String::new();
        digest.push_str("RSS Feed Digest:\n\n");
        
        for (i, summary) in best_summaries.iter().enumerate() {
            digest.push_str(&format!("{}. {}\n", i + 1, summary.summary.summary_text));
            digest.push_str(&format!("   Source: {}\n", summary.input_item_uri));
            digest.push_str(&format!("   Relevance: {:.2}\n\n", summary.relevance_score));
        }
        
        digest
    }
}

impl DigestModel for RssDigestModel {
    fn digest(
        spec: &DigestModelSpec, 
        memory: &DigestModelMemory, 
        preferences: &DigestPreferences, 
        input_items: &[InputItem]
    ) -> DigestOutput {
        info!("Creating RSS digest for {} items", input_items.len());
        
        let pondered_preferences = Self::ponder_preferences(spec, memory, preferences);
        
        let scored_summaries: Vec<ScoredSummary> = input_items
            .iter()
            .map(|item| Self::ponder_relevance_and_summarize(spec, &pondered_preferences, item))
            .collect();
            
        let best_indices = Self::select_best(spec, &pondered_preferences, &scored_summaries);
        let best_summaries: Vec<ScoredSummary> = best_indices
            .iter()
            .map(|&index| scored_summaries[index].clone())
            .collect();
            
        let digest_text = Self::compose_digest(spec, &pondered_preferences, &best_summaries);
        
        let selected_items: Vec<DigestSelectedItem> = best_indices
            .iter()
            .map(|&index| DigestSelectedItem {
                input_item_uri: input_items[index].uri.clone(),
                references: scored_summaries[index].summary.references.clone(),
            })
            .collect();
            
        debug!("Created digest with {} selected items", selected_items.len());
        
        DigestOutput {
            selected_items,
            text: digest_text,
        }
    }
    
    fn reflect(
        _spec: &DigestModelSpec,
        memory: &DigestModelMemory,
        _preferences: &DigestPreferences,
        _input_items: &[InputItem],
        _self_output: &DigestOutput,
        _opponent_output: &DigestOutput,
        win: bool
    ) -> DigestModelMemory {
        // Update memory based on feedback
        let mut new_memory = memory.text.clone();
        
        if win {
            new_memory.push_str("\n[SUCCESS] Previous digest was well-received.");
        } else {
            new_memory.push_str("\n[FEEDBACK] Previous digest could be improved. Consider adjusting relevance criteria.");
        }
        
        // Trim memory to prevent it from growing too large
        if new_memory.len() > 10000 {
            new_memory = new_memory.chars().skip(new_memory.len() - 8000).collect();
        }
        
        DigestModelMemory {
            text: new_memory,
        }
    }
}

#[derive(Debug, Clone)]
struct PonderedPreferences {
    look_out_for: String,
    keywords: Vec<String>,
    topics: Vec<String>,
    memory_context: String,
}

#[derive(Debug, Clone)]
struct FocusedSummary {
    summary_text: String,
    references: Vec<InputItemReference>,
}

#[derive(Debug, Clone)]
struct ScoredSummary {
    summary: FocusedSummary,
    relevance_score: f64,
    input_item_uri: String,
}

/// Extract important keywords from preference description
fn extract_keywords(description: &str) -> Vec<String> {
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
fn extract_topics(description: &str, memory: &DigestModelMemory) -> Vec<String> {
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

/// Calculate relevance score for an RSS item
fn calculate_relevance_score(input_item: &InputItem, preferences: &PonderedPreferences) -> f64 {
    let text = input_item.text.to_lowercase();
    let mut score: f64 = 0.0;
    
    // Keyword matching
    for keyword in &preferences.keywords {
        if text.contains(&keyword.to_lowercase()) {
            score += 0.3;
        }
    }
    
    // Topic matching  
    for topic in &preferences.topics {
        if text.contains(&topic.to_lowercase()) {
            score += 0.4;
        }
    }
    
    // Exact phrase matching gets higher score
    if text.contains(&preferences.look_out_for.to_lowercase()) {
        score += 0.5;
    }
    
    // Title matching (RSS items often have "Title: " prefix)
    if let Some(title_start) = text.find("title: ") {
        let title_portion = &text[title_start + 7..];
        if let Some(title_end) = title_portion.find('\n') {
            let title = &title_portion[..title_end];
            for keyword in &preferences.keywords {
                if title.contains(&keyword.to_lowercase()) {
                    score += 0.2; // Bonus for title keywords
                }
            }
        }
    }
    
    // Normalize score to 0-1 range
    score.min(1.0)
}

/// Create a focused summary of an RSS item
fn create_item_summary(input_item: &InputItem, _preferences: &PonderedPreferences) -> FocusedSummary {
    let text = &input_item.text;
    
    // Extract title
    let title = extract_title(text);
    
    // Extract first few sentences of description/content for summary
    let summary_text = if text.len() > 200 {
        let excerpt = &text[..200];
        if let Some(last_sentence) = excerpt.rfind('.') {
            format!("{}: {}", title, &excerpt[..last_sentence + 1])
        } else {
            format!("{}: {}...", title, excerpt)
        }
    } else {
        format!("{}: {}", title, text)
    };
    
    // Create references spanning the relevant portions
    let references = vec![InputItemReference {
        text_start_index: 0,
        text_end_index: std::cmp::min(text.len(), 200),
    }];
    
    FocusedSummary {
        summary_text,
        references,
    }
}

/// Extract title from RSS item text
fn extract_title(text: &str) -> String {
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
fn is_stop_word(word: &str) -> bool {
    matches!(
        word,
        "the" | "and" | "or" | "but" | "in" | "on" | "at" | "to" | "for" | "of" | "with" | "by" | "a" | "an" | "is" | "are" | "was" | "were" | "be" | "been" | "have" | "has" | "had" | "do" | "does" | "did" | "will" | "would" | "could" | "should" | "may" | "might" | "must" | "can" | "this" | "that" | "these" | "those"
    )
}