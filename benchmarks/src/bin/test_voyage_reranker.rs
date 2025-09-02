use anyhow::Result;
use clap::Parser;
use rand::seq::SliceRandom;
use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use voyage_reranker_test::{
    load_documents, load_preferences, load_queries, rerank_with_voyage,
};

fn randomize_documents(docs: Vec<String>) -> Vec<String> {
    let mut rng = rand::thread_rng();
    let mut shuffled = docs;
    shuffled.shuffle(&mut rng);
    shuffled
}

#[derive(Debug, Deserialize)]
struct HumanRankingEntry {
    qid: String,
    rankings: Vec<HumanRanking>,
}

#[derive(Debug, Deserialize, Clone)]
struct HumanRanking {
    doc_id: String,
    rank: usize,
}

#[derive(Debug, Serialize)]
struct BenchmarkResult {
    qid: String,
    query: String,
    // Human rankings as gold labels
    human_ndcg_at_3: f64,
    human_ndcg_at_5: f64,
    human_ndcg_at_10: f64,
    human_kendall_tau: Option<f64>,
    // MAIR original labels as gold labels
    mair_ndcg_at_3: f64,
    mair_ndcg_at_5: f64,
    mair_ndcg_at_10: f64,
    mair_kendall_tau: Option<f64>,
    // Rankings for reference
    human_ranking_order: Vec<String>,
    mair_ranking_order: Vec<String>,
    voyage_ranking_order: Vec<String>,
}

fn load_human_rankings(filepath: &str) -> Result<HashMap<String, Vec<HumanRanking>>> {
    let mut rankings = HashMap::new();
    
    if !std::path::Path::new(filepath).exists() {
        return Ok(rankings);
    }
    
    let content = fs::read_to_string(filepath)?;
    for line in content.lines() {
        if !line.trim().is_empty() {
            if let Ok(entry) = serde_json::from_str::<HumanRankingEntry>(line) {
                // Keep only the most recent ranking for each query
                rankings.insert(entry.qid, entry.rankings);
            }
        }
    }
    
    Ok(rankings)
}

fn calculate_ndcg_at_k(human_order: &[String], voyage_order: &[String], k: usize) -> f64 {
    // Create relevance scores based on human ranking (higher rank = higher relevance)
    let human_scores: HashMap<&String, f64> = human_order.iter().enumerate()
        .map(|(i, doc_id)| (doc_id, (human_order.len() - i) as f64))
        .collect();
    
    // Calculate DCG for Voyage ranking
    let mut dcg = 0.0;
    for (i, doc_id) in voyage_order.iter().take(k).enumerate() {
        if let Some(&relevance) = human_scores.get(doc_id) {
            let gain = (2_f64.powf(relevance) - 1.0) / (i as f64 + 2.0).log2();
            dcg += gain;
        }
    }
    
    // Calculate IDCG (perfect ranking based on human order)
    let mut idcg = 0.0;
    for (i, doc_id) in human_order.iter().take(k).enumerate() {
        if let Some(&relevance) = human_scores.get(doc_id) {
            let gain = (2_f64.powf(relevance) - 1.0) / (i as f64 + 2.0).log2();
            idcg += gain;
        }
    }
    
    if idcg == 0.0 {
        0.0
    } else {
        dcg / idcg
    }
}

