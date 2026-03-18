use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OrgInfo {
    pub login: String,
    pub name: Option<String>,
    pub description: Option<String>,
    pub members_count: Option<u64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OrgMember {
    pub login: String,
    pub role: Option<String>,
}
