use anyhow::Result;
use clap::Parser;
use rand::seq::SliceRandom;
use reqwest::Client;
use std::fs::OpenOptions;
use std::io::Write;
use std::path::Path;
use tokio::time::{sleep, Duration};
use voyage_reranker_test::{
    generate_personalization_preference, load_documents, load_preferences, load_queries,
};

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
    let client = Client::new();

    println!("🚀 Preference Preparation Script");
    println!("=================================");
    
    // Dataset configuration
    let dataset_name = &args.dataset;
    println!("📊 Dataset: {}", dataset_name.to_uppercase());

    // Load dataset
    println!("📄 Loading {} dataset...", dataset_name.to_uppercase());
    let queries_file = format!("data/{}_queries.jsonl", dataset_name);
    let docs_file = format!("data/{}_docs.jsonl", dataset_name);
    let preferences_file = format!("data/{}_preferences.jsonl", dataset_name);
    
    let queries = load_queries(&queries_file)?;
    let documents = load_documents(&docs_file)?;
    println!("✅ Loaded {} queries and {} documents", queries.len(), documents.len());
    // Load existing preferences to avoid regeneration
    let existing_preferences = if Path::new(&preferences_file).exists() {
        load_preferences(&preferences_file)?
    } else {
        std::collections::HashMap::new()
    };
    println!("📋 Found {} existing preferences", existing_preferences.len());

    // Filter queries: those with multiple relevant documents that need preferences
    let mut candidate_queries: Vec<_> = queries
        .iter()
        .filter(|q| {
            let relevant_count = q.labels.iter().filter(|l| l.score > 0).count();
            relevant_count >= 3 && relevant_count <= 20 // Good range for manual ranking
        })
        .filter(|q| !existing_preferences.contains_key(&q.qid)) // Skip if already have preference
        .collect();

    println!("🎯 Found {} candidate queries", candidate_queries.len());

    // Randomly select up to 100 queries
    let mut rng = rand::thread_rng();
    candidate_queries.shuffle(&mut rng);
    let target_queries: Vec<_> = candidate_queries.into_iter().take(100).collect();
    
    println!("📝 Selected {} queries for preference generation", target_queries.len());

    if target_queries.is_empty() {
        println!("✅ All target queries already have preferences! Nothing to do.");
        return Ok(());
    }

    // Process queries in batches to avoid rate limiting
    let batch_size = 5;
    let mut processed = 0;
    let total = target_queries.len();

    for (batch_idx, batch) in target_queries.chunks(batch_size).enumerate() {
        println!("\n🔄 Processing batch {} ({} queries)", batch_idx + 1, batch.len());
        
        for query in batch {
            print!("  📝 Query {}: ", query.qid);
            
            // Get relevant documents for this query
            let relevant_docs: Vec<String> = query
                .labels
                .iter()
                .filter(|label| label.score > 0)
                .filter_map(|label| documents.get(&label.id))
                .cloned()
                .collect();

            if relevant_docs.is_empty() {
                println!("❌ No relevant documents found");
                continue;
            }

            // Generate preference
            match generate_personalization_preference(&client, &query.query, &relevant_docs).await {
                Ok(preference) => {
                    // Save to file immediately
                    let preference_data = serde_json::json!({
                        "qid": query.qid,
                        "query": query.query,
                        "preference": preference,
                        "timestamp": chrono::Utc::now().to_rfc3339(),
                        "relevant_docs_count": relevant_docs.len()
                    });

                    if let Ok(json_line) = serde_json::to_string(&preference_data) {
                        match OpenOptions::new().create(true).append(true).open(&preferences_file) {
                            Ok(mut file) => {
                                if writeln!(file, "{}", json_line).is_ok() {
                                    println!("✅ Generated and saved");
                                    processed += 1;
                                } else {
                                    println!("❌ Failed to write to file");
                                }
                            }
                            Err(e) => println!("❌ Failed to open file: {}", e),
                        }
                    } else {
                        println!("❌ Failed to serialize JSON");
                    }
                }
                Err(e) => {
                    println!("❌ Generation failed: {}", e);
                    // Continue with next query instead of failing completely
                }
            }

            // Brief delay to be respectful to API
            sleep(Duration::from_millis(500)).await;
        }

        // Longer delay between batches
        if batch_idx < target_queries.chunks(batch_size).len() - 1 {
            println!("⏳ Waiting 5 seconds before next batch...");
            sleep(Duration::from_secs(5)).await;
        }
    }

    println!("\n🎉 Preference generation complete!");
    println!("📊 Summary:");
    println!("  - Processed: {}/{} queries", processed, total);
    println!("  - Success rate: {:.1}%", (processed as f64 / total as f64) * 100.0);
    println!("  - Saved to: {}", preferences_file);

    // Final verification
    let final_preferences = load_preferences(&preferences_file)?;
    println!("  - Total preferences in file: {}", final_preferences.len());

    println!("\n🔗 Next steps:");
    println!("  1. Test Voyage reranker: cargo run --bin test_voyage_reranker -- --dataset {}", dataset_name);
    println!("  2. Run the web UI: cargo run --bin ui -- --dataset {}", dataset_name);
    println!("  3. Visit: http://127.0.0.1:3000");
    println!("  4. Start ranking documents with pre-generated preferences!");

    Ok(())
}