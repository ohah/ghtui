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
