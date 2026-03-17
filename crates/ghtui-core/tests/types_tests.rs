use ghtui_core::types::common::RepoId;

#[test]
fn test_repo_id_new() {
    let repo = RepoId::new("owner", "repo");
    assert_eq!(repo.owner, "owner");
    assert_eq!(repo.name, "repo");
    assert_eq!(repo.full_name(), "owner/repo");
}

#[test]
fn test_repo_id_display() {
    let repo = RepoId::new("ohah", "ghtui");
    assert_eq!(format!("{}", repo), "ohah/ghtui");
}

#[test]
fn test_repo_id_from_str() {
    let repo: RepoId = "owner/repo".parse().unwrap();
    assert_eq!(repo.owner, "owner");
    assert_eq!(repo.name, "repo");
}

#[test]
fn test_repo_id_from_str_invalid() {
    let result = "invalid".parse::<RepoId>();
    assert!(result.is_err());

    let result = "a/b/c".parse::<RepoId>();
    assert!(result.is_err());
}

#[test]
fn test_repo_id_equality() {
    let a = RepoId::new("owner", "repo");
    let b = RepoId::new("owner", "repo");
    let c = RepoId::new("other", "repo");

    assert_eq!(a, b);
    assert_ne!(a, c);
}

#[test]
fn test_pr_state_display() {
    use ghtui_core::types::PrState;
    assert_eq!(format!("{}", PrState::Open), "open");
    assert_eq!(format!("{}", PrState::Closed), "closed");
    assert_eq!(format!("{}", PrState::Merged), "merged");
}

#[test]
fn test_issue_state_display() {
    use ghtui_core::types::IssueState;
    assert_eq!(format!("{}", IssueState::Open), "open");
    assert_eq!(format!("{}", IssueState::Closed), "closed");
}

#[test]
fn test_merge_method_as_str() {
    use ghtui_core::types::MergeMethod;
    assert_eq!(MergeMethod::Merge.as_str(), "merge");
    assert_eq!(MergeMethod::Squash.as_str(), "squash");
    assert_eq!(MergeMethod::Rebase.as_str(), "rebase");
}

#[test]
fn test_review_event_as_str() {
    use ghtui_core::types::ReviewEvent;
    assert_eq!(ReviewEvent::Approve.as_str(), "APPROVE");
    assert_eq!(ReviewEvent::RequestChanges.as_str(), "REQUEST_CHANGES");
    assert_eq!(ReviewEvent::Comment.as_str(), "COMMENT");
}

#[test]
fn test_config_defaults() {
    use ghtui_core::config::AppConfig;
    let config = AppConfig::default();
    assert_eq!(config.per_page, 30);
    assert_eq!(config.tick_rate_ms, 1000);
    assert!(config.token.is_none());
    assert!(config.default_repo.is_none());
}

#[test]
fn test_pr_filters_default() {
    use ghtui_core::types::PrFilters;
    let filters = PrFilters::default();
    assert!(filters.state.is_none());
    assert!(filters.author.is_none());
    assert!(filters.label.is_none());
}

#[test]
fn test_run_status_display() {
    use ghtui_core::types::{RunConclusion, RunStatus};
    assert_eq!(format!("{}", RunStatus::InProgress), "in_progress");
    assert_eq!(format!("{}", RunConclusion::Success), "success");
    assert_eq!(format!("{}", RunConclusion::Failure), "failure");
}

#[test]
fn test_search_kind() {
    use ghtui_core::types::SearchKind;
    assert_ne!(SearchKind::Repos, SearchKind::Issues);
    assert_ne!(SearchKind::Issues, SearchKind::Code);
}

// -- ActionsFilters tests --

#[test]
fn test_actions_filters_default() {
    use ghtui_core::types::ActionsFilters;
    let filters = ActionsFilters::default();
    assert!(filters.status.is_none());
    assert!(filters.branch.is_none());
    assert!(filters.event.is_none());
    assert!(filters.actor.is_none());
    assert!(filters.workflow_id.is_none());
    assert!(!filters.has_active_filters());
}

#[test]
fn test_actions_filters_cycle_status() {
    use ghtui_core::types::ActionsFilters;
    let mut filters = ActionsFilters::default();

    assert_eq!(filters.status_display(), "All");

    filters.cycle_status();
    assert_eq!(filters.status.as_deref(), Some("completed"));
    assert_eq!(filters.status_display(), "Completed");

    filters.cycle_status();
    assert_eq!(filters.status.as_deref(), Some("in_progress"));
    assert_eq!(filters.status_display(), "In progress");

    filters.cycle_status();
    assert_eq!(filters.status.as_deref(), Some("queued"));
    assert_eq!(filters.status_display(), "Queued");

    filters.cycle_status();
    assert_eq!(filters.status.as_deref(), Some("failure"));
    assert_eq!(filters.status_display(), "Failure");

    filters.cycle_status();
    assert_eq!(filters.status.as_deref(), Some("success"));
    assert_eq!(filters.status_display(), "Success");

    // Cycle back to None
    filters.cycle_status();
    assert!(filters.status.is_none());
    assert_eq!(filters.status_display(), "All");
}

