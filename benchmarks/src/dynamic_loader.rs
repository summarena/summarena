use anyhow::Result;
use crate::mair_loader::{QueryWithCandidates, load_mair_data_with_first_stage_candidates};
use std::path::Path;
use std::io::Read;
use flate2::read::GzDecoder;

pub struct DatasetConfig {
    pub name: String,
    pub hf_path: String, // Added hf_path for correct casing
    pub queries_path: String,
    pub docs_path: String,
    pub description: String,
    pub queries_url: Option<String>,
    pub docs_url: Option<String>,
}

impl DatasetConfig {
    pub fn from_name(name: &str) -> Self {
        let normalized_name = name.to_lowercase();
        
        // Map known datasets to their exact Hugging Face directory names
        let hf_path = match normalized_name.as_str() {
            "acordar" => "ACORDAR".to_string(),
            "fiqa" => "FiQA".to_string(), 
            "nfcorpus" => "NFCorpus".to_string(),
            "scifact" => "SciFact".to_string(),
            "quora" => "Quora".to_string(),
            "trec-covid" => "Trec-Covid".to_string(),
            "scidocs" => "SciDocs".to_string(), 
            "fever" => "Fever".to_string(),
            "climate-fever" => "Climate-FEVER".to_string(),
            "arguana" => "ArguAna".to_string(),
            "msmarco" => "MSMARCO".to_string(),
            "cqadupstack" => "CQADupStack".to_string(),
            "dbpedia-entity" => "DBPedia-Entity".to_string(),
            "webis-touche2020" => "Webis-Touche2020".to_string(),
            // For unknown datasets, try capitalizing each word
            _ => {
                name.split('-')
                    .map(|word| {
                        let mut c = word.chars();
                        match c.next() {
                            None => String::new(),
                            Some(first) => first.to_uppercase().collect::<String>() + c.as_str(),
                        }
                    })
                    .collect::<Vec<_>>()
                    .join("-")
            }
        };
        
        // Generate description based on known datasets, with fallback
        let description = match normalized_name.as_str() {
            "acordar" => "ACORDAR - Ad Hoc Content-Based RDF Dataset Retrieval".to_string(),
            "fiqa" => "FiQA - Financial Question Answering".to_string(),
            "nfcorpus" => "NFCorpus - NutritionFacts Medical Information Retrieval".to_string(),
            "scifact" => "SciFact - Scientific Fact Verification".to_string(),
            "quora" => "Quora - Question Pair Similarity".to_string(),
            "trec-covid" => "TREC-COVID - COVID-19 Literature Retrieval".to_string(),
            "scidocs" => "SciDocs - Citation Prediction".to_string(),
            "fever" => "FEVER - Fact Extraction and VERification".to_string(),
            "climate-fever" => "Climate-FEVER - Climate Change Fact Verification".to_string(),
            "arguana" => "ArguAna - Argument Retrieval".to_string(),
            "touche-2020" => "Touché-2020 - Argument Retrieval".to_string(),
            "cqadupstack" => "CQADupStack - Community Question Answering".to_string(),
            "dbpedia-entity" => "DBPedia-Entity - Entity Retrieval".to_string(),
            "webis-touche2020" => "Webis-Touché2020 - Argument Retrieval".to_string(),
            "msmarco" => "MS MARCO - Microsoft Machine Reading Comprehension".to_string(),
            _ => format!("MAIR Dataset: {hf_path}"),
        };
        
        Self {
            name: normalized_name.clone(),
            hf_path: hf_path.clone(),
            queries_path: format!("data/{normalized_name}_queries.jsonl"),
            docs_path: format!("data/{normalized_name}_docs.jsonl"),
            description,
            queries_url: Some(format!(
                "https://huggingface.co/datasets/MAIR-Bench/MAIR-Queries/resolve/main/{hf_path}/queries.jsonl"
            )),
            docs_url: Some(format!(
                "https://huggingface.co/datasets/MAIR-Bench/MAIR-Docs/resolve/main/{hf_path}/docs.jsonl"
            )),
        }
    }
    
    pub fn with_fallback_urls(mut self, queries_url: &str, docs_url: &str) -> Self {
        self.queries_url = Some(queries_url.to_string());
        self.docs_url = Some(docs_url.to_string());
        self
    }
}

pub async fn load_dataset_by_name(dataset_name: &str) -> Result<Vec<QueryWithCandidates>> {
    load_dataset_by_name_with_quiet(dataset_name, false).await
}

pub async fn load_dataset_by_name_with_quiet(dataset_name: &str, quiet: bool) -> Result<Vec<QueryWithCandidates>> {
    let config = DatasetConfig::from_name(dataset_name);
    
    if !quiet {
        // Description suppressed
    }
    load_dataset_with_fallback_quiet(config, quiet).await
}

pub async fn load_dataset_with_fallback(config: DatasetConfig) -> Result<Vec<QueryWithCandidates>> {
    load_dataset_with_fallback_quiet(config, false).await
}

