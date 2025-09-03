use anyhow::Result;
use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::env;

#[derive(Debug, Serialize)]
pub struct RerankRequest {
    pub query: String,
    pub documents: Vec<String>,
    pub model: String,
}

#[derive(Debug, Deserialize)]
pub struct RerankResponse {
    pub data: Vec<RerankResult>,
}

#[derive(Debug, Deserialize)]
pub struct RerankResult {
    pub index: usize,
    pub relevance_score: f64,
}

pub async fn rerank_with_voyage(client: &Client, query: &str, documents: Vec<String>) -> Result<Vec<(usize, f64)>> {
    let request = RerankRequest {
        query: query.to_string(),
        documents,
        model: "rerank-2".to_string(),
    };

    let response = client
        .post("https://api.voyageai.com/v1/rerank")
        .header("Authorization", format!("Bearer {}", env::var("VOYAGE_API_KEY")?))
        .header("Content-Type", "application/json")
        .json(&request)
        .send()
        .await?;

    let resp: RerankResponse = response.json().await?;
    let results = resp.data.into_iter()
        .map(|r| (r.index, r.relevance_score))
        .collect();
    
    Ok(results)
}