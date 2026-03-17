use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use super::common::User;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum RunStatus {
    Queued,
    InProgress,
    Completed,
    Waiting,
    Requested,
    Pending,
}

impl std::fmt::Display for RunStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            RunStatus::Queued => write!(f, "queued"),
            RunStatus::InProgress => write!(f, "in_progress"),
            RunStatus::Completed => write!(f, "completed"),
            RunStatus::Waiting => write!(f, "waiting"),
            RunStatus::Requested => write!(f, "requested"),
            RunStatus::Pending => write!(f, "pending"),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum RunConclusion {
    Success,
    Failure,
    Cancelled,
    Skipped,
    TimedOut,
    ActionRequired,
    Neutral,
    Stale,
    StartupFailure,
}

impl std::fmt::Display for RunConclusion {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            RunConclusion::Success => write!(f, "success"),
            RunConclusion::Failure => write!(f, "failure"),
            RunConclusion::Cancelled => write!(f, "cancelled"),
            RunConclusion::Skipped => write!(f, "skipped"),
            RunConclusion::TimedOut => write!(f, "timed_out"),
            RunConclusion::ActionRequired => write!(f, "action_required"),
            RunConclusion::Neutral => write!(f, "neutral"),
            RunConclusion::Stale => write!(f, "stale"),
            RunConclusion::StartupFailure => write!(f, "startup_failure"),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkflowRun {
    pub id: u64,
    pub name: Option<String>,
    pub head_branch: Option<String>,
    pub head_sha: String,
    pub status: Option<RunStatus>,
    pub conclusion: Option<RunConclusion>,
    pub workflow_id: u64,
    pub run_number: u64,
    pub event: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub actor: Option<User>,
    pub html_url: String,
}

#[derive(Debug, Clone)]
pub struct WorkflowRunDetail {
    pub run: WorkflowRun,
    pub jobs: Vec<Job>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Job {
    pub id: u64,
    pub name: String,
    pub status: Option<RunStatus>,
    pub conclusion: Option<RunConclusion>,
    pub started_at: Option<DateTime<Utc>>,
    pub completed_at: Option<DateTime<Utc>>,
    pub steps: Vec<JobStep>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JobStep {
    pub name: String,
    pub status: String,
    pub conclusion: Option<String>,
    pub number: u32,
    pub started_at: Option<DateTime<Utc>>,
    pub completed_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone)]
pub struct LogLine {
    pub content: String,
    pub timestamp: Option<String>,
}

#[derive(Debug, Clone, Default, PartialEq)]
pub struct ActionsFilters {
    pub status: Option<String>,
    pub branch: Option<String>,
    pub event: Option<String>,
    pub actor: Option<String>,
    pub workflow_id: Option<u64>,
}
