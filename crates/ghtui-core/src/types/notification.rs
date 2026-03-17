use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Notification {
    pub id: String,
    pub unread: bool,
    pub reason: String,
    pub updated_at: DateTime<Utc>,
    pub subject: NotificationSubject,
    pub repository: NotificationRepo,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NotificationSubject {
    pub title: String,
    #[serde(rename = "type")]
    pub subject_type: String,
    pub url: Option<String>,
    pub latest_comment_url: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NotificationRepo {
    pub full_name: String,
}

#[derive(Debug, Clone, Default)]
pub struct NotificationFilters {
    pub all: bool,
    pub participating: bool,
    pub reason: Option<String>,
    pub subject_type: Option<String>,
}

impl NotificationFilters {
    pub fn cycle_reason(&mut self) {
        self.reason = match self.reason.as_deref() {
            None => Some("review_requested".to_string()),
            Some("review_requested") => Some("assign".to_string()),
            Some("assign") => Some("mention".to_string()),
            Some("mention") => Some("subscribed".to_string()),
            Some("subscribed") => Some("ci_activity".to_string()),
            Some("ci_activity") => None,
            _ => None,
        };
    }

    pub fn reason_display(&self) -> &str {
        match self.reason.as_deref() {
            None => "All reasons",
            Some("review_requested") => "Review requested",
            Some("assign") => "Assigned",
            Some("mention") => "Mentioned",
            Some("subscribed") => "Subscribed",
            Some("ci_activity") => "CI Activity",
            Some(_) => "All reasons",
        }
    }

    pub fn cycle_type(&mut self) {
        self.subject_type = match self.subject_type.as_deref() {
            None => Some("PullRequest".to_string()),
            Some("PullRequest") => Some("Issue".to_string()),
            Some("Issue") => Some("Release".to_string()),
            Some("Release") => None,
            _ => None,
        };
    }

    pub fn type_display(&self) -> &str {
        match self.subject_type.as_deref() {
            None => "All types",
            Some("PullRequest") => "PRs",
            Some("Issue") => "Issues",
            Some("Release") => "Releases",
            Some(_) => "All types",
        }
    }

    pub fn has_active_filters(&self) -> bool {
        self.reason.is_some() || self.subject_type.is_some()
    }
}

impl Notification {
    /// Extract the issue/PR number from the subject URL.
    pub fn extract_number(&self) -> Option<u64> {
        self.subject
            .url
            .as_deref()
            .and_then(|url| url.rsplit('/').next())
            .and_then(|s| s.parse().ok())
    }

    /// Extract owner/repo from repository.full_name.
    pub fn repo_parts(&self) -> Option<(&str, &str)> {
        let parts: Vec<&str> = self.repository.full_name.splitn(2, '/').collect();
        if parts.len() == 2 {
            Some((parts[0], parts[1]))
        } else {
            None
        }
    }
}
