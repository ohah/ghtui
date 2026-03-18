use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Discussion {
    pub number: u64,
    pub title: String,
    pub author: String,
    pub created_at: String,
    pub category: String,
    pub comments_count: u64,
    pub is_answered: bool,
    pub url: String,
}
