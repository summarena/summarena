use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use tokio::fs;
use rand::seq::SliceRandom;
use rand::thread_rng;

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct MairQuery {
    pub qid: String,
    pub instruction: String,
    pub query: String,
    pub labels: Vec<MairLabel>,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct MairLabel {
    pub id: String,
    pub score: i32,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct MairDoc {
    pub id: String,
    pub doc: String,
}

#[derive(Debug, Serialize, Clone)]
pub struct QueryWithDocs {
    pub query: MairQuery,
    pub relevant_docs: Vec<MairDoc>,
}

#[derive(Debug, Serialize, Clone)]
pub struct QueryWithAllDocs {
    pub query: MairQuery,
    pub all_docs: Vec<MairDoc>,  // Mix of relevant + negative docs
    pub relevance_map: HashMap<String, i32>,  // doc_id -> relevance score
}

#[derive(Debug, Serialize, Clone)]
pub struct QueryWithCandidates {
    pub query: MairQuery,
    pub candidates: Vec<MairDoc>,  // First-stage retrieval candidates (up to 100)
    pub relevance_map: HashMap<String, i32>,  // doc_id -> relevance score
}

// Structs for parsing the first-stage results JSON
#[derive(Debug, Deserialize)]
struct FirstStageResultItem {
    results: HashMap<String, HashMap<String, f64>>,
}

type FirstStageResults = HashMap<String, Vec<FirstStageResultItem>>;


pub async fn load_first_stage_results(
    results_path: &str,
    hf_path: &str,
) -> Result<HashMap<String, Vec<String>>> {
    // Loading progress suppressed
    let results_content = fs::read_to_string(results_path).await?;
    let all_results: FirstStageResults = serde_json::from_str(&results_content)?;

    // The MAIR results file uses keys like "ACORDAR/queries"
    let key = format!("{hf_path}/queries");
    
    if let Some(results_list) = all_results.get(&key) {
        if let Some(last_result) = results_list.last() {
            let mut qid_to_doc_ids = HashMap::new();
            for (qid, doc_scores) in &last_result.results {
                // The doc_scores are a map of doc_id -> score. We just need the doc_ids.
                let doc_ids: Vec<String> = doc_scores.keys().cloned().collect();
                qid_to_doc_ids.insert(qid.clone(), doc_ids);
            }
            // Loading progress suppressed
            return Ok(qid_to_doc_ids);
        }
    }

    Err(anyhow::anyhow!("Could not find first-stage results for dataset '{}' with key '{}' in {}", hf_path, key, results_path))
}


pub async fn load_mair_data(queries_path: &str, docs_path: &str) -> Result<Vec<QueryWithDocs>> {
    // Loading progress suppressed
    let queries_content = fs::read_to_string(queries_path).await?;
    let queries: Vec<MairQuery> = queries_content
        .lines()
        .map(serde_json::from_str)
        .collect::<Result<Vec<_>, _>>()?;
    
    // Loading progress suppressed
    let docs_content = fs::read_to_string(docs_path).await?;
    let docs: HashMap<String, MairDoc> = docs_content
        .lines()
        .map(|line| {
            let doc: MairDoc = serde_json::from_str(line).unwrap();
            (doc.id.clone(), doc)
        })
        .collect();
    
    // Loading progress suppressed
    
    let mut result = Vec::new();
    
    for query in queries {
        let mut relevant_docs = Vec::new();
        
        // Get documents with relevance score >= 1
        for label in &query.labels {
            if label.score >= 1 {
                if let Some(doc) = docs.get(&label.id) {
                    relevant_docs.push(doc.clone());
                }
            }
        }
        
        // Only include queries that have relevant documents
        if !relevant_docs.is_empty() {
            result.push(QueryWithDocs {
                query,
                relevant_docs,
            });
        }
    }
    
    // Loading progress suppressed
    Ok(result)
}

pub async fn load_mair_data_with_negatives(
    queries_path: &str, 
    docs_path: &str, 
    negative_samples: usize
) -> Result<Vec<QueryWithAllDocs>> {
    // Loading progress suppressed
    let queries_content = fs::read_to_string(queries_path).await?;
    let queries: Vec<MairQuery> = queries_content
        .lines()
        .map(serde_json::from_str)
        .collect::<Result<Vec<_>, _>>()?;
    
    // Loading progress suppressed
    let docs_content = fs::read_to_string(docs_path).await?;
    let docs: HashMap<String, MairDoc> = docs_content
        .lines()
        .map(|line| {
            let doc: MairDoc = serde_json::from_str(line).unwrap();
            (doc.id.clone(), doc)
        })
        .collect();
    
    // Loading progress suppressed
    
    // Collect all document IDs that have labels (relevant + irrelevant)
    let mut labeled_doc_ids = std::collections::HashSet::new();
    for query in &queries {
        for label in &query.labels {
            labeled_doc_ids.insert(label.id.clone());
        }
    }
    
    // Get unlabeled documents for negative sampling
    let unlabeled_docs: Vec<&MairDoc> = docs
        .values()
        .filter(|doc| !labeled_doc_ids.contains(&doc.id))
        .collect();
    
    // Loading progress suppressed
    
    let mut result = Vec::new();
    let mut rng = thread_rng();
    
    for query in queries {
        let mut relevance_map = HashMap::new();
        let mut query_docs = Vec::new();
        
        // Add all labeled documents (both relevant and irrelevant)
        for label in &query.labels {
            if let Some(doc) = docs.get(&label.id) {
                query_docs.push(doc.clone());
                relevance_map.insert(label.id.clone(), label.score);
            }
        }
        
        // Add random negative samples from unlabeled documents
        let negative_count = negative_samples.min(unlabeled_docs.len());
        let mut sampled_negatives = unlabeled_docs.clone();
        sampled_negatives.shuffle(&mut rng);
        
        for negative_doc in sampled_negatives.iter().take(negative_count) {
            query_docs.push((*negative_doc).clone());
            relevance_map.insert(negative_doc.id.clone(), 0); // Score 0 for unlabeled
        }
        
        // Only include queries that have at least some relevant documents (score >= 1)
        let has_relevant = query.labels.iter().any(|label| label.score >= 1);
        if has_relevant && !query_docs.is_empty() {
            result.push(QueryWithAllDocs {
                query,
                all_docs: query_docs,
                relevance_map,
            });
        }
    }
    
    // Loading progress suppressed
    Ok(result)
}

pub async fn load_mair_data_with_first_stage_candidates(
    queries_path: &str, 
    docs_path: &str,
    hf_path: &str, // Changed from dataset_name to hf_path
    candidate_limit: usize,
) -> Result<Vec<QueryWithCandidates>> {
    load_mair_data_with_first_stage_candidates_quiet(queries_path, docs_path, hf_path, candidate_limit, false).await
}

pub async fn load_mair_data_with_first_stage_candidates_quiet(
    queries_path: &str, 
    docs_path: &str,
    hf_path: &str, 
    candidate_limit: usize,
    quiet: bool,
) -> Result<Vec<QueryWithCandidates>> {
    if !quiet {
        // Loading progress suppressed
    }
    let queries_content = fs::read_to_string(queries_path).await?;
    let queries: Vec<MairQuery> = queries_content
        .lines()
        .map(serde_json::from_str)
        .collect::<Result<Vec<_>, _>>()?;
    
    // Loading progress suppressed
    let docs_content = fs::read_to_string(docs_path).await?;
    let docs: HashMap<String, MairDoc> = docs_content
        .lines()
        .map(|line| {
            let doc: MairDoc = serde_json::from_str(line).unwrap();
            (doc.id.clone(), doc)
        })
        .collect();
    
    // Loading progress suppressed

    // Load the real first-stage results
    let first_stage_results_path = "data/text-embedding-3-small.json";
    let first_stage_map = load_first_stage_results(first_stage_results_path, hf_path).await?;
    
    let mut result = Vec::new();
    
    for query in queries {
        // Create relevance map from query labels
        let mut relevance_map = HashMap::new();
        for label in &query.labels {
            relevance_map.insert(label.id.clone(), label.score);
        }
        
        // Get the candidate doc IDs from the loaded first-stage results
        let candidate_doc_ids = match first_stage_map.get(&query.qid) {
            Some(doc_ids) => doc_ids,
            None => {
                // If a query from the dataset is not in the results file, skip it.
                // Warning suppressed
                continue;
            }
        };

        // Build the candidate list, taking up to the candidate_limit
        let mut candidates = Vec::new();
        for doc_id in candidate_doc_ids.iter().take(candidate_limit) {
            if let Some(doc) = docs.get(doc_id) {
                candidates.push(doc.clone());
                // Ensure relevance map has an entry for all candidates (score 0 if not in original labels)
                relevance_map.entry(doc_id.clone()).or_insert(0);
            } else {
                // Warning suppressed
            }
        }
        
        // Only include queries that have at least some relevant documents and candidates
        let has_relevant = query.labels.iter().any(|label| label.score >= 1);
        if has_relevant && !candidates.is_empty() {
            result.push(QueryWithCandidates {
                query,
                candidates,
                relevance_map,
            });
        }
    }
    
    // Loading progress suppressed
    Ok(result)
}