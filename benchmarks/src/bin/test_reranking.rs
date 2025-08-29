use anyhow::Result;
use clap::Parser;
use test_reranker::dynamic_loader::{discover_available_datasets, list_available_datasets, load_dataset_by_name};
use test_reranker::reranker::{calculate_ndcg_at_k_with_relevance_map, VoyageReranker};

#[derive(Debug)]
struct QueryResult {
    query_index: usize,
    query_text: String,
    baseline_ndcg: f64,
    personalized_results: Vec<PersonalizedResult>,
}

#[derive(Debug)]
struct PersonalizedResult {
    persona_description: String,
    ndcg: f64,
    improvement: f64,
}

#[derive(Parser)]
#[command(name = "test_reranking")]
#[command(about = "PAIR Reranking Test - Personalized vs Non-Personalized")]
struct Args {
    /// Run only a single query by index (0-based)
    #[arg(short, long)]
    single: Option<usize>,

    /// Number of random queries to test (default: 10)
    #[arg(short, long, default_value = "10")]
    count: usize,

    /// Custom persona/preference to test (if provided, only tests baseline + this persona)
    #[arg(short, long)]
    persona: Option<String>,

    /// Dataset to use (any MAIR dataset name) - defaults to acordar
    #[arg(short, long, default_value = "acordar")]
    dataset: String,

    /// List available datasets and exit
    #[arg(long)]
    list_datasets: bool,

    /// Discover all datasets from Hugging Face (slower but comprehensive)
    #[arg(long)]
    discover_all: bool,

    /// Number of first-stage candidates to use for reranking (default: 100, MAIR standard)
    #[arg(long, default_value = "100")]
    candidate_limit: usize,

    /// Only print final comparison results (quiet mode)
    #[arg(short, long)]
    quiet: bool,

}

