use ghtui_core::router::Route;
use ghtui_core::state::issue::InlineEditTarget;
use ghtui_core::state::pr::PrInlineEditTarget;
use ghtui_core::state::*;
use ghtui_core::types::{IssueFilters, IssueState, PrFilters, PrState};
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
            if let Some(ref detail) = state.pr_detail {
                if let Some(ref repo) = state.current_repo {
                    let number = detail.detail.pr.number;
                    let input = ghtui_core::types::ReviewInput {
                        event: ghtui_core::types::ReviewEvent::Approve,
                        body: Some("Approved".to_string()),
                        comments: vec![],
                    };
                    return vec![Command::SubmitReview(repo.clone(), number, input)];
                }
            }
            vec![]
        }
        Message::PrRequestChanges => {
            state.input_buffer.clear();
            state.modal = Some(ghtui_core::ModalKind::Confirm {
                title: "Request Changes".to_string(),
                message: "Enter review comment:".to_string(),
            });
            state.input_mode = InputMode::Insert;
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
                            .and_then(|_| None) // no branch info in list
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
                    if title == "Request Changes" =>
                {
                    let body = state.input_buffer.trim().to_string();
                    if body.is_empty() {
                        state
                            .push_toast("Comment cannot be empty".to_string(), ToastLevel::Warning);
                        return vec![];
                    }
                    if let (Some(repo), Some(detail)) = (&state.current_repo, &state.pr_detail) {
                        let input = ghtui_core::types::ReviewInput {
                            event: ghtui_core::types::ReviewEvent::RequestChanges,
                            body: Some(body),
                            comments: vec![],
                        };
                        vec![Command::SubmitReview(
                            repo.clone(),
                            detail.detail.pr.number,
                            input,
                        )]
                    } else {
                        vec![]
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
                } else if delta == usize::MAX {
                    detail.focus_prev();
                } else if delta > 0 {
                    detail.focus_next();
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

/// Count how many review comment lines appear after a given diff line
fn count_review_comment_lines(
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
        count += 1; // header line (┌─)
        count += root.body.lines().count(); // body lines (│)
        let replies = matching
            .iter()
            .filter(|rc| rc.in_reply_to_id == Some(root.id))
            .count();
        count += replies; // reply lines (│ ↳)
        count += 1; // footer line (└─)
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
fn find_cursor_file(detail: &PrDetailState) -> Option<usize> {
    find_cursor_file_info(detail).map(|(fi, _)| fi)
}

/// Find file index and whether cursor is on the file header line
fn find_cursor_file_info(detail: &PrDetailState) -> Option<(usize, bool)> {
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
fn find_cursor_line_info(detail: &PrDetailState) -> Option<(String, u32)> {
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
                    // Found the line — return new_line or old_line
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

fn action_bar_count(pr_state: &ghtui_core::types::PrState) -> usize {
    match pr_state {
        ghtui_core::types::PrState::Open => 5, // Comment, Approve, Request, Merge, Close
        ghtui_core::types::PrState::Closed => 1, // Reopen
        ghtui_core::types::PrState::Merged => 0,
    }
}

fn action_bar_action(index: usize, pr_state: &ghtui_core::types::PrState) -> Option<Message> {
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
