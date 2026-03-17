use ghtui_core::router::Route;
use ghtui_core::state::*;
use ghtui_core::{AppState, Command, Message};

pub fn update(state: &mut AppState, msg: Message) -> Vec<Command> {
    match msg {
        // Navigation
        Message::Navigate(route) => handle_navigate(state, route),
        Message::Back => {
            state.go_back();
            vec![]
        }

        // PR
        Message::PrListLoaded(prs, pagination) => {
            state.loading.remove("pr_list");
            state.pr_list = Some(PrListState::new(prs, pagination));
            vec![]
        }
        Message::PrDetailLoaded(detail) => {
            state.loading.remove("pr_detail");
            state.pr_detail = Some(PrDetailState::new(*detail));
            vec![]
        }
        Message::PrDiffLoaded(files) => {
            state.loading.remove("pr_diff");
            if let Some(ref mut detail) = state.pr_detail {
                detail.diff = Some(files);
            }
            vec![]
        }
        Message::PrMerged(number) => {
            state.push_toast(format!("PR #{} merged!", number), ToastLevel::Success);
            // Refresh the list
            refresh_current_view(state)
        }
        Message::PrClosed(number) => {
            state.push_toast(format!("PR #{} closed", number), ToastLevel::Info);
            refresh_current_view(state)
        }
        Message::PrReopened(number) => {
            state.push_toast(format!("PR #{} reopened", number), ToastLevel::Info);
            refresh_current_view(state)
        }
        Message::PrCreated(number) => {
            state.push_toast(format!("PR #{} created!", number), ToastLevel::Success);
            refresh_current_view(state)
        }
        Message::ReviewSubmitted => {
            state.push_toast("Review submitted".to_string(), ToastLevel::Success);
            refresh_current_view(state)
        }

        // Issues
        Message::IssueListLoaded(issues, pagination) => {
            state.loading.remove("issue_list");
            state.issue_list = Some(IssueListState::new(issues, pagination));
            vec![]
        }
        Message::IssueDetailLoaded(detail) => {
            state.loading.remove("issue_detail");
            state.issue_detail = Some(IssueDetailState::new(*detail));
            vec![]
        }
        Message::IssueClosed(number) => {
            state.push_toast(format!("Issue #{} closed", number), ToastLevel::Info);
            refresh_current_view(state)
        }
        Message::IssueReopened(number) => {
            state.push_toast(format!("Issue #{} reopened", number), ToastLevel::Info);
            refresh_current_view(state)
        }
        Message::IssueCreated(number) => {
            state.push_toast(format!("Issue #{} created!", number), ToastLevel::Success);
            refresh_current_view(state)
        }
        Message::CommentAdded => {
            state.push_toast("Comment added".to_string(), ToastLevel::Success);
            state.input_buffer.clear();
            state.input_mode = InputMode::Normal;
            state.modal = None;
            refresh_current_view(state)
        }

        // Actions
        Message::RunsLoaded(runs, pagination) => {
            state.loading.remove("actions_list");
            state.actions_list = Some(ActionsListState::new(runs, pagination));
            vec![]
        }
        Message::RunDetailLoaded(detail) => {
            state.loading.remove("action_detail");
            state.action_detail = Some(ActionDetailState::new(*detail));
            vec![]
        }
        Message::JobLogLoaded(job_id, lines) => {
            state.loading.remove("job_log");
            if let Some(ref mut detail) = state.action_detail {
                let _ = job_id;
                detail.log = Some(lines);
            }
            vec![]
        }
        Message::RunCancelled(run_id) => {
            state.push_toast(format!("Run #{} cancelled", run_id), ToastLevel::Info);
            refresh_current_view(state)
        }
        Message::RunRerun(run_id) => {
            state.push_toast(format!("Run #{} restarted", run_id), ToastLevel::Success);
            refresh_current_view(state)
        }

        // Notifications
        Message::NotificationsLoaded(notifications) => {
            state.loading.remove("notifications");
            state.notifications = Some(NotificationListState::new(notifications));
            vec![]
        }
        Message::NotificationMarkedRead(id) => {
            if let Some(ref mut notifs) = state.notifications {
                notifs.items.retain(|n| n.id != id);
            }
            vec![]
        }

        // Search
        Message::SearchResults(results) => {
            state.loading.remove("search");
            if let Some(ref mut search) = state.search {
                search.results = Some(results);
            }
            vec![]
        }

        // UI
        Message::InputChanged(text) => {
            if text == "\x08" {
                state.input_buffer.pop();
            } else {
                state.input_buffer.push_str(&text);
            }
            vec![]
        }
        Message::ListSelect(delta) => {
            handle_list_select(state, delta);
            vec![]
        }
        Message::TabChanged(delta) => {
            if let Some(ref mut detail) = state.pr_detail {
                if delta == usize::MAX {
                    detail.tab = detail.tab.saturating_sub(1);
                } else {
                    detail.tab = (detail.tab + delta).min(2);
                }
            }
            vec![]
        }
        Message::GlobalTabNext => {
            let total = ghtui_core::router::TAB_LABELS.len();
            state.active_tab = (state.active_tab + 1) % total;
            navigate_to_tab(state)
        }
        Message::GlobalTabPrev => {
            let total = ghtui_core::router::TAB_LABELS.len();
            state.active_tab = (state.active_tab + total - 1) % total;
            navigate_to_tab(state)
        }
        Message::GlobalTabSelect(idx) => {
            if idx < ghtui_core::router::TAB_LABELS.len() {
                state.active_tab = idx;
                navigate_to_tab(state)
            } else {
                vec![]
            }
        }
        Message::ToggleTheme => {
            state.toggle_theme();
            vec![]
        }
        Message::ModalOpen(kind) => {
            state.modal = Some(kind);
            state.input_mode = InputMode::Insert;
            vec![]
        }
        Message::ModalClose => {
            state.modal = None;
            state.input_mode = InputMode::Normal;
            vec![]
        }
        Message::Tick => {
            state.tick_toasts();
            vec![]
        }
        Message::Resize(w, h) => {
            state.terminal_size = (w, h);
            vec![]
        }

        // System
        Message::Error(e) => {
            state.loading.clear();
            state.push_toast(e.to_string(), ToastLevel::Error);
            vec![]
        }
        Message::Quit => vec![Command::Quit],
    }
}

