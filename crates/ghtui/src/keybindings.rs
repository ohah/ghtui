use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use ghtui_core::message::ModalKind;
use ghtui_core::router::Route;
use ghtui_core::state::InputMode;
use ghtui_core::state::issue::InlineEditTarget;
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
    match key.code {
        KeyCode::Esc => Some(Message::ModalClose),
        KeyCode::Enter => {
            if key.modifiers.contains(KeyModifiers::CONTROL) {
                Some(Message::ModalSubmit)
            } else {
                Some(Message::InputChanged("\n".to_string()))
            }
        }
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

    // Global keys
    match key.code {
        KeyCode::Char('q') => return Some(Message::Quit),
        KeyCode::Char('?') => return Some(Message::ModalOpen(ModalKind::Help)),
        KeyCode::Char('t') => return Some(Message::ToggleTheme),
        KeyCode::Char('S') => return Some(Message::ModalOpen(ModalKind::AccountSwitcher)),
        // Tab navigation: 1-9 for global tabs (matching GitHub web)
        KeyCode::Char('1') => return Some(Message::GlobalTabSelect(0)), // Code
        KeyCode::Char('2') => return Some(Message::GlobalTabSelect(1)), // Issues
        KeyCode::Char('3') => return Some(Message::GlobalTabSelect(2)), // Pull requests
        KeyCode::Char('4') => return Some(Message::GlobalTabSelect(3)), // Actions
        KeyCode::Char('5') => return Some(Message::GlobalTabSelect(4)), // Security
        KeyCode::Char('6') => return Some(Message::GlobalTabSelect(5)), // Insights
        KeyCode::Char('7') => return Some(Message::GlobalTabSelect(6)), // Settings
        _ => {}
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
    // (but IssueDetail handles Esc in its own handler when editing)
    if key.code == KeyCode::Esc {
        // If editing inline in IssueDetail, let the route handler deal with it
        let issue_editing = state.issue_detail.as_ref().is_some_and(|d| d.is_editing());
        if matches!(state.route, Route::IssueDetail { .. }) && issue_editing {
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
        Route::PrDetail { .. } => handle_pr_detail_keys(key),
        Route::IssueList { .. } => handle_issue_list_keys(key, state),
        Route::IssueDetail { .. } => handle_issue_detail_keys(key, state),
        Route::ActionDetail { .. } | Route::JobLog { .. } => handle_action_detail_keys(key),
        Route::Security { .. } => handle_settings_keys(key),
        Route::Insights { .. } => handle_settings_keys(key),
        Route::Settings { .. } => handle_settings_keys(key),
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

fn handle_list_keys(key: KeyEvent) -> Option<Message> {
    match key.code {
        KeyCode::Char('j') | KeyCode::Down => Some(Message::ListSelect(1)),
        KeyCode::Char('k') | KeyCode::Up => Some(Message::ListSelect(usize::MAX)),
        KeyCode::Enter => Some(Message::ListSelect(0)),
        KeyCode::Char('r') => Some(Message::Tick), // refresh
        _ => None,
    }
}

fn handle_settings_keys(key: KeyEvent) -> Option<Message> {
    match key.code {
        KeyCode::Tab => Some(Message::TabChanged(1)),
        KeyCode::BackTab => Some(Message::TabChanged(usize::MAX)),
        KeyCode::Char('j') | KeyCode::Down => Some(Message::ListSelect(1)),
        KeyCode::Char('k') | KeyCode::Up => Some(Message::ListSelect(usize::MAX)),
        _ => None,
    }
}

fn handle_action_detail_keys(key: KeyEvent) -> Option<Message> {
    match key.code {
        // Job selection
        KeyCode::Char('j') | KeyCode::Down => Some(Message::ListSelect(1)),
        KeyCode::Char('k') | KeyCode::Up => Some(Message::ListSelect(usize::MAX)),
        KeyCode::Enter => Some(Message::ListSelect(0)),
        // Log scroll
        KeyCode::PageDown => Some(Message::ScrollDown),
        KeyCode::PageUp => Some(Message::ScrollUp),
        KeyCode::Char('d') if key.modifiers.contains(KeyModifiers::CONTROL) => {
            Some(Message::ScrollDown)
        }
        KeyCode::Char('u') if key.modifiers.contains(KeyModifiers::CONTROL) => {
            Some(Message::ScrollUp)
        }
        _ => None,
    }
}

fn handle_pr_detail_keys(key: KeyEvent) -> Option<Message> {
    match key.code {
        KeyCode::Tab => Some(Message::TabChanged(1)),
        KeyCode::BackTab => Some(Message::TabChanged(usize::MAX)),
        KeyCode::Char('c') => Some(Message::ModalOpen(ModalKind::AddComment)),
        KeyCode::Char('m') => Some(Message::ModalOpen(ModalKind::MergePr)),
        KeyCode::Char('j') | KeyCode::Down => Some(Message::ListSelect(1)),
        KeyCode::Char('k') | KeyCode::Up => Some(Message::ListSelect(usize::MAX)),
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
        KeyCode::Char('o') => Some(Message::IssueSortCycle), // cycle sort order
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
            KeyCode::Enter if ctrl || is_title_edit => Some(Message::IssueEditSubmit),
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
        KeyCode::Char('L') => Some(Message::IssueLockToggle), // Shift+L: lock/unlock
        KeyCode::Char('o') => Some(Message::IssueOpenInBrowser),
        // Quick reactions
        KeyCode::Char('+') => Some(Message::IssueAddReaction("+1".to_string())),
        KeyCode::Char('-') => Some(Message::IssueAddReaction("-1".to_string())),
        KeyCode::PageDown => Some(Message::ScrollDown),
        KeyCode::PageUp => Some(Message::ScrollUp),
        _ => None,
    }
}
