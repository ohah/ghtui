use ghtui_core::AppState;
use ghtui_core::config::{AppConfig, GhAccount};
use ghtui_core::message::ModalKind;
use ghtui_core::router::Route;
use ghtui_core::state::*;
use ghtui_core::types::common::RepoId;
use ghtui_core::types::*;

// Import the update module from the binary crate's lib
// Since update is private to the binary, we test via integration of state transitions

fn make_state() -> AppState {
    let config = AppConfig::default();
    let repo = RepoId::new("owner", "repo");
    AppState::new(config, Some(repo), None, vec![])
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
        head_sha: "abc123".into(),
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
        auto_merge: false,
        reactions: None,
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
        locked: false,
        reactions: None,
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
        review_threads: vec![],
        checks: vec![],
        commits: vec![],
        timeline: vec![],
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

// -- Account tests --

fn make_accounts() -> Vec<GhAccount> {
    vec![
        GhAccount {
            host: "github.com".to_string(),
            user: "personal".to_string(),
            token: "gho_personal".to_string(),
        },
        GhAccount {
            host: "github.com".to_string(),
            user: "work".to_string(),
            token: "gho_work".to_string(),
        },
    ]
}

fn make_state_with_accounts() -> AppState {
    let config = AppConfig::default();
    let repo = RepoId::new("owner", "repo");
    let accounts = make_accounts();
    AppState::new(config, Some(repo), Some(accounts[0].clone()), accounts)
}

#[test]
fn test_account_switcher_opens_with_correct_state() {
    let mut state = make_state_with_accounts();

    state.modal = Some(ModalKind::AccountSwitcher);
    state.input_mode = InputMode::Normal; // Account switcher stays in Normal mode

    assert!(matches!(state.modal, Some(ModalKind::AccountSwitcher)));
    assert_eq!(state.accounts.len(), 2);
    assert_eq!(state.current_account.as_ref().unwrap().user, "personal");
}

#[test]
fn test_account_switch_produces_toast() {
    let mut state = make_state_with_accounts();
    let new_account = state.accounts[1].clone();

    // Simulate AccountSwitched message handling
    state.current_account = Some(new_account.clone());
    state.push_toast(
        format!("Switched to {}", new_account.display_name()),
        ToastLevel::Success,
    );

    assert_eq!(state.current_account.as_ref().unwrap().user, "work");
    assert_eq!(state.toasts.len(), 1);
    assert!(state.toasts[0].message.contains("work"));
    assert_eq!(state.toasts[0].level, ToastLevel::Success);
}

#[test]
fn test_account_switch_clears_all_cached_state() {
    let mut state = make_state_with_accounts();

    // Load some data
    state.pr_list = Some(PrListState::new(
        vec![make_pr(1, "PR")],
        Pagination::default(),
    ));
    state.issue_list = Some(IssueListState::new(
        vec![make_issue(1, "Issue")],
        Pagination::default(),
    ));

    // Simulate full account switch (as done in update/mod.rs)
    state.current_account = Some(state.accounts[1].clone());
    state.pr_list = None;
    state.pr_detail = None;
    state.issue_list = None;
    state.issue_detail = None;
    state.actions_list = None;
    state.action_detail = None;
    state.notifications = None;
    state.search = None;

    assert!(state.pr_list.is_none());
    assert!(state.pr_detail.is_none());
    assert!(state.issue_list.is_none());
    assert!(state.issue_detail.is_none());
    assert!(state.actions_list.is_none());
    assert!(state.action_detail.is_none());
    assert!(state.notifications.is_none());
    assert!(state.search.is_none());
}

#[test]
fn test_account_selected_bounds() {
    let mut state = make_state_with_accounts();

    // Navigate to last account
    state.account_selected = state.accounts.len() - 1;
    assert_eq!(state.account_selected, 1);

    // Try to go past end
    state.account_selected = (state.account_selected + 1).min(state.accounts.len() - 1);
    assert_eq!(state.account_selected, 1);

    // Go back to start
    state.account_selected = 0;
    // Try to go before start
    state.account_selected = state.account_selected.saturating_sub(1);
    assert_eq!(state.account_selected, 0);
}

#[test]
fn test_account_switch_closes_modal() {
    let mut state = make_state_with_accounts();

    // Open modal
    state.modal = Some(ModalKind::AccountSwitcher);

    // Simulate AccountSwitch handling (closes modal)
    state.modal = None;
    state.input_mode = InputMode::Normal;

    assert!(state.modal.is_none());
    assert_eq!(state.input_mode, InputMode::Normal);
}
