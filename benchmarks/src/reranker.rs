use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use crate::mair_loader::{MairDoc, QueryWithDocs, QueryWithAllDocs, QueryWithCandidates};

#[derive(Debug, Serialize)]
struct VoyageRerankRequest {
    query: String,
    documents: Vec<String>,
    model: String,
    top_k: Option<usize>,
}

#[derive(Debug, Deserialize)]
struct VoyageRerankResponse {
    data: Vec<VoyageRerankResult>,
}

#[derive(Debug, Deserialize)]
struct VoyageRerankResult {
    index: usize,
    relevance_score: f64,
}

#[derive(Clone)]
pub struct VoyageReranker {
    client: reqwest::Client,
    api_key: String,
}

impl VoyageReranker {
    pub fn new(api_key: String) -> Self {
        Self {
            client: reqwest::Client::new(),
            api_key,
        }
    }

    /// Generate personas relevant to a query using OpenAI API
    pub async fn generate_personas(&self, query: &str, instruction: &str) -> Result<Vec<String>> {
        let openai_api_key = std::env::var("OPENAI_API_KEY")
            .map_err(|_| anyhow::anyhow!("OPENAI_API_KEY environment variable must be set for persona generation"))?;

        #[derive(Serialize)]
        struct OpenAIRequest {
            model: String,
            messages: Vec<OpenAIMessage>,
            max_tokens: u32,
            temperature: f32,
        }

        #[derive(Serialize)]
        struct OpenAIMessage {
            role: String,
            content: String,
        }

        #[derive(Deserialize)]
        struct OpenAIResponse {
            choices: Vec<OpenAIChoice>,
        }

        #[derive(Deserialize)]
        struct OpenAIChoice {
            message: OpenAIMessage2,
        }

        #[derive(Deserialize)]
        struct OpenAIMessage2 {
            content: String,
        }

        let system_prompt = format!(
            "You are helping create user personas for evaluating personalized information retrieval. \
             Given a query and instruction, generate 4 distinct user personas who would search for this information \
             but have different perspectives, backgrounds, or needs. Each persona should be described in 1-2 sentences \
             focusing on their professional role, background, or specific interests that would influence how they \
             evaluate relevance.\n\n\
             Query: {query}\n\
             Instruction: {instruction}\n\n\
             Generate exactly 4 personas, one per line, without numbering or bullet points:\n\
             - First 3 personas: Create diverse, legitimate users with different professional backgrounds, expertise levels, and information needs who would genuinely benefit from this information.\n\
             - Last persona (4th): Create a deliberately misaligned or contrarian persona who might have opposing views, irrelevant priorities, or conflicting interests that would make them evaluate the same information differently or poorly. This should be a realistic but problematic user perspective that would likely worsen retrieval performance."
        );

        let request = OpenAIRequest {
            model: "gpt-4o-mini".to_string(),
            messages: vec![
                OpenAIMessage {
                    role: "user".to_string(),
                    content: system_prompt,
                }
            ],
            max_tokens: 400,
            temperature: 0.8,
        };

        let response = self
            .client
            .post("https://api.openai.com/v1/chat/completions")
            .header("Authorization", format!("Bearer {openai_api_key}"))
            .header("Content-Type", "application/json")
            .json(&request)
            .send()
            .await?;

        if !response.status().is_success() {
            let error_text = response.text().await?;
            return Err(anyhow::anyhow!("OpenAI API error: {}", error_text));
        }

        let openai_response: OpenAIResponse = response.json().await?;
        
        if openai_response.choices.is_empty() {
            return Err(anyhow::anyhow!("No response from OpenAI API"));
        }

        let personas_text = &openai_response.choices[0].message.content;
        let personas: Vec<String> = personas_text
            .lines()
            .map(|line| line.trim().to_string())
            .filter(|line| !line.is_empty())
            .collect();

        if personas.len() < 4 {
            // Warning suppressed
        }

        Ok(personas)
    }

