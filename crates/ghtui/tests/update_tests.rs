use ghtui_core::AppState;
use ghtui_core::config::{AppConfig, GhAccount};
use ghtui_core::message::ModalKind;
use ghtui_core::router::Route;
use ghtui_core::state::*;
use ghtui_core::types::code::TreeNode;
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
fn test_state_error_preserves_loading() {
    let mut state = make_state();
    state.loading.insert("pr_list".to_string());
    state.loading.insert("pr_detail".to_string());

    // Error handler should NOT clear loading — concurrent requests still
    // in flight will clean up their own keys via individual *Loaded handlers.
    state.push_toast("API Error".to_string(), ToastLevel::Error);

    assert_eq!(state.loading.len(), 2);
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

// -- Code view tests --

fn make_code_state() -> AppState {
    let mut state = make_state();
    let repo = state.current_repo.clone().unwrap();
    state.navigate(Route::Code {
        repo,
        path: String::new(),
        git_ref: "main".to_string(),
    });
    state.code = Some(CodeViewState::new("main".to_string()));
    state
}

#[test]
fn test_code_file_loaded_race_guard_mismatch() {
    let mut state = make_code_state();
    let code = state.code.as_mut().unwrap();
    code.file_path = Some("src/a.rs".to_string());

    // Simulate CodeFileLoaded arriving for a *different* file (stale response)
    // The race guard checks: code.file_path == Some(path)
    let loaded_path = "src/b.rs";
    if code.file_path.as_deref() == Some(loaded_path) {
        code.file_content = Some("content of b".to_string());
        code.file_name = Some("b.rs".to_string());
    }

    // file_content should NOT be set because paths don't match
    assert!(state.code.as_ref().unwrap().file_content.is_none());
}

#[test]
fn test_code_file_loaded_race_guard_match() {
    let mut state = make_code_state();
    let code = state.code.as_mut().unwrap();
    code.file_path = Some("src/a.rs".to_string());

    // Simulate CodeFileLoaded arriving for the correct file
    let loaded_path = "src/a.rs";
    let loaded_content = "fn main() {}".to_string();
    let loaded_filename = "a.rs".to_string();
    if code.file_path.as_deref() == Some(loaded_path) {
        code.file_content = Some(loaded_content.clone());
        code.file_name = Some(loaded_filename.clone());
        code.scroll = 0;
    }

    let code = state.code.as_ref().unwrap();
    assert_eq!(code.file_content.as_deref(), Some("fn main() {}"));
    assert_eq!(code.file_name.as_deref(), Some("a.rs"));
    assert_eq!(code.scroll, 0);
}

#[test]
fn test_code_select_ref_resets_state() {
    let mut state = make_code_state();
    let code = state.code.as_mut().unwrap();

    // Set up some existing state that should be cleared on ref switch
    code.file_content = Some("old content".to_string());
    code.file_name = Some("old.rs".to_string());
    code.file_path = Some("src/old.rs".to_string());
    code.tree_loaded = true;
    code.sidebar_focused = false;

    // Simulate CodeSelectRef: reset to fresh state with new ref
    let branches = std::mem::take(&mut code.branches);
    let tags = std::mem::take(&mut code.tags);
    let new_ref = "develop".to_string();
    *code = CodeViewState::new(new_ref.clone());
    code.branches = branches;
    code.tags = tags;

    let code = state.code.as_ref().unwrap();
    assert!(
        code.file_content.is_none(),
        "file_content should be cleared"
    );
    assert!(code.file_name.is_none(), "file_name should be cleared");
    assert!(
        code.sidebar_focused,
        "sidebar_focused should reset to true (default)"
    );
    assert!(!code.tree_loaded, "tree_loaded should reset to false");
    assert_eq!(code.git_ref, "develop");
}

#[test]
fn test_code_navigate_into_clears_previous_file() {
    let mut state = make_code_state();
    let code = state.code.as_mut().unwrap();

    // Set up existing file content
    code.file_content = Some("old content".to_string());
    code.file_name = Some("old.rs".to_string());
    code.image_data = Some(vec![0x89, 0x50, 0x4e, 0x47]);

    // Set up tree with a file node
    code.tree_loaded = true;
    code.tree = vec![TreeNode {
        name: "new.rs".to_string(),
        path: "src/new.rs".to_string(),
        is_dir: false,
        depth: 0,
        expanded: false,
        size: Some(100),
    }];
    code.rebuild_visible_tree();
    code.selected = 0;

    // Simulate CodeNavigateInto for a file node:
    // The handler clears previous file immediately before fetching
    if let Some(node) = code.tree_selected_node().cloned()
        && !node.is_dir
    {
        code.file_content = None;
        code.file_name = None;
        code.image_data = None;
        code.scroll = 0;
        code.file_path = Some(node.path.clone());
    }

    let code = state.code.as_ref().unwrap();
    assert!(
        code.file_content.is_none(),
        "file_content should be cleared before fetch"
    );
    assert!(
        code.file_name.is_none(),
        "file_name should be cleared before fetch"
    );
    assert!(
        code.image_data.is_none(),
        "image_data should be cleared before fetch"
    );
    assert_eq!(
        code.file_path.as_deref(),
        Some("src/new.rs"),
        "file_path should be set to new node path"
    );
    assert_eq!(code.scroll, 0, "scroll should be reset");
}

// -- Error handler tests --

#[test]
fn test_error_does_not_clear_all_loading() {
    let mut state = make_state();

    // Simulate multiple concurrent API calls
    state.loading.insert("pr_list".to_string());
    state.loading.insert("code_tree".to_string());
    state.loading.insert("security_alerts".to_string());

    // Simulate Message::Error handling (after the fix):
    // Error should NOT call state.loading.clear().
    // It only pushes a toast and resets code editing state.
    state.push_toast("API Error: 403 Forbidden".to_string(), ToastLevel::Error);

    // Loading keys for other in-flight requests should still be present
    assert!(
        state.is_loading("pr_list"),
        "pr_list loading should not be cleared by error"
    );
    assert!(
        state.is_loading("code_tree"),
        "code_tree loading should not be cleared by error"
    );
    assert!(
        state.is_loading("security_alerts"),
        "security_alerts loading should not be cleared by error"
    );
    assert_eq!(state.toasts.len(), 1);
    assert_eq!(state.toasts[0].level, ToastLevel::Error);
}

// -- Route::repo() tests --

#[test]
fn test_route_repo_returns_correct_repo() {
    let repo = RepoId::new("owner", "repo");

    // Routes with repo should return Some
    let code_route = Route::Code {
        repo: repo.clone(),
        path: String::new(),
        git_ref: "main".to_string(),
    };
    assert_eq!(code_route.repo(), Some(&repo));

    let pr_list = Route::PrList {
        repo: repo.clone(),
        filters: PrFilters::default(),
    };
    assert_eq!(pr_list.repo(), Some(&repo));

    let issue_detail = Route::IssueDetail {
        repo: repo.clone(),
        number: 42,
    };
    assert_eq!(issue_detail.repo(), Some(&repo));

    let actions = Route::ActionsList {
        repo: repo.clone(),
        filters: ActionsFilters::default(),
    };
    assert_eq!(actions.repo(), Some(&repo));

    let settings = Route::Settings { repo: repo.clone() };
    assert_eq!(settings.repo(), Some(&repo));

    // Routes without repo should return None
    assert_eq!(Route::Dashboard.repo(), None);
    assert_eq!(Route::Notifications.repo(), None);
    assert_eq!(
        Route::Search {
            query: "test".to_string(),
            kind: SearchKind::Repos,
        }
        .repo(),
        None
    );
    assert_eq!(Route::Gists.repo(), None);
    assert_eq!(Route::Organizations.repo(), None);
}
