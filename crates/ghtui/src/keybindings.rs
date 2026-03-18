use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use ghtui_core::message::ModalKind;
use ghtui_core::router::Route;
use ghtui_core::state::InputMode;
use ghtui_core::state::issue::InlineEditTarget;
use ghtui_core::state::pr::PrInlineEditTarget;
use ghtui_core::{AppState, Message};

pub fn handle_key(key: KeyEvent, state: &AppState) -> Option<Message> {
    // Global: Ctrl-C always quits
    if key.modifiers.contains(KeyModifiers::CONTROL) && key.code == KeyCode::Char('c') {
        return Some(Message::Quit);
    }

    match state.input_mode {
        InputMode::Insert => handle_insert_mode(key),
        InputMode::Normal => handle_normal_mode(key, state),
    }
}

fn handle_insert_mode(key: KeyEvent) -> Option<Message> {
    let ctrl = key.modifiers.contains(KeyModifiers::CONTROL);
    let alt = key.modifiers.contains(KeyModifiers::ALT);
    match key.code {
        KeyCode::Esc => Some(Message::ModalClose),
        // Ctrl+S: save/submit (reliable across all terminals)
        KeyCode::Char('s') if ctrl => Some(Message::ModalSubmit),
        // Ctrl+Enter / Alt+Enter: submit
        KeyCode::Enter if ctrl || alt => Some(Message::ModalSubmit),
        KeyCode::Enter => Some(Message::InputChanged("\n".to_string())),
        KeyCode::Char(c) => Some(Message::InputChanged(c.to_string())),
        KeyCode::Backspace => Some(Message::InputChanged("\x08".to_string())),
        _ => None,
    }
}

fn handle_normal_mode(key: KeyEvent, state: &AppState) -> Option<Message> {
    // Modal-specific keys
    if state.modal.is_some() {
        return handle_modal_keys(key, state);
    }

    // Skip global keys when editing inline or in diff comment
    let is_inline_editing = state.issue_detail.as_ref().is_some_and(|d| d.is_editing())
        || state.pr_detail.as_ref().is_some_and(|d| {
            d.is_editing() || d.diff_comment_target.is_some() || d.action_bar_focused
        });

    if !is_inline_editing {
        // Global keys
        match key.code {
            KeyCode::Char('q') => return Some(Message::Quit),
            KeyCode::Char('?') => return Some(Message::ModalOpen(ModalKind::Help)),
            KeyCode::Char('t') => return Some(Message::ToggleTheme),
            KeyCode::Char('S') => return Some(Message::ModalOpen(ModalKind::AccountSwitcher)),
            KeyCode::Char('k') if key.modifiers.contains(KeyModifiers::CONTROL) => {
                return Some(Message::SearchOpen);
            }
            // Tab navigation: 1-9 for global tabs (matching GitHub web)
            KeyCode::Char('1') => return Some(Message::GlobalTabSelect(0)),
            KeyCode::Char('2') => return Some(Message::GlobalTabSelect(1)),
            KeyCode::Char('3') => return Some(Message::GlobalTabSelect(2)),
            KeyCode::Char('4') => return Some(Message::GlobalTabSelect(3)),
            KeyCode::Char('5') => return Some(Message::GlobalTabSelect(4)),
            KeyCode::Char('6') => return Some(Message::GlobalTabSelect(5)),
            KeyCode::Char('7') => return Some(Message::GlobalTabSelect(6)),
            _ => {}
        }
    }

    // Tab / Shift-Tab for global tab navigation (except in detail views which use Tab internally)
    let in_detail = matches!(
        &state.route,
        Route::PrDetail { .. }
            | Route::IssueDetail { .. }
            | Route::ActionDetail { .. }
            | Route::JobLog { .. }
            | Route::Security { .. }
            | Route::Insights { .. }
            | Route::Settings { .. }
    );
    if !in_detail {
        match key.code {
            KeyCode::Tab => return Some(Message::GlobalTabNext),
            KeyCode::BackTab => return Some(Message::GlobalTabPrev),
            _ => {}
        }
    }

    // Esc behavior: in detail views go back, in list views do nothing
    // (but IssueDetail/PrDetail handles Esc in its own handler when editing)
    if key.code == KeyCode::Esc {
        let issue_editing = state.issue_detail.as_ref().is_some_and(|d| d.is_editing());
        let pr_editing = state.pr_detail.as_ref().is_some_and(|d| d.is_editing());
        let pr_picker = state.pr_detail.as_ref().is_some_and(|d| d.has_picker());
        let pr_diff_select = state
            .pr_detail
            .as_ref()
            .is_some_and(|d| d.tab == 3 && d.diff_select_anchor.is_some());
        let pr_diff_commenting = state
            .pr_detail
            .as_ref()
            .is_some_and(|d| d.diff_comment_target.is_some());
        if (matches!(state.route, Route::IssueDetail { .. }) && issue_editing)
            || (matches!(state.route, Route::PrDetail { .. })
                && (pr_editing || pr_picker || pr_diff_select || pr_diff_commenting))
        {
            // Fall through to route-specific handler
        } else {
            return match &state.route {
                Route::PrDetail { .. }
                | Route::IssueDetail { .. }
                | Route::ActionDetail { .. }
                | Route::JobLog { .. } => Some(Message::Back),
                _ => None,
            };
        }
    }

    // Route-specific keys
    match &state.route {
        Route::PrList { .. } => handle_pr_list_keys(key, state),
        Route::PrDetail { .. } => handle_pr_detail_keys(key, state),
        Route::IssueList { .. } => handle_issue_list_keys(key, state),
        Route::IssueDetail { .. } => handle_issue_detail_keys(key, state),
        Route::ActionsList { .. } => handle_actions_list_keys(key, state),
        Route::ActionDetail { .. } | Route::JobLog { .. } => handle_action_detail_keys(key, state),
        Route::Search { .. } => handle_search_keys(key, state),
        Route::Notifications => handle_notification_keys(key),
        Route::Security { .. } => handle_security_keys(key, state),
        Route::Insights { .. } => handle_settings_keys(key, state),
        Route::Settings { .. } => handle_settings_keys(key, state),
        _ => handle_list_keys(key),
    }
}

