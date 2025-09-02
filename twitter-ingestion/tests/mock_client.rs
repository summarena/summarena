use twitter_ingestion::{HttpClient, HttpResponse};
use anyhow::Result;
use async_trait::async_trait;
use std::collections::HashMap;
use http::StatusCode;

/// Mock HTTP client for testing
pub struct MockHttpClient {
    responses: HashMap<String, MockResponse>,
}

#[derive(Clone)]
pub struct MockResponse {
    pub status: u16,
    pub body: String,
}

impl Default for MockHttpClient {
    fn default() -> Self {
        Self::new()
    }
}

impl MockHttpClient {
    pub fn new() -> Self {
        Self {
            responses: HashMap::new(),
        }
    }

    /// Add a mock response for a specific URL
    pub fn add_response(mut self, url: &str, response: MockResponse) -> Self {
        self.responses.insert(url.to_string(), response);
        self
    }

    /// Helper to add a successful JSON response
    pub fn add_json_response(self, url: &str, json: &str) -> Self {
        self.add_response(url, MockResponse {
            status: StatusCode::OK.as_u16(),
            body: json.to_string(),
        })
    }

    /// Helper to add an error response
    pub fn add_error_response(self, url: &str, status: u16) -> Self {
        self.add_response(url, MockResponse {
            status,
            body: format!("{{\"error\": \"HTTP {}\"}}", status),
        })
    }
}

#[async_trait]
impl HttpClient for MockHttpClient {
    async fn get(&self, url: &str) -> Result<HttpResponse> {
        match self.responses.get(url) {
            Some(mock_response) => Ok(HttpResponse {
                status: mock_response.status,
                body: mock_response.body.clone(),
            }),
            None => Ok(HttpResponse {
                status: StatusCode::NOT_FOUND.as_u16(),
                body: "Not Found".to_string(),
            }),
        }
    }

    async fn get_with_auth_and_query(
        &self,
        url: &str,
        _auth_header: &str,
        _query_params: &[(String, String)],
    ) -> Result<HttpResponse> {
        // For simplicity, just use the base URL for matching
        // In a real test, you might want to include auth/query in the matching
        self.get(url).await
    }
}