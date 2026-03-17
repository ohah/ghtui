use ghtui_core::config::AppConfig;
use ghtui_core::router::Route;
use ghtui_core::state::*;
use ghtui_core::types::common::RepoId;

#[test]
fn test_app_state_new() {
    let config = AppConfig::default();
    let repo = RepoId::new("owner", "repo");
    let state = AppState::new(config, Some(repo.clone()));

    assert_eq!(state.route, Route::Dashboard);
    assert!(state.route_history.is_empty());
    assert_eq!(state.current_repo, Some(repo));
    assert!(state.loading.is_empty());
    assert!(state.toasts.is_empty());
    assert!(state.modal.is_none());
    assert_eq!(state.input_mode, InputMode::Normal);
}

#[test]
fn test_navigation() {
    let config = AppConfig::default();
    let repo = RepoId::new("owner", "repo");
    let mut state = AppState::new(config, Some(repo.clone()));

    // Navigate to PR list
    let pr_route = Route::PrList {
        repo: repo.clone(),
        filters: Default::default(),
    };
    state.navigate(pr_route.clone());

    assert!(matches!(state.route, Route::PrList { .. }));
    assert_eq!(state.route_history.len(), 1);
    assert_eq!(state.route_history[0], Route::Dashboard);

    // Navigate to PR detail
    let detail_route = Route::PrDetail {
        repo: repo.clone(),
        number: 42,
        tab: ghtui_core::PrTab::Conversation,
    };
    state.navigate(detail_route);

    assert!(matches!(state.route, Route::PrDetail { .. }));
    assert_eq!(state.route_history.len(), 2);

    // Go back
    assert!(state.go_back());
    assert!(matches!(state.route, Route::PrList { .. }));
    assert_eq!(state.route_history.len(), 1);

    // Go back again
    assert!(state.go_back());
    assert_eq!(state.route, Route::Dashboard);
    assert!(state.route_history.is_empty());

    // Can't go back further
    assert!(!state.go_back());
}

#[test]
fn test_toast_lifecycle() {
    let config = AppConfig::default();
    let mut state = AppState::new(config, None);

    assert!(state.toasts.is_empty());

    state.push_toast("Hello".to_string(), ToastLevel::Info);
    assert_eq!(state.toasts.len(), 1);
    assert_eq!(state.toasts[0].ttl, 5);

    // Tick down
    state.tick_toasts();
    assert_eq!(state.toasts[0].ttl, 4);

    // Tick down until removed
    for _ in 0..4 {
        state.tick_toasts();
    }
    assert!(state.toasts.is_empty());
}

#[test]
fn test_loading_state() {
    let config = AppConfig::default();
    let mut state = AppState::new(config, None);

    assert!(!state.is_loading("pr_list"));

    state.loading.insert("pr_list".to_string());
    assert!(state.is_loading("pr_list"));
    assert!(!state.is_loading("issue_list"));

    state.loading.remove("pr_list");
    assert!(!state.is_loading("pr_list"));
}

#[test]
fn test_pr_list_state_selection() {
    use ghtui_core::types::*;
    use chrono::Utc;

    let prs = vec![
        PullRequest {
            number: 1,
            title: "First PR".to_string(),
            state: PrState::Open,
            user: common::User { login: "user1".into(), avatar_url: "".into(), name: None },
            body: None,
            created_at: Utc::now(),
            updated_at: Utc::now(),
            merged_at: None,
            closed_at: None,
            head_ref: "feature".into(),
            base_ref: "main".into(),
            draft: false,
            labels: vec![],
            assignees: vec![],
            milestone: None,
            requested_reviewers: vec![],
            additions: None,
            deletions: None,
            changed_files: None,
            mergeable: None,
            comments: None,
            review_comments: None,
        },
        PullRequest {
            number: 2,
            title: "Second PR".to_string(),
            state: PrState::Open,
            user: common::User { login: "user2".into(), avatar_url: "".into(), name: None },
            body: None,
            created_at: Utc::now(),
            updated_at: Utc::now(),
            merged_at: None,
            closed_at: None,
            head_ref: "fix".into(),
            base_ref: "main".into(),
            draft: true,
            labels: vec![],
            assignees: vec![],
            milestone: None,
            requested_reviewers: vec![],
            additions: None,
            deletions: None,
            changed_files: None,
            mergeable: None,
            comments: None,
            review_comments: None,
        },
    ];

    let pagination = Pagination::default();
    let mut list = PrListState::new(prs, pagination);

    assert_eq!(list.selected, 0);
    assert_eq!(list.selected_pr().unwrap().number, 1);

    list.select_next();
    assert_eq!(list.selected, 1);
    assert_eq!(list.selected_pr().unwrap().number, 2);

    // Can't go past end
    list.select_next();
    assert_eq!(list.selected, 1);

    list.select_prev();
    assert_eq!(list.selected, 0);

    // Can't go before start
    list.select_prev();
    assert_eq!(list.selected, 0);
}