fn handle_modal_keys(key: KeyEvent, state: &AppState) -> Option<Message> {
    match key.code {
        KeyCode::Esc | KeyCode::Char('q') => Some(Message::ModalClose),
        _ => {
            // Account switcher modal: j/k to navigate, Enter to select
            if matches!(state.modal, Some(ModalKind::AccountSwitcher)) {
                match key.code {
                    KeyCode::Char('j') | KeyCode::Down => Some(Message::ListSelect(1)),
                    KeyCode::Char('k') | KeyCode::Up => Some(Message::ListSelect(usize::MAX)),
                    KeyCode::Enter => state
                        .accounts
                        .get(state.account_selected)
                        .map(|account| Message::AccountSwitch(account.clone())),
                    _ => None,
                }
            } else {
                None
            }
        }
    }
}

fn handle_search_keys(key: KeyEvent, state: &AppState) -> Option<Message> {
    let input_mode = state.search.as_ref().is_some_and(|s| s.input_mode);

    if input_mode {
        return match key.code {
            KeyCode::Esc => Some(Message::SearchCancel),
            KeyCode::Enter => Some(Message::SearchSubmit),
            KeyCode::Char(c) => Some(Message::SearchInput(c.to_string())),
            KeyCode::Backspace => Some(Message::SearchInput("\x08".to_string())),
            KeyCode::Tab => Some(Message::SearchCycleKind),
            _ => None,
        };
    }

    match key.code {
        KeyCode::Char('j') | KeyCode::Down => Some(Message::ListSelect(1)),
        KeyCode::Char('k') | KeyCode::Up => Some(Message::ListSelect(usize::MAX)),
        KeyCode::Enter => Some(Message::SearchNavigate),
        KeyCode::Char('/') => Some(Message::SearchOpen),
        KeyCode::Tab => Some(Message::SearchCycleKind),
        _ => None,
    }
}

fn handle_notification_keys(key: KeyEvent) -> Option<Message> {
    match key.code {
        KeyCode::Char('j') | KeyCode::Down => Some(Message::ListSelect(1)),
        KeyCode::Char('k') | KeyCode::Up => Some(Message::ListSelect(usize::MAX)),
        KeyCode::Enter => Some(Message::NotificationNavigate),
        KeyCode::Char('r') => Some(Message::Tick),
        KeyCode::Char('m') => Some(Message::NotificationMarkRead),
        KeyCode::Char('M') => Some(Message::NotificationMarkAllRead),
        KeyCode::Char('u') => Some(Message::NotificationUnsubscribe),
        KeyCode::Char('d') => Some(Message::NotificationDone),
        KeyCode::Char('s') => Some(Message::NotificationCycleReason),
        KeyCode::Char('e') => Some(Message::NotificationCycleType),
        KeyCode::Char('g') => Some(Message::NotificationToggleGrouped),
        _ => None,
    }
}

fn handle_list_keys(key: KeyEvent) -> Option<Message> {
    match key.code {
        KeyCode::Char('j') | KeyCode::Down => Some(Message::ListSelect(1)),
        KeyCode::Char('k') | KeyCode::Up => Some(Message::ListSelect(usize::MAX)),
        KeyCode::Enter => Some(Message::ListSelect(0)),
        KeyCode::Char('r') => Some(Message::Tick), // refresh
        _ => None,
    }
}

