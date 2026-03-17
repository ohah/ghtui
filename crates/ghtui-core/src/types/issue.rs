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

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct Reactions {
    #[serde(rename = "+1", default)]
    pub plus_one: u32,
    #[serde(rename = "-1", default)]
    pub minus_one: u32,
    #[serde(default)]
    pub laugh: u32,
    #[serde(default)]
    pub hooray: u32,
    #[serde(default)]
    pub confused: u32,
    #[serde(default)]
    pub heart: u32,
    #[serde(default)]
    pub rocket: u32,
    #[serde(default)]
    pub eyes: u32,
}

impl Reactions {
    pub fn total(&self) -> u32 {
        self.plus_one
            + self.minus_one
            + self.laugh
            + self.hooray
            + self.confused
            + self.heart
            + self.rocket
            + self.eyes
    }

    pub fn summary(&self) -> String {
        let mut parts = Vec::new();
        if self.plus_one > 0 {
            parts.push(format!("👍{}", self.plus_one));
        }
        if self.minus_one > 0 {
            parts.push(format!("👎{}", self.minus_one));
        }
        if self.laugh > 0 {
            parts.push(format!("😄{}", self.laugh));
        }
        if self.hooray > 0 {
            parts.push(format!("🎉{}", self.hooray));
        }
        if self.confused > 0 {
            parts.push(format!("😕{}", self.confused));
        }
        if self.heart > 0 {
            parts.push(format!("❤️{}", self.heart));
        }
        if self.rocket > 0 {
            parts.push(format!("🚀{}", self.rocket));
        }
        if self.eyes > 0 {
            parts.push(format!("👀{}", self.eyes));
        }
        parts.join(" ")
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
    #[serde(default)]
    pub locked: bool,
    #[serde(default)]
    pub reactions: Option<Reactions>,
}

#[derive(Debug, Clone)]
pub struct IssueDetail {
    pub issue: Issue,
    pub comments: Vec<IssueComment>,
    pub timeline: Vec<TimelineEvent>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IssueComment {
    pub id: u64,
    pub user: User,
    pub body: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    #[serde(default)]
    pub reactions: Option<Reactions>,
}

/// Timeline event types from GitHub Timeline API
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TimelineEvent {
    pub event: String,
    pub created_at: Option<DateTime<Utc>>,
    pub actor: Option<User>,
    pub label: Option<TimelineLabel>,
    pub assignee: Option<User>,
    pub milestone: Option<TimelineMilestone>,
    pub rename: Option<TimelineRename>,
    pub source: Option<TimelineSource>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TimelineLabel {
    pub name: String,
    pub color: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TimelineMilestone {
    pub title: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TimelineRename {
    pub from: String,
    pub to: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TimelineSource {
    pub issue: Option<TimelineSourceIssue>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TimelineSourceIssue {
    pub number: u64,
    pub title: String,
}

impl TimelineEvent {
    pub fn display(&self) -> String {
        let actor = self
            .actor
            .as_ref()
            .map(|a| a.login.as_str())
            .unwrap_or("someone");

        match self.event.as_str() {
            "labeled" => {
                let label = self.label.as_ref().map(|l| l.name.as_str()).unwrap_or("?");
                format!("{} added label '{}'", actor, label)
            }
            "unlabeled" => {
                let label = self.label.as_ref().map(|l| l.name.as_str()).unwrap_or("?");
                format!("{} removed label '{}'", actor, label)
            }
            "assigned" => {
                let assignee = self
                    .assignee
                    .as_ref()
                    .map(|a| a.login.as_str())
                    .unwrap_or("?");
                format!("{} assigned {}", actor, assignee)
            }
            "unassigned" => {
                let assignee = self
                    .assignee
                    .as_ref()
                    .map(|a| a.login.as_str())
                    .unwrap_or("?");
                format!("{} unassigned {}", actor, assignee)
            }
            "milestoned" => {
                let ms = self
                    .milestone
                    .as_ref()
                    .map(|m| m.title.as_str())
                    .unwrap_or("?");
                format!("{} added milestone '{}'", actor, ms)
            }
            "demilestoned" => {
                let ms = self
                    .milestone
                    .as_ref()
                    .map(|m| m.title.as_str())
                    .unwrap_or("?");
                format!("{} removed milestone '{}'", actor, ms)
            }
            "renamed" => {
                if let Some(ref rename) = self.rename {
                    format!(
                        "{} changed title: '{}' → '{}'",
                        actor, rename.from, rename.to
                    )
                } else {
                    format!("{} renamed the issue", actor)
                }
            }
            "closed" => format!("{} closed this", actor),
            "reopened" => format!("{} reopened this", actor),
            "locked" => format!("{} locked this", actor),
            "unlocked" => format!("{} unlocked this", actor),
            "cross-referenced" => {
                if let Some(ref source) = self.source {
                    if let Some(ref issue) = source.issue {
                        format!(
                            "{} referenced this in #{} {}",
                            actor, issue.number, issue.title
                        )
                    } else {
                        format!("{} cross-referenced this", actor)
                    }
                } else {
                    format!("{} cross-referenced this", actor)
                }
            }
            "referenced" => format!("{} referenced this", actor),
            "mentioned" => format!("{} was mentioned", actor),
            "subscribed" => format!("{} subscribed", actor),
            other => format!("{} {} this", actor, other),
        }
    }

    pub fn icon(&self) -> &'static str {
        match self.event.as_str() {
            "labeled" | "unlabeled" => "🏷️",
            "assigned" | "unassigned" => "👤",
            "milestoned" | "demilestoned" => "📌",
            "renamed" => "✏️",
            "closed" => "🔒",
            "reopened" => "🔓",
            "locked" | "unlocked" => "🔐",
            "cross-referenced" | "referenced" => "🔗",
            "mentioned" => "💬",
            _ => "•",
        }
    }
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_reactions_summary() {
        let r = Reactions {
            plus_one: 3,
            heart: 1,
            rocket: 2,
            ..Default::default()
        };
        assert_eq!(r.summary(), "👍3 ❤️1 🚀2");
        assert_eq!(r.total(), 6);
    }

    #[test]
    fn test_reactions_empty() {
        let r = Reactions::default();
        assert_eq!(r.summary(), "");
        assert_eq!(r.total(), 0);
    }

    #[test]
    fn test_timeline_event_display() {
        let event = TimelineEvent {
            event: "labeled".to_string(),
            created_at: None,
            actor: Some(User {
                login: "octocat".to_string(),
                avatar_url: String::new(),
                name: None,
            }),
            label: Some(TimelineLabel {
                name: "bug".to_string(),
                color: None,
            }),
            assignee: None,
            milestone: None,
            rename: None,
            source: None,
        };
        assert_eq!(event.display(), "octocat added label 'bug'");
        assert_eq!(event.icon(), "🏷️");
    }

    #[test]
    fn test_timeline_closed() {
        let event = TimelineEvent {
            event: "closed".to_string(),
            created_at: None,
            actor: Some(User {
                login: "admin".to_string(),
                avatar_url: String::new(),
                name: None,
            }),
            label: None,
            assignee: None,
            milestone: None,
            rename: None,
            source: None,
        };
        assert_eq!(event.display(), "admin closed this");
        assert_eq!(event.icon(), "🔒");
    }

    #[test]
    fn test_timeline_renamed() {
        let event = TimelineEvent {
            event: "renamed".to_string(),
            created_at: None,
            actor: Some(User {
                login: "dev".to_string(),
                avatar_url: String::new(),
                name: None,
            }),
            label: None,
            assignee: None,
            milestone: None,
            rename: Some(TimelineRename {
                from: "old title".to_string(),
                to: "new title".to_string(),
            }),
            source: None,
        };
        assert!(event.display().contains("old title"));
        assert!(event.display().contains("new title"));
    }
}
