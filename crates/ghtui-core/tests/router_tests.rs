use ghtui_core::router::*;
use ghtui_core::types::common::RepoId;
use ghtui_core::types::{ActionsFilters, IssueFilters, PrFilters, SearchKind};

fn test_repo() -> RepoId {
    RepoId::new("owner", "repo")
}

// -- Route::title() --

#[test]
fn test_title_dashboard() {
    assert_eq!(Route::Dashboard.title(), "Dashboard");
}

#[test]
fn test_title_code() {
    let route = Route::Code {
        repo: test_repo(),
        path: "src/main.rs".to_string(),
        git_ref: "main".to_string(),
    };
    assert_eq!(route.title(), "owner/repo - src/main.rs");
}

#[test]
fn test_title_issue_list() {
    let route = Route::IssueList {
        repo: test_repo(),
        filters: IssueFilters::default(),
    };
    assert_eq!(route.title(), "owner/repo - Issues");
}

#[test]
fn test_title_issue_detail() {
    let route = Route::IssueDetail {
        repo: test_repo(),
        number: 42,
    };
    assert_eq!(route.title(), "owner/repo - Issue #42");
}

#[test]
fn test_title_pr_list() {
    let route = Route::PrList {
        repo: test_repo(),
        filters: PrFilters::default(),
    };
    assert_eq!(route.title(), "owner/repo - Pull Requests");
}

#[test]
fn test_title_pr_detail() {
    let route = Route::PrDetail {
        repo: test_repo(),
        number: 99,
        tab: PrTab::Conversation,
    };
    assert_eq!(route.title(), "owner/repo - PR #99");
}

#[test]
fn test_title_actions_list() {
    let route = Route::ActionsList {
        repo: test_repo(),
        filters: ActionsFilters::default(),
    };
    assert_eq!(route.title(), "owner/repo - Actions");
}

#[test]
fn test_title_action_detail() {
    let route = Route::ActionDetail {
        repo: test_repo(),
        run_id: 12345,
    };
    assert_eq!(route.title(), "owner/repo - Run #12345");
}

#[test]
fn test_title_job_log() {
    let route = Route::JobLog {
        repo: test_repo(),
        run_id: 100,
        job_id: 200,
    };
    assert_eq!(route.title(), "owner/repo - Job #200");
}

#[test]
fn test_title_security() {
    let route = Route::Security { repo: test_repo() };
    assert_eq!(route.title(), "owner/repo - Security");
}

#[test]
fn test_title_insights() {
    let route = Route::Insights { repo: test_repo() };
    assert_eq!(route.title(), "owner/repo - Insights");
}

#[test]
fn test_title_settings() {
    let route = Route::Settings { repo: test_repo() };
    assert_eq!(route.title(), "owner/repo - Settings");
}

#[test]
fn test_title_notifications() {
    assert_eq!(Route::Notifications.title(), "Notifications");
}

#[test]
fn test_title_search() {
    let route = Route::Search {
        query: "bug fix".to_string(),
        kind: SearchKind::Repos,
    };
    assert!(route.title().contains("bug fix"));
}

#[test]
fn test_title_discussions() {
    let route = Route::Discussions { repo: test_repo() };
    assert_eq!(route.title(), "owner/repo - Discussions");
}

#[test]
fn test_title_gists() {
    assert_eq!(Route::Gists.title(), "Gists");
}

#[test]
fn test_title_organizations() {
    assert_eq!(Route::Organizations.title(), "Organizations");
}

// -- Route::repo() --

#[test]
fn test_repo_returns_some_for_repo_routes() {
    let repo = test_repo();
    let routes_with_repo: Vec<Route> = vec![
        Route::Code {
            repo: repo.clone(),
            path: String::new(),
            git_ref: "main".to_string(),
        },
        Route::IssueList {
            repo: repo.clone(),
            filters: IssueFilters::default(),
        },
        Route::IssueDetail {
            repo: repo.clone(),
            number: 1,
        },
        Route::PrList {
            repo: repo.clone(),
            filters: PrFilters::default(),
        },
        Route::PrDetail {
            repo: repo.clone(),
            number: 1,
            tab: PrTab::Conversation,
        },
        Route::ActionsList {
            repo: repo.clone(),
            filters: ActionsFilters::default(),
        },
        Route::ActionDetail {
            repo: repo.clone(),
            run_id: 1,
        },
        Route::JobLog {
            repo: repo.clone(),
            run_id: 1,
            job_id: 1,
        },
        Route::Security { repo: repo.clone() },
        Route::Insights { repo: repo.clone() },
        Route::Settings { repo: repo.clone() },
        Route::Discussions { repo: repo.clone() },
    ];

    for route in &routes_with_repo {
        assert_eq!(
            route.repo(),
            Some(&repo),
            "Route {:?} should have repo",
            route
        );
    }
}