fn handle_security_keys(key: KeyEvent, state: &AppState) -> Option<Message> {
    let detail_open = state.security.as_ref().is_some_and(|s| s.detail_open);

    if detail_open {
        return match key.code {
            KeyCode::Esc | KeyCode::Enter => Some(Message::SecurityToggleDetail),
            KeyCode::Char('j') | KeyCode::Down | KeyCode::PageDown => Some(Message::ScrollDown),
            KeyCode::Char('k') | KeyCode::Up | KeyCode::PageUp => Some(Message::ScrollUp),
            KeyCode::Char('o') => Some(Message::SecurityOpenInBrowser),
            KeyCode::Tab => Some(Message::TabChanged(1)),
            KeyCode::BackTab => Some(Message::TabChanged(usize::MAX)),
            _ => None,
        };
    }

    match key.code {
        KeyCode::Tab => Some(Message::TabChanged(1)),
        KeyCode::BackTab => Some(Message::TabChanged(usize::MAX)),
        KeyCode::Char('j') | KeyCode::Down => Some(Message::ListSelect(1)),
        KeyCode::Char('k') | KeyCode::Up => Some(Message::ListSelect(usize::MAX)),
        KeyCode::Enter => Some(Message::SecurityToggleDetail),
        KeyCode::Char('o') => Some(Message::SecurityOpenInBrowser),
        _ => None,
    }
}

fn handle_settings_keys(key: KeyEvent, state: &AppState) -> Option<Message> {
    let is_editing = state.settings.as_ref().is_some_and(|s| s.is_editing());

    if is_editing {
        let ctrl = key.modifiers.contains(KeyModifiers::CONTROL);
        return match key.code {
            KeyCode::Esc => Some(Message::SettingsEditCancel),
            KeyCode::Char('s') if ctrl => Some(Message::SettingsEditSubmit),
            KeyCode::Enter => Some(Message::SettingsEditSubmit),
            KeyCode::Char(c) => Some(Message::SettingsEditChar(c)),
            KeyCode::Backspace => Some(Message::SettingsEditBackspace),
            _ => None,
        };
    }

    // Check if on General tab for edit keys
    let on_general = state.settings.as_ref().is_some_and(|s| s.tab == 0);

    match key.code {
        KeyCode::Tab => Some(Message::TabChanged(1)),
        KeyCode::BackTab => Some(Message::TabChanged(usize::MAX)),
        KeyCode::Char('j') | KeyCode::Down => Some(Message::ListSelect(1)),
        KeyCode::Char('k') | KeyCode::Up => Some(Message::ListSelect(usize::MAX)),
        KeyCode::PageDown => Some(Message::ScrollDown),
        KeyCode::PageUp => Some(Message::ScrollUp),
        // Edit keys (only on General tab)
        KeyCode::Char('d') if on_general => {
            Some(Message::SettingsStartEdit("description".to_string()))
        }
        KeyCode::Char('b') if on_general => {
            Some(Message::SettingsStartEdit("default_branch".to_string()))
        }
        KeyCode::Char('T') if on_general => Some(Message::SettingsStartEdit("topics".to_string())),
        // Feature toggles (on General tab)
        KeyCode::Char('I') if on_general => {
            Some(Message::SettingsToggleFeature("has_issues".to_string()))
        }
        KeyCode::Char('P') if on_general => {
            Some(Message::SettingsToggleFeature("has_projects".to_string()))
        }
        KeyCode::Char('W') if on_general => {
            Some(Message::SettingsToggleFeature("has_wiki".to_string()))
        }
        _ => None,
    }
}

fn handle_actions_list_keys(key: KeyEvent, state: &AppState) -> Option<Message> {
    let search_mode = state.actions_list.as_ref().is_some_and(|l| l.search_mode);

    if search_mode {
        return match key.code {
            KeyCode::Esc => Some(Message::ActionsSearchCancel),
            KeyCode::Enter => Some(Message::ActionsSearchSubmit),
            KeyCode::Char(c) => Some(Message::ActionsSearchInput(c.to_string())),
            KeyCode::Backspace => Some(Message::ActionsSearchInput("\x08".to_string())),
            _ => None,
        };
    }

    match key.code {
        KeyCode::Char('j') | KeyCode::Down => Some(Message::ListSelect(1)),
        KeyCode::Char('k') | KeyCode::Up => Some(Message::ListSelect(usize::MAX)),
        KeyCode::Enter => Some(Message::ListSelect(0)),
        KeyCode::Char('r') => Some(Message::Tick),
        KeyCode::Char('s') => Some(Message::ActionsToggleStatus),
        KeyCode::Char('e') => Some(Message::ActionsCycleEvent),
        KeyCode::Char('n') => Some(Message::ActionsNextPage),
        KeyCode::Char('p') => Some(Message::ActionsPrevPage),
        KeyCode::Char('/') => Some(Message::ActionsSearchStart),
        KeyCode::Char('o') => Some(Message::ActionsOpenInBrowser),
        KeyCode::Char('F') => Some(Message::ActionsFilterClear),
        KeyCode::Char('x') => Some(Message::ActionsCancelRun),
        KeyCode::Char('R') => Some(Message::ActionsRerunRun),
        _ => None,
    }
}

