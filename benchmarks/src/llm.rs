use anyhow::Result;
use rand::seq::SliceRandom;
use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::env;

#[derive(Debug, Serialize)]
struct PersonalizationRequest {
    model: String,
    messages: Vec<Message>,
    temperature: f64,
}

#[derive(Debug, Serialize)]
struct Message {
    role: String,
    content: String,
}

#[derive(Debug, Deserialize)]
struct PersonalizationResponse {
    choices: Vec<Choice>,
}

#[derive(Debug, Deserialize)]
struct Choice {
    message: MessageContent,
}

#[derive(Debug, Deserialize)]
struct MessageContent {
    content: String,
}

pub async fn generate_personalization_preference(client: &Client, query: &str, documents: &[String]) -> Result<String> {
    // Randomize document order before using for prompt
    let mut docs_shuffled = documents.to_vec();
    let mut rng = rand::thread_rng();
    docs_shuffled.shuffle(&mut rng);
    
    let doc_previews: Vec<String> = docs_shuffled.iter()
        .take(20)
        .enumerate()
        .map(|(i, doc)| {
            let preview = if doc.len() <= 1000 {
                doc.clone()
            } else {
                // Find the last character boundary before position 1000
                let mut end = 1000;
                while end > 0 && !doc.is_char_boundary(end) {
                    end -= 1;
                }
                format!("{}...", &doc[..end])
            };
            format!("Document {}: {}", i + 1, preview)
        })
        .collect();
    
    let prompt = format!(
        "Given this search query: '{}'\n\nAnd these sample documents:\n{}\n\nGenerate a specific user personalization preference that would meaningfully change how these results should be ranked. The preference should be realistic and relate to the user's background, expertise level, location, or specific needs that would affect document relevance.\n\nRespond with just the preference statement, no explanation.\n\nExample preferences:\n- 'I am a beginner investor looking for simple explanations'\n- 'I prefer academic sources and research papers'\n- 'I need advice specific to Canadian tax law'\n- 'I work in fintech and need technical implementation details'",
        query,
        doc_previews.join("\n\n")
    );

    let request = PersonalizationRequest {
        model: "gpt-4o".to_string(), // Using GPT-4o (GPT-5 requires org verification)
        messages: vec![Message {
            role: "user".to_string(),
            content: prompt,
        }],
        temperature: 0.7,
    };

    let response = client
        .post("https://api.openai.com/v1/chat/completions")
        .header("Authorization", format!("Bearer {}", env::var("OPENAI_API_KEY")?))
        .header("Content-Type", "application/json")
        .json(&request)
        .send()
        .await?;

    let response_text = response.text().await?;
    
    // Try to parse as JSON
    match serde_json::from_str::<PersonalizationResponse>(&response_text) {
        Ok(resp) => {
            if resp.choices.is_empty() {
                Err(anyhow::anyhow!("No choices returned from OpenAI API"))
            } else {
                Ok(resp.choices[0].message.content.trim().to_string())
            }
        },
        Err(_) => {
            // Check if it's an error response
            if let Ok(error_resp) = serde_json::from_str::<serde_json::Value>(&response_text) {
                if let Some(error) = error_resp.get("error") {
                    if let Some(message) = error.get("message") {
                        return Err(anyhow::anyhow!("OpenAI API Error: {}", message.as_str().unwrap_or("Unknown error")));
                    }
                }
            }
            Err(anyhow::anyhow!("Failed to parse OpenAI response"))
        }
    }
}