async fn load_dataset_with_fallback_quiet(config: DatasetConfig, quiet: bool) -> Result<Vec<QueryWithCandidates>> {
    // Use MAIR-style first-stage candidates (up to 100)
    const DEFAULT_CANDIDATE_LIMIT: usize = 100;
    
    // Try loading from local files first
    if Path::new(&config.queries_path).exists() && Path::new(&config.docs_path).exists() {
        if !quiet {
            // Loading progress suppressed
        }
        return load_mair_data_with_first_stage_candidates(
            &config.queries_path, 
            &config.docs_path, 
            &config.hf_path, // Use hf_path
            DEFAULT_CANDIDATE_LIMIT
        ).await;
    }
    
    // Fall back to remote URLs if local files don't exist
    if let (Some(queries_url), Some(docs_url)) = (&config.queries_url, &config.docs_url) {
        // Download progress suppressed
        // Download progress suppressed
        // Download progress suppressed
        
        return load_dataset_from_urls_with_first_stage_candidates(
            queries_url, 
            docs_url, 
            Some("data"),
            Some(&config.name),
            &config.hf_path, // Use hf_path
            DEFAULT_CANDIDATE_LIMIT
        ).await;
    }
    
    // No fallback available
    Err(anyhow::anyhow!(
        "Dataset '{}' not found locally at {} and {}, and no remote URLs configured",
        config.name, config.queries_path, config.docs_path
    ))
}


async fn load_dataset_from_urls_with_first_stage_candidates(queries_url: &str, docs_url: &str, cache_dir: Option<&str>, dataset_name: Option<&str>, hf_path: &str, candidate_limit: usize) -> Result<Vec<QueryWithCandidates>> {
    let cache_dir = cache_dir.unwrap_or("data");
    
    // Create cache directory if it doesn't exist
    std::fs::create_dir_all(cache_dir)?;
    
    // Generate local file paths using dataset name if provided
    let queries_filename = if let Some(name) = dataset_name {
        format!("{}_queries.jsonl", name.to_lowercase())
    } else {
        let queries_basename = queries_url.split('/').next_back().unwrap_or("unknown");
        let queries_name = if queries_basename.ends_with(".gz") { 
            queries_basename.strip_suffix(".gz").unwrap_or(queries_basename) 
        } else { 
            queries_basename 
        };
        format!("remote_queries_{queries_name}")
    };
    
    let docs_filename = if let Some(name) = dataset_name {
        format!("{}_docs.jsonl", name.to_lowercase())
    } else {
        let docs_basename = docs_url.split('/').next_back().unwrap_or("unknown");
        let docs_name = if docs_basename.ends_with(".gz") { 
            docs_basename.strip_suffix(".gz").unwrap_or(docs_basename) 
        } else { 
            docs_basename 
        };
        format!("remote_docs_{docs_name}")
    };
        
    let queries_path = format!("{cache_dir}/{queries_filename}");
    let docs_path = format!("{cache_dir}/{docs_filename}");
    
    // Download files if they don't exist
    if !Path::new(&queries_path).exists() {
        // Download progress suppressed
        let response = reqwest::get(queries_url).await?;
        let bytes = response.bytes().await?;
        
        // Handle gzipped files
        if queries_url.ends_with(".gz") {
            let mut decoder = GzDecoder::new(&bytes[..]);
            let mut decompressed = String::new();
            decoder.read_to_string(&mut decompressed)?;
            std::fs::write(&queries_path, decompressed)?;
        } else {
            std::fs::write(&queries_path, bytes)?;
        }
    } else {
        // Cache usage suppressed
    }
    
    if !Path::new(&docs_path).exists() {
        // Download progress suppressed
        let response = reqwest::get(docs_url).await?;
        let bytes = response.bytes().await?;
        
        // Handle gzipped files  
        if docs_url.ends_with(".gz") {
            let mut decoder = GzDecoder::new(&bytes[..]);
            let mut decompressed = String::new();
            decoder.read_to_string(&mut decompressed)?;
            std::fs::write(&docs_path, decompressed)?;
        } else {
            std::fs::write(&docs_path, bytes)?;
        }
    } else {
        // Cache usage suppressed
    }
    
    // Load the downloaded/cached files with first-stage candidates
    load_mair_data_with_first_stage_candidates(
        &queries_path, 
        &docs_path, 
        hf_path, 
        candidate_limit
    ).await
}

pub fn list_available_datasets() -> Vec<DatasetConfig> {
    // Return a curated list of well-known datasets for the help command
    // This avoids the need to make HTTP requests just to show help
    vec![
        DatasetConfig::from_name("acordar"),
        DatasetConfig::from_name("fiqa"), 
        DatasetConfig::from_name("nfcorpus"),
        DatasetConfig::from_name("scifact"),
        DatasetConfig::from_name("quora"),
        DatasetConfig::from_name("trec-covid"),
        DatasetConfig::from_name("scidocs"),
        DatasetConfig::from_name("fever"),
        DatasetConfig::from_name("climate-fever"),
        DatasetConfig::from_name("arguana"),
        DatasetConfig::from_name("msmarco"),
    ]
}

pub async fn discover_available_datasets() -> Result<Vec<String>> {
    // Dynamically fetch available datasets from Hugging Face
    let url = "https://huggingface.co/api/datasets/MAIR-Bench/MAIR-Queries/tree/main";
    
    let response = reqwest::get(url).await?;
    let data: serde_json::Value = response.json().await?;
    
    let mut datasets = std::collections::HashSet::new();
    
    if let Some(entries) = data.as_array() {
        for entry in entries {
            if let Some(path) = entry.get("path").and_then(|p| p.as_str()) {
                if let Some(dataset_name) = path.split('/').next() {
                    if dataset_name != ".gitattributes" && !dataset_name.is_empty() {
                        datasets.insert(dataset_name.to_lowercase());
                    }
                }
            }
        }
    }
    
    let mut dataset_list: Vec<String> = datasets.into_iter().collect();
    dataset_list.sort();
    Ok(dataset_list)
}