fn handle_action_detail_keys(key: KeyEvent, state: &AppState) -> Option<Message> {
    use ghtui_core::state::ActionDetailFocus;

    let focus = state
        .action_detail
        .as_ref()
        .map(|d| d.focus)
        .unwrap_or(ActionDetailFocus::Jobs);

    match focus {
        ActionDetailFocus::ActionBar => match key.code {
            KeyCode::Char('h') | KeyCode::Left => Some(Message::ActionDetailActionBarLeft),
            KeyCode::Char('l') | KeyCode::Right => Some(Message::ActionDetailActionBarRight),
            KeyCode::Enter => Some(Message::ActionDetailActionBarSelect),
            KeyCode::Esc | KeyCode::Char('k') | KeyCode::Up => {
                Some(Message::ActionDetailActionBarFocus)
            }
            _ => None,
        },
        ActionDetailFocus::Log => match key.code {
            KeyCode::Char('j') | KeyCode::Down | KeyCode::PageDown => Some(Message::ScrollDown),
            KeyCode::Char('k') | KeyCode::Up | KeyCode::PageUp => Some(Message::ScrollUp),
            KeyCode::Char('d') if key.modifiers.contains(KeyModifiers::CONTROL) => {
                Some(Message::ScrollDown)
            }
            KeyCode::Char('u') if key.modifiers.contains(KeyModifiers::CONTROL) => {
                Some(Message::ScrollUp)
            }
            KeyCode::Tab => Some(Message::ActionDetailFocusJobs),
            KeyCode::BackTab => Some(Message::ActionDetailActionBarFocus),
            KeyCode::Char('x') => Some(Message::ActionDetailActionBarFocus),
            KeyCode::Char('o') => Some(Message::ActionDetailOpenInBrowser),
            _ => None,
        },
        ActionDetailFocus::Jobs => match key.code {
            KeyCode::Char('j') | KeyCode::Down => Some(Message::ListSelect(1)),
            KeyCode::Char('k') | KeyCode::Up => Some(Message::ListSelect(usize::MAX)),
            KeyCode::Enter => Some(Message::ListSelect(0)),
            KeyCode::Tab => Some(Message::ActionDetailFocusLog),
            KeyCode::BackTab => Some(Message::ActionDetailActionBarFocus), // Shift+Tab → action bar
            // Step fold/unfold: toggle all steps for selected job
            KeyCode::Char('h') | KeyCode::Left => Some(Message::ActionDetailToggleStep(0)),
            KeyCode::Char('l') | KeyCode::Right => Some(Message::ActionDetailToggleStep(0)),
            KeyCode::PageDown => Some(Message::ScrollDown),
            KeyCode::PageUp => Some(Message::ScrollUp),
            KeyCode::Char('d') if key.modifiers.contains(KeyModifiers::CONTROL) => {
                Some(Message::ScrollDown)
            }
            KeyCode::Char('u') if key.modifiers.contains(KeyModifiers::CONTROL) => {
                Some(Message::ScrollUp)
            }
            KeyCode::Char('x') => Some(Message::ActionDetailActionBarFocus), // x → action bar (quick)
            KeyCode::Char('o') => Some(Message::ActionDetailOpenInBrowser),
            _ => None,
        },
    }
}

fn handle_pr_list_keys(key: KeyEvent, state: &AppState) -> Option<Message> {
    let search_mode = state.pr_list.as_ref().is_some_and(|l| l.search_mode);

    if search_mode {
        return match key.code {
            KeyCode::Esc => Some(Message::PrSearchCancel),
            KeyCode::Enter => Some(Message::PrSearchSubmit),
            KeyCode::Char(c) => Some(Message::PrSearchInput(c.to_string())),
            KeyCode::Backspace => Some(Message::PrSearchInput("\x08".to_string())),
            _ => None,
        };
    }

    match key.code {
        KeyCode::Char('j') | KeyCode::Down => Some(Message::ListSelect(1)),
        KeyCode::Char('k') | KeyCode::Up => Some(Message::ListSelect(usize::MAX)),
        KeyCode::Enter => Some(Message::ListSelect(0)),
        KeyCode::Char('r') => Some(Message::Tick),
        KeyCode::Char('s') => Some(Message::PrToggleStateFilter),
        KeyCode::Char('n') => Some(Message::PrNextPage),
        KeyCode::Char('p') => Some(Message::PrPrevPage),
        KeyCode::Char('c') => Some(Message::ModalOpen(ModalKind::CreatePr)),
        KeyCode::Char('/') => Some(Message::PrSearchStart),
        KeyCode::Char('o') => Some(Message::PrSortCycle),
        KeyCode::Char('F') => Some(Message::PrFilterClear),
        _ => None,
    }
}

