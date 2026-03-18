use serde::{Deserialize, Serialize};

use super::common::User;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Repository {
    pub name: String,
    pub full_name: String,
    pub description: Option<String>,
    pub private: bool,
    pub fork: bool,
    pub archived: bool,
    pub disabled: bool,
    pub visibility: Option<String>,
    pub default_branch: String,
    pub language: Option<String>,
    pub stargazers_count: u64,
    pub forks_count: u64,
    pub open_issues_count: u64,
    pub watchers_count: u64,
    pub size: u64,
    pub has_issues: bool,
    pub has_projects: bool,
    pub has_wiki: bool,
    pub has_discussions: Option<bool>,
    pub allow_forking: Option<bool>,
    pub topics: Option<Vec<String>>,
    pub license: Option<License>,
    pub owner: User,
    pub html_url: String,
    pub created_at: String,
    pub updated_at: String,
    pub pushed_at: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct License {
    pub key: String,
    pub name: String,
    pub spdx_id: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BranchProtection {
    pub pattern: String,
    #[serde(default)]
    pub required_status_checks: Option<RequiredStatusChecks>,
    #[serde(default)]
    pub enforce_admins: Option<EnforceAdmins>,
    #[serde(default)]
    pub required_pull_request_reviews: Option<RequiredPullRequestReviews>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RequiredStatusChecks {
    pub strict: bool,
    #[serde(default)]
    pub contexts: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EnforceAdmins {
    pub enabled: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RequiredPullRequestReviews {
    #[serde(default)]
    pub required_approving_review_count: Option<u32>,
    #[serde(default)]
    pub dismiss_stale_reviews: bool,
    #[serde(default)]
    pub require_code_owner_reviews: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Collaborator {
    pub login: String,
    pub avatar_url: String,
    #[serde(default)]
    pub role_name: Option<String>,
    pub permissions: Option<CollaboratorPermissions>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Webhook {
    pub id: u64,
    pub name: String,
    pub active: bool,
    pub events: Vec<String>,
    pub config: WebhookConfig,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WebhookConfig {
    pub url: Option<String>,
    pub content_type: Option<String>,
    pub insecure_ssl: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeployKey {
    pub id: u64,
    pub key: String,
    pub title: String,
    pub verified: bool,
    pub read_only: bool,
    pub created_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CollaboratorPermissions {
    #[serde(default)]
    pub admin: bool,
    #[serde(default)]
    pub maintain: bool,
    #[serde(default)]
    pub push: bool,
    #[serde(default)]
    pub triage: bool,
    #[serde(default)]
    pub pull: bool,
}
