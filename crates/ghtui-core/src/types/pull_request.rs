use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use super::common::{Label, Milestone, User};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum PrState {
    Open,
    Closed,
    Merged,
}

impl std::fmt::Display for PrState {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            PrState::Open => write!(f, "open"),
            PrState::Closed => write!(f, "closed"),
            PrState::Merged => write!(f, "merged"),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MergeMethod {
    Merge,
    Squash,
    Rebase,
}

impl MergeMethod {
    pub fn as_str(&self) -> &str {
        match self {
            MergeMethod::Merge => "merge",
            MergeMethod::Squash => "squash",
            MergeMethod::Rebase => "rebase",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ReviewState {
    Approved,
    ChangesRequested,
    Commented,
    Pending,
    Dismissed,
}

impl std::fmt::Display for ReviewState {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ReviewState::Approved => write!(f, "approved"),
            ReviewState::ChangesRequested => write!(f, "changes_requested"),
            ReviewState::Commented => write!(f, "commented"),
            ReviewState::Pending => write!(f, "pending"),
            ReviewState::Dismissed => write!(f, "dismissed"),
        }
    }
}

#[derive(Debug, Clone, Serialize)]
pub struct PullRequest {
    pub number: u64,
    pub title: String,
    pub state: PrState,
    pub user: User,
    pub body: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub merged_at: Option<DateTime<Utc>>,
    pub closed_at: Option<DateTime<Utc>>,
    pub head_ref: String,
    pub head_sha: String,
    pub base_ref: String,
    pub draft: bool,
    pub labels: Vec<Label>,
    pub assignees: Vec<User>,
    pub milestone: Option<Milestone>,
    pub requested_reviewers: Vec<User>,
    pub additions: Option<u32>,
    pub deletions: Option<u32>,
    pub changed_files: Option<u32>,
    pub mergeable: Option<bool>,
    pub comments: Option<u32>,
    pub review_comments: Option<u32>,
}

// GitHub API returns head/base as nested objects: { "ref": "...", ... }
#[derive(Deserialize)]
struct GitRef {
    #[serde(rename = "ref")]
    ref_name: String,
    sha: String,
}

#[derive(Deserialize)]
struct ApiPullRequest {
    number: u64,
    title: String,
    state: PrState,
    user: User,
    body: Option<String>,
    created_at: DateTime<Utc>,
    updated_at: DateTime<Utc>,
    merged_at: Option<DateTime<Utc>>,
    closed_at: Option<DateTime<Utc>>,
    head: GitRef,
    base: GitRef,
    #[serde(default)]
    draft: bool,
    #[serde(default)]
    labels: Vec<Label>,
    #[serde(default)]
    assignees: Vec<User>,
    milestone: Option<Milestone>,
    #[serde(default)]
    requested_reviewers: Vec<User>,
    additions: Option<u32>,
    deletions: Option<u32>,
    changed_files: Option<u32>,
    mergeable: Option<bool>,
    comments: Option<u32>,
    review_comments: Option<u32>,
}

impl<'de> Deserialize<'de> for PullRequest {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let api = ApiPullRequest::deserialize(deserializer)?;
        Ok(PullRequest {
            number: api.number,
            title: api.title,
            state: api.state,
            user: api.user,
            body: api.body,
            created_at: api.created_at,
            updated_at: api.updated_at,
            merged_at: api.merged_at,
            closed_at: api.closed_at,
            head_ref: api.head.ref_name,
            head_sha: api.head.sha,
            base_ref: api.base.ref_name,
            draft: api.draft,
            labels: api.labels,
            assignees: api.assignees,
            milestone: api.milestone,
            requested_reviewers: api.requested_reviewers,
            additions: api.additions,
            deletions: api.deletions,
            changed_files: api.changed_files,
            mergeable: api.mergeable,
            comments: api.comments,
            review_comments: api.review_comments,
        })
    }
}

#[derive(Debug, Clone)]
pub struct PrCommit {
    pub sha: String,
    pub message: String,
    pub author: String,
    pub date: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone)]
pub struct PullRequestDetail {
    pub pr: PullRequest,
    pub reviews: Vec<Review>,
    pub comments: Vec<PrComment>,
    pub review_comments: Vec<ReviewComment>,
    pub checks: Vec<CheckStatus>,
    pub timeline: Vec<super::issue::TimelineEvent>,
    pub commits: Vec<PrCommit>,
}

#[derive(Debug, Clone)]
pub struct Review {
    pub id: u64,
    pub user: User,
    pub state: ReviewState,
    pub body: Option<String>,
    pub submitted_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PrComment {
    pub id: u64,
    pub user: User,
    pub body: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone)]
pub struct ReviewComment {
    pub id: u64,
    pub user: User,
    pub body: String,
    pub path: String,
    pub line: Option<u32>,
    pub original_line: Option<u32>,
    pub diff_hunk: String,
    pub created_at: DateTime<Utc>,
    pub in_reply_to_id: Option<u64>,
}

#[derive(Debug, Clone)]
pub struct CheckStatus {
    pub name: String,
    pub status: String,
    pub conclusion: Option<String>,
    pub url: Option<String>,
}

#[derive(Debug, Clone)]
pub struct DiffFile {
    pub filename: String,
    pub status: DiffFileStatus,
    pub additions: u32,
    pub deletions: u32,
    pub hunks: Vec<DiffHunk>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DiffFileStatus {
    Added,
    Removed,
    Modified,
    Renamed,
}

#[derive(Debug, Clone)]
pub struct DiffHunk {
    pub header: String,
    pub old_start: u32,
    pub new_start: u32,
    pub lines: Vec<DiffLine>,
}

#[derive(Debug, Clone)]
pub struct DiffLine {
    pub kind: DiffLineKind,
    pub content: String,
    pub old_line: Option<u32>,
    pub new_line: Option<u32>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DiffLineKind {
    Context,
    Add,
    Remove,
    Header,
}

#[derive(Debug, Clone, Default, PartialEq)]
pub struct PrFilters {
    pub state: Option<PrState>,
    pub author: Option<String>,
    pub assignee: Option<String>,
    pub label: Option<String>,
    pub sort: Option<String>,
    pub direction: Option<String>,
}

#[derive(Debug, Clone)]
pub struct CreatePrInput {
    pub title: String,
    pub body: String,
    pub head: String,
    pub base: String,
    pub draft: bool,
}

#[derive(Debug, Clone)]
pub struct ReviewInput {
    pub event: ReviewEvent,
    pub body: Option<String>,
    pub comments: Vec<ReviewCommentInput>,
}

#[derive(Debug, Clone, Copy)]
pub enum ReviewEvent {
    Approve,
    RequestChanges,
    Comment,
}

impl ReviewEvent {
    pub fn as_str(&self) -> &str {
        match self {
            ReviewEvent::Approve => "APPROVE",
            ReviewEvent::RequestChanges => "REQUEST_CHANGES",
            ReviewEvent::Comment => "COMMENT",
        }
    }
}

#[derive(Debug, Clone)]
pub struct ReviewCommentInput {
    pub path: String,
    pub line: u32,
    pub body: String,
}
