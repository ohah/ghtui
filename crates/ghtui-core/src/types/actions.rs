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

impl ActionsFilters {
    /// Cycle status filter: None → completed → in_progress → queued → failure → success → None
    pub fn cycle_status(&mut self) {
        self.status = match self.status.as_deref() {
            None => Some("completed".to_string()),
            Some("completed") => Some("in_progress".to_string()),
            Some("in_progress") => Some("queued".to_string()),
            Some("queued") => Some("failure".to_string()),
            Some("failure") => Some("success".to_string()),
            Some("success") => None,
            _ => None,
        };
    }

    pub fn status_display(&self) -> &str {
        match self.status.as_deref() {
            None => "All",
            Some("completed") => "Completed",
            Some("in_progress") => "In progress",
            Some("queued") => "Queued",
            Some("failure") => "Failure",
            Some("success") => "Success",
            _ => "All",
        }
    }

    /// Cycle event filter: None → push → pull_request → schedule → workflow_dispatch → None
    pub fn cycle_event(&mut self) {
        self.event = match self.event.as_deref() {
            None => Some("push".to_string()),
            Some("push") => Some("pull_request".to_string()),
            Some("pull_request") => Some("schedule".to_string()),
            Some("schedule") => Some("workflow_dispatch".to_string()),
            Some("workflow_dispatch") => None,
            _ => None,
        };
    }

    pub fn event_display(&self) -> &str {
        match self.event.as_deref() {
            None => "All events",
            Some("push") => "push",
            Some("pull_request") => "pull_request",
            Some("schedule") => "schedule",
            Some("workflow_dispatch") => "workflow_dispatch",
            Some(_) => "All events",
        }
    }

    pub fn has_active_filters(&self) -> bool {
        self.status.is_some()
            || self.branch.is_some()
            || self.event.is_some()
            || self.actor.is_some()
            || self.workflow_id.is_some()
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Workflow {
    pub id: u64,
    pub name: String,
    pub path: String,
    pub state: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Artifact {
    pub id: u64,
    pub name: String,
    pub size_in_bytes: u64,
    pub archive_download_url: String,
    pub expired: bool,
    pub created_at: Option<DateTime<Utc>>,
    pub expires_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PendingDeployment {
    pub id: u64,
    pub environment: PendingEnvironment,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PendingEnvironment {
    pub id: u64,
    pub name: String,
}
