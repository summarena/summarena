use crate::data::{load_documents, load_queries};
use crate::preferences::load_preferences;
use crate::types::Query;
use anyhow::Result;
use axum::{
    Json, Router,
    extract::{Path, State},
    http::StatusCode,
    response::Html,
    routing::{get, post},
};
use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs::OpenOptions;
use std::io::Write;
use std::sync::Arc;
use tower_http::{cors::CorsLayer, services::ServeDir};

#[derive(Clone)]
pub struct AppState {
    pub queries: Vec<Query>,
    pub documents: HashMap<String, String>,
    pub human_rankings: Arc<tokio::sync::Mutex<HashMap<String, Vec<HumanRanking>>>>,
    pub preferences: Arc<tokio::sync::Mutex<HashMap<String, String>>>,
    pub client: Client,
    pub dataset: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct HumanRanking {
    pub doc_id: String,
    pub rank: usize,
    pub relevance_score: Option<i32>,
}

#[derive(Debug, Serialize)]
pub struct QueryResponse {
    pub qid: String,
    pub query: String,
    pub documents: Vec<DocumentWithId>,
    pub preference: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct DocumentWithId {
    pub id: String,
    pub content: String,
}

#[derive(Debug, Deserialize)]
pub struct SaveRankingRequest {
    pub qid: String,
    pub rankings: Vec<HumanRanking>,
}

#[derive(Debug, Serialize)]
pub struct SavedData {
    pub qid: String,
    pub query: String,
    pub preference: String,
    pub rankings: Vec<HumanRanking>,
    pub timestamp: String,
}

pub async fn create_app(dataset: String) -> Result<Router> {
    let queries = load_queries(&format!("data/{}_queries.jsonl", dataset))?;
    let documents = load_documents(&format!("data/{}_docs.jsonl", dataset))?;
    let preferences_map = load_preferences(&format!("data/{}_preferences.jsonl", dataset))?;

    let state = AppState {
        queries,
        documents,
        human_rankings: Arc::new(tokio::sync::Mutex::new(HashMap::new())),
        preferences: Arc::new(tokio::sync::Mutex::new(preferences_map)),
        client: Client::new(),
        dataset: dataset.clone(),
    };

    let app = Router::new()
        .route("/", get(serve_index))
        .route("/api/queries", get(get_queries))
        .route("/api/query/:qid", get(get_query_basic))
        .route("/api/query/:qid/documents", get(get_query_documents))
        .route("/api/query/:qid/preference", get(get_query_preference))
        .route("/api/rankings", post(save_rankings))
        .route("/api/rankings/:qid", get(get_rankings))
        .nest_service("/static", ServeDir::new("static"))
        .layer(CorsLayer::permissive())
        .with_state(state);

    Ok(app)
}

async fn serve_index() -> Html<&'static str> {
    Html(include_str!("../static/index.html"))
}

async fn get_queries(State(state): State<AppState>) -> Json<Vec<String>> {
    let query_ids: Vec<String> = state
        .queries
        .iter()
        .filter(|q| {
            let relevant_count = q.labels.iter().filter(|l| l.score > 0).count();
            relevant_count >= 3 && relevant_count <= 20 // Same filter as prep script
        })
        .map(|q| q.qid.clone())
        .collect();

    Json(query_ids)
}

async fn get_query_basic(
    Path(qid): Path<String>,
    State(state): State<AppState>,
) -> Result<Json<QueryResponse>, StatusCode> {
    let query = state.queries
        .iter()
        .find(|q| q.qid == qid)
        .ok_or(StatusCode::NOT_FOUND)?;

    let documents: Vec<DocumentWithId> = query
        .labels
        .iter()
        .filter(|label| label.score > 0)
        .filter_map(|label| {
            state.documents.get(&label.id).map(|content| DocumentWithId {
                id: label.id.clone(),
                content: content.clone(),
            })
        })
        .collect();

    // Get preference from memory if it exists
    let preferences = state.preferences.lock().await;
    let preference = preferences.get(&qid).cloned();

    Ok(Json(QueryResponse {
        qid: query.qid.clone(),
        query: query.query.clone(),
        documents,
        preference,
    }))
}

async fn get_query_documents(
    Path(qid): Path<String>,
    State(state): State<AppState>,
) -> Result<Json<Vec<DocumentWithId>>, StatusCode> {
    let query = state.queries
        .iter()
        .find(|q| q.qid == qid)
        .ok_or(StatusCode::NOT_FOUND)?;

    let documents: Vec<DocumentWithId> = query
        .labels
        .iter()
        .filter(|label| label.score > 0)
        .filter_map(|label| {
            state.documents.get(&label.id).map(|content| DocumentWithId {
                id: label.id.clone(),
                content: content.clone(),
            })
        })
        .collect();

    Ok(Json(documents))
}

async fn get_query_preference(
    Path(qid): Path<String>,
    State(state): State<AppState>,
) -> Result<Json<Option<String>>, StatusCode> {
    // Check if query exists
    let _query = state.queries
        .iter()
        .find(|q| q.qid == qid)
        .ok_or(StatusCode::NOT_FOUND)?;

    let preferences = state.preferences.lock().await;
    let preference = preferences.get(&qid).cloned();

    Ok(Json(preference))
}

async fn save_rankings(
    State(state): State<AppState>,
    Json(request): Json<SaveRankingRequest>,
) -> StatusCode {
    // Save to memory
    let mut rankings = state.human_rankings.lock().await;
    rankings.insert(request.qid.clone(), request.rankings.clone());
    drop(rankings);

    // Get query and preference for file saving
    let query = state.queries.iter().find(|q| q.qid == request.qid);
    let preferences = state.preferences.lock().await;
    let preference = preferences.get(&request.qid);

    if let (Some(query), Some(preference)) = (query, preference) {
        // Save rankings to separate file
        let rankings_filename = format!("data/{}_human_rankings.jsonl", state.dataset);
        let ranking_data = serde_json::json!({
            "qid": request.qid,
            "rankings": request.rankings,
            "timestamp": chrono::Utc::now().to_rfc3339()
        });

        if let Ok(json_line) = serde_json::to_string(&ranking_data) {
            match OpenOptions::new()
                .create(true)
                .append(true)
                .open(&rankings_filename)
            {
                Ok(mut file) => {
                    if let Err(e) = writeln!(file, "{}", json_line) {
                        eprintln!("Error writing to {}: {}", rankings_filename, e);
                    } else {
                        println!("Saved ranking for query {} to rankings file", request.qid);
                    }
                }
                Err(e) => eprintln!("Error opening {}: {}", rankings_filename, e),
            }
        }

        // Save preference to separate file
        let preferences_filename = format!("data/{}_preferences.jsonl", state.dataset);
        let preference_data = serde_json::json!({
            "qid": request.qid,
            "query": query.query,
            "preference": preference.clone(),
            "timestamp": chrono::Utc::now().to_rfc3339()
        });

        if let Ok(json_line) = serde_json::to_string(&preference_data) {
            match OpenOptions::new()
                .create(true)
                .append(true)
                .open(&preferences_filename)
            {
                Ok(mut file) => {
                    if let Err(e) = writeln!(file, "{}", json_line) {
                        eprintln!("Error writing to {}: {}", preferences_filename, e);
                    } else {
                        println!(
                            "Saved preference for query {} to preferences file",
                            request.qid
                        );
                    }
                }
                Err(e) => eprintln!("Error opening {}: {}", preferences_filename, e),
            }
        }
    }

    StatusCode::OK
}

async fn get_rankings(
    Path(qid): Path<String>,
    State(state): State<AppState>,
) -> Json<Option<Vec<HumanRanking>>> {
    let rankings = state.human_rankings.lock().await;
    Json(rankings.get(&qid).cloned())
}

pub async fn start_server(dataset: String) -> Result<()> {
    let app = create_app(dataset).await?;

    let listener = tokio::net::TcpListener::bind("127.0.0.1:3000").await?;
    println!("Server running on http://127.0.0.1:3000");

    axum::serve(listener, app).await?;

    Ok(())
}
