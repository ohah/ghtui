use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use super::common::{Label, Milestone, User};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum IssueState {
    Open,
    Closed,
}

impl std::fmt::Display for IssueState {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            IssueState::Open => write!(f, "open"),
            IssueState::Closed => write!(f, "closed"),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Issue {
    pub number: u64,
    pub title: String,
    pub state: IssueState,
    pub user: User,
    pub body: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub closed_at: Option<DateTime<Utc>>,
    #[serde(default)]
    pub labels: Vec<Label>,
    #[serde(default)]
    pub assignees: Vec<User>,
    pub milestone: Option<Milestone>,
    pub comments: Option<u32>,
}

#[derive(Debug, Clone)]
pub struct IssueDetail {
    pub issue: Issue,
    pub comments: Vec<IssueComment>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IssueComment {
    pub id: u64,
    pub user: User,
    pub body: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Default, PartialEq)]
pub struct IssueFilters {
    pub state: Option<IssueState>,
    pub author: Option<String>,
    pub assignee: Option<String>,
    pub label: Option<String>,
    pub sort: Option<String>,
    pub direction: Option<String>,
}

#[derive(Debug, Clone)]
pub struct CreateIssueInput {
    pub title: String,
    pub body: String,
    pub labels: Vec<String>,
    pub assignees: Vec<String>,
}