fn handle_pr_detail_keys(key: KeyEvent, state: &AppState) -> Option<Message> {
    let is_editing = state.pr_detail.as_ref().is_some_and(|d| d.is_editing());

    if is_editing {
        let is_title_edit = state
            .pr_detail
            .as_ref()
            .is_some_and(|d| matches!(d.edit_target, Some(PrInlineEditTarget::PrTitle)));

        let ctrl = key.modifiers.contains(KeyModifiers::CONTROL);
        let alt = key.modifiers.contains(KeyModifiers::ALT);

        return match key.code {
            KeyCode::Esc => Some(Message::PrEditCancel),
            KeyCode::Char('s') if ctrl => Some(Message::PrEditSubmit),
            KeyCode::Enter if ctrl || alt || is_title_edit => Some(Message::PrEditSubmit),
            KeyCode::Enter => Some(Message::PrEditNewline),
            KeyCode::Char('z') if ctrl => Some(Message::PrEditUndo),
            KeyCode::Char('y') if ctrl => Some(Message::PrEditRedo),
            KeyCode::Char(c) => Some(Message::PrEditChar(c)),
            KeyCode::Backspace => Some(Message::PrEditBackspace),
            KeyCode::Delete => Some(Message::PrEditDelete),
            KeyCode::Tab => Some(Message::PrEditTab),
            KeyCode::Left if ctrl || alt => Some(Message::PrEditWordLeft),
            KeyCode::Right if ctrl || alt => Some(Message::PrEditWordRight),
            KeyCode::Left => Some(Message::PrEditCursorLeft),
            KeyCode::Right => Some(Message::PrEditCursorRight),
            KeyCode::Up => Some(Message::PrEditCursorUp),
            KeyCode::Down => Some(Message::PrEditCursorDown),
            KeyCode::Home => Some(Message::PrEditHome),
            KeyCode::End => Some(Message::PrEditEnd),
            KeyCode::PageUp => Some(Message::PrEditPageUp),
            KeyCode::PageDown => Some(Message::PrEditPageDown),
            _ => None,
        };
    }

    // Picker mode
    let has_picker = state.pr_detail.as_ref().is_some_and(|d| d.has_picker());

    if has_picker {
        if let Some(ref detail) = state.pr_detail {
            if detail.label_picker.is_some() {
                return match key.code {
                    KeyCode::Esc => Some(Message::PrLabelCancel),
                    KeyCode::Enter | KeyCode::Char(' ') => detail
                        .label_picker
                        .as_ref()
                        .map(|p| Message::PrLabelSelect(p.cursor)),
                    KeyCode::Char('j') | KeyCode::Down => Some(Message::ListSelect(1)),
                    KeyCode::Char('k') | KeyCode::Up => Some(Message::ListSelect(usize::MAX)),
                    KeyCode::Char('s') => Some(Message::PrLabelApply),
                    _ => None,
                };
            }
            if detail.assignee_picker.is_some() {
                return match key.code {
                    KeyCode::Esc => Some(Message::PrAssigneeCancel),
                    KeyCode::Enter | KeyCode::Char(' ') => detail
                        .assignee_picker
                        .as_ref()
                        .map(|p| Message::PrAssigneeSelect(p.cursor)),
                    KeyCode::Char('j') | KeyCode::Down => Some(Message::ListSelect(1)),
                    KeyCode::Char('k') | KeyCode::Up => Some(Message::ListSelect(usize::MAX)),
                    KeyCode::Char('s') => Some(Message::PrAssigneeApply),
                    _ => None,
                };
            }
            if detail.milestone_picker.is_some() {
                return match key.code {
                    KeyCode::Esc => Some(Message::PrMilestoneCancel),
                    KeyCode::Enter | KeyCode::Char(' ') => detail
                        .milestone_picker
                        .as_ref()
                        .map(|p| Message::PrMilestoneSelect(p.cursor)),
                    KeyCode::Char('j') | KeyCode::Down => Some(Message::ListSelect(1)),
                    KeyCode::Char('k') | KeyCode::Up => Some(Message::ListSelect(usize::MAX)),
                    KeyCode::Char('s') => Some(Message::PrMilestoneApply),
                    KeyCode::Char('0') => Some(Message::PrMilestoneClear),
                    _ => None,
                };
            }
        }
        return None;
    }

    // Check if we're on the diff tab
    let on_diff_tab = state.pr_detail.as_ref().is_some_and(|d| d.tab == 3);

    if on_diff_tab {
        // Inline comment editor mode
        let diff_commenting = state
            .pr_detail
            .as_ref()
            .is_some_and(|d| d.diff_comment_target.is_some());

        if diff_commenting {
            let ctrl = key.modifiers.contains(KeyModifiers::CONTROL);
            let alt = key.modifiers.contains(KeyModifiers::ALT);
            return match key.code {
                KeyCode::Esc => Some(Message::PrDiffCommentCancel),
                // Ctrl+S: save/submit (reliable across all terminals)
                KeyCode::Char('s') if ctrl => Some(Message::PrDiffCommentSubmit),
                // Alt+Enter: submit fallback
                KeyCode::Enter if alt => Some(Message::PrDiffCommentSubmit),
                // Ctrl+Enter: submit (works in some terminals)
                KeyCode::Enter if ctrl => Some(Message::PrDiffCommentSubmit),
                // Ctrl+G: insert suggestion template
                KeyCode::Char('g') if ctrl => Some(Message::PrDiffInsertSuggestion),
                KeyCode::Enter => Some(Message::PrEditNewline),
                KeyCode::Char(c) => Some(Message::PrEditChar(c)),
                KeyCode::Backspace => Some(Message::PrEditBackspace),
                KeyCode::Delete => Some(Message::PrEditDelete),
                KeyCode::Left => Some(Message::PrEditCursorLeft),
                KeyCode::Right => Some(Message::PrEditCursorRight),
                KeyCode::Up => Some(Message::PrEditCursorUp),
                KeyCode::Down => Some(Message::PrEditCursorDown),
                _ => None,
            };
        }

        // File tree focused
        let tree_focused = state
            .pr_detail
            .as_ref()
            .is_some_and(|d| d.file_tree_focused && d.show_file_tree);

        if tree_focused {
            return match key.code {
                KeyCode::Char('j') | KeyCode::Down => Some(Message::PrDiffTreeDown),
                KeyCode::Char('k') | KeyCode::Up => Some(Message::PrDiffTreeUp),
                KeyCode::Enter => Some(Message::PrDiffTreeSelect),
                KeyCode::Char('f') => Some(Message::PrDiffToggleTree),
                KeyCode::Char('V') => Some(Message::PrDiffMarkViewed),
                KeyCode::Tab => Some(Message::PrDiffTreeFocus), // switch to diff
                KeyCode::BackTab => Some(Message::TabChanged(usize::MAX)),
                KeyCode::Esc => Some(Message::PrDiffTreeFocus), // unfocus tree
                KeyCode::Char('o') => Some(Message::PrOpenInBrowser),
                _ => None,
            };
        }

        // Diff focused
        let shift = key.modifiers.contains(KeyModifiers::SHIFT);
        return match key.code {
            KeyCode::Tab => {
                // If tree is visible, focus tree; otherwise next global tab
                let has_tree = state.pr_detail.as_ref().is_some_and(|d| d.show_file_tree);
                if has_tree {
                    Some(Message::PrDiffTreeFocus)
                } else {
                    Some(Message::TabChanged(1))
                }
            }
            KeyCode::BackTab => Some(Message::TabChanged(usize::MAX)),
            KeyCode::Char('f') => Some(Message::PrDiffToggleTree),
            KeyCode::Char('j') | KeyCode::Down if shift => Some(Message::PrDiffSelectDown),
            KeyCode::Char('k') | KeyCode::Up if shift => Some(Message::PrDiffSelectUp),
            KeyCode::Char('J') => Some(Message::PrDiffSelectDown),
            KeyCode::Char('K') => Some(Message::PrDiffSelectUp),
            KeyCode::Char('j') | KeyCode::Down => Some(Message::PrDiffCursorDown),
            KeyCode::Char('k') | KeyCode::Up => Some(Message::PrDiffCursorUp),
            KeyCode::Enter => Some(Message::PrDiffToggleCollapse),
            KeyCode::Char('l') | KeyCode::Right => Some(Message::PrDiffExpand),
            KeyCode::Char('h') | KeyCode::Left => Some(Message::PrDiffCollapse),
            KeyCode::Char('V') => Some(Message::PrDiffMarkViewed),
            KeyCode::Esc => Some(Message::PrDiffClearSelection),
            KeyCode::PageDown => Some(Message::ScrollDown),
            KeyCode::PageUp => Some(Message::ScrollUp),
            KeyCode::Char('s') => Some(Message::PrDiffToggleSideBySide),
            KeyCode::Char('z') => Some(Message::PrReviewThreadToggleResolve),
            KeyCode::Char('o') => Some(Message::PrOpenInBrowser),
            _ => None,
        };
    }

    // Action bar focused
    let action_focused = state
        .pr_detail
        .as_ref()
        .is_some_and(|d| d.action_bar_focused);
    if action_focused {
        return match key.code {
            KeyCode::Char('h') | KeyCode::Left => Some(Message::PrActionBarLeft),
            KeyCode::Char('l') | KeyCode::Right => Some(Message::PrActionBarRight),
            KeyCode::Enter => Some(Message::PrActionBarSelect),
            KeyCode::Esc | KeyCode::Char('k') | KeyCode::Up => Some(Message::PrActionBarFocus),
            KeyCode::Tab => Some(Message::TabChanged(1)),
            KeyCode::BackTab => Some(Message::TabChanged(usize::MAX)),
            _ => None,
        };
    }

    // Normal mode (conversation/checks tabs)
    match key.code {
        KeyCode::Tab => Some(Message::TabChanged(1)),
        KeyCode::BackTab => Some(Message::TabChanged(usize::MAX)),
        KeyCode::Char('j') | KeyCode::Down => Some(Message::ListSelect(1)),
        KeyCode::Char('k') | KeyCode::Up => Some(Message::ListSelect(usize::MAX)),
        KeyCode::Char('e') => Some(Message::PrStartEditBody),
        KeyCode::Char('c') => Some(Message::PrStartComment),
        KeyCode::Char('r') => Some(Message::PrStartReply),
        KeyCode::Char('l') => Some(Message::PrLabelToggle),
        KeyCode::Char('a') => Some(Message::PrAssigneeToggle),
        KeyCode::Char('v') => Some(Message::PrReviewerToggle),
        KeyCode::Char('m') => Some(Message::ModalOpen(ModalKind::MergePr)),
        KeyCode::Char('d') => Some(Message::PrDeleteComment),
        KeyCode::Char('x') => Some(Message::PrToggleState),
        KeyCode::Char('A') => Some(Message::PrApprove),
        KeyCode::Char('R') => Some(Message::PrRequestChanges),
        KeyCode::Char('D') => Some(Message::PrDraftToggle),
        KeyCode::Char('G') => Some(Message::PrAutoMergeToggle),
        KeyCode::Char('M') => Some(Message::PrMilestoneToggle),
        KeyCode::Char('b') => Some(Message::PrChangeBase),
        KeyCode::Char('o') => Some(Message::PrOpenInBrowser),
        KeyCode::Char('+') => Some(Message::PrAddReaction("+1".to_string())),
        KeyCode::Char('-') => Some(Message::PrAddReaction("-1".to_string())),
        KeyCode::PageDown => Some(Message::ScrollDown),
        KeyCode::PageUp => Some(Message::ScrollUp),
        _ => None,
    }
}

