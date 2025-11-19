use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchResult {
    pub backend: String,
    pub name: String,
    pub identifier: String,
    pub description: String,
    pub version: Option<String>,
}