    pub async fn rerank(
        &self,
        query: &str,
        documents: &[String],
        top_k: Option<usize>,
    ) -> Result<Vec<(usize, f64)>> {
        let request = VoyageRerankRequest {
            query: query.to_string(),
            documents: documents.to_vec(),
            model: "rerank-2".to_string(),
            top_k,
        };

        let response = self
            .client
            .post("https://api.voyageai.com/v1/rerank")
            .header("Authorization", format!("Bearer {}", self.api_key))
            .header("Content-Type", "application/json")
            .json(&request)
            .send()
            .await?;

        if !response.status().is_success() {
            let error_text = response.text().await?;
            return Err(anyhow::anyhow!("Voyage API error: {}", error_text));
        }

        let response_text = response.text().await?;
        // API response logging suppressed
        let rerank_response: VoyageRerankResponse = serde_json::from_str(&response_text)?;
        
        Ok(rerank_response
            .data
            .into_iter()
            .map(|result| (result.index, result.relevance_score))
            .collect())
    }

    /// Rerank documents with preference-aware query modification
    pub async fn rerank_with_preferences(
        &self,
        query_data: &QueryWithDocs,
        preferences: &[String],
        top_k: Option<usize>,
    ) -> Result<Vec<MairDoc>> {
        // Modify query with preferences
        let enhanced_query = if preferences.is_empty() {
            query_data.query.query.clone()
        } else {
            format!(
                "{}\\n\\nUser preferences: {}",
                query_data.query.query,
                preferences.join("; ")
            )
        };

        // Extract document texts for reranking
        let doc_texts: Vec<String> = query_data.relevant_docs
            .iter()
            .map(|doc| {
                // Try to extract meaningful content from the document
                self.extract_doc_text(&doc.doc)
            })
            .collect();

        // Reranking progress suppressed

        // Call Voyage API
        let rankings = self.rerank(&enhanced_query, &doc_texts, top_k).await?;

        // Sort by relevance score (descending)
        let mut sorted_rankings = rankings;
        sorted_rankings.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));

        // Reorder documents according to rankings
        let mut reordered_docs = Vec::new();
        for (doc_index, _score) in sorted_rankings {
            if doc_index < query_data.relevant_docs.len() {
                let doc = query_data.relevant_docs[doc_index].clone();
                // Doc scoring suppressed
                reordered_docs.push(doc);
            }
        }

        Ok(reordered_docs)
    }

    /// Rerank all documents (relevant + negative) with preference-aware query modification
    pub async fn rerank_all_docs_with_preferences(
        &self,
        query_data: &QueryWithAllDocs,
        preferences: &[String],
        top_k: Option<usize>,
    ) -> Result<Vec<MairDoc>> {
        // Modify query with preferences
        let enhanced_query = if preferences.is_empty() {
            query_data.query.query.clone()
        } else {
            format!(
                "{}\\n\\nUser preferences: {}",
                query_data.query.query,
                preferences.join("; ")
            )
        };

        // Extract document texts for reranking (all docs, not just relevant)
        let doc_texts: Vec<String> = query_data.all_docs
            .iter()
            .map(|doc| {
                self.extract_doc_text(&doc.doc)
            })
            .collect();

        // Reranking progress suppressed

        // Use the Voyage API to rerank
        let rankings = self.rerank(&enhanced_query, &doc_texts, top_k).await?;

        // Reorder documents based on ranking results
        let mut reordered_docs = Vec::new();
        for (index, _score) in rankings {
            if index < query_data.all_docs.len() {
                let doc = &query_data.all_docs[index];
                // Doc scoring suppressed
                reordered_docs.push(doc.clone());
            }
        }

        Ok(reordered_docs)
    }

    pub fn extract_doc_text(&self, doc_content: &str) -> String {
        // Try to parse as JSON and extract meaningful fields
        if let Ok(parsed) = serde_json::from_str::<serde_json::Value>(&doc_content.replace('\'', "\"")) {
            let mut text_parts = Vec::new();
            
            if let Some(title) = parsed.get("title").and_then(|v| v.as_str()) {
                text_parts.push(format!("Title: {title}"));
            }
            
            if let Some(description) = parsed.get("description").and_then(|v| v.as_str()) {
                text_parts.push(format!("Description: {description}"));
            }
            
            if let Some(tags) = parsed.get("tags").and_then(|v| v.as_str()) {
                text_parts.push(format!("Tags: {tags}"));
            }
            
            if !text_parts.is_empty() {
                return text_parts.join("\\n");
            }
        }
        
        // Fallback to original content, truncated if too long
        let truncated = if doc_content.len() > 1000 {
            // Safely truncate at UTF-8 character boundary
            let mut end = 1000;
            while end > 0 && !doc_content.is_char_boundary(end) {
                end -= 1;
            }
            format!("{}...", &doc_content[..end])
        } else {
            doc_content.to_string()
        };
        
        truncated
    }

    /// Rerank first-stage candidates with preference-aware query modification (MAIR-style evaluation)
    pub async fn rerank_candidates_with_preferences(
        &self,
        query_data: &QueryWithCandidates,
        preferences: &[String],
        top_k: Option<usize>,
    ) -> Result<Vec<MairDoc>> {
        // Modify query with preferences
        let enhanced_query = if preferences.is_empty() {
            query_data.query.query.clone()
        } else {
            format!(
                "{}\\n\\nUser preferences: {}",
                query_data.query.query,
                preferences.join("; ")
            )
        };

        // Extract document texts for reranking (all candidates)
        let doc_texts: Vec<String> = query_data.candidates
            .iter()
            .map(|doc| {
                self.extract_doc_text(&doc.doc)
            })
            .collect();

        // Reranking progress suppressed

        // Use the Voyage API to rerank
        let rankings = self.rerank(&enhanced_query, &doc_texts, top_k).await?;

        // Reorder documents based on ranking results
        let mut reordered_docs = Vec::new();
        for (index, _score) in rankings {
            if index < query_data.candidates.len() {
                let doc = &query_data.candidates[index];
                // Doc scoring suppressed
                reordered_docs.push(doc.clone());
            }
        }

        Ok(reordered_docs)
    }
}

