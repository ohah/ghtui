use ghtui_core::config::AppConfig;
use ghtui_core::message::ModalKind;
use ghtui_core::router::Route;
use ghtui_core::state::*;
use ghtui_core::types::common::RepoId;
use ghtui_core::types::*;
use ghtui_core::{AppState, Command, Message};

// Import the update module from the binary crate's lib
// Since update is private to the binary, we test via integration of state transitions

fn make_state() -> AppState {
    let config = AppConfig::default();
    let repo = RepoId::new("owner", "repo");
    AppState::new(config, Some(repo))
}

fn make_pr(number: u64, title: &str) -> PullRequest {
    PullRequest {
        number,
        title: title.to_string(),
        state: PrState::Open,
        user: common::User {
            login: "test".into(),
            avatar_url: "".into(),
            name: None,
        },
        body: None,
        created_at: chrono::Utc::now(),
        updated_at: chrono::Utc::now(),
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
    }
}

fn make_issue(number: u64, title: &str) -> Issue {
    Issue {
        number,
        title: title.to_string(),
        state: IssueState::Open,
        user: common::User {
            login: "test".into(),
            avatar_url: "".into(),
            name: None,
        },
        body: None,
        created_at: chrono::Utc::now(),
        updated_at: chrono::Utc::now(),
        closed_at: None,
        labels: vec![],
        assignees: vec![],
        milestone: None,
        comments: None,
    }
}

#[test]
fn test_state_navigation_to_pr_list() {
    let mut state = make_state();
    let repo = state.current_repo.clone().unwrap();

    state.navigate(Route::PrList {
        repo,
        filters: PrFilters::default(),
    });

    assert!(matches!(state.route, Route::PrList { .. }));
    assert_eq!(state.route_history.len(), 1);
}

#[test]
fn test_state_pr_list_loaded() {
    let mut state = make_state();
    state.loading.insert("pr_list".to_string());

    let prs = vec![make_pr(1, "First"), make_pr(2, "Second")];
    state.pr_list = Some(PrListState::new(prs, Pagination::default()));
    state.loading.remove("pr_list");

    assert!(!state.is_loading("pr_list"));
    assert!(state.pr_list.is_some());
    assert_eq!(state.pr_list.as_ref().unwrap().items.len(), 2);
}

#[test]
fn test_state_issue_list_loaded() {
    let mut state = make_state();
    state.loading.insert("issue_list".to_string());

    let issues = vec![make_issue(1, "Bug"), make_issue(2, "Feature")];
    state.issue_list = Some(IssueListState::new(issues, Pagination::default()));
    state.loading.remove("issue_list");

    assert!(!state.is_loading("issue_list"));
    assert_eq!(state.issue_list.as_ref().unwrap().items.len(), 2);
}

#[test]
fn test_state_toast_on_success() {
    let mut state = make_state();
    state.push_toast("PR #42 merged!".to_string(), ToastLevel::Success);

    assert_eq!(state.toasts.len(), 1);
    assert_eq!(state.toasts[0].level, ToastLevel::Success);
    assert!(state.toasts[0].message.contains("42"));
}

#[test]
fn test_state_error_clears_loading() {
    let mut state = make_state();
    state.loading.insert("pr_list".to_string());
    state.loading.insert("pr_detail".to_string());

    // Simulating error handling
    state.loading.clear();
    state.push_toast("API Error".to_string(), ToastLevel::Error);

    assert!(state.loading.is_empty());
    assert_eq!(state.toasts.len(), 1);
    assert_eq!(state.toasts[0].level, ToastLevel::Error);
}

#[test]
fn test_state_modal_open_close() {
    let mut state = make_state();

    assert!(state.modal.is_none());
    assert_eq!(state.input_mode, InputMode::Normal);

    // Open modal
    state.modal = Some(ModalKind::AddComment);
    state.input_mode = InputMode::Insert;

    assert!(state.modal.is_some());
    assert_eq!(state.input_mode, InputMode::Insert);

    // Close modal
    state.modal = None;
    state.input_mode = InputMode::Normal;

    assert!(state.modal.is_none());
    assert_eq!(state.input_mode, InputMode::Normal);
}

#[test]
fn test_state_input_buffer() {
    let mut state = make_state();

    assert!(state.input_buffer.is_empty());

    state.input_buffer.push_str("Hello");
    assert_eq!(state.input_buffer, "Hello");

    state.input_buffer.push_str(" World");
    assert_eq!(state.input_buffer, "Hello World");

    state.input_buffer.pop();
    assert_eq!(state.input_buffer, "Hello Worl");

    state.input_buffer.clear();
    assert!(state.input_buffer.is_empty());
}

#[test]
fn test_pr_detail_tab_switching() {
    let detail = PullRequestDetail {
        pr: make_pr(1, "Test"),
        reviews: vec![],
        comments: vec![],
        review_comments: vec![],
        checks: vec![],
    };

    let mut detail_state = PrDetailState::new(detail);
    assert_eq!(detail_state.tab, 0);

    detail_state.tab = 1;
    assert_eq!(detail_state.tab, 1);

    detail_state.tab = 2;
    assert_eq!(detail_state.tab, 2);
}

#[test]
fn test_actions_list_selection() {
    use chrono::Utc;

    let runs = vec![
        WorkflowRun {
            id: 1,
            name: Some("CI".to_string()),
            head_branch: Some("main".to_string()),
            head_sha: "abc123".to_string(),
            status: Some(RunStatus::Completed),
            conclusion: Some(RunConclusion::Success),
            workflow_id: 1,
            run_number: 100,
            event: "push".to_string(),
            created_at: Utc::now(),
            updated_at: Utc::now(),
            actor: None,
            html_url: "".to_string(),
        },
        WorkflowRun {
            id: 2,
            name: Some("Deploy".to_string()),
            head_branch: Some("main".to_string()),
            head_sha: "def456".to_string(),
            status: Some(RunStatus::InProgress),
            conclusion: None,
            workflow_id: 2,
            run_number: 101,
            event: "push".to_string(),
            created_at: Utc::now(),
            updated_at: Utc::now(),
            actor: None,
            html_url: "".to_string(),
        },
    ];

    let mut list = ActionsListState::new(runs, Pagination::default());
    assert_eq!(list.selected, 0);
    assert_eq!(list.selected_run().unwrap().id, 1);

    list.select_next();
    assert_eq!(list.selected_run().unwrap().id, 2);
}

#[test]
fn test_notification_list_selection() {
    use chrono::Utc;

    let notifs = vec![Notification {
        id: "1".to_string(),
        unread: true,
        reason: "review_requested".to_string(),
        updated_at: Utc::now(),
        subject: NotificationSubject {
            title: "Fix bug".to_string(),
            subject_type: "PullRequest".to_string(),
            url: None,
            latest_comment_url: None,
        },
        repository: NotificationRepo {
            full_name: "owner/repo".to_string(),
        },
    }];

    let mut list = NotificationListState::new(notifs);
    assert_eq!(list.selected, 0);
    assert!(list.selected_notification().is_some());

    // Can't go further with only 1 item
    list.select_next();
    assert_eq!(list.selected, 0);
}

#[test]
fn test_multiple_toasts() {
    let mut state = make_state();

    state.push_toast("First".to_string(), ToastLevel::Info);
    state.push_toast("Second".to_string(), ToastLevel::Success);
    state.push_toast("Third".to_string(), ToastLevel::Error);

    assert_eq!(state.toasts.len(), 3);

    // Tick 5 times to expire all
    for _ in 0..5 {
        state.tick_toasts();
    }
    assert!(state.toasts.is_empty());
}