#[tokio::main]
async fn main() -> Result<()> {
    let args = Args::parse();

    // Handle dataset listing
    if args.list_datasets {
        println!("Popular datasets:");
        for dataset in list_available_datasets() {
            println!("  {} - {}", dataset.name, dataset.description);
        }
        println!("\nNote: Many more datasets are available. Use --discover-all to see all 120+ datasets (requires internet).");
        return Ok(());
    }

    // Handle comprehensive dataset discovery
    if args.discover_all {
        println!("Discovering all available datasets from Hugging Face...");
        match discover_available_datasets().await {
            Ok(datasets) => {
                println!("All available datasets ({} total):", datasets.len());
                for (i, dataset) in datasets.iter().enumerate() {
                    println!("  {}. {}", i + 1, dataset);
                }
                println!("\nYou can use any of these with: --dataset <name>");
            }
            Err(e) => {
                eprintln!("Failed to discover datasets: {e}");
                eprintln!("Showing popular datasets instead:");
                for dataset in list_available_datasets() {
                    println!("  {} - {}", dataset.name, dataset.description);
                }
            }
        }
        return Ok(());
    }

    // Title suppressed

    // Check for API key
    let api_key = std::env::var("VOYAGE_API_KEY")
        .expect("VOYAGE_API_KEY environment variable must be set for reranking");
    let reranker = VoyageReranker::new_with_quiet(api_key, args.quiet);

    // Load predefined dataset with first-stage candidates (MAIR-style)
    let queries = load_dataset_by_name(&args.dataset).await?;

    // Override the candidate limit if specified
    let queries = if args.candidate_limit != 100 {
        // Reload progress suppressed
        let config = test_reranker::dynamic_loader::DatasetConfig::from_name(&args.dataset);
        if std::path::Path::new(&config.queries_path).exists()
            && std::path::Path::new(&config.docs_path).exists()
        {
            test_reranker::mair_loader::load_mair_data_with_first_stage_candidates(
                &config.queries_path,
                &config.docs_path,
                &config.hf_path,
                args.candidate_limit,
            )
            .await?
        } else {
            // Use the already loaded queries with default candidate limit
            queries
        }
    } else {
        queries
    };

    // Determine which queries to test
    let queries_to_test: Vec<_> = if let Some(single_index) = args.single {
        if single_index >= queries.len() {
            eprintln!("Error: Query index {} is out of range (0-{})", single_index, queries.len() - 1);
            std::process::exit(1);
        }
        // Test info suppressed
        vec![&queries[single_index]]
    } else {
        let take_n = args.count.min(queries.len());
        // Test info suppressed
        use rand::seq::SliceRandom;
        use rand::thread_rng;

        let mut rng = thread_rng();
        let mut random_queries: Vec<_> = queries.iter().collect();
        random_queries.shuffle(&mut rng);
        random_queries.into_iter().take(take_n).collect()
    };

    // Collect results for quiet mode
    let mut quiet_results = Vec::new();

    for (i, query_data) in queries_to_test.iter().enumerate() {
        // Query details suppressed

        // Baseline test info suppressed
        
        // Extract document texts for plain reranking
        let doc_texts: Vec<String> = query_data.candidates
            .iter()
            .map(|doc| reranker.extract_doc_text(&doc.doc))
            .collect();
        
        let baseline_ndcg = match reranker.rerank(&query_data.query.query, &doc_texts, None).await {
            Ok(rankings) => {
                // Reorder documents based on ranking results
                let mut plain_reranked_docs = Vec::new();
                for (index, _score) in rankings {
                    if index < query_data.candidates.len() {
                        plain_reranked_docs.push(query_data.candidates[index].clone());
                    }
                }
                
                let plain_reranked_ndcg = calculate_ndcg_at_k_with_relevance_map(
                    &plain_reranked_docs,
                    &query_data.relevance_map,
                    10,
                );
                let original_ndcg = calculate_ndcg_at_k_with_relevance_map(
                    &query_data.candidates,
                    &query_data.relevance_map,
                    10,
                );
                
                // Baseline results suppressed
                
                plain_reranked_ndcg
            }
            Err(e) => {
                // Error suppressed
                // Fallback to original ordering
                calculate_ndcg_at_k_with_relevance_map(
                    &query_data.candidates,
                    &query_data.relevance_map,
                    10,
                )
            }
        };

        // Small delay to avoid rate limiting
        tokio::time::sleep(tokio::time::Duration::from_millis(500)).await;

        // Always generate personas using LLM
        let test_preferences = if let Some(ref custom_persona) = args.persona {
            // Custom persona provided via CLI
            vec![
                vec![], // No preferences (baseline)
                vec![custom_persona.clone()],
            ]
        } else {
            // Generate personas using LLM for this specific query
            let reranker = &reranker;
            match reranker
                .generate_personas(&query_data.query.query, &query_data.query.instruction)
                .await
            {
                Ok(generated_personas) => {
                    let mut prefs = vec![vec![]]; // Start with baseline (no preferences)
                    prefs.extend(generated_personas.into_iter().map(|p| vec![p]));
                    prefs
                }
                Err(e) => {
                    eprintln!("Failed to generate personas: {}", e);
                    eprintln!("Skipping this query due to persona generation failure.");
                    continue; // Skip this query if persona generation fails
                }
            }
        };

        // Skip the empty preferences test since we already did baseline
        let test_preferences: Vec<_> = test_preferences.into_iter().filter(|p| !p.is_empty()).collect();
        
        // Collect personalized results for this query
        let mut personalized_results = Vec::new();
        
        for (pref_idx, preferences) in test_preferences.iter().enumerate() {
            // Test info suppressed

            match reranker
                .rerank_candidates_with_preferences(query_data, preferences, None)
                .await
            {
                Ok(reranked_docs) => {
                    let reranked_ndcg = calculate_ndcg_at_k_with_relevance_map(
                        &reranked_docs,
                        &query_data.relevance_map,
                        10,
                    );
                    let improvement = reranked_ndcg - baseline_ndcg;

                    // Collect result for quiet mode
                    let persona_description = preferences.join("; ");
                    personalized_results.push(PersonalizedResult {
                        persona_description,
                        ndcg: reranked_ndcg,
                        improvement,
                    });

                    // Personalized results suppressed
                }
                Err(e) => {
                    // Error suppressed
                }
            }

            // Small delay to avoid rate limiting
            tokio::time::sleep(tokio::time::Duration::from_millis(500)).await;
        }

        // Collect this query's results
        quiet_results.push(QueryResult {
            query_index: i + 1,
            query_text: query_data.query.query.clone(),
            baseline_ndcg,
            personalized_results,
        });

        // Query end suppressed
    }

    // Print final results
    {
        println!("FINAL COMPARISON RESULTS");
        println!("{}", "=".repeat(60));
        
        for result in &quiet_results {
            println!("\nQuery {}: {}", result.query_index, 
                     result.query_text.chars().take(60).collect::<String>() + 
                     if result.query_text.len() > 60 { "..." } else { "" });
            println!("Baseline NDCG@10: {:.4}", result.baseline_ndcg);
            
            for persona_result in &result.personalized_results {
                println!("  Persona: {} → NDCG@10: {:.4} (Δ{:.4})", 
                         persona_result.persona_description.chars().take(50).collect::<String>() +
                         if persona_result.persona_description.len() > 50 { "..." } else { "" },
                         persona_result.ndcg, 
                         persona_result.improvement);
            }
        }
        
        // Summary statistics
        if !quiet_results.is_empty() {
            let avg_baseline: f64 = quiet_results.iter().map(|r| r.baseline_ndcg).sum::<f64>() / quiet_results.len() as f64;
            let all_improvements: Vec<f64> = quiet_results.iter()
                .flat_map(|r| r.personalized_results.iter().map(|p| p.improvement))
                .collect();
            
            if !all_improvements.is_empty() {
                let avg_improvement = all_improvements.iter().sum::<f64>() / all_improvements.len() as f64;
                let positive_improvements = all_improvements.iter().filter(|&&x| x > 0.0).count();
                
                println!("\n{}", "=".repeat(60));
                println!("SUMMARY");
                println!("Average baseline NDCG@10: {:.4}", avg_baseline);
                println!("Average improvement: {:.4}", avg_improvement);
                println!("Positive improvements: {}/{} ({:.1}%)", 
                         positive_improvements, 
                         all_improvements.len(),
                         positive_improvements as f64 / all_improvements.len() as f64 * 100.0);
            }
        }
    }

    Ok(())
}

// extract_title_and_preview function removed (unused)