pub fn calculate_ndcg_at_k(
    ranked_docs: &[MairDoc],
    original_query: &QueryWithDocs,
    k: usize,
) -> f64 {
    let k = k.min(ranked_docs.len());
    if k == 0 {
        return 0.0;
    }

    // Create a map from doc_id to relevance score
    let relevance_map: HashMap<String, i32> = original_query.query.labels
        .iter()
        .map(|label| (label.id.clone(), label.score))
        .collect();

    // Calculate DCG
    let mut dcg = 0.0;
    for (i, doc) in ranked_docs.iter().take(k).enumerate() {
        let relevance = relevance_map.get(&doc.id).unwrap_or(&0);
        let gain = (2_i32.pow(*relevance as u32) - 1) as f64;
        dcg += gain / (2.0 + i as f64).log2();
    }

    // Calculate IDCG (ideal DCG)
    let mut ideal_relevances: Vec<i32> = original_query.query.labels
        .iter()
        .filter(|label| label.score >= 1) // Only consider relevant docs
        .map(|label| label.score)
        .collect();
    ideal_relevances.sort_by(|a, b| b.cmp(a));

    let mut idcg = 0.0;
    for (i, relevance) in ideal_relevances.iter().take(k).enumerate() {
        let gain = (2_i32.pow(*relevance as u32) - 1) as f64;
        idcg += gain / (2.0 + i as f64).log2();
    }

    if idcg == 0.0 {
        0.0
    } else {
        dcg / idcg
    }
}

pub fn calculate_ndcg_at_k_with_relevance_map(
    ranked_docs: &[MairDoc],
    relevance_map: &HashMap<String, i32>,
    k: usize,
) -> f64 {
    let k = k.min(ranked_docs.len());
    if k == 0 {
        return 0.0;
    }

    // Calculate DCG
    let mut dcg = 0.0;
    for (i, doc) in ranked_docs.iter().take(k).enumerate() {
        let relevance = relevance_map.get(&doc.id).unwrap_or(&0);
        let gain = (2_i32.pow(*relevance as u32) - 1) as f64;
        dcg += gain / (2.0 + i as f64).log2();
    }

    // Calculate IDCG (ideal DCG) 
    let mut ideal_relevances: Vec<i32> = relevance_map
        .values()
        .filter(|&&score| score >= 1) // Only consider relevant docs
        .cloned()
        .collect();
    ideal_relevances.sort_by(|a, b| b.cmp(a));

    let mut idcg = 0.0;
    for (i, relevance) in ideal_relevances.iter().take(k).enumerate() {
        let gain = (2_i32.pow(*relevance as u32) - 1) as f64;
        idcg += gain / (2.0 + i as f64).log2();
    }

    if idcg == 0.0 {
        0.0
    } else {
        dcg / idcg
    }
}