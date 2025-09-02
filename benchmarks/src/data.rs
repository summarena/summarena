use anyhow::Result;
use std::collections::HashMap;
use std::fs;
use crate::types::{Query, Document};

pub fn load_queries(file_path: &str) -> Result<Vec<Query>> {
    let content = fs::read_to_string(file_path)?;
    let mut queries = Vec::new();
    
    for line in content.lines() {
        if !line.trim().is_empty() {
            let query: Query = serde_json::from_str(line)?;
            queries.push(query);
        }
    }
    
    Ok(queries)
}

pub fn load_documents(file_path: &str) -> Result<HashMap<String, String>> {
    let content = fs::read_to_string(file_path)?;
    let mut docs = HashMap::new();
    
    for line in content.lines() {
        if !line.trim().is_empty() {
            let doc: Document = serde_json::from_str(line)?;
            docs.insert(doc.id, doc.doc);
        }
    }
    
    Ok(docs)
}