mod helpers;
use helpers::*;

use ghtui_core::router::Route;
use ghtui_core::state::actions::ActionBarItem;
use ghtui_core::state::issue::InlineEditTarget;
use ghtui_core::state::pr::PrInlineEditTarget;
use ghtui_core::state::settings::SettingsEditField;
use ghtui_core::state::*;
use ghtui_core::types::{ActionsFilters, IssueFilters, IssueState, PrFilters, PrState};
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
            state.code = None;
            state.discussions = None;
            state.gists = None;
            state.org = None;
            state.recent_repos.clear();
            // Refresh current view
            refresh_current_view(state)
        }

        // PR
        Message::PrListLoaded(prs, pagination, filters) => {
            state.loading.remove("pr_list");
            state.pr_list = Some(PrListState::with_filters(prs, pagination, filters));
            vec![]
        }
        Message::PrDetailLoaded(detail) => {
            state.loading.remove("pr_detail");
            // Preserve UI state (tab, scroll, focus) if refreshing
            let old = state.pr_detail.take();
            let mut new_state = PrDetailState::new(*detail);
            if let Some(old) = old {
                new_state.tab = old.tab;
                new_state.scroll = old.scroll;
                new_state.diff_scroll = old.diff_scroll;
                new_state.diff_cursor = old.diff_cursor;
                new_state.diff_collapsed = old.diff_collapsed;
                new_state.diff = old.diff; // keep existing diff until re-fetched
                new_state.show_file_tree = old.show_file_tree;
                new_state.file_tree_focused = old.file_tree_focused;
                new_state.file_tree_selected = old.file_tree_selected;
                new_state.focus = old.focus;
                new_state.viewed_files = old.viewed_files;
            }
            state.pr_detail = Some(new_state);
            // Fetch diff after detail is loaded (avoids race condition)
            if let Some(ref repo) = state.current_repo {
                if let Route::PrDetail { number, .. } = &state.route {
                    state.loading.insert("pr_diff".to_string());
                    return vec![Command::FetchPrDiff(repo.clone(), *number)];
                }
            }
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
        Message::PrUpdated(number) => {
            state.push_toast(format!("PR #{} updated", number), ToastLevel::Success);
            refresh_current_view(state)
        }
        Message::ReviewSubmitted => {
            state.push_toast("Review submitted".to_string(), ToastLevel::Success);
            refresh_current_view(state)
        }
        Message::PrToggleStateFilter => {
            if let Some(ref repo) = state.current_repo {
                let filters = if let Some(ref mut list) = state.pr_list {
                    list.toggle_state_filter();
                    list.filters.clone()
                } else {
                    PrFilters {
                        state: Some(PrState::Closed),
                        ..Default::default()
                    }
                };
                state.loading.insert("pr_list".to_string());
                vec![Command::FetchPrList(repo.clone(), filters, 1)]
            } else {
                vec![]
            }
        }
        Message::PrSortCycle => {
            if let Some(ref repo) = state.current_repo {
                let filters = if let Some(ref mut list) = state.pr_list {
                    list.cycle_sort();
                    list.filters.clone()
                } else {
                    PrFilters::default()
                };
                state.loading.insert("pr_list".to_string());
                vec![Command::FetchPrList(repo.clone(), filters, 1)]
            } else {
                vec![]
            }
        }
        Message::PrNextPage => {
            if let (Some(repo), Some(list)) = (&state.current_repo, &state.pr_list) {
                if list.pagination.has_next {
                    let next_page = list.pagination.page + 1;
                    let filters = list.filters.clone();
                    state.loading.insert("pr_list".to_string());
                    vec![Command::FetchPrList(repo.clone(), filters, next_page)]
                } else {
                    vec![]
                }
            } else {
                vec![]
            }
        }
        Message::PrPrevPage => {
            if let (Some(repo), Some(list)) = (&state.current_repo, &state.pr_list) {
                if list.pagination.page > 1 {
                    let prev_page = list.pagination.page - 1;
                    let filters = list.filters.clone();
                    state.loading.insert("pr_list".to_string());
                    vec![Command::FetchPrList(repo.clone(), filters, prev_page)]
                } else {
                    vec![]
                }
            } else {
                vec![]
            }
        }
        Message::PrSearchStart => {
            if let Some(ref mut list) = state.pr_list {
                list.search_mode = true;
                list.search_query.clear();
            }
            vec![]
        }
        Message::PrSearchInput(text) => {
            if let Some(ref mut list) = state.pr_list {
                if text == "\x08" {
                    list.search_query.pop();
                } else {
                    list.search_query.push_str(&text);
                }
            }
            vec![]
        }
        Message::PrSearchSubmit => {
            if let (Some(list), Some(repo)) = (&mut state.pr_list, &state.current_repo) {
                list.search_mode = false;
                let query = list.search_query.clone();
                if query.is_empty() {
                    state.loading.insert("pr_list".to_string());
                    vec![Command::FetchPrList(repo.clone(), list.filters.clone(), 1)]
                } else {
                    state.loading.insert("pr_list".to_string());
                    vec![Command::SearchPulls(repo.clone(), query)]
                }
            } else {
                vec![]
            }
        }
        Message::PrSearchCancel => {
            if let Some(ref mut list) = state.pr_list {
                list.search_mode = false;
                list.search_query.clear();
            }
            vec![]
        }
        Message::PrFilterByLabel(label) => {
            if let Some(ref mut list) = state.pr_list {
                list.filters.label = Some(label);
            }
            if let Some(ref repo) = state.current_repo {
                if let Some(ref list) = state.pr_list {
                    state.loading.insert("pr_list".to_string());
                    return vec![Command::FetchPrList(repo.clone(), list.filters.clone(), 1)];
                }
            }
            vec![]
        }
        Message::PrFilterByAuthor(author) => {
            if let Some(ref mut list) = state.pr_list {
                list.filters.author = Some(author);
            }
            if let Some(ref repo) = state.current_repo {
                if let Some(ref list) = state.pr_list {
                    state.loading.insert("pr_list".to_string());
                    return vec![Command::FetchPrList(repo.clone(), list.filters.clone(), 1)];
                }
            }
            vec![]
        }
        Message::PrFilterByAssignee(assignee) => {
            if let Some(ref mut list) = state.pr_list {
                list.filters.assignee = Some(assignee);
            }
            if let Some(ref repo) = state.current_repo {
                if let Some(ref list) = state.pr_list {
                    state.loading.insert("pr_list".to_string());
                    return vec![Command::FetchPrList(repo.clone(), list.filters.clone(), 1)];
                }
            }
            vec![]
        }
        Message::PrFilterClear => {
            if let Some(ref mut list) = state.pr_list {
                let state_filter = list.filters.state;
                list.filters = PrFilters {
                    state: state_filter,
                    ..Default::default()
                };
            }
            if let Some(ref repo) = state.current_repo {
                if let Some(ref list) = state.pr_list {
                    state.loading.insert("pr_list".to_string());
                    return vec![Command::FetchPrList(repo.clone(), list.filters.clone(), 1)];
                }
            }
            vec![]
        }
        // PR detail: close/reopen
        Message::PrToggleState => {
            if let Some(ref detail) = state.pr_detail {
                if let Some(ref repo) = state.current_repo {
                    let number = detail.detail.pr.number;
                    return match detail.detail.pr.state {
                        PrState::Open => vec![Command::ClosePr(repo.clone(), number)],
                        PrState::Closed => vec![Command::ReopenPr(repo.clone(), number)],
                        PrState::Merged => vec![], // Can't reopen merged
                    };
                }
            }
            vec![]
        }
        Message::PrOpenInBrowser => {
            if let Some(ref detail) = state.pr_detail {
                if let Some(ref repo) = state.current_repo {
                    let url = format!(
                        "https://github.com/{}/pull/{}",
                        repo.full_name(),
                        detail.detail.pr.number
                    );
                    return vec![Command::OpenInBrowser(url)];
                }
            }
            vec![]
        }
        Message::PrApprove => {
            if let Some(ref mut detail) = state.pr_detail {
                detail.editor = ghtui_core::editor::TextEditor::new();
                detail.edit_target = Some(PrInlineEditTarget::ReviewApprove);
            }
            vec![]
        }
        Message::PrRequestChanges => {
            if let Some(ref mut detail) = state.pr_detail {
                detail.editor = ghtui_core::editor::TextEditor::new();
                detail.edit_target = Some(PrInlineEditTarget::ReviewRequestChanges);
            }
            vec![]
        }
        Message::PrActionBarFocus => {
            if let Some(ref mut detail) = state.pr_detail {
                detail.action_bar_focused = !detail.action_bar_focused;
            }
            vec![]
        }
        Message::PrActionBarLeft => {
            if let Some(ref mut detail) = state.pr_detail {
                detail.action_bar_selected = detail.action_bar_selected.saturating_sub(1);
            }
            vec![]
        }
        Message::PrActionBarRight => {
            if let Some(ref mut detail) = state.pr_detail {
                let max = action_bar_count(&detail.detail.pr.state);
                detail.action_bar_selected =
                    (detail.action_bar_selected + 1).min(max.saturating_sub(1));
            }
            vec![]
        }
        Message::PrActionBarSelect => {
            if let Some(ref detail) = state.pr_detail {
                let msg =
                    match action_bar_action(detail.action_bar_selected, &detail.detail.pr.state) {
                        Some(m) => m,
                        None => return vec![],
                    };
                return update(state, msg);
            }
            vec![]
        }
        Message::PrChangeBase => {
            if let Some(ref detail) = state.pr_detail {
                state.input_buffer = detail.detail.pr.base_ref.clone();
                state.modal = Some(ghtui_core::ModalKind::Confirm {
                    title: "Change Base Branch".to_string(),
                    message: "Enter target branch name:".to_string(),
                });
                state.input_mode = InputMode::Insert;
            }
            vec![]
        }
        Message::PrDeleteComment => {
            if let Some(ref detail) = state.pr_detail {
                if let Some(idx) = detail.selected_comment() {
                    if let Some(comment) = detail.detail.comments.get(idx) {
                        if let Some(ref repo) = state.current_repo {
                            return vec![Command::DeletePrComment(repo.clone(), comment.id)];
                        }
                    }
                }
            }
            vec![]
        }
        // PR label picker
        Message::PrLabelToggle => {
            if let Some(ref repo) = state.current_repo {
                state.loading.insert("repo_labels".to_string());
                vec![Command::FetchRepoLabels(repo.clone())]
            } else {
                vec![]
            }
        }
        Message::PrLabelsLoaded(labels) => {
            state.loading.remove("repo_labels");
            if let Some(ref mut detail) = state.pr_detail {
                let current_labels: Vec<String> = detail
                    .detail
                    .pr
                    .labels
                    .iter()
                    .map(|l| l.name.clone())
                    .collect();
                detail.label_picker = Some(ghtui_core::state::issue::LabelPickerState {
                    available: labels,
                    selected_names: current_labels,
                    cursor: 0,
                });
            }
            vec![]
        }
        Message::PrLabelSelect(idx) => {
            if let Some(ref mut detail) = state.pr_detail {
                if let Some(ref mut picker) = detail.label_picker {
                    if let Some(label) = picker.available.get(idx) {
                        let name = label.name.clone();
                        if picker.selected_names.contains(&name) {
                            picker.selected_names.retain(|n| n != &name);
                        } else {
                            picker.selected_names.push(name);
                        }
                    }
                }
            }
            vec![]
        }
        Message::PrLabelApply => {
            if let Some(ref detail) = state.pr_detail {
                if let (Some(picker), Some(repo)) = (&detail.label_picker, &state.current_repo) {
                    let number = detail.detail.pr.number;
                    let labels = picker.selected_names.clone();
                    let cmds = vec![Command::SetPrLabels(repo.clone(), number, labels)];
                    if let Some(ref mut detail) = state.pr_detail {
                        detail.label_picker = None;
                    }
                    return cmds;
                }
            }
            if let Some(ref mut detail) = state.pr_detail {
                detail.label_picker = None;
            }
            vec![]
        }
        Message::PrLabelCancel => {
            if let Some(ref mut detail) = state.pr_detail {
                detail.label_picker = None;
            }
            vec![]
        }
        // PR assignee picker
        Message::PrAssigneeToggle => {
            if let Some(ref repo) = state.current_repo {
                state.loading.insert("collaborators_picker".to_string());
                vec![Command::FetchCollaboratorsForPicker(repo.clone())]
            } else {
                vec![]
            }
        }
        Message::PrCollaboratorsLoaded(logins) => {
            state.loading.remove("collaborators_picker");
            if let Some(ref mut detail) = state.pr_detail {
                let current: Vec<String> = detail
                    .detail
                    .pr
                    .assignees
                    .iter()
                    .map(|a| a.login.clone())
                    .collect();
                detail.assignee_picker = Some(ghtui_core::state::issue::AssigneePickerState {
                    available: logins,
                    selected_names: current,
                    cursor: 0,
                });
            }
            vec![]
        }
        Message::PrAssigneeSelect(idx) => {
            if let Some(ref mut detail) = state.pr_detail {
                if let Some(ref mut picker) = detail.assignee_picker {
                    if let Some(login) = picker.available.get(idx).cloned() {
                        if picker.selected_names.contains(&login) {
                            picker.selected_names.retain(|n| n != &login);
                        } else {
                            picker.selected_names.push(login);
                        }
                    }
                }
            }
            vec![]
        }
        Message::PrAssigneeApply => {
            if let Some(ref detail) = state.pr_detail {
                if let (Some(picker), Some(repo)) = (&detail.assignee_picker, &state.current_repo) {
                    let number = detail.detail.pr.number;
                    let assignees = picker.selected_names.clone();
                    let cmds = vec![Command::SetPrAssignees(repo.clone(), number, assignees)];
                    if let Some(ref mut detail) = state.pr_detail {
                        detail.assignee_picker = None;
                    }
                    return cmds;
                }
            }
            if let Some(ref mut detail) = state.pr_detail {
                detail.assignee_picker = None;
            }
            vec![]
        }
        Message::PrAssigneeCancel => {
            if let Some(ref mut detail) = state.pr_detail {
                detail.assignee_picker = None;
            }
            vec![]
        }
        // PR milestone picker
        Message::PrMilestoneToggle => {
            if let Some(ref repo) = state.current_repo {
                state.loading.insert("milestones".to_string());
                vec![Command::FetchMilestones(repo.clone())]
            } else {
                vec![]
            }
        }
        Message::PrMilestoneSelect(idx) => {
            if let Some(ref mut detail) = state.pr_detail {
                if let Some(ref mut picker) = detail.milestone_picker {
                    if let Some(ms) = picker.available.get(idx) {
                        let num = ms.number as u64;
                        if picker.selected == Some(num) {
                            picker.selected = None;
                        } else {
                            picker.selected = Some(num);
                        }
                    }
                }
            }
            vec![]
        }
        Message::PrMilestoneApply => {
            if let Some(ref detail) = state.pr_detail {
                if let (Some(picker), Some(repo)) = (&detail.milestone_picker, &state.current_repo)
                {
                    let number = detail.detail.pr.number;
                    let ms = picker.selected;
                    let cmds = vec![Command::SetMilestone(repo.clone(), number, ms)];
                    if let Some(ref mut d) = state.pr_detail {
                        d.milestone_picker = None;
                    }
                    return cmds;
                }
            }
            if let Some(ref mut d) = state.pr_detail {
                d.milestone_picker = None;
            }
            vec![]
        }
        Message::PrMilestoneClear => {
            if let Some(ref detail) = state.pr_detail {
                if let Some(ref repo) = state.current_repo {
                    let number = detail.detail.pr.number;
                    if let Some(ref mut d) = state.pr_detail {
                        d.milestone_picker = None;
                    }
                    return vec![Command::SetMilestone(repo.clone(), number, None)];
                }
            }
            vec![]
        }
        Message::PrMilestoneCancel => {
            if let Some(ref mut d) = state.pr_detail {
                d.milestone_picker = None;
            }
            vec![]
        }
        // PR draft toggle
        Message::PrDraftToggle => {
            if let Some(ref detail) = state.pr_detail {
                if let Some(ref repo) = state.current_repo {
                    let number = detail.detail.pr.number;
                    let new_draft = !detail.detail.pr.draft;
                    return vec![Command::SetPrDraft(repo.clone(), number, new_draft)];
                }
            }
            vec![]
        }
        // PR auto-merge toggle
        Message::PrAutoMergeToggle => {
            if let Some(ref detail) = state.pr_detail {
                if let Some(ref repo) = state.current_repo {
                    let number = detail.detail.pr.number;
                    let enable = !detail.detail.pr.auto_merge;
                    return vec![Command::SetAutoMerge(repo.clone(), number, enable)];
                }
            }
            vec![]
        }
        // PR diff toggle side-by-side mode
        Message::PrDiffToggleSideBySide => {
            if let Some(ref mut detail) = state.pr_detail {
                detail.diff_side_by_side = !detail.diff_side_by_side;
            }
            vec![]
        }
        // PR review thread resolve/unresolve
        Message::PrReviewThreadToggleResolve => {
            if let (Some(detail), Some(repo)) = (&state.pr_detail, &state.current_repo) {
                if let Some(ref files) = detail.diff {
                    let cursor_comment_id = find_review_comment_at_cursor(detail, files);
                    if let Some(comment_id) = cursor_comment_id {
                        // Find root of the thread (in_reply_to_id chain)
                        let root_id =
                            find_thread_root_id(comment_id, &detail.detail.review_comments);
                        // Find matching thread
                        if let Some(thread) = detail
                            .detail
                            .review_threads
                            .iter()
                            .find(|t| t.root_comment_id == root_id)
                        {
                            let resolve = !thread.is_resolved;
                            let number = detail.detail.pr.number;
                            return vec![Command::ResolveReviewThread(
                                repo.clone(),
                                number,
                                thread.node_id.clone(),
                                resolve,
                            )];
                        }
                    }
                    state.push_toast(
                        "No review thread at cursor".to_string(),
                        ToastLevel::Warning,
                    );
                }
            }
            vec![]
        }
        // PR diff mark viewed (local only)
        Message::PrDiffMarkViewed => {
            if let Some(ref mut detail) = state.pr_detail {
                // Determine which file to mark based on context
                let filename = if detail.tab == 3 && detail.file_tree_focused {
                    // In file tree: use selected file
                    detail
                        .diff
                        .as_ref()
                        .and_then(|files| files.get(detail.file_tree_selected))
                        .map(|f| f.filename.clone())
                } else if detail.tab == 3 {
                    // In diff view: find file at cursor
                    detail.diff.as_ref().and_then(|files| {
                        let mut line = 0usize;
                        for (i, file) in files.iter().enumerate() {
                            let collapsed = detail.diff_collapsed.contains(&i);
                            let file_lines = if collapsed {
                                1
                            } else {
                                1 + file.hunks.iter().map(|h| 1 + h.lines.len()).sum::<usize>()
                            };
                            if detail.diff_cursor < line + file_lines {
                                return Some(file.filename.clone());
                            }
                            line += file_lines;
                        }
                        None
                    })
                } else {
                    None
                };
                if let Some(name) = filename {
                    if detail.viewed_files.contains(&name) {
                        detail.viewed_files.remove(&name);
                    } else {
                        detail.viewed_files.insert(name);
                    }
                }
            }
            vec![]
        }
        // PR reviewer (modal input)
        Message::PrReviewerToggle => {
            let current = state
                .pr_detail
                .as_ref()
                .map(|d| {
                    d.detail
                        .pr
                        .requested_reviewers
                        .iter()
                        .map(|u| u.login.clone())
                        .collect::<Vec<_>>()
                        .join(", ")
                })
                .unwrap_or_default();
            state.input_buffer = current;
            state.modal = Some(ghtui_core::ModalKind::Confirm {
                title: "Request Reviewers".to_string(),
                message: "Enter reviewer logins (comma-separated):".to_string(),
            });
            state.input_mode = InputMode::Insert;
            vec![]
        }
        Message::PrReviewerApply | Message::PrReviewerCancel => vec![],
        // PR reactions
        Message::PrAddReaction(reaction) => {
            if let Some(ref detail) = state.pr_detail {
                if let Some(ref repo) = state.current_repo {
                    use ghtui_core::state::pr::PrSection;
                    match &detail.focus {
                        PrSection::Body | PrSection::Title => {
                            let number = detail.detail.pr.number;
                            return vec![Command::AddReaction(
                                repo.clone(),
                                number,
                                reaction,
                                true,
                            )];
                        }
                        PrSection::Comment(idx) => {
                            if let Some(comment) = detail.detail.comments.get(*idx) {
                                return vec![Command::AddReaction(
                                    repo.clone(),
                                    comment.id,
                                    reaction,
                                    false,
                                )];
                            }
                        }
                        _ => {}
                    }
                }
            }
            vec![]
        }
        // PR inline editing
        Message::PrStartEditTitle => {
            if let Some(ref mut detail) = state.pr_detail {
                detail.start_edit_title();
            }
            vec![]
        }
        Message::PrStartEditBody => {
            if let Some(ref mut detail) = state.pr_detail {
                use ghtui_core::state::pr::PrSection;
                match &detail.focus {
                    PrSection::Title => detail.start_edit_title(),
                    PrSection::Body => detail.start_edit_body(),
                    PrSection::Comment(idx) => detail.start_edit_comment(*idx),
                    _ => {}
                }
            }
            vec![]
        }
        Message::PrStartComment => {
            if let Some(ref mut detail) = state.pr_detail {
                detail.start_new_comment();
            }
            vec![]
        }
        Message::PrStartReply => {
            if let Some(ref mut detail) = state.pr_detail {
                if let Some(idx) = detail.selected_comment() {
                    detail.start_quote_reply(idx);
                } else {
                    detail.start_new_comment();
                }
            }
            vec![]
        }
        Message::PrEditChar(c) => {
            if let Some(ref mut detail) = state.pr_detail {
                if detail.diff_comment_target.is_some() {
                    detail.diff_comment_editor.insert_char(c);
                } else {
                    detail.editor.insert_char(c);
                }
            }
            vec![]
        }
        Message::PrEditNewline => {
            if let Some(ref mut detail) = state.pr_detail {
                if detail.diff_comment_target.is_some() {
                    detail.diff_comment_editor.insert_newline();
                } else {
                    detail.editor.insert_newline();
                }
            }
            vec![]
        }
        Message::PrEditBackspace => {
            if let Some(ref mut detail) = state.pr_detail {
                if detail.diff_comment_target.is_some() {
                    detail.diff_comment_editor.backspace();
                } else {
                    detail.editor.backspace();
                }
            }
            vec![]
        }
        Message::PrEditCursorLeft => {
            if let Some(ref mut detail) = state.pr_detail {
                if detail.diff_comment_target.is_some() {
                    detail.diff_comment_editor.move_left();
                } else {
                    detail.editor.move_left();
                }
            }
            vec![]
        }
        Message::PrEditCursorRight => {
            if let Some(ref mut detail) = state.pr_detail {
                if detail.diff_comment_target.is_some() {
                    detail.diff_comment_editor.move_right();
                } else {
                    detail.editor.move_right();
                }
            }
            vec![]
        }
        Message::PrEditCursorUp => {
            if let Some(ref mut detail) = state.pr_detail {
                if detail.diff_comment_target.is_some() {
                    detail.diff_comment_editor.move_up();
                } else {
                    detail.editor.move_up();
                }
            }
            vec![]
        }
        Message::PrEditCursorDown => {
            if let Some(ref mut detail) = state.pr_detail {
                if detail.diff_comment_target.is_some() {
                    detail.diff_comment_editor.move_down();
                } else {
                    detail.editor.move_down();
                }
            }
            vec![]
        }
        Message::PrEditDelete => {
            if let Some(ref mut detail) = state.pr_detail {
                if detail.diff_comment_target.is_some() {
                    detail.diff_comment_editor.delete();
                } else {
                    detail.editor.delete();
                }
            }
            vec![]
        }
        Message::PrEditHome => {
            if let Some(ref mut detail) = state.pr_detail {
                detail.editor.move_home();
            }
            vec![]
        }
        Message::PrEditEnd => {
            if let Some(ref mut detail) = state.pr_detail {
                detail.editor.move_end();
            }
            vec![]
        }
        Message::PrEditTab => {
            if let Some(ref mut detail) = state.pr_detail {
                detail.editor.insert_tab();
            }
            vec![]
        }
        Message::PrEditWordLeft => {
            if let Some(ref mut detail) = state.pr_detail {
                detail.editor.move_word_left();
            }
            vec![]
        }
        Message::PrEditWordRight => {
            if let Some(ref mut detail) = state.pr_detail {
                detail.editor.move_word_right();
            }
            vec![]
        }
        Message::PrEditPageUp => {
            if let Some(ref mut detail) = state.pr_detail {
                detail.editor.page_up();
            }
            vec![]
        }
        Message::PrEditPageDown => {
            if let Some(ref mut detail) = state.pr_detail {
                detail.editor.page_down();
            }
            vec![]
        }
        Message::PrEditUndo => {
            if let Some(ref mut detail) = state.pr_detail {
                detail.editor.undo();
            }
            vec![]
        }
        Message::PrEditRedo => {
            if let Some(ref mut detail) = state.pr_detail {
                detail.editor.redo();
            }
            vec![]
        }
        Message::PrEditSubmit => {
            if let Some(ref detail) = state.pr_detail {
                if let Some(ref repo) = state.current_repo {
                    let cmds = match &detail.edit_target {
                        Some(PrInlineEditTarget::PrTitle) => {
                            let title = detail.editor_text().trim().to_string();
                            let number = detail.detail.pr.number;
                            if title.is_empty() {
                                state.push_toast(
                                    "Title cannot be empty".to_string(),
                                    ToastLevel::Warning,
                                );
                                return vec![];
                            }
                            vec![Command::UpdatePr(repo.clone(), number, Some(title), None)]
                        }
                        Some(PrInlineEditTarget::PrBody) => {
                            let body = detail.editor_text();
                            let number = detail.detail.pr.number;
                            vec![Command::UpdatePr(repo.clone(), number, None, Some(body))]
                        }
                        Some(PrInlineEditTarget::Comment(idx)) => {
                            if let Some(comment) = detail.detail.comments.get(*idx) {
                                let body = detail.editor_text();
                                if body.trim().is_empty() {
                                    state.push_toast(
                                        "Comment cannot be empty".to_string(),
                                        ToastLevel::Warning,
                                    );
                                    return vec![];
                                }
                                vec![Command::UpdatePrComment(
                                    repo.clone(),
                                    detail.detail.pr.number,
                                    comment.id,
                                    body,
                                )]
                            } else {
                                vec![]
                            }
                        }
                        Some(
                            PrInlineEditTarget::NewComment | PrInlineEditTarget::QuoteReply(_),
                        ) => {
                            let body = detail.editor_text();
                            if body.trim().is_empty() {
                                state.push_toast(
                                    "Comment cannot be empty".to_string(),
                                    ToastLevel::Warning,
                                );
                                return vec![];
                            }
                            let number = detail.detail.pr.number;
                            vec![Command::AddPrComment(repo.clone(), number, body)]
                        }
                        Some(PrInlineEditTarget::ReviewApprove) => {
                            let body = detail.editor_text();
                            let number = detail.detail.pr.number;
                            let input = ghtui_core::types::ReviewInput {
                                event: ghtui_core::types::ReviewEvent::Approve,
                                body: Some(if body.trim().is_empty() {
                                    "Approved".to_string()
                                } else {
                                    body
                                }),
                                comments: vec![],
                            };
                            vec![Command::SubmitReview(repo.clone(), number, input)]
                        }
                        Some(PrInlineEditTarget::ReviewRequestChanges) => {
                            let body = detail.editor_text();
                            if body.trim().is_empty() {
                                state.push_toast(
                                    "Review comment cannot be empty".to_string(),
                                    ToastLevel::Warning,
                                );
                                return vec![];
                            }
                            let number = detail.detail.pr.number;
                            let input = ghtui_core::types::ReviewInput {
                                event: ghtui_core::types::ReviewEvent::RequestChanges,
                                body: Some(body),
                                comments: vec![],
                            };
                            vec![Command::SubmitReview(repo.clone(), number, input)]
                        }
                        None => vec![],
                    };
                    if let Some(ref mut detail) = state.pr_detail {
                        detail.cancel_edit();
                    }
                    return cmds;
                }
            }
            if let Some(ref mut detail) = state.pr_detail {
                detail.cancel_edit();
            }
            vec![]
        }
        Message::PrEditCancel => {
            if let Some(ref mut detail) = state.pr_detail {
                detail.cancel_edit();
            }
            vec![]
        }
        // PR diff navigation
        Message::PrDiffCursorDown => {
            if let Some(ref mut detail) = state.pr_detail {
                detail.diff_cursor += 1;
                detail.diff_select_anchor = None;
            }
            vec![]
        }
        Message::PrDiffCursorUp => {
            if let Some(ref mut detail) = state.pr_detail {
                detail.diff_cursor = detail.diff_cursor.saturating_sub(1);
                detail.diff_select_anchor = None;
            }
            vec![]
        }
        Message::PrDiffSelectDown => {
            if let Some(ref mut detail) = state.pr_detail {
                if detail.diff_select_anchor.is_none() {
                    detail.diff_select_anchor = Some(detail.diff_cursor);
                }
                detail.diff_cursor += 1;
            }
            vec![]
        }
        Message::PrDiffSelectUp => {
            if let Some(ref mut detail) = state.pr_detail {
                if detail.diff_select_anchor.is_none() {
                    detail.diff_select_anchor = Some(detail.diff_cursor);
                }
                detail.diff_cursor = detail.diff_cursor.saturating_sub(1);
            }
            vec![]
        }
        Message::PrDiffToggleCollapse => {
            if let Some(ref mut detail) = state.pr_detail {
                if let Some((fi, is_header)) = find_cursor_file_info(detail) {
                    if is_header {
                        // File header → toggle fold
                        if detail.diff_collapsed.contains(&fi) {
                            detail.diff_collapsed.remove(&fi);
                        } else {
                            detail.diff_collapsed.insert(fi);
                        }
                    } else {
                        // Code line → open inline comment editor
                        if let Some(target) = find_cursor_line_info(detail) {
                            detail.diff_comment_target = Some(target);
                            detail.diff_comment_editor = ghtui_core::editor::TextEditor::new();
                        }
                    }
                }
            }
            vec![]
        }
        Message::PrDiffExpand => {
            if let Some(ref mut detail) = state.pr_detail {
                if let Some(fi) = find_cursor_file(detail) {
                    detail.diff_collapsed.remove(&fi);
                }
            }
            vec![]
        }
        Message::PrDiffCollapse => {
            if let Some(ref mut detail) = state.pr_detail {
                if let Some(fi) = find_cursor_file(detail) {
                    detail.diff_collapsed.insert(fi);
                }
            }
            vec![]
        }
        Message::PrDiffCommentSubmit => {
            if let Some(ref detail) = state.pr_detail {
                if let (Some((path, line)), Some(repo)) =
                    (&detail.diff_comment_target, &state.current_repo)
                {
                    let body = detail.diff_comment_editor.content();
                    if body.trim().is_empty() {
                        state
                            .push_toast("Comment cannot be empty".to_string(), ToastLevel::Warning);
                        return vec![];
                    }
                    let number = detail.detail.pr.number;
                    let input = ghtui_core::types::ReviewInput {
                        event: ghtui_core::types::ReviewEvent::Comment,
                        body: None,
                        comments: vec![ghtui_core::types::ReviewCommentInput {
                            path: path.clone(),
                            line: *line,
                            body,
                        }],
                    };
                    if let Some(ref mut d) = state.pr_detail {
                        d.diff_comment_target = None;
                        d.diff_comment_editor = ghtui_core::editor::TextEditor::new();
                    }
                    return vec![Command::SubmitReview(repo.clone(), number, input)];
                }
            }
            vec![]
        }
        Message::PrDiffInsertSuggestion => {
            if let Some(ref mut detail) = state.pr_detail {
                if let Some((ref path, line)) = detail.diff_comment_target.clone() {
                    // Find the original code at this line
                    let original_code = detail
                        .diff
                        .as_ref()
                        .and_then(|files| {
                            files.iter().find(|f| f.filename == *path).and_then(|f| {
                                f.hunks.iter().find_map(|h| {
                                    h.lines.iter().find_map(|dl| {
                                        if dl.new_line == Some(line) || dl.old_line == Some(line) {
                                            Some(dl.content.clone())
                                        } else {
                                            None
                                        }
                                    })
                                })
                            })
                        })
                        .unwrap_or_default();

                    let template = format!("```suggestion\n{}\n```", original_code);
                    for c in template.chars() {
                        if c == '\n' {
                            detail.diff_comment_editor.insert_newline();
                        } else {
                            detail.diff_comment_editor.insert_char(c);
                        }
                    }
                    detail.diff_comment_editor.move_up();
                }
            }
            vec![]
        }
        Message::PrDiffCommentCancel => {
            if let Some(ref mut detail) = state.pr_detail {
                detail.diff_comment_target = None;
                detail.diff_comment_editor = ghtui_core::editor::TextEditor::new();
            }
            vec![]
        }
        Message::PrDiffClearSelection => {
            if let Some(ref mut detail) = state.pr_detail {
                detail.diff_select_anchor = None;
            }
            vec![]
        }
        Message::PrDiffToggleTree => {
            if let Some(ref mut detail) = state.pr_detail {
                detail.show_file_tree = !detail.show_file_tree;
                if !detail.show_file_tree {
                    detail.file_tree_focused = false;
                }
            }
            vec![]
        }
        Message::PrDiffTreeFocus => {
            if let Some(ref mut detail) = state.pr_detail {
                if detail.show_file_tree {
                    detail.file_tree_focused = !detail.file_tree_focused;
                }
            }
            vec![]
        }
        Message::PrDiffTreeUp => {
            if let Some(ref mut detail) = state.pr_detail {
                detail.file_tree_selected = detail.file_tree_selected.saturating_sub(1);
            }
            vec![]
        }
        Message::PrDiffTreeDown => {
            if let Some(ref mut detail) = state.pr_detail {
                if let Some(ref files) = detail.diff {
                    let max = files.len().saturating_sub(1);
                    detail.file_tree_selected = (detail.file_tree_selected + 1).min(max);
                }
            }
            vec![]
        }
        Message::PrDiffTreeSelect => {
            // Jump diff cursor to selected file
            if let Some(ref mut detail) = state.pr_detail {
                if let Some(ref files) = detail.diff {
                    let target_fi = detail.file_tree_selected;
                    // Calculate line position of target file header
                    let summary_lines = files.len() + 3;
                    let mut line = summary_lines;
                    for (fi, file) in files.iter().enumerate() {
                        if fi == target_fi {
                            detail.diff_cursor = line;
                            detail.file_tree_focused = false; // switch focus to diff
                            // Expand if collapsed
                            detail.diff_collapsed.remove(&fi);
                            break;
                        }
                        let collapsed = detail.diff_collapsed.contains(&fi);
                        if collapsed {
                            line += 1;
                        } else {
                            line +=
                                1 + file.hunks.iter().map(|h| 1 + h.lines.len()).sum::<usize>() + 1;
                        }
                    }
                }
            }
            vec![]
        }

        // Issues
        Message::IssueListLoaded(issues, pagination, filters) => {
            state.loading.remove("issue_list");
            state.issue_list = Some(IssueListState::with_filters(issues, pagination, filters));
            // Also fetch pinned issue numbers
            if let Some(ref repo) = state.current_repo {
                vec![Command::FetchPinnedIssues(repo.clone())]
            } else {
                vec![]
            }
        }
        Message::IssuePinnedNumbersLoaded(numbers) => {
            if let Some(ref mut list) = state.issue_list {
                list.pinned_numbers = numbers.clone();
                // Sort: pinned issues first, then original order
                list.items.sort_by(|a, b| {
                    let a_pinned = numbers.contains(&a.number);
                    let b_pinned = numbers.contains(&b.number);
                    b_pinned.cmp(&a_pinned) // true (pinned) comes first
                });
            }
            vec![]
        }
        Message::IssueDetailLoaded(detail) => {
            state.loading.remove("issue_detail");
            let old = state.issue_detail.take();
            let mut new_state = IssueDetailState::new(*detail);
            if let Some(old) = old {
                new_state.scroll = old.scroll;
                new_state.focus = old.focus;
            }
            state.issue_detail = Some(new_state);
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
        Message::IssueSortCycle => {
            if let Some(ref repo) = state.current_repo {
                let filters = if let Some(ref mut list) = state.issue_list {
                    list.cycle_sort();
                    list.filters.clone()
                } else {
                    IssueFilters::default()
                };
                state.loading.insert("issue_list".to_string());
                vec![Command::FetchIssueList(repo.clone(), filters, 1)]
            } else {
                vec![]
            }
        }
        Message::IssueLockToggle => {
            if let Some(ref detail) = state.issue_detail {
                if let Some(ref repo) = state.current_repo {
                    let number = detail.detail.issue.number;
                    let locked = detail.detail.issue.locked;
                    return if locked {
                        vec![Command::UnlockIssue(repo.clone(), number)]
                    } else {
                        vec![Command::LockIssue(repo.clone(), number)]
                    };
                }
            }
            vec![]
        }
        Message::IssuePinToggle => {
            if let Some(ref detail) = state.issue_detail {
                if let Some(ref repo) = state.current_repo {
                    let number = detail.detail.issue.number;
                    // Try pin first; if already pinned, the API will error and we unpin
                    return vec![Command::PinIssue(repo.clone(), number)];
                }
            }
            vec![]
        }
        Message::IssueFilterByLabel(label) => {
            if let Some(ref mut list) = state.issue_list {
                list.filters.label = Some(label);
            }
            if let Some(ref repo) = state.current_repo {
                if let Some(ref list) = state.issue_list {
                    state.loading.insert("issue_list".to_string());
                    return vec![Command::FetchIssueList(
                        repo.clone(),
                        list.filters.clone(),
                        1,
                    )];
                }
            }
            vec![]
        }
        Message::IssueFilterByAuthor(author) => {
            if let Some(ref mut list) = state.issue_list {
                list.filters.author = Some(author);
            }
            if let Some(ref repo) = state.current_repo {
                if let Some(ref list) = state.issue_list {
                    state.loading.insert("issue_list".to_string());
                    return vec![Command::FetchIssueList(
                        repo.clone(),
                        list.filters.clone(),
                        1,
                    )];
                }
            }
            vec![]
        }
        Message::IssueFilterByAssignee(assignee) => {
            if let Some(ref mut list) = state.issue_list {
                list.filters.assignee = Some(assignee);
            }
            if let Some(ref repo) = state.current_repo {
                if let Some(ref list) = state.issue_list {
                    state.loading.insert("issue_list".to_string());
                    return vec![Command::FetchIssueList(
                        repo.clone(),
                        list.filters.clone(),
                        1,
                    )];
                }
            }
            vec![]
        }
        Message::IssueFilterClear => {
            if let Some(ref mut list) = state.issue_list {
                let state_filter = list.filters.state;
                list.filters = IssueFilters {
                    state: state_filter,
                    ..Default::default()
                };
            }
            if let Some(ref repo) = state.current_repo {
                if let Some(ref list) = state.issue_list {
                    state.loading.insert("issue_list".to_string());
                    return vec![Command::FetchIssueList(
                        repo.clone(),
                        list.filters.clone(),
                        1,
                    )];
                }
            }
            vec![]
        }
        Message::IssueTransfer => {
            // Use input_buffer for destination repo
            state.input_buffer.clear();
            state.modal = Some(ghtui_core::ModalKind::Confirm {
                title: "Transfer Issue".to_string(),
                message: "Enter destination repo (owner/name):".to_string(),
            });
            state.input_mode = ghtui_core::state::InputMode::Insert;
            vec![]
        }
        Message::IssueTemplatesLoaded(_templates) => {
            // Templates info stored for CreateIssue modal
            vec![]
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

        // Issue search
        Message::IssueSearchStart => {
            if let Some(ref mut list) = state.issue_list {
                list.search_mode = true;
                list.search_query.clear();
            }
            vec![]
        }
        Message::IssueSearchInput(text) => {
            if let Some(ref mut list) = state.issue_list {
                if text == "\x08" {
                    list.search_query.pop();
                } else {
                    list.search_query.push_str(&text);
                }
            }
            vec![]
        }
        Message::IssueSearchSubmit => {
            if let (Some(list), Some(repo)) = (&mut state.issue_list, &state.current_repo) {
                list.search_mode = false;
                let query = list.search_query.clone();
                if query.is_empty() {
                    // Reset to normal list
                    state.loading.insert("issue_list".to_string());
                    vec![Command::FetchIssueList(
                        repo.clone(),
                        list.filters.clone(),
                        1,
                    )]
                } else {
                    state.loading.insert("issue_list".to_string());
                    vec![Command::SearchIssues(repo.clone(), query)]
                }
            } else {
                vec![]
            }
        }
        Message::IssueSearchCancel => {
            if let Some(ref mut list) = state.issue_list {
                list.search_mode = false;
                list.search_query.clear();
            }
            vec![]
        }

        // Label picker
        Message::IssueLabelToggle => {
            if let Some(ref repo) = state.current_repo {
                // Fetch available labels and open picker
                state.loading.insert("repo_labels".to_string());
                vec![Command::FetchRepoLabels(repo.clone())]
            } else {
                vec![]
            }
        }
        Message::IssueLabelsLoaded(labels) => {
            state.loading.remove("repo_labels");
            if matches!(state.route, Route::PrDetail { .. }) {
                // Route to PR detail
                return update(state, Message::PrLabelsLoaded(labels));
            }
            if let Some(ref mut detail) = state.issue_detail {
                let current_labels: Vec<String> = detail
                    .detail
                    .issue
                    .labels
                    .iter()
                    .map(|l| l.name.clone())
                    .collect();
                detail.label_picker = Some(ghtui_core::state::issue::LabelPickerState {
                    available: labels,
                    selected_names: current_labels,
                    cursor: 0,
                });
            }
            vec![]
        }
        Message::IssueLabelSelect(idx) => {
            if let Some(ref mut detail) = state.issue_detail {
                if let Some(ref mut picker) = detail.label_picker {
                    if let Some(label) = picker.available.get(idx) {
                        let name = label.name.clone();
                        if picker.selected_names.contains(&name) {
                            picker.selected_names.retain(|n| n != &name);
                        } else {
                            picker.selected_names.push(name);
                        }
                    }
                }
            }
            vec![]
        }
        Message::IssueLabelApply => {
            if let Some(ref detail) = state.issue_detail {
                if let (Some(picker), Some(repo)) = (&detail.label_picker, &state.current_repo) {
                    let number = detail.detail.issue.number;
                    let labels = picker.selected_names.clone();
                    let cmds = vec![Command::SetIssueLabels(repo.clone(), number, labels)];
                    if let Some(ref mut detail) = state.issue_detail {
                        detail.label_picker = None;
                    }
                    return cmds;
                }
            }
            if let Some(ref mut detail) = state.issue_detail {
                detail.label_picker = None;
            }
            vec![]
        }
        Message::IssueLabelCancel => {
            if let Some(ref mut detail) = state.issue_detail {
                detail.label_picker = None;
            }
            vec![]
        }

        // Assignee picker
        Message::IssueAssigneeToggle => {
            if let Some(ref repo) = state.current_repo {
                state.loading.insert("collaborators_picker".to_string());
                vec![Command::FetchCollaboratorsForPicker(repo.clone())]
            } else {
                vec![]
            }
        }
        Message::IssueCollaboratorsLoaded(logins) => {
            state.loading.remove("collaborators_picker");
            if matches!(state.route, Route::PrDetail { .. }) {
                return update(state, Message::PrCollaboratorsLoaded(logins));
            }
            if let Some(ref mut detail) = state.issue_detail {
                let current: Vec<String> = detail
                    .detail
                    .issue
                    .assignees
                    .iter()
                    .map(|a| a.login.clone())
                    .collect();
                detail.assignee_picker = Some(ghtui_core::state::issue::AssigneePickerState {
                    available: logins,
                    selected_names: current,
                    cursor: 0,
                });
            }
            vec![]
        }
        Message::IssueAssigneeSelect(idx) => {
            if let Some(ref mut detail) = state.issue_detail {
                if let Some(ref mut picker) = detail.assignee_picker {
                    if let Some(login) = picker.available.get(idx).cloned() {
                        if picker.selected_names.contains(&login) {
                            picker.selected_names.retain(|n| n != &login);
                        } else {
                            picker.selected_names.push(login);
                        }
                    }
                }
            }
            vec![]
        }
        Message::IssueAssigneeApply => {
            if let Some(ref detail) = state.issue_detail {
                if let (Some(picker), Some(repo)) = (&detail.assignee_picker, &state.current_repo) {
                    let number = detail.detail.issue.number;
                    let assignees = picker.selected_names.clone();
                    let cmds = vec![Command::SetIssueAssignees(repo.clone(), number, assignees)];
                    if let Some(ref mut detail) = state.issue_detail {
                        detail.assignee_picker = None;
                    }
                    return cmds;
                }
            }
            if let Some(ref mut detail) = state.issue_detail {
                detail.assignee_picker = None;
            }
            vec![]
        }
        Message::IssueAssigneeCancel => {
            if let Some(ref mut detail) = state.issue_detail {
                detail.assignee_picker = None;
            }
            vec![]
        }

        // Comment delete
        Message::IssueDeleteComment => {
            if let Some(ref detail) = state.issue_detail {
                if let Some(idx) = detail.selected_comment() {
                    if let Some(comment) = detail.detail.comments.get(idx) {
                        if let Some(ref repo) = state.current_repo {
                            return vec![Command::DeleteComment(repo.clone(), comment.id)];
                        }
                    }
                }
            }
            vec![]
        }
        Message::CommentDeleted => {
            state.push_toast("Comment deleted".to_string(), ToastLevel::Success);
            refresh_current_view(state)
        }

        // Close/Reopen
        Message::IssueToggleState => {
            if let Some(ref detail) = state.issue_detail {
                if let Some(ref repo) = state.current_repo {
                    let number = detail.detail.issue.number;
                    return match detail.detail.issue.state {
                        IssueState::Open => vec![Command::CloseIssue(repo.clone(), number)],
                        IssueState::Closed => vec![Command::ReopenIssue(repo.clone(), number)],
                    };
                }
            }
            vec![]
        }

        // Open in browser
        Message::IssueOpenInBrowser => {
            if let Some(ref detail) = state.issue_detail {
                if let Some(ref repo) = state.current_repo {
                    let url = format!(
                        "https://github.com/{}/issues/{}",
                        repo.full_name(),
                        detail.detail.issue.number
                    );
                    return vec![Command::OpenInBrowser(url)];
                }
            }
            vec![]
        }

        // Reactions
        Message::IssueAddReaction(reaction) => {
            if let Some(ref detail) = state.issue_detail {
                if let Some(ref repo) = state.current_repo {
                    use ghtui_core::state::issue::IssueSection;
                    match &detail.focus {
                        IssueSection::Body | IssueSection::Title => {
                            let number = detail.detail.issue.number;
                            return vec![Command::AddReaction(
                                repo.clone(),
                                number,
                                reaction,
                                true,
                            )];
                        }
                        IssueSection::Comment(idx) => {
                            if let Some(comment) = detail.detail.comments.get(*idx) {
                                return vec![Command::AddReaction(
                                    repo.clone(),
                                    comment.id,
                                    reaction,
                                    false,
                                )];
                            }
                        }
                        _ => {}
                    }
                }
            }
            vec![]
        }
        Message::ReactionAdded => {
            state.push_toast("Reaction added".to_string(), ToastLevel::Success);
            refresh_current_view(state)
        }

        // Milestone picker
        Message::IssueMilestoneToggle => {
            if let Some(ref repo) = state.current_repo {
                state.loading.insert("milestones".to_string());
                vec![Command::FetchMilestones(repo.clone())]
            } else {
                vec![]
            }
        }
        Message::IssueMilestonesLoaded(milestones) => {
            state.loading.remove("milestones");
            // Route to PR detail if on PR view
            if matches!(state.route, Route::PrDetail { .. }) {
                if let Some(ref mut detail) = state.pr_detail {
                    let current = detail.detail.pr.milestone.as_ref().map(|m| m.number as u64);
                    detail.milestone_picker =
                        Some(ghtui_core::state::issue::MilestonePickerState {
                            available: milestones,
                            selected: current,
                            cursor: 0,
                        });
                }
                return vec![];
            }
            if let Some(ref mut detail) = state.issue_detail {
                let current = detail
                    .detail
                    .issue
                    .milestone
                    .as_ref()
                    .map(|m| m.number as u64);
                detail.milestone_picker = Some(ghtui_core::state::issue::MilestonePickerState {
                    available: milestones,
                    selected: current,
                    cursor: 0,
                });
            }
            vec![]
        }
        Message::IssueMilestoneSelect(idx) => {
            if let Some(ref mut detail) = state.issue_detail {
                if let Some(ref mut picker) = detail.milestone_picker {
                    if let Some(ms) = picker.available.get(idx) {
                        let num = ms.number as u64;
                        if picker.selected == Some(num) {
                            picker.selected = None;
                        } else {
                            picker.selected = Some(num);
                        }
                    }
                }
            }
            vec![]
        }
        Message::IssueMilestoneApply => {
            if let Some(ref detail) = state.issue_detail {
                if let (Some(picker), Some(repo)) = (&detail.milestone_picker, &state.current_repo)
                {
                    let number = detail.detail.issue.number;
                    let ms = picker.selected;
                    let cmds = vec![Command::SetMilestone(repo.clone(), number, ms)];
                    if let Some(ref mut detail) = state.issue_detail {
                        detail.milestone_picker = None;
                    }
                    return cmds;
                }
            }
            if let Some(ref mut detail) = state.issue_detail {
                detail.milestone_picker = None;
            }
            vec![]
        }
        Message::IssueMilestoneClear => {
            if let Some(ref detail) = state.issue_detail {
                if let Some(ref repo) = state.current_repo {
                    let number = detail.detail.issue.number;
                    if let Some(ref mut d) = state.issue_detail {
                        d.milestone_picker = None;
                    }
                    return vec![Command::SetMilestone(repo.clone(), number, None)];
                }
            }
            vec![]
        }
        Message::IssueMilestoneCancel => {
            if let Some(ref mut detail) = state.issue_detail {
                detail.milestone_picker = None;
            }
            vec![]
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
                use ghtui_core::state::issue::IssueSection;
                match &detail.focus {
                    IssueSection::Title => detail.start_edit_title(),
                    IssueSection::Body => detail.start_edit_body(),
                    IssueSection::Comment(idx) => detail.start_edit_comment(*idx),
                    _ => {}
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
                if let Some(idx) = detail.selected_comment() {
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
        Message::IssueEditDelete => {
            if let Some(ref mut detail) = state.issue_detail {
                detail.editor.delete();
            }
            vec![]
        }
        Message::IssueEditTab => {
            if let Some(ref mut detail) = state.issue_detail {
                detail.editor.insert_tab();
            }
            vec![]
        }
        Message::IssueEditWordLeft => {
            if let Some(ref mut detail) = state.issue_detail {
                detail.editor.move_word_left();
            }
            vec![]
        }
        Message::IssueEditWordRight => {
            if let Some(ref mut detail) = state.issue_detail {
                detail.editor.move_word_right();
            }
            vec![]
        }
        Message::IssueEditPageUp => {
            if let Some(ref mut detail) = state.issue_detail {
                detail.editor.page_up();
            }
            vec![]
        }
        Message::IssueEditPageDown => {
            if let Some(ref mut detail) = state.issue_detail {
                detail.editor.page_down();
            }
            vec![]
        }
        Message::IssueEditUndo => {
            if let Some(ref mut detail) = state.issue_detail {
                detail.editor.undo();
            }
            vec![]
        }
        Message::IssueEditRedo => {
            if let Some(ref mut detail) = state.issue_detail {
                detail.editor.redo();
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
                                state.push_toast(
                                    "Title cannot be empty".to_string(),
                                    ToastLevel::Warning,
                                );
                                return vec![];
                            }
                            vec![Command::UpdateIssue(
                                repo.clone(),
                                number,
                                Some(title),
                                None,
                            )]
                        }
                        Some(InlineEditTarget::IssueBody) => {
                            let body = detail.editor_text();
                            let number = detail.detail.issue.number;
                            vec![Command::UpdateIssue(repo.clone(), number, None, Some(body))]
                        }
                        Some(InlineEditTarget::Comment(idx)) => {
                            if let Some(comment) = detail.detail.comments.get(*idx) {
                                let body = detail.editor_text();
                                if body.trim().is_empty() {
                                    state.push_toast(
                                        "Comment cannot be empty".to_string(),
                                        ToastLevel::Warning,
                                    );
                                    return vec![];
                                }
                                vec![Command::UpdateComment(
                                    repo.clone(),
                                    detail.detail.issue.number,
                                    comment.id,
                                    body,
                                )]
                            } else {
                                vec![]
                            }
                        }
                        Some(InlineEditTarget::NewComment | InlineEditTarget::QuoteReply(_)) => {
                            let body = detail.editor_text();
                            if body.trim().is_empty() {
                                state.push_toast(
                                    "Comment cannot be empty".to_string(),
                                    ToastLevel::Warning,
                                );
                                return vec![];
                            }
                            let number = detail.detail.issue.number;
                            vec![Command::AddComment(repo.clone(), number, body)]
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
        Message::RunsLoaded(runs, pagination, filters) => {
            state.loading.remove("actions_list");
            let old_workflows = state
                .actions_list
                .as_ref()
                .map(|l| l.workflows.clone())
                .unwrap_or_default();
            let mut new_state = ActionsListState::with_filters(runs, pagination, filters);
            new_state.workflows = old_workflows;
            state.actions_list = Some(new_state);
            vec![]
        }
        Message::RunDetailLoaded(detail) => {
            state.loading.remove("action_detail");
            let workflow_id = detail.run.workflow_id;
            state.action_detail = Some(ActionDetailState::new(*detail));
            // Fetch workflow file if we know the path
            if let Some(ref repo) = state.current_repo {
                let workflow_path = state
                    .actions_list
                    .as_ref()
                    .and_then(|l| l.workflows.iter().find(|w| w.id == workflow_id))
                    .map(|w| w.path.clone());
                if let Some(path) = workflow_path {
                    return vec![Command::FetchWorkflowFile(repo.clone(), path)];
                }
            }
            vec![]
        }
        Message::JobLogLoaded(job_id, lines) => {
            state.loading.remove("job_log");
            if let Some(ref mut detail) = state.action_detail {
                let _ = job_id;
                // Check if the selected job is still running → enable log streaming
                let job_in_progress =
                    detail
                        .detail
                        .jobs
                        .get(detail.selected_job)
                        .is_some_and(|j| {
                            matches!(
                                j.status,
                                Some(ghtui_core::types::RunStatus::InProgress)
                                    | Some(ghtui_core::types::RunStatus::Queued)
                            )
                        });
                detail.log_streaming = job_in_progress;
                detail.log_poll_counter = 0;
                // Auto-scroll to bottom if streaming
                if detail.auto_scroll && detail.log_streaming {
                    let line_count = lines.len();
                    detail.log_scroll = line_count.saturating_sub(20);
                }
                detail.set_log(lines);
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
        Message::WorkflowsLoaded(workflows) => {
            if let Some(ref mut list) = state.actions_list {
                list.workflows = workflows;
            }
            vec![]
        }
        Message::ActionsToggleStatus => {
            if let Some(ref repo) = state.current_repo {
                let filters = if let Some(ref mut list) = state.actions_list {
                    list.cycle_status();
                    list.filters.clone()
                } else {
                    ActionsFilters::default()
                };
                state.loading.insert("actions_list".to_string());
                vec![Command::FetchRuns(repo.clone(), filters, 1)]
            } else {
                vec![]
            }
        }
        Message::ActionsCycleEvent => {
            if let Some(ref repo) = state.current_repo {
                let filters = if let Some(ref mut list) = state.actions_list {
                    list.cycle_event();
                    list.filters.clone()
                } else {
                    ActionsFilters::default()
                };
                state.loading.insert("actions_list".to_string());
                vec![Command::FetchRuns(repo.clone(), filters, 1)]
            } else {
                vec![]
            }
        }
        Message::ActionsNextPage => {
            if let (Some(repo), Some(list)) = (&state.current_repo, &state.actions_list) {
                if list.pagination.has_next {
                    let next_page = list.pagination.page + 1;
                    let filters = list.filters.clone();
                    state.loading.insert("actions_list".to_string());
                    vec![Command::FetchRuns(repo.clone(), filters, next_page)]
                } else {
                    vec![]
                }
            } else {
                vec![]
            }
        }
        Message::ActionsPrevPage => {
            if let (Some(repo), Some(list)) = (&state.current_repo, &state.actions_list) {
                if list.pagination.page > 1 {
                    let prev_page = list.pagination.page - 1;
                    let filters = list.filters.clone();
                    state.loading.insert("actions_list".to_string());
                    vec![Command::FetchRuns(repo.clone(), filters, prev_page)]
                } else {
                    vec![]
                }
            } else {
                vec![]
            }
        }
        Message::ActionsSearchStart => {
            if let Some(ref mut list) = state.actions_list {
                list.search_mode = true;
                list.search_query.clear();
            }
            vec![]
        }
        Message::ActionsSearchInput(text) => {
            if let Some(ref mut list) = state.actions_list {
                if text == "\x08" {
                    list.search_query.pop();
                } else {
                    list.search_query.push_str(&text);
                }
            }
            vec![]
        }
        Message::ActionsSearchSubmit => {
            if let (Some(list), Some(repo)) = (&mut state.actions_list, &state.current_repo) {
                list.search_mode = false;
                let query = list.search_query.clone();
                if query.is_empty() {
                    state.loading.insert("actions_list".to_string());
                    vec![Command::FetchRuns(repo.clone(), list.filters.clone(), 1)]
                } else {
                    // Filter by branch name (closest to search for actions)
                    let mut filters = list.filters.clone();
                    filters.branch = Some(query);
                    list.filters = filters.clone();
                    state.loading.insert("actions_list".to_string());
                    vec![Command::FetchRuns(repo.clone(), filters, 1)]
                }
            } else {
                vec![]
            }
        }
        Message::ActionsSearchCancel => {
            if let Some(ref mut list) = state.actions_list {
                list.search_mode = false;
                list.search_query.clear();
            }
            vec![]
        }
        Message::ActionsFilterClear => {
            if let Some(ref mut list) = state.actions_list {
                list.filters = ActionsFilters::default();
            }
            if let Some(ref repo) = state.current_repo {
                state.loading.insert("actions_list".to_string());
                vec![Command::FetchRuns(
                    repo.clone(),
                    ActionsFilters::default(),
                    1,
                )]
            } else {
                vec![]
            }
        }
        Message::ActionsSelectWorkflow(workflow_id) => {
            if let Some(ref repo) = state.current_repo {
                let filters = if let Some(ref mut list) = state.actions_list {
                    list.select_workflow(workflow_id);
                    list.filters.clone()
                } else {
                    ActionsFilters::default()
                };
                state.loading.insert("actions_list".to_string());
                vec![Command::FetchRuns(repo.clone(), filters, 1)]
            } else {
                vec![]
            }
        }
        Message::ActionsOpenInBrowser => {
            if let Some(ref list) = state.actions_list {
                if let Some(run) = list.selected_run() {
                    return vec![Command::OpenInBrowser(run.html_url.clone())];
                }
            }
            vec![]
        }
        Message::ActionsCancelRun => {
            if let (Some(list), Some(repo)) = (&state.actions_list, &state.current_repo) {
                if let Some(run) = list.selected_run() {
                    return vec![Command::CancelRun(repo.clone(), run.id)];
                }
            }
            vec![]
        }
        Message::ActionsRerunRun => {
            if let (Some(list), Some(repo)) = (&state.actions_list, &state.current_repo) {
                if let Some(run) = list.selected_run() {
                    return vec![Command::RerunRun(repo.clone(), run.id)];
                }
            }
            vec![]
        }
        // Workflow sidebar
        Message::ActionsToggleWorkflowSidebar => {
            if let Some(ref mut list) = state.actions_list {
                list.show_workflow_sidebar = !list.show_workflow_sidebar;
                list.workflow_sidebar_focused = list.show_workflow_sidebar;
            }
            vec![]
        }
        Message::ActionsWorkflowSidebarUp => {
            if let Some(ref mut list) = state.actions_list {
                list.workflow_sidebar_selected = list.workflow_sidebar_selected.saturating_sub(1);
            }
            vec![]
        }
        Message::ActionsWorkflowSidebarDown => {
            if let Some(ref mut list) = state.actions_list {
                let max = list.workflows.len(); // 0="All", 1..N=workflows
                if list.workflow_sidebar_selected < max {
                    list.workflow_sidebar_selected += 1;
                }
            }
            vec![]
        }
        Message::ActionsWorkflowSidebarSelect => {
            if let (Some(list), Some(repo)) = (&mut state.actions_list, &state.current_repo) {
                let workflow_id = if list.workflow_sidebar_selected == 0 {
                    None
                } else {
                    list.workflows
                        .get(list.workflow_sidebar_selected - 1)
                        .map(|w| w.id)
                };
                list.filters.workflow_id = workflow_id;
                list.workflow_sidebar_focused = false;
                return vec![Command::FetchRuns(repo.clone(), list.filters.clone(), 1)];
            }
            vec![]
        }
        // Dispatch modal
        Message::ActionsDispatchOpen => {
            if let (Some(list), Some(repo)) = (&state.actions_list, &state.current_repo) {
                let workflow = if list.workflow_sidebar_selected > 0 {
                    list.workflows.get(list.workflow_sidebar_selected - 1)
                } else if !list.workflows.is_empty() {
                    Some(&list.workflows[0])
                } else {
                    None
                };
                if let Some(wf) = workflow {
                    return vec![Command::FetchWorkflowInputs(
                        repo.clone(),
                        wf.id,
                        wf.name.clone(),
                        wf.path.clone(),
                    )];
                }
            }
            vec![]
        }
        Message::WorkflowInputsLoaded(workflow_id, workflow_name, inputs) => {
            if let Some(ref mut list) = state.actions_list {
                use ghtui_core::state::actions::{DispatchInputField, DispatchState};
                let fields: Vec<DispatchInputField> = inputs
                    .iter()
                    .map(|i| DispatchInputField {
                        name: i.name.clone(),
                        value: i.default.clone().unwrap_or_default(),
                        input_type: i.input_type.clone(),
                        required: i.required,
                        options: i.options.clone(),
                        description: i.description.clone(),
                    })
                    .collect();
                list.dispatch = Some(DispatchState {
                    workflow_id,
                    workflow_name,
                    git_ref: "main".to_string(),
                    inputs: fields,
                    focused_field: 0,
                    editing: false,
                    edit_buffer: String::new(),
                });
            }
            vec![]
        }
        Message::ActionsDispatchClose => {
            if let Some(ref mut list) = state.actions_list {
                list.dispatch = None;
            }
            vec![]
        }
        Message::ActionsDispatchFieldNext => {
            if let Some(ref mut list) = state.actions_list {
                if let Some(ref mut d) = list.dispatch {
                    if d.focused_field < d.inputs.len() {
                        d.focused_field += 1;
                    }
                }
            }
            vec![]
        }
        Message::ActionsDispatchFieldPrev => {
            if let Some(ref mut list) = state.actions_list {
                if let Some(ref mut d) = list.dispatch {
                    d.focused_field = d.focused_field.saturating_sub(1);
                }
            }
            vec![]
        }
        Message::ActionsDispatchEditStart => {
            if let Some(ref mut list) = state.actions_list {
                if let Some(ref mut d) = list.dispatch {
                    d.editing = true;
                    d.edit_buffer = if d.focused_field == 0 {
                        d.git_ref.clone()
                    } else {
                        d.inputs
                            .get(d.focused_field - 1)
                            .map(|f| f.value.clone())
                            .unwrap_or_default()
                    };
                }
            }
            vec![]
        }
        Message::ActionsDispatchEditChar(c) => {
            if let Some(ref mut list) = state.actions_list {
                if let Some(ref mut d) = list.dispatch {
                    if d.editing {
                        d.edit_buffer.push(c);
                    }
                }
            }
            vec![]
        }
        Message::ActionsDispatchEditBackspace => {
            if let Some(ref mut list) = state.actions_list {
                if let Some(ref mut d) = list.dispatch {
                    if d.editing {
                        d.edit_buffer.pop();
                    }
                }
            }
            vec![]
        }
        Message::ActionsDispatchEditDone => {
            if let Some(ref mut list) = state.actions_list {
                if let Some(ref mut d) = list.dispatch {
                    if d.editing {
                        if d.focused_field == 0 {
                            d.git_ref = d.edit_buffer.clone();
                        } else if let Some(field) = d.inputs.get_mut(d.focused_field - 1) {
                            field.value = d.edit_buffer.clone();
                        }
                        d.editing = false;
                    }
                }
            }
            vec![]
        }
        Message::ActionsDispatchSubmit => {
            if let (Some(list), Some(repo)) = (&mut state.actions_list, &state.current_repo) {
                if let Some(dispatch) = list.dispatch.take() {
                    let mut inputs = serde_json::Map::new();
                    for field in &dispatch.inputs {
                        if !field.value.is_empty() {
                            inputs.insert(
                                field.name.clone(),
                                serde_json::Value::String(field.value.clone()),
                            );
                        }
                    }
                    return vec![Command::DispatchWorkflow(
                        repo.clone(),
                        dispatch.workflow_id,
                        dispatch.git_ref,
                        serde_json::Value::Object(inputs),
                    )];
                }
            }
            vec![]
        }
        Message::ActionDetailToggleStep(_) => {
            if let Some(ref mut detail) = state.action_detail {
                detail.toggle_steps_collapsed();
            }
            vec![]
        }
        Message::ActionDetailFocusJobs => {
            if let Some(ref mut detail) = state.action_detail {
                detail.focus = ActionDetailFocus::Jobs;
            }
            vec![]
        }
        Message::ActionDetailFocusLog => {
            if let Some(ref mut detail) = state.action_detail {
                detail.focus = ActionDetailFocus::Log;
            }
            vec![]
        }
        Message::ActionDetailActionBarFocus => {
            if let Some(ref mut detail) = state.action_detail {
                detail.focus = if detail.focus == ActionDetailFocus::ActionBar {
                    ActionDetailFocus::Jobs
                } else {
                    ActionDetailFocus::ActionBar
                };
            }
            vec![]
        }
        Message::ActionDetailActionBarLeft => {
            if let Some(ref mut detail) = state.action_detail {
                detail.action_bar_selected = detail.action_bar_selected.saturating_sub(1);
            }
            vec![]
        }
        Message::ActionDetailActionBarRight => {
            if let Some(ref mut detail) = state.action_detail {
                let max = detail.action_bar_items.len().saturating_sub(1);
                detail.action_bar_selected = (detail.action_bar_selected + 1).min(max);
            }
            vec![]
        }
        Message::ActionDetailActionBarSelect => {
            if let (Some(detail), Some(repo)) = (&state.action_detail, &state.current_repo) {
                let run_id = detail.detail.run.id;
                let action = detail.action_bar_items.get(detail.action_bar_selected);
                match action {
                    Some(ActionBarItem::Cancel) => {
                        return vec![Command::CancelRun(repo.clone(), run_id)];
                    }
                    Some(ActionBarItem::Rerun) => {
                        return vec![Command::RerunRun(repo.clone(), run_id)];
                    }
                    Some(ActionBarItem::RerunFailed) => {
                        return vec![Command::RerunFailedJobs(repo.clone(), run_id)];
                    }
                    Some(ActionBarItem::Delete) => {
                        return vec![Command::DeleteRun(repo.clone(), run_id)];
                    }
                    Some(ActionBarItem::OpenInBrowser) => {
                        return vec![Command::OpenInBrowser(detail.detail.run.html_url.clone())];
                    }
                    None => {}
                }
            }
            vec![]
        }
        Message::ActionDetailOpenInBrowser => {
            if let Some(ref detail) = state.action_detail {
                return vec![Command::OpenInBrowser(detail.detail.run.html_url.clone())];
            }
            vec![]
        }
        Message::RunRerunFailed(run_id) => {
            state.push_toast(
                format!("Failed jobs of run #{} restarted", run_id),
                ToastLevel::Success,
            );
            refresh_current_view(state)
        }
        Message::RunDeleted(run_id) => {
            state.push_toast(format!("Run #{} deleted", run_id), ToastLevel::Info);
            // Go back to list after delete
            state.go_back();
            refresh_current_view(state)
        }
        Message::ArtifactsLoaded(artifacts) => {
            if let Some(ref mut detail) = state.action_detail {
                detail.artifacts = artifacts;
            }
            vec![]
        }
        Message::ArtifactDownloaded(name, url) => {
            if let Some(ref mut detail) = state.action_detail {
                detail.downloading_artifact = None;
            }
            state.push_toast(
                format!("Artifact '{}' downloading...", name),
                ToastLevel::Success,
            );
            vec![Command::OpenInBrowser(url)]
        }
        Message::WorkflowDispatched => {
            state.push_toast("Workflow dispatched".to_string(), ToastLevel::Success);
            refresh_current_view(state)
        }
        Message::WorkflowFileLoaded(content) => {
            if let Some(ref mut detail) = state.action_detail {
                detail.workflow_file = Some(content);
            }
            vec![]
        }
        Message::PendingDeploymentsLoaded(deployments) => {
            if let Some(ref mut detail) = state.action_detail {
                detail.pending_deployments = deployments;
            }
            vec![]
        }
        Message::DeploymentApproved => {
            state.push_toast("Deployment approved".to_string(), ToastLevel::Success);
            refresh_current_view(state)
        }
        Message::DeploymentRejected => {
            state.push_toast("Deployment rejected".to_string(), ToastLevel::Info);
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
                let max = notifs.filtered_items().len().saturating_sub(1);
                notifs.selected = notifs.selected.min(max);
            }
            vec![]
        }
        Message::NotificationNavigate => {
            if let Some(ref notifs) = state.notifications {
                if let Some(notif) = notifs.selected_notification() {
                    let number = notif.extract_number();
                    let repo_parts = notif.repo_parts();
                    let subject_type = notif.subject.subject_type.clone();
                    if let (Some(number), Some((owner, name))) = (number, repo_parts) {
                        let repo = ghtui_core::types::common::RepoId::new(owner, name);
                        return match subject_type.as_str() {
                            "PullRequest" => {
                                let route = Route::PrDetail {
                                    repo,
                                    number,
                                    tab: ghtui_core::PrTab::Conversation,
                                };
                                handle_navigate(state, route)
                            }
                            "Issue" => {
                                let route = Route::IssueDetail { repo, number };
                                handle_navigate(state, route)
                            }
                            _ => vec![],
                        };
                    }
                }
            }
            vec![]
        }
        Message::NotificationMarkRead => {
            if let Some(ref notifs) = state.notifications {
                if let Some(notif) = notifs.selected_notification() {
                    let id = notif.id.clone();
                    return vec![Command::MarkNotificationRead(id)];
                }
            }
            vec![]
        }
        Message::NotificationMarkAllRead => vec![Command::MarkAllNotificationsRead],
        Message::NotificationAllMarkedRead => {
            state.push_toast(
                "All notifications marked read".to_string(),
                ToastLevel::Success,
            );
            if let Some(ref mut notifs) = state.notifications {
                notifs.items.clear();
                notifs.selected = 0;
            }
            vec![]
        }
        Message::NotificationUnsubscribe => {
            if let Some(ref notifs) = state.notifications {
                if let Some(notif) = notifs.selected_notification() {
                    let id = notif.id.clone();
                    return vec![Command::UnsubscribeThread(id)];
                }
            }
            vec![]
        }
        Message::NotificationUnsubscribed(id) => {
            state.push_toast("Unsubscribed".to_string(), ToastLevel::Info);
            if let Some(ref mut notifs) = state.notifications {
                notifs.items.retain(|n| n.id != id);
                let max = notifs.filtered_items().len().saturating_sub(1);
                notifs.selected = notifs.selected.min(max);
            }
            vec![]
        }
        Message::NotificationDone => {
            if let Some(ref notifs) = state.notifications {
                if let Some(notif) = notifs.selected_notification() {
                    let id = notif.id.clone();
                    return vec![Command::MarkThreadDone(id)];
                }
            }
            vec![]
        }
        Message::NotificationDoneResult(id) => {
            state.push_toast("Notification done".to_string(), ToastLevel::Info);
            if let Some(ref mut notifs) = state.notifications {
                notifs.items.retain(|n| n.id != id);
                let max = notifs.filtered_items().len().saturating_sub(1);
                notifs.selected = notifs.selected.min(max);
            }
            vec![]
        }
        Message::NotificationCycleReason => {
            if let Some(ref mut notifs) = state.notifications {
                notifs.cycle_reason();
            }
            vec![]
        }
        Message::NotificationCycleType => {
            if let Some(ref mut notifs) = state.notifications {
                notifs.cycle_type();
            }
            vec![]
        }
        Message::NotificationToggleGrouped => {
            if let Some(ref mut notifs) = state.notifications {
                notifs.toggle_grouped();
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
        Message::CodeFrequencyLoaded(freq) => {
            state.loading.remove("code_frequency");
            if let Some(ref mut ins) = state.insights {
                ins.code_frequency = freq;
            }
            vec![]
        }
        Message::ForksLoaded(forks) => {
            state.loading.remove("forks");
            if let Some(ref mut ins) = state.insights {
                ins.forks = forks;
            }
            vec![]
        }
        Message::DependencyGraphLoaded(deps) => {
            state.loading.remove("dependency_graph");
            if let Some(ref mut ins) = state.insights {
                ins.dependencies = deps;
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
        Message::SearchOpen => {
            // If already on search route, just toggle input mode
            if matches!(state.route, Route::Search { .. }) {
                if let Some(ref mut search) = state.search {
                    search.input_mode = true;
                    search.input_query.clear();
                }
                return vec![];
            }
            let kind = state
                .search
                .as_ref()
                .map(|s| s.kind)
                .unwrap_or(ghtui_core::types::SearchKind::Repos);
            let route = Route::Search {
                query: String::new(),
                kind,
            };
            handle_navigate(state, route)
        }
        Message::SearchInput(text) => {
            if let Some(ref mut search) = state.search {
                if text == "\x08" {
                    search.input_query.pop();
                } else {
                    search.input_query.push_str(&text);
                }
            }
            vec![]
        }
        Message::SearchSubmit => {
            if let Some(ref mut search) = state.search {
                search.input_mode = false;
                search.query = search.input_query.clone();
                search.history_cursor = None;
                if !search.query.is_empty() {
                    search.push_history(&search.query.clone());
                    let kind = search.kind;
                    let query = search.query.clone();
                    state.loading.insert("search".to_string());
                    return vec![Command::Search(query, kind, 1)];
                }
            }
            vec![]
        }
        Message::SearchCancel => {
            if let Some(ref mut search) = state.search {
                search.input_mode = false;
                search.input_query = search.query.clone();
                search.history_cursor = None;
            }
            vec![]
        }
        Message::SearchHistoryPrev => {
            if let Some(ref mut search) = state.search {
                search.history_prev();
            }
            vec![]
        }
        Message::SearchHistoryNext => {
            if let Some(ref mut search) = state.search {
                search.history_next();
            }
            vec![]
        }
        Message::SearchCycleKind => {
            if let Some(ref mut search) = state.search {
                search.cycle_kind();
                // Re-search if we have a query
                if !search.query.is_empty() {
                    let kind = search.kind;
                    let query = search.query.clone();
                    search.selected = 0;
                    state.loading.insert("search".to_string());
                    return vec![Command::Search(query, kind, 1)];
                }
            }
            vec![]
        }
        Message::SearchNavigate => {
            // Navigate to the selected search result
            if let Some(ref search) = state.search {
                if let Some(ref results) = search.results {
                    if let Some(item) = results.items.get(search.selected) {
                        match item {
                            ghtui_core::types::SearchResultItem::Issue {
                                repo,
                                number,
                                is_pr,
                                ..
                            } => {
                                if let Ok(repo_id) =
                                    repo.parse::<ghtui_core::types::common::RepoId>()
                                {
                                    if *is_pr {
                                        let route = Route::PrDetail {
                                            repo: repo_id,
                                            number: *number,
                                            tab: ghtui_core::PrTab::Conversation,
                                        };
                                        return handle_navigate(state, route);
                                    } else {
                                        let route = Route::IssueDetail {
                                            repo: repo_id,
                                            number: *number,
                                        };
                                        return handle_navigate(state, route);
                                    }
                                }
                            }
                            ghtui_core::types::SearchResultItem::Repo { full_name, .. } => {
                                return vec![Command::OpenInBrowser(format!(
                                    "https://github.com/{}",
                                    full_name
                                ))];
                            }
                            ghtui_core::types::SearchResultItem::Code { repo, path, .. } => {
                                return vec![Command::OpenInBrowser(format!(
                                    "https://github.com/{}/blob/HEAD/{}",
                                    repo, path
                                ))];
                            }
                        }
                    }
                }
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
        Message::SecurityAdvisoriesLoaded(advisories) => {
            state.loading.remove("security_advisories");
            if let Some(ref mut sec) = state.security {
                sec.advisories = advisories;
            }
            vec![]
        }
        Message::SecurityToggleDetail => {
            if let Some(ref mut sec) = state.security {
                sec.detail_open = !sec.detail_open;
                sec.detail_scroll = 0;
            }
            vec![]
        }
        Message::SecurityOpenInBrowser => {
            if let Some(ref sec) = state.security {
                let url = match sec.tab {
                    0 => sec
                        .dependabot_alerts
                        .get(sec.selected)
                        .map(|a| a.html_url.clone()),
                    1 => sec
                        .code_scanning_alerts
                        .get(sec.selected)
                        .map(|a| a.html_url.clone()),
                    2 => sec
                        .secret_scanning_alerts
                        .get(sec.selected)
                        .map(|a| a.html_url.clone()),
                    _ => None,
                };
                if let Some(url) = url {
                    return vec![Command::OpenInBrowser(url)];
                }
            }
            vec![]
        }
        Message::SecurityDismissAlert => {
            if let (Some(sec), Some(repo)) = (&state.security, &state.current_repo) {
                let repo = repo.clone();
                match sec.tab {
                    0 => {
                        // Dependabot
                        if let Some(alert) = sec.dependabot_alerts.get(sec.selected) {
                            let number = alert.number;
                            state.push_toast("Dismissing alert...".to_string(), ToastLevel::Info);
                            return vec![Command::DismissDependabotAlert(
                                repo,
                                number,
                                "no_bandwidth".to_string(),
                            )];
                        }
                    }
                    1 => {
                        // Code scanning
                        if let Some(alert) = sec.code_scanning_alerts.get(sec.selected) {
                            let number = alert.number;
                            state.push_toast("Dismissing alert...".to_string(), ToastLevel::Info);
                            return vec![Command::DismissCodeScanningAlert(
                                repo,
                                number,
                                "won't fix".to_string(),
                            )];
                        }
                    }
                    2 => {
                        // Secret scanning
                        if let Some(alert) = sec.secret_scanning_alerts.get(sec.selected) {
                            let number = alert.number;
                            state.push_toast("Dismissing alert...".to_string(), ToastLevel::Info);
                            return vec![Command::ResolveSecretScanningAlert(
                                repo,
                                number,
                                "false_positive".to_string(),
                            )];
                        }
                    }
                    _ => {}
                }
            }
            vec![]
        }
        Message::SecurityReopenAlert => {
            if let (Some(sec), Some(repo)) = (&state.security, &state.current_repo) {
                if sec.tab == 0 {
                    if let Some(alert) = sec.dependabot_alerts.get(sec.selected) {
                        let number = alert.number;
                        let repo = repo.clone();
                        state.push_toast("Reopening alert...".to_string(), ToastLevel::Info);
                        return vec![Command::ReopenDependabotAlert(repo, number)];
                    }
                }
            }
            vec![]
        }
        Message::SecurityAlertUpdated(tab) => {
            if let Some(ref repo) = state.current_repo {
                let repo = repo.clone();
                return match tab {
                    0 => {
                        state.loading.insert("dependabot".to_string());
                        vec![Command::FetchDependabotAlerts(repo)]
                    }
                    1 => {
                        state.loading.insert("code_scanning".to_string());
                        vec![Command::FetchCodeScanningAlerts(repo)]
                    }
                    2 => {
                        state.loading.insert("secret_scanning".to_string());
                        vec![Command::FetchSecretScanningAlerts(repo)]
                    }
                    _ => vec![],
                };
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
        Message::SettingsWebhooksLoaded(hooks) => {
            state.loading.remove("webhooks");
            if let Some(ref mut settings) = state.settings {
                settings.webhooks = hooks;
            }
            vec![]
        }
        Message::SettingsDeployKeysLoaded(keys) => {
            state.loading.remove("deploy_keys");
            if let Some(ref mut settings) = state.settings {
                settings.deploy_keys = keys;
            }
            vec![]
        }
        Message::SettingsRepoUpdated(repo) => {
            state.push_toast("Settings updated".to_string(), ToastLevel::Success);
            if let Some(ref mut settings) = state.settings {
                settings.repo = *repo;
                settings.cancel_edit();
            }
            vec![]
        }
        Message::SettingsStartEdit(field) => {
            if let Some(ref mut settings) = state.settings {
                let edit_field = match field.as_str() {
                    "description" => SettingsEditField::Description,
                    "default_branch" => SettingsEditField::DefaultBranch,
                    "topics" => SettingsEditField::Topics,
                    _ => return vec![],
                };
                settings.start_edit(edit_field);
            }
            vec![]
        }
        Message::SettingsEditChar(c) => {
            if let Some(ref mut settings) = state.settings {
                settings.edit_buffer.push(c);
            }
            vec![]
        }
        Message::SettingsEditBackspace => {
            if let Some(ref mut settings) = state.settings {
                settings.edit_buffer.pop();
            }
            vec![]
        }
        Message::SettingsEditCancel => {
            if let Some(ref mut settings) = state.settings {
                settings.cancel_edit();
            }
            vec![]
        }
        Message::SettingsEditSubmit => {
            if let (Some(settings), Some(repo)) = (&state.settings, &state.current_repo) {
                let updates = match &settings.editing {
                    Some(SettingsEditField::Description) => {
                        serde_json::json!({ "description": settings.edit_buffer })
                    }
                    Some(SettingsEditField::DefaultBranch) => {
                        serde_json::json!({ "default_branch": settings.edit_buffer })
                    }
                    Some(SettingsEditField::Topics) => {
                        let topics: Vec<&str> =
                            settings.edit_buffer.split(',').map(|s| s.trim()).collect();
                        serde_json::json!({ "topics": topics })
                    }
                    None => return vec![],
                };
                return vec![Command::UpdateRepo(repo.clone(), updates)];
            }
            vec![]
        }
        Message::SettingsToggleFeature(feature) => {
            if let (Some(settings), Some(repo)) = (&state.settings, &state.current_repo) {
                let current = match feature.as_str() {
                    "has_issues" => settings.repo.has_issues,
                    "has_projects" => settings.repo.has_projects,
                    "has_wiki" => settings.repo.has_wiki,
                    "has_discussions" => settings.repo.has_discussions.unwrap_or(false),
                    _ => return vec![],
                };
                let updates = serde_json::json!({ feature: !current });
                return vec![Command::UpdateRepo(repo.clone(), updates)];
            }
            vec![]
        }
        Message::SettingsToggleVisibility => {
            if let Some(ref settings) = state.settings {
                let new_private = !settings.repo.private;
                let updates = serde_json::json!({
                    "private": new_private,
                    "visibility": if new_private { "private" } else { "public" },
                });
                let label = if new_private { "private" } else { "public" };
                state.push_toast(
                    format!("Changing visibility to {}...", label),
                    ToastLevel::Warning,
                );
                if let Some(ref repo) = state.current_repo {
                    return vec![Command::UpdateRepo(repo.clone(), updates)];
                }
            }
            vec![]
        }
        Message::SettingsSidebarFocus => {
            if let Some(ref mut settings) = state.settings {
                settings.sidebar_focused = !settings.sidebar_focused;
                // Reset item selection when switching to content
                if !settings.sidebar_focused {
                    settings.selected = 0;
                }
            }
            vec![]
        }
        Message::SettingsDeleteCollaborator => {
            let info = state
                .settings
                .as_ref()
                .and_then(|s| s.collaborators.get(s.selected).map(|c| c.login.clone()));
            if let (Some(username), Some(repo)) = (info, state.current_repo.clone()) {
                state.push_toast(
                    format!("Removing collaborator @{}...", username),
                    ToastLevel::Warning,
                );
                return vec![Command::RemoveCollaborator(repo, username)];
            }
            vec![]
        }
        Message::SettingsDeleteWebhook => {
            let info = state
                .settings
                .as_ref()
                .and_then(|s| s.webhooks.get(s.selected).map(|h| h.id));
            if let (Some(hook_id), Some(repo)) = (info, state.current_repo.clone()) {
                state.push_toast("Deleting webhook...".to_string(), ToastLevel::Warning);
                return vec![Command::DeleteWebhook(repo, hook_id)];
            }
            vec![]
        }
        Message::SettingsToggleWebhook => {
            let info = state
                .settings
                .as_ref()
                .and_then(|s| s.webhooks.get(s.selected).map(|h| (h.id, !h.active)));
            if let (Some((hook_id, new_active)), Some(repo)) = (info, state.current_repo.clone()) {
                let label = if new_active { "Enabling" } else { "Disabling" };
                state.push_toast(format!("{} webhook...", label), ToastLevel::Info);
                return vec![Command::ToggleWebhook(repo, hook_id, new_active)];
            }
            vec![]
        }
        Message::SettingsDeleteDeployKey => {
            let info = state
                .settings
                .as_ref()
                .and_then(|s| s.deploy_keys.get(s.selected).map(|k| k.id));
            if let (Some(key_id), Some(repo)) = (info, state.current_repo.clone()) {
                state.push_toast("Deleting deploy key...".to_string(), ToastLevel::Warning);
                return vec![Command::DeleteDeployKey(repo, key_id)];
            }
            vec![]
        }
        Message::SettingsDeleteBranchProtection => {
            let info = state.settings.as_ref().and_then(|s| {
                s.branch_protections
                    .get(s.selected)
                    .map(|bp| bp.pattern.clone())
            });
            if let (Some(branch), Some(repo)) = (info, state.current_repo.clone()) {
                state.push_toast(
                    format!("Deleting protection for '{}'...", branch),
                    ToastLevel::Warning,
                );
                return vec![Command::DeleteBranchProtection(repo, branch)];
            }
            vec![]
        }
        Message::SettingsToggleBranchEnforceAdmins => {
            if let Some(ref settings) = state.settings {
                if let Some(bp) = settings.branch_protections.get(settings.selected) {
                    let current = bp
                        .enforce_admins
                        .as_ref()
                        .map(|e| e.enabled)
                        .unwrap_or(false);
                    let pattern = bp.pattern.clone();
                    let label = if current { "Disabling" } else { "Enabling" };
                    state.push_toast(
                        format!("{} enforce admins for '{}'...", label, pattern),
                        ToastLevel::Info,
                    );
                    if let Some(ref repo) = state.current_repo {
                        return vec![Command::ToggleBranchEnforceAdmins(
                            repo.clone(),
                            pattern,
                            !current,
                        )];
                    }
                }
            }
            vec![]
        }
        Message::SettingsItemUpdated(tab) => {
            state.push_toast("Settings updated".to_string(), ToastLevel::Success);
            if let Some(ref mut settings) = state.settings {
                settings.selected = 0;
            }
            if let Some(ref repo) = state.current_repo {
                let repo = repo.clone();
                return match tab {
                    1 => {
                        state.loading.insert("branch_protections".to_string());
                        vec![Command::FetchBranchProtections(repo)]
                    }
                    2 => {
                        state.loading.insert("collaborators".to_string());
                        vec![Command::FetchCollaborators(repo)]
                    }
                    3 => {
                        state.loading.insert("webhooks".to_string());
                        vec![Command::FetchWebhooks(repo)]
                    }
                    4 => {
                        state.loading.insert("deploy_keys".to_string());
                        vec![Command::FetchDeployKeys(repo)]
                    }
                    _ => vec![],
                };
            }
            vec![]
        }

        // Mouse click
        Message::MouseClick(_col, row) => {
            // Row 0 = repo header, Row 1 = tab bar, Row 2+ = content
            if row == 0 {
                // Click on repo header → go to dashboard
                return update(state, Message::GoHome);
            }
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
                // Content area click — route-specific handling
                let content_row = (row - 2) as usize;

                // PR/Issue detail: row 2-6 = header, row 7 area = sub-tab bar
                // Approximate sub-tab row for detail views
                // Get terminal width for responsive checks
                let term_width = state.terminal_size.0;
                let sidebar_visible = term_width >= 80;

                match &state.route {
                    Route::PrDetail { .. } => {
                        // Dynamic header height based on editing state
                        let is_title_editing = state.pr_detail.as_ref().is_some_and(|d| {
                            matches!(
                                d.edit_target,
                                Some(ghtui_core::state::pr::PrInlineEditTarget::PrTitle)
                            )
                        });
                        let header_rows = if is_title_editing { 6 } else { 5 };
                        // Sub-tab bar is right after header (1 row)
                        let tab_row = header_rows;

                        if content_row == tab_row {
                            // Click on sub-tab bar — use dynamic tab widths
                            if let Some(ref mut detail) = state.pr_detail {
                                // Rebuild tab names with counts (matching pr_detail.rs rendering)
                                let commit_count = detail.detail.commits.len();
                                let check_count = detail.detail.checks.len();
                                let file_count = detail.diff.as_ref().map(|f| f.len()).unwrap_or(0);
                                let tabs = [
                                    "Conversation".to_string(),
                                    format!("Commits ({})", commit_count),
                                    if check_count > 0 {
                                        format!("Checks ({})", check_count)
                                    } else {
                                        "Checks".to_string()
                                    },
                                    format!("Files changed ({})", file_count),
                                ];
                                let mut x: u16 = 1; // leading space
                                for (i, name) in tabs.iter().enumerate() {
                                    let w = name.len() as u16 + 2;
                                    if _col >= x && _col < x + w {
                                        detail.tab = i;
                                        return vec![];
                                    }
                                    x += w + 3; // " | " separator
                                }
                            }
                            vec![]
                        } else if content_row > tab_row {
                            let item_index = content_row.saturating_sub(tab_row + 1);
                            handle_mouse_list_select(state, item_index)
                        } else {
                            vec![]
                        }
                    }
                    // Sidebar views: only route to sidebar if sidebar is visible
                    Route::Security { .. } | Route::Insights { .. } | Route::Settings { .. } => {
                        if sidebar_visible && _col < 30 {
                            if content_row > 0 {
                                let idx = content_row.saturating_sub(1);
                                match &state.route {
                                    Route::Security { .. } => {
                                        if let Some(ref mut sec) = state.security {
                                            if idx < sec.tab_count() {
                                                sec.tab = idx;
                                                sec.selected = 0;
                                            }
                                        }
                                    }
                                    Route::Insights { .. } => {
                                        if let Some(ref mut ins) = state.insights {
                                            if idx < ins.tab_count() {
                                                ins.tab = idx;
                                            }
                                        }
                                    }
                                    Route::Settings { .. } => {
                                        if let Some(ref mut settings) = state.settings {
                                            if idx < settings.tab_count() {
                                                settings.tab = idx;
                                                settings.selected = 0;
                                            }
                                        }
                                    }
                                    _ => {}
                                }
                            }
                            vec![]
                        } else if content_row > 0 {
                            let item_index = content_row.saturating_sub(1);
                            handle_mouse_list_select(state, item_index)
                        } else {
                            vec![]
                        }
                    }
                    // Code view: left 35 cols = file tree click
                    Route::Code { .. } => {
                        if sidebar_visible && _col < 35 {
                            // File tree click
                            if content_row > 0 {
                                let idx = content_row.saturating_sub(1);
                                if let Some(ref mut code) = state.code {
                                    code.selected = idx.min(if code.tree_loaded {
                                        code.tree_visible.len().saturating_sub(1)
                                    } else {
                                        code.entries.len().saturating_sub(1)
                                    });
                                }
                            }
                            vec![]
                        } else if content_row > 0 {
                            let item_index = content_row.saturating_sub(1);
                            handle_mouse_list_select(state, item_index)
                        } else {
                            vec![]
                        }
                    }
                    // List views with filter bar: extra row offset
                    Route::IssueList { .. } | Route::PrList { .. } | Route::ActionsList { .. } => {
                        // filter bar (1) + border (1) + header (1) = 3 rows before items
                        if content_row > 2 {
                            let item_index = content_row.saturating_sub(3);
                            handle_mouse_list_select(state, item_index)
                        } else {
                            vec![]
                        }
                    }
                    // Default: just border offset
                    _ => {
                        if content_row > 0 {
                            let item_index = content_row.saturating_sub(1);
                            handle_mouse_list_select(state, item_index)
                        } else {
                            vec![]
                        }
                    }
                }
            } else {
                vec![]
            }
        }

        // Mouse double-click — open/toggle the clicked item
        Message::MouseDoubleClick(_col, row) => {
            let term_width = state.terminal_size.0;
            let sidebar_visible = term_width >= 80;

            if row >= 2 {
                let content_row = (row - 2) as usize;

                match &state.route {
                    // Code tab: double-click file tree → open/toggle
                    Route::Code { .. } => {
                        if sidebar_visible && _col < 35 && content_row > 0 {
                            let idx = content_row.saturating_sub(1);
                            if let Some(ref mut code) = state.code {
                                code.selected = idx.min(if code.tree_loaded {
                                    code.tree_visible.len().saturating_sub(1)
                                } else {
                                    code.entries.len().saturating_sub(1)
                                });
                            }
                            // Navigate into (open file or toggle dir)
                            return update(state, Message::CodeNavigateInto);
                        }
                        vec![]
                    }
                    // List views: double-click → open detail
                    // filter bar (1) + border (1) + header (1) = 3 rows before items
                    Route::IssueList { .. } | Route::PrList { .. } | Route::ActionsList { .. } => {
                        if content_row > 2 {
                            let item_index = content_row.saturating_sub(3);
                            let cmds = handle_mouse_list_select(state, item_index);
                            if !cmds.is_empty() {
                                return cmds;
                            }
                            // Select + open
                            return update(state, Message::ListSelect(0)); // 0 = Enter/open
                        }
                        vec![]
                    }
                    _ => vec![],
                }
            } else {
                vec![]
            }
        }

        // Scroll — context-aware
        Message::ScrollUp => {
            if matches!(state.route, Route::Code { .. }) {
                if let Some(ref mut code) = state.code {
                    if code.commit_detail.is_some() {
                        code.commit_scroll = code.commit_scroll.saturating_sub(3);
                    } else if !code.sidebar_focused {
                        code.scroll = code.scroll.saturating_sub(3);
                    } else {
                        code.select_prev();
                    }
                    return vec![];
                }
            }
            if matches!(state.route, Route::Security { .. }) {
                if let Some(ref mut sec) = state.security {
                    if sec.detail_open {
                        sec.detail_scroll = sec.detail_scroll.saturating_sub(3);
                        return vec![];
                    }
                }
            }
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
            if matches!(state.route, Route::PrDetail { .. }) {
                if let Some(ref mut detail) = state.pr_detail {
                    if detail.tab == 3 {
                        detail.diff_scroll = detail.diff_scroll.saturating_sub(3);
                    } else {
                        detail.scroll = detail.scroll.saturating_sub(3);
                    }
                    return vec![];
                }
            }
            update(state, Message::ListSelect(usize::MAX))
        }
        Message::ScrollDown => {
            if matches!(state.route, Route::Code { .. }) {
                if let Some(ref mut code) = state.code {
                    if code.commit_detail.is_some() {
                        code.commit_scroll += 3;
                    } else if !code.sidebar_focused {
                        code.scroll += 3;
                    } else {
                        code.select_next();
                    }
                    return vec![];
                }
            }
            if matches!(state.route, Route::Security { .. }) {
                if let Some(ref mut sec) = state.security {
                    if sec.detail_open {
                        sec.detail_scroll += 3;
                        return vec![];
                    }
                }
            }
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
            if matches!(state.route, Route::PrDetail { .. }) {
                if let Some(ref mut detail) = state.pr_detail {
                    if detail.tab == 3 {
                        detail.diff_scroll += 3;
                    } else {
                        detail.scroll += 3;
                    }
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
                match try_move_subtab(detail.tab, delta, 4) {
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
        Message::GoHome => {
            state.route = Route::Dashboard;
            state.reset_repo_state();
            refresh_current_view(state)
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
            if idx == 8 {
                // Discussions tab (accessible via key 8 or command palette)
                if let Some(ref repo) = state.current_repo {
                    let route = Route::Discussions { repo: repo.clone() };
                    return handle_navigate(state, route);
                }
                vec![]
            } else if idx < ghtui_core::router::TAB_LABELS.len() {
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
                        match detail.selected_comment() {
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
                        if let Some(idx) = detail.selected_comment() {
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
                        state
                            .push_toast("Comment cannot be empty".to_string(), ToastLevel::Warning);
                        return vec![];
                    }
                    if let Some(ref repo) = state.current_repo {
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
                            match detail.selected_comment() {
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
                                            state.push_toast(
                                                "Comment cannot be empty".to_string(),
                                                ToastLevel::Warning,
                                            );
                                            return vec![];
                                        }
                                        {
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
                        state
                            .push_toast("Comment cannot be empty".to_string(), ToastLevel::Warning);
                        return vec![];
                    }
                    if let Some(ref repo) = state.current_repo {
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
                Some(ghtui_core::ModalKind::CreatePr) => {
                    let input = state.input_buffer.clone();
                    let mut lines = input.splitn(3, '\n');
                    let title = lines.next().unwrap_or("").trim().to_string();
                    let base = lines.next().unwrap_or("main").trim().to_string();
                    let body = lines.next().unwrap_or("").trim().to_string();
                    if title.is_empty() {
                        state.push_toast("Title cannot be empty".to_string(), ToastLevel::Warning);
                        return vec![];
                    }
                    if let Some(ref repo) = state.current_repo {
                        // Get current branch as head
                        let head = state
                            .pr_list
                            .as_ref()
                            .and(None) // no branch info in list
                            .unwrap_or_else(|| "HEAD".to_string());
                        let base = if base.is_empty() {
                            "main".to_string()
                        } else {
                            base
                        };
                        let pr_input = ghtui_core::types::CreatePrInput {
                            title,
                            body,
                            head,
                            base,
                            draft: false,
                        };
                        vec![Command::CreatePr(repo.clone(), pr_input)]
                    } else {
                        vec![]
                    }
                }
                Some(ghtui_core::ModalKind::MergePr) => {
                    let method_str = state.input_buffer.trim().to_lowercase();
                    let method = match method_str.as_str() {
                        "merge" => ghtui_core::types::MergeMethod::Merge,
                        "squash" => ghtui_core::types::MergeMethod::Squash,
                        _ => ghtui_core::types::MergeMethod::Rebase, // default
                    };
                    if let (Some(repo), Some(detail)) = (&state.current_repo, &state.pr_detail) {
                        vec![Command::MergePr(
                            repo.clone(),
                            detail.detail.pr.number,
                            method,
                        )]
                    } else {
                        vec![]
                    }
                }
                Some(ghtui_core::ModalKind::Confirm { ref title, .. })
                    if title == "Transfer Issue" =>
                {
                    let dest = state.input_buffer.trim().to_string();
                    if dest.contains('/') {
                        if let (Some(repo), Some(detail)) =
                            (&state.current_repo, &state.issue_detail)
                        {
                            vec![Command::TransferIssue(
                                repo.clone(),
                                detail.detail.issue.number,
                                dest,
                            )]
                        } else {
                            vec![]
                        }
                    } else {
                        state.push_toast(
                            "Invalid format. Use owner/repo".to_string(),
                            ToastLevel::Warning,
                        );
                        return vec![];
                    }
                }
                Some(ghtui_core::ModalKind::Confirm { ref title, .. })
                    if title == "Request Reviewers" =>
                {
                    let input = state.input_buffer.trim().to_string();
                    if input.is_empty() {
                        return vec![];
                    }
                    let reviewers: Vec<String> = input
                        .split(',')
                        .map(|s| s.trim().to_string())
                        .filter(|s| !s.is_empty())
                        .collect();
                    if let (Some(repo), Some(detail)) = (&state.current_repo, &state.pr_detail) {
                        vec![Command::SetPrReviewers(
                            repo.clone(),
                            detail.detail.pr.number,
                            reviewers,
                        )]
                    } else {
                        vec![]
                    }
                }
                Some(ghtui_core::ModalKind::Confirm { ref title, .. })
                    if title == "Change Base Branch" =>
                {
                    let base = state.input_buffer.trim().to_string();
                    if base.is_empty() {
                        state.push_toast(
                            "Branch name cannot be empty".to_string(),
                            ToastLevel::Warning,
                        );
                        return vec![];
                    }
                    if let (Some(repo), Some(detail)) = (&state.current_repo, &state.pr_detail) {
                        vec![Command::ChangePrBase(
                            repo.clone(),
                            detail.detail.pr.number,
                            base,
                        )]
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
            // Log streaming: re-fetch log every 5 ticks (~5 seconds) for in-progress jobs
            if matches!(state.route, Route::ActionDetail { .. }) {
                if let Some(ref mut detail) = state.action_detail {
                    if detail.log_streaming {
                        detail.log_poll_counter += 1;
                        if detail.log_poll_counter >= 5 {
                            detail.log_poll_counter = 0;
                            if let Some(job) = detail.detail.jobs.get(detail.selected_job) {
                                let job_id = job.id;
                                if let Route::ActionDetail { ref repo, run_id } = state.route {
                                    return vec![Command::FetchJobLog(
                                        repo.clone(),
                                        run_id,
                                        job_id,
                                    )];
                                }
                            }
                        }
                    }
                }
            }
            vec![]
        }
        Message::Resize(w, h) => {
            state.terminal_size = (w, h);
            vec![]
        }

        // System
        Message::Error(e) => {
            state.loading.clear();
            // Reset code editing state on error to avoid stuck editor
            if let Some(ref mut code) = state.code {
                if code.editing {
                    code.editing = false;
                }
            }
            state.push_toast(e.to_string(), ToastLevel::Error);
            vec![]
        }
        // Command Palette
        Message::PaletteOpen => {
            state.command_palette = Some(CommandPaletteState::new());
            vec![]
        }
        Message::PaletteClose => {
            state.command_palette = None;
            vec![]
        }
        Message::PaletteInput(query) => {
            if let Some(ref mut palette) = state.command_palette {
                palette.query = query;
                palette.filter();
            }
            vec![]
        }
        Message::PaletteSelect => {
            if let Some(palette) = state.command_palette.take() {
                if let Some(&idx) = palette.filtered.get(palette.selected) {
                    // We need to dispatch the selected message
                    // Since messages are not Clone, we reconstruct from the item
                    let items = CommandPaletteState::new().items;
                    if idx < items.len() {
                        // Re-create the message from a fresh palette
                        let fresh = CommandPaletteState::new();
                        // Use the index to pick the right item
                        return update(state, fresh.items.into_iter().nth(idx).unwrap().message);
                    }
                }
            }
            vec![]
        }
        Message::PaletteUp => {
            if let Some(ref mut palette) = state.command_palette {
                if palette.selected > 0 {
                    palette.selected -= 1;
                }
            }
            vec![]
        }
        Message::PaletteDown => {
            if let Some(ref mut palette) = state.command_palette {
                if palette.selected + 1 < palette.filtered.len() {
                    palette.selected += 1;
                }
            }
            vec![]
        }

        Message::Quit => vec![Command::Quit],
        Message::InsightsSidebarFocus => {
            if let Some(ref mut ins) = state.insights {
                ins.sidebar_focused = !ins.sidebar_focused;
            }
            vec![]
        }
        Message::SecuritySidebarFocus => {
            if let Some(ref mut sec) = state.security {
                sec.sidebar_focused = !sec.sidebar_focused;
            }
            vec![]
        }

        // Code tab
        Message::CodeTreeLoaded(nodes) => {
            state.loading.remove("code_tree");
            state.loading.remove("code_contents");
            let mut cmds = vec![];
            if let (Some(code), Some(repo)) = (&mut state.code, &state.current_repo) {
                // Detect README in root
                if code.readme_content.is_none() {
                    if let Some(readme_node) = nodes.iter().find(|n| {
                        n.depth == 0 && !n.is_dir && {
                            let lower = n.name.to_lowercase();
                            lower == "readme.md" || lower == "readme" || lower == "readme.txt"
                        }
                    }) {
                        state.loading.insert("code_readme".to_string());
                        cmds.push(Command::FetchFileContent(
                            repo.clone(),
                            readme_node.path.clone(),
                            code.git_ref.clone(),
                        ));
                    }
                }
                code.tree = nodes;
                code.tree_loaded = true;
                // Expand root-level directories by default
                code.expanded_dirs.clear();
                for node in &code.tree {
                    if node.depth == 0 && node.is_dir {
                        code.expanded_dirs.insert(node.path.clone());
                    }
                }
                code.rebuild_visible_tree();
                code.selected = 0;
            }
            cmds
        }
        Message::CodeToggleExpand => {
            if let Some(ref mut code) = state.code {
                code.toggle_expand();
            }
            vec![]
        }
        Message::CodeContentsLoaded(entries) => {
            state.loading.remove("code_contents");
            let mut cmds = vec![];
            // Auto-detect README in root directory listing
            if let (Some(code), Some(repo)) = (&mut state.code, &state.current_repo) {
                if code.current_path.is_empty() && code.readme_content.is_none() {
                    if let Some(readme) = entries.iter().find(|e| {
                        let lower = e.name.to_lowercase();
                        lower == "readme.md" || lower == "readme" || lower == "readme.txt"
                    }) {
                        state.loading.insert("code_readme".to_string());
                        cmds.push(Command::FetchFileContent(
                            repo.clone(),
                            readme.path.clone(),
                            code.git_ref.clone(),
                        ));
                    }
                }
                code.entries = entries;
                code.selected = 0;
            }
            cmds
        }
        Message::CodeFileLoaded(filename, content) => {
            state.loading.remove("code_file");
            state.loading.remove("code_readme");
            if let Some(ref mut code) = state.code {
                // If this is a README file loaded from root, store as readme_content
                let lower = filename.to_lowercase();
                if code.current_path.is_empty()
                    && (lower == "readme.md" || lower == "readme" || lower == "readme.txt")
                    && code.file_content.is_none()
                {
                    code.readme_content = Some(content);
                } else {
                    code.file_content = Some(content);
                    code.file_name = Some(filename);
                    code.scroll = 0;
                    code.sidebar_focused = false;
                }
            }
            vec![]
        }
        Message::CodeImageLoaded(filename, bytes) => {
            state.loading.remove("code_file");
            if let Some(ref mut code) = state.code {
                code.image_data = Some(bytes);
                code.file_content = None; // clear text content
                code.file_name = Some(filename);
                code.scroll = 0;
                code.sidebar_focused = false;
            }
            vec![]
        }
        Message::CodeReadmeLoaded(content) => {
            state.loading.remove("code_readme");
            if let Some(ref mut code) = state.code {
                code.readme_content = Some(content);
            }
            vec![]
        }
        Message::CodeNavigateInto => {
            if let Some(ref mut code) = state.code {
                // Tree mode
                if code.tree_loaded {
                    if let Some(node) = code.tree_selected_node().cloned() {
                        if node.is_dir {
                            code.toggle_expand();
                        } else if let Some(ref repo) = state.current_repo {
                            code.file_path = Some(node.path.clone());
                            state.loading.insert("code_file".to_string());
                            let filename = node.path.rsplit('/').next().unwrap_or(&node.path);
                            if is_image_file(filename) {
                                return vec![Command::FetchFileBytes(
                                    repo.clone(),
                                    node.path,
                                    code.git_ref.clone(),
                                )];
                            }
                            return vec![Command::FetchFileContent(
                                repo.clone(),
                                node.path,
                                code.git_ref.clone(),
                            )];
                        }
                    }
                    return vec![];
                }
                // Flat mode fallback
                if let Some(entry) = code.entries.get(code.selected).cloned() {
                    if let Some(ref repo) = state.current_repo {
                        match entry.entry_type {
                            ghtui_core::types::code::FileEntryType::Dir => {
                                code.path_stack.push(code.current_path.clone());
                                code.current_path = entry.path.clone();
                                code.file_content = None;
                                code.file_name = None;
                                code.selected = 0;
                                state.loading.insert("code_contents".to_string());
                                return vec![Command::FetchContents(
                                    repo.clone(),
                                    entry.path,
                                    code.git_ref.clone(),
                                )];
                            }
                            ghtui_core::types::code::FileEntryType::File => {
                                code.file_path = Some(entry.path.clone());
                                state.loading.insert("code_file".to_string());
                                if is_image_file(&entry.name) {
                                    return vec![Command::FetchFileBytes(
                                        repo.clone(),
                                        entry.path,
                                        code.git_ref.clone(),
                                    )];
                                }
                                return vec![Command::FetchFileContent(
                                    repo.clone(),
                                    entry.path,
                                    code.git_ref.clone(),
                                )];
                            }
                        }
                    }
                }
            }
            vec![]
        }
        Message::CodeNavigateBack => {
            if let Some(ref mut code) = state.code {
                // If viewing a file or image, close view first
                if code.file_content.is_some() || code.image_data.is_some() {
                    code.file_content = None;
                    code.file_name = None;
                    code.file_path = None;
                    code.image_data = None;
                    code.scroll = 0;
                    code.sidebar_focused = true;
                    return vec![];
                }
                // Tree mode: collapse parent directory of selected node
                if code.tree_loaded {
                    if let Some(node) = code.tree_selected_node().cloned() {
                        if node.is_dir && code.expanded_dirs.contains(&node.path) {
                            // Collapse this dir
                            code.expanded_dirs.remove(&node.path);
                            code.rebuild_visible_tree();
                        } else {
                            // Find parent dir and collapse it
                            let parts: Vec<&str> = node.path.split('/').collect();
                            if parts.len() > 1 {
                                let parent = parts[..parts.len() - 1].join("/");
                                code.expanded_dirs.remove(&parent);
                                code.rebuild_visible_tree();
                                // Move selection to the parent dir
                                for (vi, &ti) in code.tree_visible.iter().enumerate() {
                                    if let Some(n) = code.tree.get(ti) {
                                        if n.path == parent {
                                            code.selected = vi;
                                            break;
                                        }
                                    }
                                }
                            }
                        }
                    }
                    return vec![];
                }
                // Flat mode fallback: go up in directory
                if let Some(parent_path) = code.path_stack.pop() {
                    code.current_path = parent_path.clone();
                    code.selected = 0;
                    if let Some(ref repo) = state.current_repo {
                        state.loading.insert("code_contents".to_string());
                        return vec![Command::FetchContents(
                            repo.clone(),
                            parent_path,
                            code.git_ref.clone(),
                        )];
                    }
                }
            }
            vec![]
        }
        Message::CodeSidebarFocus => {
            if let Some(ref mut code) = state.code {
                code.sidebar_focused = !code.sidebar_focused;
            }
            vec![]
        }
        Message::CodeBranchesLoaded(branches) => {
            state.loading.remove("code_branches");
            if let Some(ref mut code) = state.code {
                code.branches = branches;
            }
            vec![]
        }
        Message::CodeTagsLoaded(tags) => {
            state.loading.remove("code_tags");
            if let Some(ref mut code) = state.code {
                code.tags = tags;
            }
            vec![]
        }
        Message::CodeOpenRefPicker => {
            if let Some(ref mut code) = state.code {
                code.build_ref_picker_items();
                code.ref_picker_open = true;
            }
            vec![]
        }
        Message::CodeCloseRefPicker => {
            if let Some(ref mut code) = state.code {
                code.ref_picker_open = false;
            }
            vec![]
        }
        Message::CodeSelectRef => {
            if let Some(ref mut code) = state.code {
                if let Some((ref_name, _is_branch)) =
                    code.ref_picker_items.get(code.ref_picker_selected).cloned()
                {
                    code.ref_picker_open = false;
                    code.git_ref = ref_name;
                    code.file_content = None;
                    code.file_name = None;
                    code.file_path = None;
                    code.readme_content = None;
                    code.entries.clear();
                    code.path_stack.clear();
                    code.current_path.clear();
                    code.selected = 0;
                    code.scroll = 0;
                    code.commits.clear();
                    code.commit_detail = None;
                    code.show_commits = false;
                    code.commit_selected = 0;
                    // Clear tree state
                    code.tree.clear();
                    code.expanded_dirs.clear();
                    code.tree_visible.clear();
                    code.tree_loaded = false;
                    if let Some(ref repo) = state.current_repo {
                        state.loading.insert("code_tree".to_string());
                        state.loading.insert("code_contents".to_string());
                        return vec![Command::FetchTree(repo.clone(), code.git_ref.clone())];
                    }
                }
            }
            vec![]
        }
        Message::CodeCommitsLoaded(commits) => {
            state.loading.remove("code_commits");
            if let Some(ref mut code) = state.code {
                code.commits = commits;
                code.commit_selected = 0;
            }
            vec![]
        }
        Message::CodeCommitDetailLoaded(detail) => {
            state.loading.remove("code_commit_detail");
            if let Some(ref mut code) = state.code {
                code.commit_detail = Some(*detail);
                code.commit_scroll = 0;
            }
            vec![]
        }
        Message::CodeToggleCommits => {
            if let Some(ref mut code) = state.code {
                code.show_commits = !code.show_commits;
                code.commit_detail = None;
                code.commit_scroll = 0;
                if code.show_commits && code.commits.is_empty() {
                    if let Some(ref repo) = state.current_repo {
                        state.loading.insert("code_commits".to_string());
                        return vec![Command::FetchCommits(
                            repo.clone(),
                            code.git_ref.clone(),
                            code.current_path.clone(),
                            30,
                        )];
                    }
                }
            }
            vec![]
        }
        Message::CodeOpenCommitDetail => {
            if let Some(ref mut code) = state.code {
                if let Some(entry) = code.commits.get(code.commit_selected) {
                    let sha = entry.sha.clone();
                    if let Some(ref repo) = state.current_repo {
                        state.loading.insert("code_commit_detail".to_string());
                        return vec![Command::FetchCommitDetail(repo.clone(), sha)];
                    }
                }
            }
            vec![]
        }
        Message::CodeCloseCommitDetail => {
            if let Some(ref mut code) = state.code {
                code.commit_detail = None;
                code.commit_scroll = 0;
            }
            vec![]
        }

        // Code file editing
        Message::CodeStartEdit => {
            if let Some(ref mut code) = state.code {
                if let Some(ref content) = code.file_content {
                    code.editor = ghtui_core::editor::TextEditor::from_string(content);
                    // Set viewport height based on terminal size (minus borders/status)
                    let vh = state.terminal_size.1.saturating_sub(4) as usize;
                    code.editor.set_viewport_height(vh.max(5));
                    code.editing = true;
                }
            }
            vec![]
        }
        Message::CodeEditChar(c) => {
            if let Some(ref mut code) = state.code {
                code.editor.insert_char(c);
            }
            vec![]
        }
        Message::CodeEditNewline => {
            if let Some(ref mut code) = state.code {
                code.editor.insert_newline();
            }
            vec![]
        }
        Message::CodeEditBackspace => {
            if let Some(ref mut code) = state.code {
                code.editor.backspace();
            }
            vec![]
        }
        Message::CodeEditDelete => {
            if let Some(ref mut code) = state.code {
                code.editor.delete();
            }
            vec![]
        }
        Message::CodeEditTab => {
            if let Some(ref mut code) = state.code {
                code.editor.insert_tab();
            }
            vec![]
        }
        Message::CodeEditCursorLeft => {
            if let Some(ref mut code) = state.code {
                code.editor.move_left();
            }
            vec![]
        }
        Message::CodeEditCursorRight => {
            if let Some(ref mut code) = state.code {
                code.editor.move_right();
            }
            vec![]
        }
        Message::CodeEditCursorUp => {
            if let Some(ref mut code) = state.code {
                code.editor.move_up();
            }
            vec![]
        }
        Message::CodeEditCursorDown => {
            if let Some(ref mut code) = state.code {
                code.editor.move_down();
            }
            vec![]
        }
        Message::CodeEditWordLeft => {
            if let Some(ref mut code) = state.code {
                code.editor.move_word_left();
            }
            vec![]
        }
        Message::CodeEditWordRight => {
            if let Some(ref mut code) = state.code {
                code.editor.move_word_right();
            }
            vec![]
        }
        Message::CodeEditHome => {
            if let Some(ref mut code) = state.code {
                code.editor.move_home();
            }
            vec![]
        }
        Message::CodeEditEnd => {
            if let Some(ref mut code) = state.code {
                code.editor.move_end();
            }
            vec![]
        }
        Message::CodeEditPageUp => {
            if let Some(ref mut code) = state.code {
                code.editor.page_up();
            }
            vec![]
        }
        Message::CodeEditPageDown => {
            if let Some(ref mut code) = state.code {
                code.editor.page_down();
            }
            vec![]
        }
        Message::CodeEditUndo => {
            if let Some(ref mut code) = state.code {
                code.editor.undo();
            }
            vec![]
        }
        Message::CodeEditRedo => {
            if let Some(ref mut code) = state.code {
                code.editor.redo();
            }
            vec![]
        }
        Message::CodeEditSubmit => {
            if let Some(ref code) = state.code {
                if let Some(ref repo) = state.current_repo {
                    let content = code.editor.content();
                    let filename = code.file_name.clone().unwrap_or_default();
                    let file_path = code.file_path.clone().unwrap_or_default();
                    let message = format!("Update {}", filename);
                    let branch = code.git_ref.clone();

                    // Find the file SHA from entries
                    let sha = code
                        .entries
                        .iter()
                        .find(|e| Some(&e.path) == code.file_path.as_ref())
                        .map(|e| e.sha.clone())
                        .unwrap_or_default();

                    state.loading.insert("code_file_update".to_string());
                    return vec![Command::UpdateFileContent(
                        repo.clone(),
                        file_path,
                        content,
                        message,
                        sha,
                        branch,
                    )];
                }
            }
            vec![]
        }
        Message::CodeEditCancel => {
            if let Some(ref mut code) = state.code {
                code.editing = false;
                code.editor = ghtui_core::editor::TextEditor::new();
            }
            vec![]
        }
        Message::CodeEditSelectLeft => {
            if let Some(ref mut code) = state.code {
                code.editor.move_left_selecting();
            }
            vec![]
        }
        Message::CodeEditSelectRight => {
            if let Some(ref mut code) = state.code {
                code.editor.move_right_selecting();
            }
            vec![]
        }
        Message::CodeEditSelectUp => {
            if let Some(ref mut code) = state.code {
                code.editor.move_up_selecting();
            }
            vec![]
        }
        Message::CodeEditSelectDown => {
            if let Some(ref mut code) = state.code {
                code.editor.move_down_selecting();
            }
            vec![]
        }
        Message::CodeEditSelectAll => {
            if let Some(ref mut code) = state.code {
                code.editor.select_all();
            }
            vec![]
        }
        Message::CodeEditCut => {
            if let Some(ref mut code) = state.code {
                if let Some(text) = code.editor.selected_text() {
                    code.editor.delete_selection();
                    return vec![Command::SetClipboard(text)];
                }
            }
            vec![]
        }
        Message::CodeEditCopy => {
            if let Some(ref code) = state.code {
                if let Some(text) = code.editor.selected_text() {
                    return vec![Command::SetClipboard(text)];
                }
            }
            vec![]
        }
        Message::CodeEditPaste(text) => {
            if let Some(ref mut code) = state.code {
                if !text.is_empty() {
                    code.editor.delete_selection();
                    code.editor.insert_str(&text);
                }
            }
            vec![]
        }
        Message::CodeEditMoveLineStart => {
            if let Some(ref mut code) = state.code {
                code.editor.move_home();
            }
            vec![]
        }
        Message::CodeEditMoveLineEnd => {
            if let Some(ref mut code) = state.code {
                code.editor.move_end();
            }
            vec![]
        }
        Message::CodeEditMoveDocTop => {
            if let Some(ref mut code) = state.code {
                code.editor.move_to_top();
            }
            vec![]
        }
        Message::CodeEditMoveDocBottom => {
            if let Some(ref mut code) = state.code {
                code.editor.move_to_bottom();
            }
            vec![]
        }
        // Cmd+Shift selecting movements
        Message::CodeEditSelectToLineStart => {
            if let Some(ref mut code) = state.code {
                code.editor.move_home_selecting();
            }
            vec![]
        }
        Message::CodeEditSelectToLineEnd => {
            if let Some(ref mut code) = state.code {
                code.editor.move_end_selecting();
            }
            vec![]
        }
        Message::CodeEditSelectToDocTop => {
            if let Some(ref mut code) = state.code {
                code.editor.move_to_top_selecting();
            }
            vec![]
        }
        Message::CodeEditSelectToDocBottom => {
            if let Some(ref mut code) = state.code {
                code.editor.move_to_bottom_selecting();
            }
            vec![]
        }
        Message::CodeEditSelectWordLeft => {
            if let Some(ref mut code) = state.code {
                code.editor.move_word_left_selecting();
            }
            vec![]
        }
        Message::CodeEditSelectWordRight => {
            if let Some(ref mut code) = state.code {
                code.editor.move_word_right_selecting();
            }
            vec![]
        }

        // Discussions
        Message::DiscussionsLoaded(discussions) => {
            state.loading.remove("discussions");
            state.discussions = Some(ghtui_core::state::DiscussionsState::new(discussions));
            vec![]
        }
        Message::DiscussionsOpenInBrowser => {
            if let Some(ref disc) = state.discussions {
                if let Some(item) = disc.items.get(disc.selected) {
                    return vec![Command::OpenInBrowser(item.url.clone())];
                }
            }
            vec![]
        }

        // Gists
        Message::GistsLoaded(gists) => {
            state.loading.remove("gists");
            state.gists = Some(ghtui_core::state::GistsState::new(gists));
            vec![]
        }
        Message::GistsOpenInBrowser => {
            if let Some(ref g) = state.gists {
                if let Some(item) = g.items.get(g.selected) {
                    return vec![Command::OpenInBrowser(item.html_url.clone())];
                }
            }
            vec![]
        }

        // Organizations
        Message::OrgsLoaded(orgs) => {
            state.loading.remove("orgs");
            state.org = Some(ghtui_core::state::OrgState::new(orgs));
            // Fetch members of first org
            if let Some(ref org_state) = state.org {
                if let Some(first_org) = org_state.orgs.first() {
                    state.loading.insert("org_members".to_string());
                    return vec![Command::FetchOrgMembers(first_org.login.clone())];
                }
            }
            vec![]
        }
        Message::OrgMembersLoaded(members) => {
            state.loading.remove("org_members");
            if let Some(ref mut org_state) = state.org {
                org_state.members = members;
            }
            vec![]
        }

        // Multi-repo dashboard
        Message::RecentReposLoaded(repos) => {
            state.loading.remove("recent_repos");
            state.recent_repos = repos;
            vec![]
        }

        Message::CodeFileUpdated => {
            state.loading.remove("code_file_update");
            let mut cmds = vec![];
            if let Some(ref mut code) = state.code {
                code.editing = false;
                code.editor = ghtui_core::editor::TextEditor::new();
                let filename = code.file_name.clone().unwrap_or_default();
                let file_path = code.file_path.clone();
                let git_ref = code.git_ref.clone();
                let current_path = code.current_path.clone();

                state.push_toast(
                    format!("Committed: Update {}", filename),
                    ToastLevel::Success,
                );

                // Re-fetch file content to get new SHA and updated content
                if let (Some(fp), Some(repo)) = (file_path, &state.current_repo) {
                    state.loading.insert("code_file".to_string());
                    cmds.push(Command::FetchFileContent(repo.clone(), fp, git_ref.clone()));
                    // Also re-fetch the directory listing to update SHA
                    state.loading.insert("code_contents".to_string());
                    cmds.push(Command::FetchContents(repo.clone(), current_path, git_ref));
                }
            }
            cmds
        }
    }
}
