use ghtui_core::router::Route;
use ghtui_core::state::issue::InlineEditTarget;
use ghtui_core::state::*;
use ghtui_core::types::{IssueFilters, IssueState};
use ghtui_core::{AppState, Command, Message};

pub fn update(state: &mut AppState, msg: Message) -> Vec<Command> {
    match msg {
        // Navigation
        Message::Navigate(route) => handle_navigate(state, route),
        Message::Back => {
            state.go_back();
            vec![]
        }

        // Account
        Message::AccountSwitch(account) => {
            state.modal = None;
            state.input_mode = InputMode::Normal;
            vec![Command::SwitchAccount(account)]
        }
        Message::AccountSwitched(account) => {
            state.push_toast(
                format!("Switched to {}", account.display_name()),
                ToastLevel::Success,
            );
            state.current_account = Some(account);
            // Clear cached data
            state.pr_list = None;
            state.pr_detail = None;
            state.issue_list = None;
            state.issue_detail = None;
            state.actions_list = None;
            state.action_detail = None;
            state.notifications = None;
            state.search = None;
            state.insights = None;
            state.security = None;
            state.settings = None;
            // Refresh current view
            refresh_current_view(state)
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
        Message::IssueListLoaded(issues, pagination, filters) => {
            state.loading.remove("issue_list");
            state.issue_list = Some(IssueListState::with_filters(issues, pagination, filters));
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
        Message::IssueUpdated(number) => {
            state.push_toast(format!("Issue #{} updated", number), ToastLevel::Success);
            refresh_current_view(state)
        }
        Message::CommentAdded => {
            state.push_toast("Comment added".to_string(), ToastLevel::Success);
            state.input_buffer.clear();
            state.input_mode = InputMode::Normal;
            state.modal = None;
            refresh_current_view(state)
        }
        Message::CommentUpdated => {
            state.push_toast("Comment updated".to_string(), ToastLevel::Success);
            state.input_buffer.clear();
            state.input_mode = InputMode::Normal;
            state.modal = None;
            refresh_current_view(state)
        }
        Message::IssueToggleStateFilter => {
            if let Some(ref repo) = state.current_repo {
                let filters = if let Some(ref mut list) = state.issue_list {
                    list.toggle_state_filter();
                    list.filters.clone()
                } else {
                    IssueFilters {
                        state: Some(IssueState::Closed),
                        ..Default::default()
                    }
                };
                state.loading.insert("issue_list".to_string());
                vec![Command::FetchIssueList(repo.clone(), filters, 1)]
            } else {
                vec![]
            }
        }
        Message::IssueNextPage => {
            if let (Some(repo), Some(list)) = (&state.current_repo, &state.issue_list) {
                if list.pagination.has_next {
                    let next_page = list.pagination.page + 1;
                    let filters = list.filters.clone();
                    state.loading.insert("issue_list".to_string());
                    vec![Command::FetchIssueList(repo.clone(), filters, next_page)]
                } else {
                    vec![]
                }
            } else {
                vec![]
            }
        }
        Message::IssuePrevPage => {
            if let (Some(repo), Some(list)) = (&state.current_repo, &state.issue_list) {
                if list.pagination.page > 1 {
                    let prev_page = list.pagination.page - 1;
                    let filters = list.filters.clone();
                    state.loading.insert("issue_list".to_string());
                    vec![Command::FetchIssueList(repo.clone(), filters, prev_page)]
                } else {
                    vec![]
                }
            } else {
                vec![]
            }
        }

        // Inline editing
        Message::IssueStartEditTitle => {
            if let Some(ref mut detail) = state.issue_detail {
                detail.start_edit_title();
            }
            vec![]
        }
        Message::IssueStartEditBody => {
            if let Some(ref mut detail) = state.issue_detail {
                match detail.selected_comment {
                    None => detail.start_edit_body(),
                    Some(idx) => detail.start_edit_comment(idx),
                }
            }
            vec![]
        }
        Message::IssueStartComment => {
            if let Some(ref mut detail) = state.issue_detail {
                detail.start_new_comment();
            }
            vec![]
        }
        Message::IssueStartReply => {
            if let Some(ref mut detail) = state.issue_detail {
                if let Some(idx) = detail.selected_comment {
                    detail.start_quote_reply(idx);
                } else {
                    detail.start_new_comment();
                }
            }
            vec![]
        }
        Message::IssueEditChar(c) => {
            if let Some(ref mut detail) = state.issue_detail {
                detail.editor.insert_char(c);
            }
            vec![]
        }
        Message::IssueEditNewline => {
            if let Some(ref mut detail) = state.issue_detail {
                detail.editor.insert_newline();
            }
            vec![]
        }
        Message::IssueEditBackspace => {
            if let Some(ref mut detail) = state.issue_detail {
                detail.editor.backspace();
            }
            vec![]
        }
        Message::IssueEditCursorLeft => {
            if let Some(ref mut detail) = state.issue_detail {
                detail.editor.move_left();
            }
            vec![]
        }
        Message::IssueEditCursorRight => {
            if let Some(ref mut detail) = state.issue_detail {
                detail.editor.move_right();
            }
            vec![]
        }
        Message::IssueEditCursorUp => {
            if let Some(ref mut detail) = state.issue_detail {
                detail.editor.move_up();
            }
            vec![]
        }
        Message::IssueEditCursorDown => {
            if let Some(ref mut detail) = state.issue_detail {
                detail.editor.move_down();
            }
            vec![]
        }
        Message::IssueEditHome => {
            if let Some(ref mut detail) = state.issue_detail {
                detail.editor.move_home();
            }
            vec![]
        }
        Message::IssueEditEnd => {
            if let Some(ref mut detail) = state.issue_detail {
                detail.editor.move_end();
            }
            vec![]
        }
        Message::IssueEditSubmit => {
            if let Some(ref detail) = state.issue_detail {
                if let Some(ref repo) = state.current_repo {
                    let cmds = match &detail.edit_target {
                        Some(InlineEditTarget::IssueTitle) => {
                            let title = detail.editor_text().trim().to_string();
                            let number = detail.detail.issue.number;
                            if title.is_empty() {
                                vec![]
                            } else {
                                vec![Command::UpdateIssue(
                                    repo.clone(),
                                    number,
                                    Some(title),
                                    None,
                                )]
                            }
                        }
                        Some(InlineEditTarget::IssueBody) => {
                            let body = detail.editor_text().clone();
                            let number = detail.detail.issue.number;
                            vec![Command::UpdateIssue(repo.clone(), number, None, Some(body))]
                        }
                        Some(InlineEditTarget::Comment(idx)) => {
                            if let Some(comment) = detail.detail.comments.get(*idx) {
                                let body = detail.editor_text().clone();
                                if body.trim().is_empty() {
                                    vec![]
                                } else {
                                    vec![Command::UpdateComment(
                                        repo.clone(),
                                        detail.detail.issue.number,
                                        comment.id,
                                        body,
                                    )]
                                }
                            } else {
                                vec![]
                            }
                        }
                        Some(InlineEditTarget::NewComment | InlineEditTarget::QuoteReply(_)) => {
                            let body = detail.editor_text().clone();
                            if body.trim().is_empty() {
                                vec![]
                            } else {
                                let number = detail.detail.issue.number;
                                vec![Command::AddComment(repo.clone(), number, body)]
                            }
                        }
                        None => vec![],
                    };
                    // Clear editing state
                    if let Some(ref mut detail) = state.issue_detail {
                        detail.cancel_edit();
                    }
                    return cmds;
                }
            }
            if let Some(ref mut detail) = state.issue_detail {
                detail.cancel_edit();
            }
            vec![]
        }
        Message::IssueEditCancel => {
            if let Some(ref mut detail) = state.issue_detail {
                detail.cancel_edit();
            }
            vec![]
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

        // Insights
        Message::ContributorStatsLoaded(stats) => {
            state.loading.remove("insights");
            state.loading.remove("contributors");
            if state.insights.is_none() {
                state.insights = Some(InsightsState::default());
            }
            if let Some(ref mut ins) = state.insights {
                ins.contributors = stats;
            }
            vec![]
        }
        Message::CommitActivityLoaded(activity) => {
            state.loading.remove("commit_activity");
            if let Some(ref mut ins) = state.insights {
                ins.commit_activity = activity;
            }
            vec![]
        }
        Message::TrafficClonesLoaded(clones) => {
            state.loading.remove("traffic_clones");
            if let Some(ref mut ins) = state.insights {
                ins.traffic_clones = Some(clones);
            }
            vec![]
        }
        Message::TrafficViewsLoaded(views) => {
            state.loading.remove("traffic_views");
            if let Some(ref mut ins) = state.insights {
                ins.traffic_views = Some(views);
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

        // Security
        Message::DependabotAlertsLoaded(alerts) => {
            state.loading.remove("security");
            state.loading.remove("dependabot");
            if let Some(ref mut sec) = state.security {
                sec.dependabot_alerts = alerts;
            }
            vec![]
        }
        Message::CodeScanningAlertsLoaded(alerts) => {
            state.loading.remove("code_scanning");
            if let Some(ref mut sec) = state.security {
                sec.code_scanning_alerts = alerts;
            }
            vec![]
        }
        Message::SecretScanningAlertsLoaded(alerts) => {
            state.loading.remove("secret_scanning");
            if let Some(ref mut sec) = state.security {
                sec.secret_scanning_alerts = alerts;
            }
            vec![]
        }

        // Settings
        Message::SettingsRepoLoaded(repo) => {
            state.loading.remove("settings");
            state.settings = Some(SettingsState::new(*repo));
            // Also fetch branch protections and collaborators
            if let Some(ref repo_id) = state.current_repo {
                state.loading.insert("branch_protections".to_string());
                state.loading.insert("collaborators".to_string());
                vec![
                    Command::FetchBranchProtections(repo_id.clone()),
                    Command::FetchCollaborators(repo_id.clone()),
                ]
            } else {
                vec![]
            }
        }
        Message::SettingsBranchProtectionsLoaded(protections) => {
            state.loading.remove("branch_protections");
            if let Some(ref mut settings) = state.settings {
                settings.branch_protections = protections;
            }
            vec![]
        }
        Message::SettingsCollaboratorsLoaded(collaborators) => {
            state.loading.remove("collaborators");
            if let Some(ref mut settings) = state.settings {
                settings.collaborators = collaborators;
            }
            vec![]
        }

        // Mouse click
        Message::MouseClick(_col, row) => {
            // Row 0 = repo header, Row 1 = tab bar, Row 2+ = content
            if row == 1 {
                // Tab bar click — compute which tab was clicked based on column position
                // Tab labels with spacing: " N Label " format
                let mut x: u16 = 0;
                for (i, label) in ghtui_core::router::TAB_LABELS.iter().enumerate() {
                    let key_width = 3u16; // " N "
                    let label_width = label.len() as u16;
                    let sep_width = if i < ghtui_core::router::TAB_LABELS.len() - 1 {
                        1
                    } else {
                        0
                    };
                    let tab_end = x + key_width + label_width + sep_width;
                    if _col >= x && _col < tab_end {
                        state.active_tab = i;
                        return navigate_to_tab(state);
                    }
                    x = tab_end;
                }
                vec![]
            } else if row >= 2 {
                // Content area click — select list item by row offset
                let content_row = (row - 2) as usize;
                // Find which item is at this row (accounting for border)
                if content_row > 0 {
                    let item_index = content_row - 1; // -1 for top border
                    handle_mouse_list_select(state, item_index)
                } else {
                    vec![]
                }
            } else {
                vec![]
            }
        }

        // Scroll — context-aware
        Message::ScrollUp => {
            if matches!(state.route, Route::ActionDetail { .. }) {
                if let Some(ref mut detail) = state.action_detail {
                    if detail.log.is_some() {
                        detail.log_scroll = detail.log_scroll.saturating_sub(3);
                        return vec![];
                    }
                }
            }
            if matches!(state.route, Route::IssueDetail { .. }) {
                if let Some(ref mut detail) = state.issue_detail {
                    detail.scroll = detail.scroll.saturating_sub(3);
                    return vec![];
                }
            }
            update(state, Message::ListSelect(usize::MAX))
        }
        Message::ScrollDown => {
            if matches!(state.route, Route::ActionDetail { .. }) {
                if let Some(ref mut detail) = state.action_detail {
                    if detail.log.is_some() {
                        detail.log_scroll += 3;
                        return vec![];
                    }
                }
            }
            if matches!(state.route, Route::IssueDetail { .. }) {
                if let Some(ref mut detail) = state.issue_detail {
                    detail.scroll += 3;
                    return vec![];
                }
            }
            update(state, Message::ListSelect(1))
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
            // Handle account switcher navigation
            if matches!(state.modal, Some(ghtui_core::ModalKind::AccountSwitcher)) {
                if !state.accounts.is_empty() {
                    if delta == usize::MAX {
                        state.account_selected = state.account_selected.saturating_sub(1);
                    } else if delta > 0 && delta != usize::MAX {
                        state.account_selected =
                            (state.account_selected + 1).min(state.accounts.len() - 1);
                    }
                }
                return vec![];
            }
            handle_list_select(state, delta)
        }
        Message::TabChanged(delta) => {
            let overflow = if matches!(state.route, Route::Settings { .. }) {
                if let Some(ref mut settings) = state.settings {
                    match try_move_subtab(settings.tab, delta, settings.tab_count()) {
                        Some(new_tab) => {
                            settings.tab = new_tab;
                            false
                        }
                        None => true,
                    }
                } else {
                    false
                }
            } else if matches!(state.route, Route::Insights { .. }) {
                if let Some(ref mut ins) = state.insights {
                    match try_move_subtab(ins.tab, delta, ins.tab_count()) {
                        Some(new_tab) => {
                            ins.tab = new_tab;
                            false
                        }
                        None => true,
                    }
                } else {
                    false
                }
            } else if matches!(state.route, Route::Security { .. }) {
                if let Some(ref mut sec) = state.security {
                    match try_move_subtab(sec.tab, delta, sec.tab_count()) {
                        Some(new_tab) => {
                            sec.tab = new_tab;
                            false
                        }
                        None => true,
                    }
                } else {
                    false
                }
            } else if let Some(ref mut detail) = state.pr_detail {
                match try_move_subtab(detail.tab, delta, 3) {
                    Some(new_tab) => {
                        detail.tab = new_tab;
                        false
                    }
                    None => true,
                }
            } else {
                false
            };

            if overflow {
                // Sub-tab overflowed: move to next/prev global tab
                let total = ghtui_core::router::TAB_LABELS.len();
                if delta == usize::MAX {
                    state.active_tab = (state.active_tab + total - 1) % total;
                } else {
                    state.active_tab = (state.active_tab + 1) % total;
                }
                return navigate_to_tab(state);
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
            state.input_buffer.clear();
            // Pre-fill input for edit/reply modals
            match &kind {
                ghtui_core::ModalKind::EditIssue => {
                    if let Some(ref detail) = state.issue_detail {
                        match detail.selected_comment {
                            None => {
                                // Editing the issue itself: title\nbody
                                let issue = &detail.detail.issue;
                                state.input_buffer = format!(
                                    "{}\n{}",
                                    issue.title,
                                    issue.body.as_deref().unwrap_or("")
                                );
                            }
                            Some(idx) => {
                                // Editing a comment
                                if let Some(comment) = detail.detail.comments.get(idx) {
                                    state.input_buffer = comment.body.clone();
                                }
                            }
                        }
                    }
                }
                ghtui_core::ModalKind::AddComment => {
                    // If a comment is selected, quote it for reply
                    if let Some(ref detail) = state.issue_detail {
                        if let Some(idx) = detail.selected_comment {
                            if let Some(comment) = detail.detail.comments.get(idx) {
                                let quoted: String = comment
                                    .body
                                    .lines()
                                    .map(|l| format!("> {}", l))
                                    .collect::<Vec<_>>()
                                    .join("\n");
                                state.input_buffer =
                                    format!("> @{}\n{}\n\n", comment.user.login, quoted);
                            }
                        }
                    }
                }
                _ => {}
            }
            state.modal = Some(kind);
            state.input_mode = InputMode::Insert;
            vec![]
        }
        Message::ModalSubmit => {
            let cmds = match state.modal {
                Some(ghtui_core::ModalKind::AddComment) => {
                    let body = state.input_buffer.clone();
                    if body.trim().is_empty() {
                        vec![]
                    } else if let Some(ref repo) = state.current_repo {
                        match &state.route {
                            Route::IssueDetail { number, .. } => {
                                vec![Command::AddComment(repo.clone(), *number, body)]
                            }
                            Route::PrDetail { number, .. } => {
                                vec![Command::AddComment(repo.clone(), *number, body)]
                            }
                            _ => vec![],
                        }
                    } else {
                        vec![]
                    }
                }
                Some(ghtui_core::ModalKind::EditIssue) => {
                    if let Some(ref repo) = state.current_repo {
                        if let Some(ref detail) = state.issue_detail {
                            match detail.selected_comment {
                                None => {
                                    // Edit issue title/body
                                    let input = state.input_buffer.clone();
                                    let mut lines = input.splitn(2, '\n');
                                    let title = lines.next().unwrap_or("").trim().to_string();
                                    let body = lines.next().map(|b| b.trim().to_string());
                                    let number = detail.detail.issue.number;
                                    let title_opt =
                                        if title.is_empty() { None } else { Some(title) };
                                    vec![Command::UpdateIssue(
                                        repo.clone(),
                                        number,
                                        title_opt,
                                        body,
                                    )]
                                }
                                Some(idx) => {
                                    // Edit comment
                                    if let Some(comment) = detail.detail.comments.get(idx) {
                                        let body = state.input_buffer.clone();
                                        if body.trim().is_empty() {
                                            vec![]
                                        } else {
                                            let number = detail.detail.issue.number;
                                            vec![Command::UpdateComment(
                                                repo.clone(),
                                                number,
                                                comment.id,
                                                body,
                                            )]
                                        }
                                    } else {
                                        vec![]
                                    }
                                }
                            }
                        } else {
                            vec![]
                        }
                    } else {
                        vec![]
                    }
                }
                Some(ghtui_core::ModalKind::EditComment(comment_id)) => {
                    let body = state.input_buffer.clone();
                    if body.trim().is_empty() {
                        vec![]
                    } else if let Some(ref repo) = state.current_repo {
                        if let Some(ref detail) = state.issue_detail {
                            vec![Command::UpdateComment(
                                repo.clone(),
                                detail.detail.issue.number,
                                comment_id,
                                body,
                            )]
                        } else {
                            vec![]
                        }
                    } else {
                        vec![]
                    }
                }
                Some(ghtui_core::ModalKind::CreateIssue) => {
                    let input = state.input_buffer.clone();
                    let mut lines = input.splitn(2, '\n');
                    let title = lines.next().unwrap_or("").trim().to_string();
                    let body = lines.next().unwrap_or("").trim().to_string();
                    if title.is_empty() {
                        state.push_toast("Title cannot be empty".to_string(), ToastLevel::Warning);
                        return vec![];
                    }
                    if let Some(ref repo) = state.current_repo {
                        let input = ghtui_core::types::CreateIssueInput {
                            title,
                            body,
                            labels: vec![],
                            assignees: vec![],
                        };
                        vec![Command::CreateIssue(repo.clone(), input)]
                    } else {
                        vec![]
                    }
                }
                _ => vec![],
            };
            state.modal = None;
            state.input_mode = InputMode::Normal;
            state.input_buffer.clear();
            cmds
        }
        Message::ModalClose => {
            state.modal = None;
            state.input_mode = InputMode::Normal;
            state.input_buffer.clear();
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
        Route::Security { repo } => {
            state.security = Some(SecurityState::new());
            state.loading.insert("security".to_string());
            state.loading.insert("dependabot".to_string());
            state.loading.insert("code_scanning".to_string());
            state.loading.insert("secret_scanning".to_string());
            vec![
                Command::FetchDependabotAlerts(repo.clone()),
                Command::FetchCodeScanningAlerts(repo.clone()),
                Command::FetchSecretScanningAlerts(repo.clone()),
            ]
        }
        Route::Insights { repo } => {
            state.loading.insert("insights".to_string());
            state.loading.insert("contributors".to_string());
            state.loading.insert("commit_activity".to_string());
            state.loading.insert("traffic_clones".to_string());
            state.loading.insert("traffic_views".to_string());
            vec![
                Command::FetchContributorStats(repo.clone()),
                Command::FetchCommitActivity(repo.clone()),
                Command::FetchTrafficClones(repo.clone()),
                Command::FetchTrafficViews(repo.clone()),
            ]
        }
        Route::Settings { repo } => {
            state.loading.insert("settings".to_string());
            vec![Command::FetchRepoSettings(repo.clone())]
        }
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
        TAB_SECURITY => Route::Security { repo },
        TAB_INSIGHTS => Route::Insights { repo },
        TAB_SETTINGS => Route::Settings { repo },
        _ => return vec![],
    };

    handle_navigate(state, route)
}

/// Try to move sub-tab. Returns None if overflow (should go to global tab).
fn try_move_subtab(current: usize, delta: usize, count: usize) -> Option<usize> {
    if count == 0 {
        return None;
    }
    if delta == usize::MAX {
        // Previous
        if current == 0 {
            None // overflow: go to previous global tab
        } else {
            Some(current - 1)
        }
    } else {
        // Next
        let next = current + delta;
        if next >= count {
            None // overflow: go to next global tab
        } else {
            Some(next)
        }
    }
}

fn handle_list_select(state: &mut AppState, delta: usize) -> Vec<Command> {
    match &state.route {
        Route::PrList { .. } => {
            if let Some(ref mut list) = state.pr_list {
                if delta == 0 {
                    if let Some(pr) = list.selected_pr() {
                        let repo = state.current_repo.clone().unwrap();
                        let number = pr.number;
                        let route = Route::PrDetail {
                            repo,
                            number,
                            tab: ghtui_core::PrTab::Conversation,
                        };
                        return handle_navigate(state, route);
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
                        return handle_navigate(state, route);
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
                        return handle_navigate(state, route);
                    }
                } else if delta == usize::MAX {
                    list.select_prev();
                } else {
                    list.select_next();
                }
            }
        }
        Route::ActionDetail { repo, run_id } => {
            if let Some(ref mut detail) = state.action_detail {
                if delta == 0 {
                    // Enter: fetch log for selected job
                    if let Some(job) = detail.detail.jobs.get(detail.selected_job) {
                        let job_id = job.id;
                        detail.log = None;
                        detail.log_scroll = 0;
                        state.loading.insert("job_log".to_string());
                        return vec![Command::FetchJobLog(repo.clone(), *run_id, job_id)];
                    }
                } else if delta == usize::MAX {
                    detail.selected_job = detail.selected_job.saturating_sub(1);
                } else {
                    let max = detail.detail.jobs.len().saturating_sub(1);
                    detail.selected_job = (detail.selected_job + 1).min(max);
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
        Route::IssueDetail { .. } => {
            if let Some(ref mut detail) = state.issue_detail {
                if delta == usize::MAX {
                    detail.select_prev_comment();
                } else if delta > 0 && delta != 0 {
                    detail.select_next_comment();
                }
            }
        }
        Route::Insights { .. } => {
            if let Some(ref mut ins) = state.insights {
                if delta == usize::MAX {
                    ins.scroll = ins.scroll.saturating_sub(1);
                } else if delta > 0 {
                    ins.scroll += 1;
                }
            }
        }
        Route::Settings { .. } => {
            if let Some(ref mut settings) = state.settings {
                if delta == usize::MAX {
                    settings.scroll = settings.scroll.saturating_sub(1);
                } else if delta > 0 {
                    settings.scroll += 1;
                }
            }
        }
        _ => {}
    }
    vec![]
}

fn handle_mouse_list_select(state: &mut AppState, item_index: usize) -> Vec<Command> {
    match &state.route {
        Route::PrList { .. } => {
            if let Some(ref mut list) = state.pr_list {
                if item_index < list.items.len() {
                    list.selected = item_index;
                }
            }
        }
        Route::IssueList { .. } => {
            if let Some(ref mut list) = state.issue_list {
                if item_index < list.items.len() {
                    list.selected = item_index;
                }
            }
        }
        Route::ActionsList { .. } => {
            if let Some(ref mut list) = state.actions_list {
                if item_index < list.items.len() {
                    list.selected = item_index;
                }
            }
        }
        Route::Notifications => {
            if let Some(ref mut list) = state.notifications {
                if item_index < list.items.len() {
                    list.selected = item_index;
                }
            }
        }
        _ => {}
    }
    vec![]
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
