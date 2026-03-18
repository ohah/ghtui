use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GistEntry {
    pub id: String,
    pub description: Option<String>,
    pub public: bool,
    pub files_count: usize,
    pub created_at: String,
    pub html_url: String,
}
