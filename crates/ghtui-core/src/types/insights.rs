use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Contributor {
    pub login: Option<String>,
    pub avatar_url: Option<String>,
    pub contributions: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContributorStats {
    pub author: Option<ContributorAuthor>,
    pub total: u64,
    #[serde(default)]
    pub weeks: Vec<WeeklyStats>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContributorAuthor {
    pub login: String,
    pub avatar_url: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WeeklyStats {
    #[serde(rename = "w")]
    pub week: i64,
    #[serde(rename = "a")]
    pub additions: u64,
    #[serde(rename = "d")]
    pub deletions: u64,
    #[serde(rename = "c")]
    pub commits: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CommitActivity {
    #[serde(default)]
    pub days: Vec<u64>,
    pub total: u64,
    pub week: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TrafficClones {
    pub count: u64,
    pub uniques: u64,
    #[serde(default)]
    pub clones: Vec<TrafficEntry>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TrafficViews {
    pub count: u64,
    pub uniques: u64,
    #[serde(default)]
    pub views: Vec<TrafficEntry>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TrafficEntry {
    pub timestamp: String,
    pub count: u64,
    pub uniques: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CodeFrequency(pub i64, pub i64, pub i64); // [week, additions, deletions]