fn handle_issue_list_keys(key: KeyEvent, state: &AppState) -> Option<Message> {
    // Search mode
    let search_mode = state.issue_list.as_ref().is_some_and(|l| l.search_mode);

    if search_mode {
        return match key.code {
            KeyCode::Esc => Some(Message::IssueSearchCancel),
            KeyCode::Enter => Some(Message::IssueSearchSubmit),
            KeyCode::Char(c) => Some(Message::IssueSearchInput(c.to_string())),
            KeyCode::Backspace => Some(Message::IssueSearchInput("\x08".to_string())),
            _ => None,
        };
    }

    match key.code {
        KeyCode::Char('j') | KeyCode::Down => Some(Message::ListSelect(1)),
        KeyCode::Char('k') | KeyCode::Up => Some(Message::ListSelect(usize::MAX)),
        KeyCode::Enter => Some(Message::ListSelect(0)),
        KeyCode::Char('r') => Some(Message::Tick),
        KeyCode::Char('s') => Some(Message::IssueToggleStateFilter),
        KeyCode::Char('n') => Some(Message::IssueNextPage),
        KeyCode::Char('p') => Some(Message::IssuePrevPage),
        KeyCode::Char('c') => Some(Message::ModalOpen(ModalKind::CreateIssue)),
        KeyCode::Char('/') => Some(Message::IssueSearchStart),
        KeyCode::Char('o') => Some(Message::IssueSortCycle),
        KeyCode::Char('F') => Some(Message::IssueFilterClear), // Shift+F: clear filters
        _ => None,
    }
}