fn calculate_kendall_tau(human_order: &[String], voyage_order: &[String]) -> Option<f64> {
    if human_order.len() < 2 || voyage_order.len() < 2 {
        return None;
    }
    
    // Create position maps
    let human_pos: HashMap<&String, usize> = human_order.iter().enumerate()
        .map(|(i, id)| (id, i)).collect();
    let voyage_pos: HashMap<&String, usize> = voyage_order.iter().enumerate()
        .map(|(i, id)| (id, i)).collect();
    
    // Find common documents
    let common_docs: Vec<&String> = human_order.iter()
        .filter(|id| voyage_pos.contains_key(id))
        .collect();
    
    if common_docs.len() < 2 {
        return None;
    }
    
    let n = common_docs.len();
    let mut concordant = 0;
    let mut discordant = 0;
    
    for i in 0..n {
        for j in (i + 1)..n {
            let doc_i = common_docs[i];
            let doc_j = common_docs[j];
            
            let human_order_ij = human_pos[doc_i] < human_pos[doc_j];
            let voyage_order_ij = voyage_pos[doc_i] < voyage_pos[doc_j];
            
            if human_order_ij == voyage_order_ij {
                concordant += 1;
            } else {
                discordant += 1;
            }
        }
    }
    
    let total_pairs = (n * (n - 1)) / 2;
    if total_pairs == 0 {
        return None;
    }
    
    Some((concordant as f64 - discordant as f64) / total_pairs as f64)
}

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// Dataset name (e.g., fiqa, msmarco, scidocs)
    #[arg(short, long)]
    dataset: String,
}