#[test]
fn test_repo_returns_none_for_non_repo_routes() {
    let routes_without_repo: Vec<Route> = vec![
        Route::Dashboard,
        Route::Notifications,
        Route::Search {
            query: "test".to_string(),
            kind: SearchKind::Repos,
        },
        Route::Gists,
        Route::Organizations,
    ];

    for route in &routes_without_repo {
        assert_eq!(route.repo(), None, "Route {:?} should not have repo", route);
    }
}

// -- Route::tab_index() --

#[test]
fn test_tab_index_code() {
    let repo = test_repo();
    assert_eq!(Route::Dashboard.tab_index(), Some(TAB_CODE));
    assert_eq!(
        Route::Code {
            repo,
            path: String::new(),
            git_ref: "main".to_string(),
        }
        .tab_index(),
        Some(TAB_CODE)
    );
}

#[test]
fn test_tab_index_issues() {
    let repo = test_repo();
    assert_eq!(
        Route::IssueList {
            repo: repo.clone(),
            filters: IssueFilters::default(),
        }
        .tab_index(),
        Some(TAB_ISSUES)
    );
    assert_eq!(
        Route::IssueDetail { repo, number: 1 }.tab_index(),
        Some(TAB_ISSUES)
    );
}

#[test]
fn test_tab_index_prs() {
    let repo = test_repo();
    assert_eq!(
        Route::PrList {
            repo: repo.clone(),
            filters: PrFilters::default(),
        }
        .tab_index(),
        Some(TAB_PRS)
    );
    assert_eq!(
        Route::PrDetail {
            repo,
            number: 1,
            tab: PrTab::Diff,
        }
        .tab_index(),
        Some(TAB_PRS)
    );
}

#[test]
fn test_tab_index_actions() {
    let repo = test_repo();
    assert_eq!(
        Route::ActionsList {
            repo: repo.clone(),
            filters: ActionsFilters::default(),
        }
        .tab_index(),
        Some(TAB_ACTIONS)
    );
    assert_eq!(
        Route::ActionDetail {
            repo: repo.clone(),
            run_id: 1,
        }
        .tab_index(),
        Some(TAB_ACTIONS)
    );
    assert_eq!(
        Route::JobLog {
            repo,
            run_id: 1,
            job_id: 1,
        }
        .tab_index(),
        Some(TAB_ACTIONS)
    );
}

#[test]
fn test_tab_index_other_tabs() {
    let repo = test_repo();
    assert_eq!(
        Route::Security { repo: repo.clone() }.tab_index(),
        Some(TAB_SECURITY)
    );
    assert_eq!(
        Route::Insights { repo: repo.clone() }.tab_index(),
        Some(TAB_INSIGHTS)
    );
    assert_eq!(Route::Settings { repo }.tab_index(), Some(TAB_SETTINGS));
}

#[test]
fn test_tab_index_none_for_non_tab_routes() {
    assert_eq!(Route::Notifications.tab_index(), None);
    assert_eq!(
        Route::Search {
            query: String::new(),
            kind: SearchKind::Code,
        }
        .tab_index(),
        None
    );
    assert_eq!(Route::Discussions { repo: test_repo() }.tab_index(), None);
    assert_eq!(Route::Gists.tab_index(), None);
    assert_eq!(Route::Organizations.tab_index(), None);
}

// -- Constants --

#[test]
fn test_tab_constants() {
    assert_eq!(TAB_CODE, 0);
    assert_eq!(TAB_ISSUES, 1);
    assert_eq!(TAB_PRS, 2);
    assert_eq!(TAB_ACTIONS, 3);
    assert_eq!(TAB_SECURITY, 4);
    assert_eq!(TAB_INSIGHTS, 5);
    assert_eq!(TAB_SETTINGS, 6);
}

#[test]
fn test_tab_labels_count() {
    assert_eq!(TAB_LABELS.len(), 7);
    assert_eq!(TAB_LABELS[TAB_CODE], "Code");
    assert_eq!(TAB_LABELS[TAB_ISSUES], "Issues");
    assert_eq!(TAB_LABELS[TAB_PRS], "Pull requests");
    assert_eq!(TAB_LABELS[TAB_ACTIONS], "Actions");
    assert_eq!(TAB_LABELS[TAB_SECURITY], "Security");
    assert_eq!(TAB_LABELS[TAB_INSIGHTS], "Insights");
    assert_eq!(TAB_LABELS[TAB_SETTINGS], "Settings");
}

// -- PrTab --

#[test]
fn test_pr_tab_default() {
    let tab = PrTab::default();
    assert_eq!(tab, PrTab::Conversation);
}

#[test]
fn test_pr_tab_variants() {
    // Ensure all variants exist and are distinct
    assert_ne!(PrTab::Conversation, PrTab::Diff);
    assert_ne!(PrTab::Diff, PrTab::Checks);
    assert_ne!(PrTab::Conversation, PrTab::Checks);
}

// -- Route::default() --

#[test]
fn test_route_default() {
    let route = Route::default();
    assert_eq!(route, Route::Dashboard);
}