fn handle_navigate(state: &mut AppState, route: Route) -> Vec<Command> {
    let cmds = match &route {
        Route::PrList { repo, filters } => {
            state.loading.insert("pr_list".to_string());
            vec![Command::FetchPrList(repo.clone(), filters.clone(), 1)]
        }
        Route::PrDetail { repo, number, .. } => {
            state.loading.insert("pr_detail".to_string());
            state.loading.insert("pr_diff".to_string());
            vec![
                Command::FetchPrDetail(repo.clone(), *number),
                Command::FetchPrDiff(repo.clone(), *number),
            ]
        }
        Route::IssueList { repo, filters } => {
            state.loading.insert("issue_list".to_string());
            vec![Command::FetchIssueList(repo.clone(), filters.clone(), 1)]
        }
        Route::IssueDetail { repo, number } => {
            state.loading.insert("issue_detail".to_string());
            vec![Command::FetchIssueDetail(repo.clone(), *number)]
        }
        Route::ActionsList { repo, filters } => {
            state.loading.insert("actions_list".to_string());
            vec![Command::FetchRuns(repo.clone(), filters.clone(), 1)]
        }
        Route::ActionDetail { repo, run_id } => {
            state.loading.insert("action_detail".to_string());
            vec![Command::FetchRunDetail(repo.clone(), *run_id)]
        }
        Route::JobLog {
            repo,
            run_id,
            job_id,
        } => {
            state.loading.insert("job_log".to_string());
            vec![Command::FetchJobLog(repo.clone(), *run_id, *job_id)]
        }
        Route::Notifications => {
            state.loading.insert("notifications".to_string());
            vec![Command::FetchNotifications(
                ghtui_core::types::NotificationFilters::default(),
            )]
        }
        Route::Search { query, kind } => {
            if !query.is_empty() {
                state.loading.insert("search".to_string());
                state.search = Some(ghtui_core::state::SearchViewState::new(
                    query.clone(),
                    *kind,
                ));
                vec![Command::Search(query.clone(), *kind, 1)]
            } else {
                state.search = Some(ghtui_core::state::SearchViewState::new(
                    String::new(),
                    *kind,
                ));
                vec![]
            }
        }
        Route::Code { .. } => vec![],
        Route::Projects { .. } => vec![],
        Route::Wiki { .. } => vec![],
        Route::Security { .. } => vec![],
        Route::Insights { .. } => vec![],
        Route::Settings { .. } => vec![],
        Route::Dashboard => vec![],
    };

    // Sync active_tab with route
    if let Some(idx) = route.tab_index() {
        state.active_tab = idx;
    }
    state.navigate(route);
    cmds
}

