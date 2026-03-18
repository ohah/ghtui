use ghtui_core::router::Route;
use ghtui_core::state::SecurityState;
use ghtui_core::state::pr::PrDetailState;
use ghtui_core::{AppState, Command, Message};

/// Check if a filename has an image extension.
pub(crate) fn is_image_file(filename: &str) -> bool {
    let ext = filename.rsplit('.').next().unwrap_or("").to_lowercase();
    matches!(
        ext.as_str(),
        "png" | "jpg" | "jpeg" | "gif" // only formats enabled in image crate
    )
}

pub(crate) fn handle_navigate(state: &mut AppState, route: Route) -> Vec<Command> {
    let mut cmds = match &route {
        Route::PrList { repo, filters } => {
            state.loading.insert("pr_list".to_string());
            vec![Command::FetchPrList(repo.clone(), filters.clone(), 1)]
        }
        Route::PrDetail { repo, number, .. } => {
            state.loading.insert("pr_detail".to_string());
            // Diff is fetched after detail loads (see PrDetailLoaded handler)
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
            vec![
                Command::FetchRuns(repo.clone(), filters.clone(), 1),
                Command::FetchWorkflows(repo.clone()),
            ]
        }
        Route::ActionDetail { repo, run_id } => {
            state.loading.insert("action_detail".to_string());
            vec![
                Command::FetchRunDetail(repo.clone(), *run_id),
                Command::FetchRunArtifacts(repo.clone(), *run_id),
                Command::FetchPendingDeployments(repo.clone(), *run_id),
            ]
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
        Route::Code {
            repo,
            path: _,
            git_ref,
        } => {
            state.code = Some(ghtui_core::state::CodeViewState::new(git_ref.clone()));
            state.loading.insert("code_tree".to_string());
            state.loading.insert("code_contents".to_string());
            state.loading.insert("code_branches".to_string());
            state.loading.insert("code_tags".to_string());
            vec![
                Command::FetchTree(repo.clone(), git_ref.clone()),
                Command::FetchBranches(repo.clone()),
                Command::FetchTags(repo.clone()),
            ]
        }
        Route::Security { repo } => {
            state.security = Some(SecurityState::new());
            state.loading.insert("security".to_string());
            state.loading.insert("dependabot".to_string());
            state.loading.insert("code_scanning".to_string());
            state.loading.insert("secret_scanning".to_string());
            state.loading.insert("security_advisories".to_string());
            vec![
                Command::FetchDependabotAlerts(repo.clone()),
                Command::FetchCodeScanningAlerts(repo.clone()),
                Command::FetchSecretScanningAlerts(repo.clone()),
                Command::FetchSecurityAdvisories(repo.clone()),
            ]
        }
        Route::Insights { repo } => {
            state.loading.insert("insights".to_string());
            state.loading.insert("contributors".to_string());
            state.loading.insert("commit_activity".to_string());
            state.loading.insert("traffic_clones".to_string());
            state.loading.insert("traffic_views".to_string());
            state.loading.insert("code_frequency".to_string());
            state.loading.insert("forks".to_string());
            state.loading.insert("dependency_graph".to_string());
            vec![
                Command::FetchContributorStats(repo.clone()),
                Command::FetchCommitActivity(repo.clone()),
                Command::FetchTrafficClones(repo.clone()),
                Command::FetchTrafficViews(repo.clone()),
                Command::FetchCodeFrequency(repo.clone()),
                Command::FetchForks(repo.clone()),
                Command::FetchDependencyGraph(repo.clone()),
            ]
        }
        Route::Settings { repo } => {
            state.loading.insert("settings".to_string());
            state.loading.insert("webhooks".to_string());
            state.loading.insert("deploy_keys".to_string());
            vec![
                Command::FetchRepoSettings(repo.clone()),
                Command::FetchWebhooks(repo.clone()),
                Command::FetchDeployKeys(repo.clone()),
            ]
        }
        Route::Discussions { repo } => {
            state.loading.insert("discussions".to_string());
            vec![Command::FetchDiscussions(repo.clone())]
        }
        Route::Gists => {
            state.loading.insert("gists".to_string());
            vec![Command::FetchGists]
        }
        Route::Organizations => {
            state.loading.insert("orgs".to_string());
            vec![Command::FetchOrgs]
        }
        Route::Dashboard => {
            state.loading.insert("recent_repos".to_string());
            vec![Command::FetchRecentRepos]
        }
    };

    // Sync current_repo and active_tab with route
    if let Some(repo) = route.repo() {
        let repo_changed = state.current_repo.as_ref() != Some(repo);
        state.current_repo = Some(repo.clone());
        // Fetch open issue/PR counts when repo changes
        if repo_changed {
            state.open_issue_count = None;
            state.open_pr_count = None;
            cmds.push(Command::FetchRepoCounts(repo.clone()));
        }
    }
    if let Some(idx) = route.tab_index() {
        state.active_tab = idx;
    }
    state.navigate(route);
    cmds
}

pub(crate) fn navigate_to_tab(state: &mut AppState) -> Vec<Command> {
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
pub(crate) fn try_move_subtab(current: usize, delta: usize, count: usize) -> Option<usize> {
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

pub(crate) fn handle_list_select(state: &mut AppState, delta: usize) -> Vec<Command> {
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
        Route::Search { .. } => {
            if let Some(ref mut search) = state.search {
                if delta == usize::MAX {
                    search.select_prev();
                } else if delta > 0 {
                    search.select_next();
                }
            }
        }
        Route::IssueDetail { .. } => {
            if let Some(ref mut detail) = state.issue_detail {
                // Picker mode
                if let Some(ref mut picker) = detail.label_picker {
                    let max = picker.available.len().saturating_sub(1);
                    if delta == usize::MAX {
                        picker.cursor = picker.cursor.saturating_sub(1);
                    } else if delta > 0 {
                        picker.cursor = (picker.cursor + 1).min(max);
                    }
                } else if let Some(ref mut picker) = detail.assignee_picker {
                    let max = picker.available.len().saturating_sub(1);
                    if delta == usize::MAX {
                        picker.cursor = picker.cursor.saturating_sub(1);
                    } else if delta > 0 {
                        picker.cursor = (picker.cursor + 1).min(max);
                    }
                } else if let Some(ref mut picker) = detail.milestone_picker {
                    let max = picker.available.len().saturating_sub(1);
                    if delta == usize::MAX {
                        picker.cursor = picker.cursor.saturating_sub(1);
                    } else if delta > 0 {
                        picker.cursor = (picker.cursor + 1).min(max);
                    }
                } else if delta == usize::MAX {
                    detail.focus_prev();
                } else if delta > 0 {
                    detail.focus_next();
                }
            }
        }
        Route::PrDetail { .. } => {
            if let Some(ref mut detail) = state.pr_detail {
                if let Some(ref mut picker) = detail.label_picker {
                    let max = picker.available.len().saturating_sub(1);
                    if delta == usize::MAX {
                        picker.cursor = picker.cursor.saturating_sub(1);
                    } else if delta > 0 {
                        picker.cursor = (picker.cursor + 1).min(max);
                    }
                } else if let Some(ref mut picker) = detail.assignee_picker {
                    let max = picker.available.len().saturating_sub(1);
                    if delta == usize::MAX {
                        picker.cursor = picker.cursor.saturating_sub(1);
                    } else if delta > 0 {
                        picker.cursor = (picker.cursor + 1).min(max);
                    }
                } else if let Some(ref mut picker) = detail.milestone_picker {
                    let max = picker.available.len().saturating_sub(1);
                    if delta == usize::MAX {
                        picker.cursor = picker.cursor.saturating_sub(1);
                    } else if delta > 0 {
                        picker.cursor = (picker.cursor + 1).min(max);
                    }
                } else {
                    match detail.tab {
                        0 => {
                            // Conversation tab: focus navigation + auto-scroll
                            if delta == usize::MAX {
                                detail.focus_prev();
                                detail.scroll = detail.scroll.saturating_sub(3);
                            } else if delta > 0 {
                                detail.focus_next();
                                detail.scroll += 3;
                            }
                        }
                        1 => {
                            // Commits tab: navigate commit list
                            let max = detail.detail.commits.len().saturating_sub(1);
                            if delta == usize::MAX {
                                detail.commit_selected = detail.commit_selected.saturating_sub(1);
                            } else if delta > 0 {
                                detail.commit_selected = (detail.commit_selected + 1).min(max);
                            }
                        }
                        2 => {
                            // Checks tab: scroll
                            if delta == usize::MAX {
                                detail.scroll = detail.scroll.saturating_sub(1);
                            } else if delta > 0 {
                                detail.scroll += 1;
                            }
                        }
                        _ => {}
                    }
                }
            }
        }
        Route::Code { .. } => {
            if let Some(ref mut code) = state.code {
                // Ref picker navigation
                if code.ref_picker_open {
                    if delta == usize::MAX {
                        code.ref_picker_prev();
                    } else if delta > 0 {
                        code.ref_picker_next();
                    }
                } else if code.sidebar_focused {
                    if delta == usize::MAX {
                        code.select_prev();
                    } else if delta > 0 {
                        code.select_next();
                    }
                } else {
                    // Content focused: scroll file content or commit detail
                    if code.commit_detail.is_some() {
                        if delta == usize::MAX {
                            code.commit_scroll = code.commit_scroll.saturating_sub(1);
                        } else if delta > 0 {
                            code.commit_scroll += 1;
                        }
                    } else if delta == usize::MAX {
                        code.scroll = code.scroll.saturating_sub(1);
                    } else if delta > 0 {
                        code.scroll += 1;
                    }
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
                if settings.sidebar_focused {
                    // Sidebar: scroll only (tab navigation is via TabChanged)
                    if delta == usize::MAX {
                        settings.scroll = settings.scroll.saturating_sub(1);
                    } else if delta > 0 {
                        settings.scroll += 1;
                    }
                } else {
                    // Content focused: navigate items within current tab
                    let max_items = match settings.tab {
                        2 => settings.collaborators.len(),
                        3 => settings.webhooks.len(),
                        4 => settings.deploy_keys.len(),
                        _ => 0, // General/BranchProtection: use scroll
                    };
                    if max_items > 0 {
                        if delta == usize::MAX {
                            settings.selected = settings.selected.saturating_sub(1);
                        } else if delta > 0 {
                            settings.selected =
                                (settings.selected + 1).min(max_items.saturating_sub(1));
                        }
                    } else {
                        // General/BranchProtection: scroll content
                        if delta == usize::MAX {
                            settings.scroll = settings.scroll.saturating_sub(1);
                        } else if delta > 0 {
                            settings.scroll += 1;
                        }
                    }
                }
            }
        }
        Route::Discussions { .. } => {
            if let Some(ref mut disc) = state.discussions {
                if !disc.items.is_empty() {
                    if delta == usize::MAX {
                        disc.selected = disc.selected.saturating_sub(1);
                    } else if delta > 0 {
                        disc.selected = (disc.selected + 1).min(disc.items.len().saturating_sub(1));
                    }
                }
            }
        }
        Route::Gists => {
            if let Some(ref mut g) = state.gists {
                if !g.items.is_empty() {
                    if delta == usize::MAX {
                        g.selected = g.selected.saturating_sub(1);
                    } else if delta > 0 {
                        g.selected = (g.selected + 1).min(g.items.len().saturating_sub(1));
                    }
                }
            }
        }
        Route::Organizations => {
            if let Some(ref mut org_state) = state.org {
                if !org_state.orgs.is_empty() {
                    let old_selected = org_state.selected_org;
                    if delta == usize::MAX {
                        org_state.selected_org = org_state.selected_org.saturating_sub(1);
                    } else if delta > 0 {
                        org_state.selected_org = (org_state.selected_org + 1)
                            .min(org_state.orgs.len().saturating_sub(1));
                    }
                    // If selection changed, fetch members for new org
                    if old_selected != org_state.selected_org {
                        let login = org_state.orgs[org_state.selected_org].login.clone();
                        state.loading.insert("org_members".to_string());
                        return vec![Command::FetchOrgMembers(login)];
                    }
                }
            }
        }
        Route::Dashboard => {
            if !state.recent_repos.is_empty() {
                if delta == usize::MAX {
                    state.dashboard_selected = state.dashboard_selected.saturating_sub(1);
                } else if delta > 0 {
                    state.dashboard_selected = (state.dashboard_selected + 1)
                        .min(state.recent_repos.len().saturating_sub(1));
                }
            }
        }
        _ => {}
    }
    vec![]
}

pub(crate) fn handle_mouse_list_select(state: &mut AppState, item_index: usize) -> Vec<Command> {
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

pub(crate) fn refresh_current_view(state: &mut AppState) -> Vec<Command> {
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
        Route::Dashboard => {
            state.loading.insert("recent_repos".to_string());
            vec![Command::FetchRecentRepos]
        }
        _ => vec![],
    }
}

/// Count how many review comment lines appear after a given diff line
pub(crate) fn count_review_comment_lines(
    review_comments: &[ghtui_core::types::ReviewComment],
    filename: &str,
    line_num: Option<u32>,
) -> usize {
    let Some(ln) = line_num else { return 0 };
    let matching: Vec<_> = review_comments
        .iter()
        .filter(|rc| rc.path == filename && (rc.line == Some(ln) || rc.original_line == Some(ln)))
        .collect();
    if matching.is_empty() {
        return 0;
    }
    let mut count = 0;
    let roots: Vec<_> = matching
        .iter()
        .filter(|rc| rc.in_reply_to_id.is_none())
        .collect();
    for root in &roots {
        count += 1; // header line
        count += root.body.lines().count(); // body lines
        let replies = matching
            .iter()
            .filter(|rc| rc.in_reply_to_id == Some(root.id))
            .count();
        count += replies; // reply lines
        count += 1; // footer line
    }
    // Orphan comments
    let orphans = matching
        .iter()
        .filter(|rc| {
            rc.in_reply_to_id.is_some() && !roots.iter().any(|r| Some(r.id) == rc.in_reply_to_id)
        })
        .count();
    count += orphans * 2; // header + footer per orphan
    count
}

/// Find which file index the diff cursor is currently on
pub(crate) fn find_cursor_file(detail: &PrDetailState) -> Option<usize> {
    find_cursor_file_info(detail).map(|(fi, _)| fi)
}

/// Find file index and whether cursor is on the file header line
pub(crate) fn find_cursor_file_info(detail: &PrDetailState) -> Option<(usize, bool)> {
    let files = detail.diff.as_ref()?;
    let review_comments = &detail.detail.review_comments;
    let summary_lines = files.len() + 3; // header + empty + files + empty
    if detail.diff_cursor < summary_lines {
        return None;
    }
    let mut line = summary_lines;
    for (fi, file) in files.iter().enumerate() {
        let collapsed = detail.diff_collapsed.contains(&fi);
        let file_header_line = line;
        if collapsed {
            if detail.diff_cursor == file_header_line {
                return Some((fi, true));
            }
            line += 1;
            continue;
        }
        // Count: file header + hunks (each with header + lines + review comments) + trailing empty
        let mut file_lines = 1; // file header
        for hunk in &file.hunks {
            file_lines += 1; // hunk header
            for diff_line in &hunk.lines {
                file_lines += 1; // the code line itself
                let target = diff_line.new_line.or(diff_line.old_line);
                file_lines += count_review_comment_lines(review_comments, &file.filename, target);
            }
        }
        file_lines += 1; // trailing empty line

        if detail.diff_cursor >= file_header_line
            && detail.diff_cursor < file_header_line + file_lines
        {
            let is_header = detail.diff_cursor == file_header_line;
            return Some((fi, is_header));
        }
        line += file_lines;
    }
    None
}

/// Find the file path and line number at the current diff cursor position
pub(crate) fn find_cursor_line_info(detail: &PrDetailState) -> Option<(String, u32)> {
    let files = detail.diff.as_ref()?;
    let review_comments = &detail.detail.review_comments;
    let summary_lines = files.len() + 3;
    if detail.diff_cursor < summary_lines {
        return None;
    }
    let mut line = summary_lines;
    for (fi, file) in files.iter().enumerate() {
        let collapsed = detail.diff_collapsed.contains(&fi);
        if collapsed {
            line += 1;
            continue;
        }
        line += 1; // file header
        for hunk in &file.hunks {
            line += 1; // hunk header
            for diff_line in &hunk.lines {
                if detail.diff_cursor == line {
                    // Found the line -- return new_line or old_line
                    let ln = diff_line.new_line.or(diff_line.old_line)?;
                    return Some((file.filename.clone(), ln));
                }
                line += 1;
                let target = diff_line.new_line.or(diff_line.old_line);
                line += count_review_comment_lines(review_comments, &file.filename, target);
            }
        }
        line += 1; // trailing empty
    }
    None
}

pub(crate) fn action_bar_count(pr_state: &ghtui_core::types::PrState) -> usize {
    match pr_state {
        ghtui_core::types::PrState::Open => 5, // Comment, Approve, Request, Merge, Close
        ghtui_core::types::PrState::Closed => 1, // Reopen
        ghtui_core::types::PrState::Merged => 0,
    }
}

pub(crate) fn action_bar_action(
    index: usize,
    pr_state: &ghtui_core::types::PrState,
) -> Option<Message> {
    match pr_state {
        ghtui_core::types::PrState::Open => match index {
            0 => Some(Message::PrStartComment),
            1 => Some(Message::PrApprove),
            2 => Some(Message::PrRequestChanges),
            3 => Some(Message::ModalOpen(ghtui_core::ModalKind::MergePr)),
            4 => Some(Message::PrToggleState),
            _ => None,
        },
        ghtui_core::types::PrState::Closed => match index {
            0 => Some(Message::PrToggleState),
            _ => None,
        },
        ghtui_core::types::PrState::Merged => None,
    }
}

/// Find a review comment ID near the diff cursor position.
/// Reuses `count_review_comment_lines` to stay consistent with rendering.
pub(crate) fn find_review_comment_at_cursor(
    detail: &PrDetailState,
    files: &[ghtui_core::types::DiffFile],
) -> Option<u64> {
    let review_comments = &detail.detail.review_comments;
    let summary_lines = files.len() + 3;
    if detail.diff_cursor < summary_lines {
        return None;
    }
    let mut line = summary_lines;
    for (fi, file) in files.iter().enumerate() {
        let collapsed = detail.diff_collapsed.contains(&fi);
        if collapsed {
            line += 1;
            continue;
        }
        line += 1; // file header
        for hunk in &file.hunks {
            line += 1; // hunk header
            for diff_line in &hunk.lines {
                let code_line = line;
                line += 1;
                let target = diff_line.new_line.or(diff_line.old_line);
                let comment_lines =
                    count_review_comment_lines(review_comments, &file.filename, target);
                if comment_lines > 0
                    && detail.diff_cursor >= code_line
                    && detail.diff_cursor < code_line + 1 + comment_lines
                {
                    // Find the first root comment at this position
                    let root = review_comments.iter().find(|c| {
                        c.path == file.filename
                            && c.in_reply_to_id.is_none()
                            && (c.line == target || c.original_line == target)
                    });
                    return root.map(|c| c.id);
                }
                line += comment_lines;
            }
        }
        line += 1; // trailing empty
    }
    None
}

/// Trace back in_reply_to_id chain to find the root comment ID of a thread.
pub(crate) fn find_thread_root_id(
    mut comment_id: u64,
    review_comments: &[ghtui_core::types::ReviewComment],
) -> u64 {
    let mut visited = std::collections::HashSet::new();
    while let Some(c) = review_comments.iter().find(|c| c.id == comment_id) {
        match c.in_reply_to_id {
            Some(parent_id) if visited.insert(comment_id) => comment_id = parent_id,
            _ => return c.id,
        }
    }
    comment_id
}
