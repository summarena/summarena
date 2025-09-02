use anyhow::Result;
use clap::Parser;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use voyage_reranker_test::{load_queries, Query};

#[derive(Debug, Deserialize)]
struct HumanRankingEntry {
    qid: String,
    rankings: Vec<HumanRanking>,
    timestamp: String,
}

#[derive(Debug, Deserialize, Clone)]
struct HumanRanking {
    doc_id: String,
    rank: usize,
    relevance_score: Option<i32>,
}

#[derive(Debug, Serialize)]
struct RankingAnalysis {
    qid: String,
    query: String,
    total_relevant_docs: usize,
    human_ranked_docs: usize,
    
    // Ranking change statistics
    kendall_tau: Option<f64>,
    spearman_correlation: Option<f64>,
    average_rank_change: f64,
    max_rank_change: i32,
    
    // Document movement analysis
    docs_moved_up: usize,
    docs_moved_down: usize,
    docs_unchanged: usize,
    
    // Top-k analysis
    top3_overlap: f64,
    top5_overlap: f64,
    top10_overlap: f64,
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

fn calculate_kendall_tau(original: &[String], human: &[String]) -> Option<f64> {
    if original.len() < 2 || human.len() < 2 {
        return None;
    }
    
    // Create position maps
    let orig_pos: HashMap<&String, usize> = original.iter().enumerate()
        .map(|(i, id)| (id, i)).collect();
    let human_pos: HashMap<&String, usize> = human.iter().enumerate()
        .map(|(i, id)| (id, i)).collect();
    
    // Find common documents
    let common_docs: Vec<&String> = original.iter()
        .filter(|id| human_pos.contains_key(id))
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
            
            let orig_order = orig_pos[doc_i] < orig_pos[doc_j];
            let human_order = human_pos[doc_i] < human_pos[doc_j];
            
            if orig_order == human_order {
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

fn calculate_top_k_overlap(original: &[String], human: &[String], k: usize) -> f64 {
    let orig_top_k: std::collections::HashSet<_> = original.iter().take(k).collect();
    let human_top_k: std::collections::HashSet<_> = human.iter().take(k).collect();
    
    let intersection = orig_top_k.intersection(&human_top_k).count();
    let union_size = k.min(original.len()).max(k.min(human.len()));
    
    if union_size == 0 {
        0.0
    } else {
        intersection as f64 / union_size as f64
    }
}

fn analyze_rankings(queries: &[Query], human_rankings: &HashMap<String, Vec<HumanRanking>>) -> Vec<RankingAnalysis> {
    let mut analyses = Vec::new();
    
    for query in queries {
        if let Some(human_ranking) = human_rankings.get(&query.qid) {
            // Get original ranking (sorted by relevance score descending)
            let mut original_docs: Vec<_> = query.labels.iter()
                .filter(|label| label.score > 0)
                .collect();
            original_docs.sort_by(|a, b| b.score.cmp(&a.score));
            let original_order: Vec<String> = original_docs.iter().map(|l| l.id.clone()).collect();
            
            // Get human ranking order
            let mut human_sorted = human_ranking.clone();
            human_sorted.sort_by_key(|r| r.rank);
            let human_order: Vec<String> = human_sorted.iter().map(|r| r.doc_id.clone()).collect();
            
            // Calculate rank changes
            let orig_pos: HashMap<&String, usize> = original_order.iter().enumerate()
                .map(|(i, id)| (id, i)).collect();
            let human_pos: HashMap<&String, usize> = human_order.iter().enumerate()
                .map(|(i, id)| (id, i)).collect();
            
            let mut rank_changes = Vec::new();
            let mut docs_moved_up = 0;
            let mut docs_moved_down = 0;
            let mut docs_unchanged = 0;
            
            for doc_id in &human_order {
                if let Some(&orig_rank) = orig_pos.get(doc_id) {
                    if let Some(&human_rank) = human_pos.get(doc_id) {
                        let change = human_rank as i32 - orig_rank as i32;
                        rank_changes.push(change);
                        
                        if change > 0 {
                            docs_moved_down += 1;
                        } else if change < 0 {
                            docs_moved_up += 1;
                        } else {
                            docs_unchanged += 1;
                        }
                    }
                }
            }
            
            let average_rank_change = if rank_changes.is_empty() {
                0.0
            } else {
                rank_changes.iter().map(|&x| x.abs() as f64).sum::<f64>() / rank_changes.len() as f64
            };
            
            let max_rank_change = rank_changes.iter().map(|&x| x.abs()).max().unwrap_or(0);
            
            let analysis = RankingAnalysis {
                qid: query.qid.clone(),
                query: query.query.clone(),
                total_relevant_docs: original_order.len(),
                human_ranked_docs: human_order.len(),
                kendall_tau: calculate_kendall_tau(&original_order, &human_order),
                spearman_correlation: None, // Could implement if needed
                average_rank_change,
                max_rank_change,
                docs_moved_up,
                docs_moved_down,
                docs_unchanged,
                top3_overlap: calculate_top_k_overlap(&original_order, &human_order, 3),
                top5_overlap: calculate_top_k_overlap(&original_order, &human_order, 5),
                top10_overlap: calculate_top_k_overlap(&original_order, &human_order, 10),
            };
            
            analyses.push(analysis);
        }
    }
    
    analyses
}

fn print_summary_statistics(analyses: &[RankingAnalysis]) {
    if analyses.is_empty() {
        println!("No analyses to summarize");
        return;
    }
    
    println!("=== RANKING ANALYSIS SUMMARY ===\n");
    
    println!("Total queries analyzed: {}", analyses.len());
    
    // Kendall Tau statistics
    let kendall_values: Vec<f64> = analyses.iter()
        .filter_map(|a| a.kendall_tau)
        .collect();
    
    if !kendall_values.is_empty() {
        let avg_kendall = kendall_values.iter().sum::<f64>() / kendall_values.len() as f64;
        let min_kendall = kendall_values.iter().fold(f64::INFINITY, |a, &b| a.min(b));
        let max_kendall = kendall_values.iter().fold(f64::NEG_INFINITY, |a, &b| a.max(b));
        
        println!("Kendall Tau Correlation:");
        println!("  Average: {:.3}", avg_kendall);
        println!("  Range: {:.3} to {:.3}", min_kendall, max_kendall);
        println!("  (1.0 = perfect agreement, 0.0 = no correlation, -1.0 = perfect disagreement)\n");
    }
    
    // Rank change statistics
    let avg_rank_change = analyses.iter().map(|a| a.average_rank_change).sum::<f64>() / analyses.len() as f64;
    let max_rank_change = analyses.iter().map(|a| a.max_rank_change).max().unwrap_or(0);
    
    println!("Rank Change Statistics:");
    println!("  Average rank change magnitude: {:.2} positions", avg_rank_change);
    println!("  Maximum rank change: {} positions", max_rank_change);
    
    // Movement statistics
    let total_moved_up: usize = analyses.iter().map(|a| a.docs_moved_up).sum();
    let total_moved_down: usize = analyses.iter().map(|a| a.docs_moved_down).sum();
    let total_unchanged: usize = analyses.iter().map(|a| a.docs_unchanged).sum();
    let total_docs = total_moved_up + total_moved_down + total_unchanged;
    
    if total_docs > 0 {
        println!("\nDocument Movement:");
        println!("  Moved up: {} ({:.1}%)", total_moved_up, 100.0 * total_moved_up as f64 / total_docs as f64);
        println!("  Moved down: {} ({:.1}%)", total_moved_down, 100.0 * total_moved_down as f64 / total_docs as f64);
        println!("  Unchanged: {} ({:.1}%)", total_unchanged, 100.0 * total_unchanged as f64 / total_docs as f64);
    }
    
    // Top-k overlap statistics
    let avg_top3 = analyses.iter().map(|a| a.top3_overlap).sum::<f64>() / analyses.len() as f64;
    let avg_top5 = analyses.iter().map(|a| a.top5_overlap).sum::<f64>() / analyses.len() as f64;
    let avg_top10 = analyses.iter().map(|a| a.top10_overlap).sum::<f64>() / analyses.len() as f64;
    
    println!("\nTop-K Overlap (how much top-ranked documents stay the same):");
    println!("  Top-3 overlap: {:.1}%", avg_top3 * 100.0);
    println!("  Top-5 overlap: {:.1}%", avg_top5 * 100.0);
    println!("  Top-10 overlap: {:.1}%", avg_top10 * 100.0);
}

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// Dataset name (e.g., fiqa, msmarco, scidocs)
    #[arg(short, long)]
    dataset: String,
}

fn main() -> Result<()> {
    let args = Args::parse();
    let dataset = &args.dataset;
    
    println!("Loading {} queries and human rankings...", dataset.to_uppercase());
    
    let queries = load_queries(&format!("data/{}_queries.jsonl", dataset))?;
    let human_rankings = load_human_rankings(&format!("data/{}_human_rankings.jsonl", dataset))?;
    
    println!("Found {} queries and {} human rankings", queries.len(), human_rankings.len());
    
    let analyses = analyze_rankings(&queries, &human_rankings);
    
    // Print detailed results
    println!("\n=== DETAILED ANALYSIS ===\n");
    for analysis in &analyses {
        println!("Query {}: {}", analysis.qid, analysis.query);
        println!("  Original docs: {}, Human ranked: {}", analysis.total_relevant_docs, analysis.human_ranked_docs);
        if let Some(tau) = analysis.kendall_tau {
            println!("  Kendall Tau: {:.3}", tau);
        }
        println!("  Avg rank change: {:.2}, Max change: {}", analysis.average_rank_change, analysis.max_rank_change);
        println!("  Movement: ↑{} ↓{} ={}", analysis.docs_moved_up, analysis.docs_moved_down, analysis.docs_unchanged);
        println!("  Top-K overlap: T3={:.1}% T5={:.1}% T10={:.1}%", 
                 analysis.top3_overlap * 100.0, analysis.top5_overlap * 100.0, analysis.top10_overlap * 100.0);
        println!();
    }
    
    print_summary_statistics(&analyses);
    
    // Save detailed results to JSON
    let output_file = format!("data/{}_ranking_analysis_results.json", dataset);
    let json_output = serde_json::to_string_pretty(&analyses)?;
    fs::write(&output_file, json_output)?;
    println!("\nDetailed results saved to {}", output_file);
    
    Ok(())
}