#[test]
fn test_actions_filters_cycle_event() {
    use ghtui_core::types::ActionsFilters;
    let mut filters = ActionsFilters::default();

    assert_eq!(filters.event_display(), "All events");

    filters.cycle_event();
    assert_eq!(filters.event.as_deref(), Some("push"));
    assert_eq!(filters.event_display(), "push");

    filters.cycle_event();
    assert_eq!(filters.event.as_deref(), Some("pull_request"));

    filters.cycle_event();
    assert_eq!(filters.event.as_deref(), Some("schedule"));

    filters.cycle_event();
    assert_eq!(filters.event.as_deref(), Some("workflow_dispatch"));

    // Cycle back to None
    filters.cycle_event();
    assert!(filters.event.is_none());
}

#[test]
fn test_actions_filters_has_active() {
    use ghtui_core::types::ActionsFilters;
    let mut filters = ActionsFilters::default();
    assert!(!filters.has_active_filters());

    filters.status = Some("completed".to_string());
    assert!(filters.has_active_filters());

    filters.status = None;
    filters.branch = Some("main".to_string());
    assert!(filters.has_active_filters());

    filters.branch = None;
    filters.workflow_id = Some(123);
    assert!(filters.has_active_filters());
}

// -- NotificationFilters tests --

#[test]
fn test_notification_filters_default() {
    use ghtui_core::types::NotificationFilters;
    let filters = NotificationFilters::default();
    assert!(filters.reason.is_none());
    assert!(filters.subject_type.is_none());
    assert!(!filters.has_active_filters());
}

#[test]
fn test_notification_filters_cycle_reason() {
    use ghtui_core::types::NotificationFilters;
    let mut filters = NotificationFilters::default();

    filters.cycle_reason();
    assert_eq!(filters.reason.as_deref(), Some("review_requested"));
    assert_eq!(filters.reason_display(), "Review requested");

    filters.cycle_reason();
    assert_eq!(filters.reason.as_deref(), Some("assign"));

    filters.cycle_reason();
    assert_eq!(filters.reason.as_deref(), Some("mention"));

    filters.cycle_reason();
    assert_eq!(filters.reason.as_deref(), Some("subscribed"));

    filters.cycle_reason();
    assert_eq!(filters.reason.as_deref(), Some("ci_activity"));

    filters.cycle_reason();
    assert!(filters.reason.is_none());
}

#[test]
fn test_notification_filters_cycle_type() {
    use ghtui_core::types::NotificationFilters;
    let mut filters = NotificationFilters::default();

    filters.cycle_type();
    assert_eq!(filters.subject_type.as_deref(), Some("PullRequest"));
    assert_eq!(filters.type_display(), "PRs");

    filters.cycle_type();
    assert_eq!(filters.subject_type.as_deref(), Some("Issue"));

    filters.cycle_type();
    assert_eq!(filters.subject_type.as_deref(), Some("Release"));

    filters.cycle_type();
    assert!(filters.subject_type.is_none());
}

#[test]
fn test_notification_extract_number() {
    use chrono::Utc;
    use ghtui_core::types::*;

    let notif = Notification {
        id: "1".to_string(),
        unread: true,
        reason: "review_requested".to_string(),
        updated_at: Utc::now(),
        subject: NotificationSubject {
            title: "Fix bug".to_string(),
            subject_type: "PullRequest".to_string(),
            url: Some("https://api.github.com/repos/owner/repo/pulls/42".to_string()),
            latest_comment_url: None,
        },
        repository: NotificationRepo {
            full_name: "owner/repo".to_string(),
        },
    };

    assert_eq!(notif.extract_number(), Some(42));
    assert_eq!(notif.repo_parts(), Some(("owner", "repo")));
}

#[test]
fn test_notification_extract_number_none() {
    use chrono::Utc;
    use ghtui_core::types::*;

    let notif = Notification {
        id: "1".to_string(),
        unread: false,
        reason: "subscribed".to_string(),
        updated_at: Utc::now(),
        subject: NotificationSubject {
            title: "Release v1.0".to_string(),
            subject_type: "Release".to_string(),
            url: None,
            latest_comment_url: None,
        },
        repository: NotificationRepo {
            full_name: "owner/repo".to_string(),
        },
    };

    assert_eq!(notif.extract_number(), None);
}
