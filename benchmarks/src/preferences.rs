use anyhow::Result;
use serde_json::Value;
use std::collections::HashMap;
use std::fs;

pub fn load_preferences(filename: &str) -> Result<HashMap<String, String>> {
    let mut preferences = HashMap::new();
    
    if !std::path::Path::new(filename).exists() {
        return Ok(preferences);
    }
    
    let content = fs::read_to_string(filename)?;
    
    for line in content.lines() {
        if !line.trim().is_empty() {
            let data: Value = serde_json::from_str(line)?;
            if let (Some(qid), Some(preference)) = (
                data.get("qid").and_then(|v| v.as_str()),
                data.get("preference").and_then(|v| v.as_str())
            ) {
                preferences.insert(qid.to_string(), preference.to_string());
            }
        }
    }
    
    Ok(preferences)
}