fn handle_issue_detail_keys(key: KeyEvent, state: &AppState) -> Option<Message> {
    let is_editing = state.issue_detail.as_ref().is_some_and(|d| d.is_editing());

    if is_editing {
        // Inline editing mode
        let is_title_edit = state
            .issue_detail
            .as_ref()
            .is_some_and(|d| matches!(d.edit_target, Some(InlineEditTarget::IssueTitle)));

        let ctrl = key.modifiers.contains(KeyModifiers::CONTROL);
        let alt = key.modifiers.contains(KeyModifiers::ALT);

        return match key.code {
            KeyCode::Esc => Some(Message::IssueEditCancel),
            KeyCode::Char('s') if ctrl => Some(Message::IssueEditSubmit),
            KeyCode::Enter if ctrl || alt || is_title_edit => Some(Message::IssueEditSubmit),
            KeyCode::Enter => Some(Message::IssueEditNewline),
            // Ctrl+Z/Y undo/redo
            KeyCode::Char('z') if ctrl => Some(Message::IssueEditUndo),
            KeyCode::Char('y') if ctrl => Some(Message::IssueEditRedo),
            // Regular char input
            KeyCode::Char(c) => Some(Message::IssueEditChar(c)),
            KeyCode::Backspace => Some(Message::IssueEditBackspace),
            KeyCode::Delete => Some(Message::IssueEditDelete),
            KeyCode::Tab => Some(Message::IssueEditTab),
            // Ctrl+Left/Right or Alt+Left/Right for word movement
            KeyCode::Left if ctrl || alt => Some(Message::IssueEditWordLeft),
            KeyCode::Right if ctrl || alt => Some(Message::IssueEditWordRight),
            KeyCode::Left => Some(Message::IssueEditCursorLeft),
            KeyCode::Right => Some(Message::IssueEditCursorRight),
            KeyCode::Up => Some(Message::IssueEditCursorUp),
            KeyCode::Down => Some(Message::IssueEditCursorDown),
            KeyCode::Home => Some(Message::IssueEditHome),
            KeyCode::End => Some(Message::IssueEditEnd),
            KeyCode::PageUp => Some(Message::IssueEditPageUp),
            KeyCode::PageDown => Some(Message::IssueEditPageDown),
            _ => None,
        };
    }

    // Picker mode (label, assignee, milestone)
    let has_picker = state.issue_detail.as_ref().is_some_and(|d| d.has_picker());

    if has_picker {
        if let Some(ref detail) = state.issue_detail {
            // Determine which picker is active
            if detail.label_picker.is_some() {
                return match key.code {
                    KeyCode::Esc => Some(Message::IssueLabelCancel),
                    KeyCode::Enter | KeyCode::Char(' ') => detail
                        .label_picker
                        .as_ref()
                        .map(|p| Message::IssueLabelSelect(p.cursor)),
                    KeyCode::Char('j') | KeyCode::Down => Some(Message::ListSelect(1)),
                    KeyCode::Char('k') | KeyCode::Up => Some(Message::ListSelect(usize::MAX)),
                    KeyCode::Char('s') => Some(Message::IssueLabelApply),
                    _ => None,
                };
            }
            if detail.assignee_picker.is_some() {
                return match key.code {
                    KeyCode::Esc => Some(Message::IssueAssigneeCancel),
                    KeyCode::Enter | KeyCode::Char(' ') => detail
                        .assignee_picker
                        .as_ref()
                        .map(|p| Message::IssueAssigneeSelect(p.cursor)),
                    KeyCode::Char('j') | KeyCode::Down => Some(Message::ListSelect(1)),
                    KeyCode::Char('k') | KeyCode::Up => Some(Message::ListSelect(usize::MAX)),
                    KeyCode::Char('s') => Some(Message::IssueAssigneeApply),
                    _ => None,
                };
            }
            if detail.milestone_picker.is_some() {
                return match key.code {
                    KeyCode::Esc => Some(Message::IssueMilestoneCancel),
                    KeyCode::Enter | KeyCode::Char(' ') => detail
                        .milestone_picker
                        .as_ref()
                        .map(|p| Message::IssueMilestoneSelect(p.cursor)),
                    KeyCode::Char('j') | KeyCode::Down => Some(Message::ListSelect(1)),
                    KeyCode::Char('k') | KeyCode::Up => Some(Message::ListSelect(usize::MAX)),
                    KeyCode::Char('s') => Some(Message::IssueMilestoneApply),
                    KeyCode::Char('0') => Some(Message::IssueMilestoneClear),
                    _ => None,
                };
            }
        }
        return None;
    }

    // Normal section navigation mode
    match key.code {
        KeyCode::Char('j') | KeyCode::Down => Some(Message::ListSelect(1)), // focus next section
        KeyCode::Char('k') | KeyCode::Up => Some(Message::ListSelect(usize::MAX)), // focus prev
        KeyCode::Char('e') => Some(Message::IssueStartEditBody),            // edit focused item
        KeyCode::Char('c') => Some(Message::IssueStartComment),
        KeyCode::Char('r') => Some(Message::IssueStartReply),
        KeyCode::Char('l') => Some(Message::IssueLabelToggle),
        KeyCode::Char('a') => Some(Message::IssueAssigneeToggle),
        KeyCode::Char('m') => Some(Message::IssueMilestoneToggle),
        KeyCode::Char('d') => Some(Message::IssueDeleteComment),
        KeyCode::Char('x') => Some(Message::IssueToggleState),
        KeyCode::Char('L') => Some(Message::IssueLockToggle),
        KeyCode::Char('P') => Some(Message::IssuePinToggle),
        KeyCode::Char('X') => Some(Message::IssueTransfer), // Shift+X: transfer
        KeyCode::Char('o') => Some(Message::IssueOpenInBrowser),
        // Quick reactions
        KeyCode::Char('+') => Some(Message::IssueAddReaction("+1".to_string())),
        KeyCode::Char('-') => Some(Message::IssueAddReaction("-1".to_string())),
        KeyCode::PageDown => Some(Message::ScrollDown),
        KeyCode::PageUp => Some(Message::ScrollUp),
        _ => None,
    }
}