fn navigate_to_tab(state: &mut AppState) -> Vec<Command> {
    use ghtui_core::router::*;
    use ghtui_core::types::*;

    let Some(ref repo) = state.current_repo else {
        return vec![];
    };
    let repo = repo.clone();

    let route = match state.active_tab {
        TAB_CODE => Route::Code {
            repo,
            path: "/".to_string(),
            git_ref: "main".to_string(),
        },
        TAB_ISSUES => Route::IssueList {
            repo,
            filters: IssueFilters::default(),
        },
        TAB_PRS => Route::PrList {
            repo,
            filters: PrFilters::default(),
        },
        TAB_ACTIONS => Route::ActionsList {
            repo,
            filters: ActionsFilters::default(),
        },
        TAB_PROJECTS => Route::Projects { repo },
        TAB_WIKI => Route::Wiki { repo },
        TAB_SECURITY => Route::Security { repo },
        TAB_INSIGHTS => Route::Insights { repo },
        TAB_SETTINGS => Route::Settings { repo },
        _ => return vec![],
    };

    handle_navigate(state, route)
}

fn handle_list_select(state: &mut AppState, delta: usize) {
    match &state.route {
        Route::PrList { .. } => {
            if let Some(ref mut list) = state.pr_list {
                if delta == 0 {
                    // Open selected
                    if let Some(pr) = list.selected_pr() {
                        let repo = state.current_repo.clone().unwrap();
                        let number = pr.number;
                        let route = Route::PrDetail {
                            repo,
                            number,
                            tab: ghtui_core::PrTab::Conversation,
                        };
                        state.navigate(route);
                    }
                } else if delta == usize::MAX {
                    list.select_prev();
                } else {
                    list.select_next();
                }
            }
        }
        Route::IssueList { .. } => {
            if let Some(ref mut list) = state.issue_list {
                if delta == 0 {
                    if let Some(issue) = list.selected_issue() {
                        let repo = state.current_repo.clone().unwrap();
                        let number = issue.number;
                        let route = Route::IssueDetail { repo, number };
                        state.navigate(route);
                    }
                } else if delta == usize::MAX {
                    list.select_prev();
                } else {
                    list.select_next();
                }
            }
        }
        Route::ActionsList { .. } => {
            if let Some(ref mut list) = state.actions_list {
                if delta == 0 {
                    if let Some(run) = list.selected_run() {
                        let repo = state.current_repo.clone().unwrap();
                        let run_id = run.id;
                        let route = Route::ActionDetail { repo, run_id };
                        state.navigate(route);
                    }
                } else if delta == usize::MAX {
                    list.select_prev();
                } else {
                    list.select_next();
                }
            }
        }
        Route::Notifications => {
            if let Some(ref mut list) = state.notifications {
                if delta == usize::MAX {
                    list.select_prev();
                } else if delta > 0 {
                    list.select_next();
                }
            }
        }
        _ => {}
    }
}

fn refresh_current_view(state: &mut AppState) -> Vec<Command> {
    match &state.route {
        Route::PrList { repo, filters } => {
            state.loading.insert("pr_list".to_string());
            vec![Command::FetchPrList(repo.clone(), filters.clone(), 1)]
        }
        Route::PrDetail { repo, number, .. } => {
            state.loading.insert("pr_detail".to_string());
            vec![Command::FetchPrDetail(repo.clone(), *number)]
        }
        Route::IssueList { repo, filters } => {
            state.loading.insert("issue_list".to_string());
            vec![Command::FetchIssueList(repo.clone(), filters.clone(), 1)]
        }
        Route::IssueDetail { repo, number } => {
            state.loading.insert("issue_detail".to_string());
            vec![Command::FetchIssueDetail(repo.clone(), *number)]
        }
        Route::ActionsList { repo, filters } => {
            state.loading.insert("actions_list".to_string());
            vec![Command::FetchRuns(repo.clone(), filters.clone(), 1)]
        }
        _ => vec![],
    }
}
