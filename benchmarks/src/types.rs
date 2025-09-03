use serde::{Deserialize};

#[derive(Debug, Deserialize, Clone)]
pub struct Query {
    pub qid: String,
    pub query: String,
    pub labels: Vec<Label>,
}

#[derive(Debug, Deserialize, Clone)]
pub struct Label {
    pub id: String,
    pub score: i32,
}

#[derive(Debug, Deserialize)]
pub struct Document {
    pub id: String,
    pub doc: String,
}