#[tokio::main]
async fn main() -> Result<()> {
    let args = Args::parse();
    let dataset = &args.dataset;
    
    let client = Client::new();

    println!("Loading {} dataset...", dataset.to_uppercase());
    let queries = load_queries(&format!("data/{}_queries.jsonl", dataset))?;
    let documents = load_documents(&format!("data/{}_docs.jsonl", dataset))?;

    // Load saved preferences to ensure consistency with UI
    println!("Loading saved preferences...");
    let saved_preferences = load_preferences(&format!("data/{}_preferences.jsonl", dataset))?;
    println!("Found {} saved preferences", saved_preferences.len());

    // Load human rankings to use as gold labels
    println!("Loading human rankings (gold labels)...");
    let human_rankings = load_human_rankings(&format!("data/{}_human_rankings.jsonl", dataset))?;
    println!("Found {} human rankings", human_rankings.len());

    // Filter for queries that were manually labeled (same criteria used for manual labeling)
    let manually_labeled_queries: Vec<_> = queries
        .iter()
        .filter(|q| {
            let relevant_count = q.labels.iter().filter(|l| l.score > 0).count();
            relevant_count >= 3 && relevant_count <= 20 // These are the queries you manually labeled
        })
        .collect();

    println!("Found {} queries matching manual labeling criteria", manually_labeled_queries.len());
    println!("These are the same queries you've been ranking in the web interface\n");

    let mut benchmark_results = Vec::new();

    for query in manually_labeled_queries {
        println!("\n=== Query {} ===", query.qid);
        println!("Query: {}", query.query);

        // Get relevant documents (score > 0)
        let relevant_docs: Vec<String> = query
            .labels
            .iter()
            .filter(|label| label.score > 0)
            .filter_map(|label| documents.get(&label.id))
            .cloned()
            .collect();

        if relevant_docs.is_empty() {
            println!("No relevant documents found");
            continue;
        }

        println!("Found {} relevant documents", relevant_docs.len());

        // Print original labeled order (document IDs)
        println!("Original labeled order:");
        for (i, label) in query.labels.iter().filter(|l| l.score > 0).enumerate() {
            println!("  {}: doc_id {}", i + 1, label.id);
        }

        // Randomize document order to prevent bias
        let randomized_docs = randomize_documents(relevant_docs.clone());

        // Skip if no human ranking (we need this as gold label)
        let human_ranking = if let Some(ranking) = human_rankings.get(&query.qid) {
            ranking
        } else {
            println!("No human ranking found for query {} - skipping", query.qid);
            continue;
        };

        // Use saved preference (skip if not available since we only test manually labeled queries)
        let preference = if let Some(saved_pref) = saved_preferences.get(&query.qid) {
            println!("Using saved preference: {}", saved_pref);
            saved_pref.clone()
        } else {
            println!("No saved preference found for query {} - skipping", query.qid);
            continue;
        };

        // Create personalized query
        let personalized_query = format!("{}\n\nUser preference: {}", query.query, preference);

        // Test Voyage reranker with personalized query
        println!("Testing Voyage reranker with personalized query...");
        match rerank_with_voyage(&client, &personalized_query, randomized_docs.clone()).await {
            Ok(results) => {
                // Extract document IDs in Voyage ranking order
                let voyage_doc_ids: Vec<String> = results.iter()
                    .map(|(original_index, _score)| {
                        randomized_docs[*original_index].clone()
                    })
                    .filter_map(|doc_content| {
                        // Find document ID by matching content
                        query.labels.iter()
                            .find(|label| {
                                if let Some(stored_content) = documents.get(&label.id) {
                                    stored_content == &doc_content
                                } else {
                                    false
                                }
                            })
                            .map(|label| label.id.clone())
                    })
                    .collect();

                // Get human ranking order
                let mut human_sorted = human_ranking.clone();
                human_sorted.sort_by_key(|r| r.rank);
                let human_doc_ids: Vec<String> = human_sorted.iter()
                    .map(|r| r.doc_id.clone())
                    .collect();

                // Get MAIR original ranking order (sorted by relevance score descending)
                let mut mair_docs: Vec<_> = query.labels.iter()
                    .filter(|label| label.score > 0)
                    .collect();
                mair_docs.sort_by(|a, b| b.score.cmp(&a.score));
                let mair_doc_ids: Vec<String> = mair_docs.iter().map(|l| l.id.clone()).collect();

                // Calculate metrics against human rankings
                let human_ndcg_at_3 = calculate_ndcg_at_k(&human_doc_ids, &voyage_doc_ids, 3);
                let human_ndcg_at_5 = calculate_ndcg_at_k(&human_doc_ids, &voyage_doc_ids, 5);
                let human_ndcg_at_10 = calculate_ndcg_at_k(&human_doc_ids, &voyage_doc_ids, 10);
                let human_kendall_tau = calculate_kendall_tau(&human_doc_ids, &voyage_doc_ids);

                // Calculate metrics against MAIR original labels
                let mair_ndcg_at_3 = calculate_ndcg_at_k(&mair_doc_ids, &voyage_doc_ids, 3);
                let mair_ndcg_at_5 = calculate_ndcg_at_k(&mair_doc_ids, &voyage_doc_ids, 5);
                let mair_ndcg_at_10 = calculate_ndcg_at_k(&mair_doc_ids, &voyage_doc_ids, 10);
                let mair_kendall_tau = calculate_kendall_tau(&mair_doc_ids, &voyage_doc_ids);

                // Print results
                println!("MAIR original order: {:?}", mair_doc_ids);
                println!("Human ranking order: {:?}", human_doc_ids);
                println!("Voyage ranking order: {:?}", voyage_doc_ids);
                println!("\n--- Voyage vs Human Rankings ---");
                println!("NDCG@3: {:.3}", human_ndcg_at_3);
                println!("NDCG@5: {:.3}", human_ndcg_at_5);
                println!("NDCG@10: {:.3}", human_ndcg_at_10);
                if let Some(tau) = human_kendall_tau {
                    println!("Kendall Tau: {:.3}", tau);
                }
                println!("\n--- Voyage vs MAIR Original ---");
                println!("NDCG@3: {:.3}", mair_ndcg_at_3);
                println!("NDCG@5: {:.3}", mair_ndcg_at_5);
                println!("NDCG@10: {:.3}", mair_ndcg_at_10);
                if let Some(tau) = mair_kendall_tau {
                    println!("Kendall Tau: {:.3}", tau);
                }

                // Store benchmark result
                benchmark_results.push(BenchmarkResult {
                    qid: query.qid.clone(),
                    query: query.query.clone(),
                    human_ndcg_at_3,
                    human_ndcg_at_5,
                    human_ndcg_at_10,
                    human_kendall_tau,
                    mair_ndcg_at_3,
                    mair_ndcg_at_5,
                    mair_ndcg_at_10,
                    mair_kendall_tau,
                    human_ranking_order: human_doc_ids,
                    mair_ranking_order: mair_doc_ids,
                    voyage_ranking_order: voyage_doc_ids,
                });
            }
            Err(e) => println!("Reranking failed: {}", e),
        }
    }

    // Print summary statistics
    if !benchmark_results.is_empty() {
        println!("\n=== BENCHMARK SUMMARY ===");
        println!("Total queries tested: {}", benchmark_results.len());
        
        // Human rankings as gold labels
        let human_avg_ndcg_3 = benchmark_results.iter().map(|r| r.human_ndcg_at_3).sum::<f64>() / benchmark_results.len() as f64;
        let human_avg_ndcg_5 = benchmark_results.iter().map(|r| r.human_ndcg_at_5).sum::<f64>() / benchmark_results.len() as f64;
        let human_avg_ndcg_10 = benchmark_results.iter().map(|r| r.human_ndcg_at_10).sum::<f64>() / benchmark_results.len() as f64;
        
        let human_kendall_values: Vec<f64> = benchmark_results.iter().filter_map(|r| r.human_kendall_tau).collect();
        let human_avg_kendall = if human_kendall_values.is_empty() { 0.0 } else {
            human_kendall_values.iter().sum::<f64>() / human_kendall_values.len() as f64
        };
        
        // MAIR original labels as gold labels
        let mair_avg_ndcg_3 = benchmark_results.iter().map(|r| r.mair_ndcg_at_3).sum::<f64>() / benchmark_results.len() as f64;
        let mair_avg_ndcg_5 = benchmark_results.iter().map(|r| r.mair_ndcg_at_5).sum::<f64>() / benchmark_results.len() as f64;
        let mair_avg_ndcg_10 = benchmark_results.iter().map(|r| r.mair_ndcg_at_10).sum::<f64>() / benchmark_results.len() as f64;
        
        let mair_kendall_values: Vec<f64> = benchmark_results.iter().filter_map(|r| r.mair_kendall_tau).collect();
        let mair_avg_kendall = if mair_kendall_values.is_empty() { 0.0 } else {
            mair_kendall_values.iter().sum::<f64>() / mair_kendall_values.len() as f64
        };
        
        println!("\n--- VOYAGE vs HUMAN PERSONALIZED RANKINGS ---");
        println!("Average NDCG@3: {:.3}", human_avg_ndcg_3);
        println!("Average NDCG@5: {:.3}", human_avg_ndcg_5);
        println!("Average NDCG@10: {:.3}", human_avg_ndcg_10);
        println!("Average Kendall Tau: {:.3}", human_avg_kendall);
        
        println!("\n--- VOYAGE vs MAIR ORIGINAL LABELS ---");
        println!("Average NDCG@3: {:.3}", mair_avg_ndcg_3);
        println!("Average NDCG@5: {:.3}", mair_avg_ndcg_5);
        println!("Average NDCG@10: {:.3}", mair_avg_ndcg_10);
        println!("Average Kendall Tau: {:.3}", mair_avg_kendall);
        
        // Performance comparison
        println!("\n--- PERFORMANCE COMPARISON ---");
        println!("NDCG@3: Human {:.3} vs MAIR {:.3} (diff: {:.3})", 
                 human_avg_ndcg_3, mair_avg_ndcg_3, human_avg_ndcg_3 - mair_avg_ndcg_3);
        println!("NDCG@5: Human {:.3} vs MAIR {:.3} (diff: {:.3})", 
                 human_avg_ndcg_5, mair_avg_ndcg_5, human_avg_ndcg_5 - mair_avg_ndcg_5);
        println!("NDCG@10: Human {:.3} vs MAIR {:.3} (diff: {:.3})", 
                 human_avg_ndcg_10, mair_avg_ndcg_10, human_avg_ndcg_10 - mair_avg_ndcg_10);
        println!("Kendall Tau: Human {:.3} vs MAIR {:.3} (diff: {:.3})", 
                 human_avg_kendall, mair_avg_kendall, human_avg_kendall - mair_avg_kendall);
        
        // Save detailed results
        let output_file = format!("data/{}_voyage_benchmark_comparison.json", dataset);
        let json_output = serde_json::to_string_pretty(&benchmark_results)?;
        fs::write(&output_file, json_output)?;
        println!("\nDetailed results saved to {}", output_file);
    }

    Ok(())
}
