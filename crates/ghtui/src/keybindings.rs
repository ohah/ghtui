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
    // Command palette intercepts all keys when open
    if state.command_palette.is_some() {
        return handle_palette_keys(key, state);
    }

    // Modal-specific keys
    if state.modal.is_some() {
        return handle_modal_keys(key, state);
    }

    // Ctrl+P opens command palette (before any other processing)
    if key.modifiers.contains(KeyModifiers::CONTROL) && key.code == KeyCode::Char('p') {
        return Some(Message::PaletteOpen);
    }

    // Skip global keys when editing inline or in diff comment
    let is_inline_editing = state.issue_detail.as_ref().is_some_and(|d| d.is_editing())
        || state.pr_detail.as_ref().is_some_and(|d| {
            d.is_editing() || d.diff_comment_target.is_some() || d.action_bar_focused
        })
        || state.code.as_ref().is_some_and(|c| c.editing);

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
            KeyCode::Char('8') => return Some(Message::GlobalTabSelect(8)),
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
            | Route::Code { .. }
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
        let code_editing = state.code.as_ref().is_some_and(|c| c.editing);
        let code_active = matches!(state.route, Route::Code { .. })
            && (code_editing
                || state.code.as_ref().is_some_and(|c| {
                    c.ref_picker_open || c.commit_detail.is_some() || c.show_commits
                }));
        if (matches!(state.route, Route::IssueDetail { .. }) && issue_editing)
            || (matches!(state.route, Route::PrDetail { .. })
                && (pr_editing || pr_picker || pr_diff_select || pr_diff_commenting))
            || code_active
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
        Route::Code { .. } => handle_code_keys(key, state),
        Route::Security { .. } => handle_security_keys(key, state),
        Route::Insights { .. } => handle_insights_keys(key, state),
        Route::Settings { .. } => handle_settings_keys(key, state),
        Route::Discussions { .. } => handle_discussions_keys(key, state),
        Route::Gists => handle_gists_keys(key, state),
        Route::Organizations => handle_org_keys(key, state),
        Route::Dashboard => handle_dashboard_keys(key, state),
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

fn handle_palette_keys(key: KeyEvent, state: &AppState) -> Option<Message> {
    let palette = state.command_palette.as_ref().unwrap();
    match key.code {
        KeyCode::Esc => Some(Message::PaletteClose),
        KeyCode::Enter => Some(Message::PaletteSelect),
        KeyCode::Char('k') | KeyCode::Up => Some(Message::PaletteUp),
        KeyCode::Char('j') | KeyCode::Down => Some(Message::PaletteDown),
        KeyCode::Backspace => {
            let mut q = palette.query.clone();
            q.pop();
            Some(Message::PaletteInput(q))
        }
        KeyCode::Char(c) => {
            let mut q = palette.query.clone();
            q.push(c);
            Some(Message::PaletteInput(q))
        }
        _ => None,
    }
}

fn handle_search_keys(key: KeyEvent, state: &AppState) -> Option<Message> {
    let input_mode = state.search.as_ref().is_some_and(|s| s.input_mode);

    if input_mode {
        return match key.code {
            KeyCode::Esc => Some(Message::SearchCancel),
            KeyCode::Enter => Some(Message::SearchSubmit),
            KeyCode::Up => Some(Message::SearchHistoryPrev),
            KeyCode::Down => Some(Message::SearchHistoryNext),
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

fn handle_insights_keys(key: KeyEvent, state: &AppState) -> Option<Message> {
    let sidebar_focused = state.insights.as_ref().is_some_and(|s| s.sidebar_focused);

    if sidebar_focused {
        return match key.code {
            KeyCode::Char('j') | KeyCode::Down => Some(Message::TabChanged(1)),
            KeyCode::Char('k') | KeyCode::Up => Some(Message::TabChanged(usize::MAX)),
            KeyCode::Enter | KeyCode::Tab | KeyCode::BackTab => Some(Message::InsightsSidebarFocus),
            _ => None,
        };
    }

    // Content focused
    match key.code {
        KeyCode::Tab | KeyCode::BackTab | KeyCode::Esc => Some(Message::InsightsSidebarFocus),
        KeyCode::Char('j') | KeyCode::Down => Some(Message::ListSelect(1)),
        KeyCode::Char('k') | KeyCode::Up => Some(Message::ListSelect(usize::MAX)),
        KeyCode::PageDown => Some(Message::ScrollDown),
        KeyCode::PageUp => Some(Message::ScrollUp),
        _ => None,
    }
}

fn handle_security_keys(key: KeyEvent, state: &AppState) -> Option<Message> {
    let sidebar_focused = state.security.as_ref().is_some_and(|s| s.sidebar_focused);
    let detail_open = state.security.as_ref().is_some_and(|s| s.detail_open);

    if detail_open {
        return match key.code {
            KeyCode::Esc | KeyCode::Enter => Some(Message::SecurityToggleDetail),
            KeyCode::Char('j') | KeyCode::Down | KeyCode::PageDown => Some(Message::ScrollDown),
            KeyCode::Char('k') | KeyCode::Up | KeyCode::PageUp => Some(Message::ScrollUp),
            KeyCode::Char('o') => Some(Message::SecurityOpenInBrowser),
            _ => None,
        };
    }

    if sidebar_focused {
        return match key.code {
            KeyCode::Char('j') | KeyCode::Down => Some(Message::TabChanged(1)),
            KeyCode::Char('k') | KeyCode::Up => Some(Message::TabChanged(usize::MAX)),
            KeyCode::Enter | KeyCode::Tab | KeyCode::BackTab => Some(Message::SecuritySidebarFocus),
            _ => None,
        };
    }

    // Content focused
    match key.code {
        KeyCode::Tab | KeyCode::BackTab | KeyCode::Esc => Some(Message::SecuritySidebarFocus),
        KeyCode::Char('j') | KeyCode::Down => Some(Message::ListSelect(1)),
        KeyCode::Char('k') | KeyCode::Up => Some(Message::ListSelect(usize::MAX)),
        KeyCode::Enter => Some(Message::SecurityToggleDetail),
        KeyCode::Char('o') => Some(Message::SecurityOpenInBrowser),
        KeyCode::Char('d') => Some(Message::SecurityDismissAlert),
        KeyCode::Char('r') => Some(Message::SecurityReopenAlert),
        _ => None,
    }
}

fn handle_code_keys(key: KeyEvent, state: &AppState) -> Option<Message> {
    let code = state.code.as_ref();
    let sidebar_focused = code.map(|c| c.sidebar_focused).unwrap_or(true);
    let ref_picker_open = code.is_some_and(|c| c.ref_picker_open);
    let show_commits = code.is_some_and(|c| c.show_commits);
    let has_commit_detail = code.is_some_and(|c| c.commit_detail.is_some());
    let is_editing = code.is_some_and(|c| c.editing);

    // File editing mode — fullscreen editor
    if is_editing {
        let ctrl = key.modifiers.contains(KeyModifiers::CONTROL);
        return match key.code {
            KeyCode::Esc => Some(Message::CodeEditCancel),
            KeyCode::Char('s') if ctrl => Some(Message::CodeEditSubmit),
            KeyCode::Char('z') if ctrl => Some(Message::CodeEditUndo),
            KeyCode::Char('y') if ctrl => Some(Message::CodeEditRedo),
            KeyCode::Char(c) => Some(Message::CodeEditChar(c)),
            KeyCode::Enter => Some(Message::CodeEditNewline),
            KeyCode::Backspace => Some(Message::CodeEditBackspace),
            KeyCode::Delete => Some(Message::CodeEditDelete),
            KeyCode::Tab => Some(Message::CodeEditTab),
            KeyCode::Left if ctrl => Some(Message::CodeEditWordLeft),
            KeyCode::Right if ctrl => Some(Message::CodeEditWordRight),
            KeyCode::Left => Some(Message::CodeEditCursorLeft),
            KeyCode::Right => Some(Message::CodeEditCursorRight),
            KeyCode::Up => Some(Message::CodeEditCursorUp),
            KeyCode::Down => Some(Message::CodeEditCursorDown),
            KeyCode::Home => Some(Message::CodeEditHome),
            KeyCode::End => Some(Message::CodeEditEnd),
            KeyCode::PageUp => Some(Message::CodeEditPageUp),
            KeyCode::PageDown => Some(Message::CodeEditPageDown),
            _ => None,
        };
    }

    // Ref picker mode
    if ref_picker_open {
        return match key.code {
            KeyCode::Char('j') | KeyCode::Down => Some(Message::ListSelect(1)),
            KeyCode::Char('k') | KeyCode::Up => Some(Message::ListSelect(usize::MAX)),
            KeyCode::Enter => Some(Message::CodeSelectRef),
            KeyCode::Esc => Some(Message::CodeCloseRefPicker),
            _ => None,
        };
    }

    // Commit detail view
    if has_commit_detail {
        return match key.code {
            KeyCode::Esc | KeyCode::Backspace => Some(Message::CodeCloseCommitDetail),
            KeyCode::Char('j') | KeyCode::Down => Some(Message::ScrollDown),
            KeyCode::Char('k') | KeyCode::Up => Some(Message::ScrollUp),
            KeyCode::PageDown => Some(Message::ScrollDown),
            KeyCode::PageUp => Some(Message::ScrollUp),
            _ => None,
        };
    }

    // Commit list mode
    if show_commits && sidebar_focused {
        return match key.code {
            KeyCode::Char('j') | KeyCode::Down => Some(Message::ListSelect(1)),
            KeyCode::Char('k') | KeyCode::Up => Some(Message::ListSelect(usize::MAX)),
            KeyCode::Enter => Some(Message::CodeOpenCommitDetail),
            KeyCode::Char('c') => Some(Message::CodeToggleCommits),
            KeyCode::Esc => Some(Message::CodeToggleCommits),
            KeyCode::Char('b') => Some(Message::CodeOpenRefPicker),
            KeyCode::Tab | KeyCode::BackTab => Some(Message::CodeSidebarFocus),
            _ => None,
        };
    }

    if sidebar_focused {
        // File tree focused
        return match key.code {
            KeyCode::Char('j') | KeyCode::Down => Some(Message::ListSelect(1)),
            KeyCode::Char('k') | KeyCode::Up => Some(Message::ListSelect(usize::MAX)),
            KeyCode::Enter | KeyCode::Char('l') | KeyCode::Right => Some(Message::CodeNavigateInto),
            KeyCode::Backspace | KeyCode::Char('h') | KeyCode::Left => {
                Some(Message::CodeNavigateBack)
            }
            KeyCode::Tab | KeyCode::BackTab => Some(Message::CodeSidebarFocus),
            KeyCode::Char('b') => Some(Message::CodeOpenRefPicker),
            KeyCode::Char('c') => Some(Message::CodeToggleCommits),
            _ => None,
        };
    }

    // Content focused: scroll + edit
    match key.code {
        KeyCode::Char('j') | KeyCode::Down => Some(Message::ListSelect(1)),
        KeyCode::Char('k') | KeyCode::Up => Some(Message::ListSelect(usize::MAX)),
        KeyCode::PageDown => Some(Message::ScrollDown),
        KeyCode::PageUp => Some(Message::ScrollUp),
        KeyCode::Tab | KeyCode::BackTab | KeyCode::Esc => Some(Message::CodeSidebarFocus),
        KeyCode::Backspace => Some(Message::CodeNavigateBack),
        KeyCode::Char('b') => Some(Message::CodeOpenRefPicker),
        KeyCode::Char('c') => Some(Message::CodeToggleCommits),
        KeyCode::Char('e') => Some(Message::CodeStartEdit),
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

    let sidebar_focused = state
        .settings
        .as_ref()
        .map(|s| s.sidebar_focused)
        .unwrap_or(true);

    if sidebar_focused {
        // Sidebar focused: j/k navigate tabs, Enter/Tab switch to content
        return match key.code {
            KeyCode::Char('j') | KeyCode::Down => Some(Message::TabChanged(1)),
            KeyCode::Char('k') | KeyCode::Up => Some(Message::TabChanged(usize::MAX)),
            KeyCode::Enter | KeyCode::Tab | KeyCode::BackTab => Some(Message::SettingsSidebarFocus),
            _ => None,
        };
    }

    // Content focused
    let current_tab = state.settings.as_ref().map(|s| s.tab).unwrap_or(0);
    let on_general = current_tab == 0;
    let on_branch_protection = current_tab == 1;
    let on_collaborators = current_tab == 2;
    let on_webhooks = current_tab == 3;
    let on_deploy_keys = current_tab == 4;

    match key.code {
        // Tab / Esc go back to sidebar
        KeyCode::Tab | KeyCode::BackTab | KeyCode::Esc => Some(Message::SettingsSidebarFocus),
        // j/k navigate items in content
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
        KeyCode::Char('V') if on_general => Some(Message::SettingsToggleVisibility),
        // Collaborator tab: d to delete
        KeyCode::Char('d') if on_collaborators => Some(Message::SettingsDeleteCollaborator),
        // Webhook tab: d to delete, a to toggle active
        KeyCode::Char('d') if on_webhooks => Some(Message::SettingsDeleteWebhook),
        KeyCode::Char('a') if on_webhooks => Some(Message::SettingsToggleWebhook),
        // Deploy key tab: d to delete
        KeyCode::Char('d') if on_deploy_keys => Some(Message::SettingsDeleteDeployKey),
        // Branch protection tab: d to delete, e to toggle enforce admins
        KeyCode::Char('d') if on_branch_protection => Some(Message::SettingsDeleteBranchProtection),
        KeyCode::Char('e') if on_branch_protection => {
            Some(Message::SettingsToggleBranchEnforceAdmins)
        }
        _ => None,
    }
}

fn handle_actions_list_keys(key: KeyEvent, state: &AppState) -> Option<Message> {
    let list = state.actions_list.as_ref()?;

    if list.search_mode {
        return match key.code {
            KeyCode::Esc => Some(Message::ActionsSearchCancel),
            KeyCode::Enter => Some(Message::ActionsSearchSubmit),
            KeyCode::Char(c) => Some(Message::ActionsSearchInput(c.to_string())),
            KeyCode::Backspace => Some(Message::ActionsSearchInput("\x08".to_string())),
            _ => None,
        };
    }

    // Dispatch modal keys
    if let Some(ref dispatch) = list.dispatch {
        if dispatch.editing {
            return match key.code {
                KeyCode::Esc | KeyCode::Enter => Some(Message::ActionsDispatchEditDone),
                KeyCode::Char(c) => Some(Message::ActionsDispatchEditChar(c)),
                KeyCode::Backspace => Some(Message::ActionsDispatchEditBackspace),
                _ => None,
            };
        }
        let ctrl = key.modifiers.contains(KeyModifiers::CONTROL);
        return match key.code {
            KeyCode::Esc => Some(Message::ActionsDispatchClose),
            KeyCode::Char('s') if ctrl => Some(Message::ActionsDispatchSubmit),
            KeyCode::Enter => Some(Message::ActionsDispatchEditStart),
            KeyCode::Char('j') | KeyCode::Down => Some(Message::ActionsDispatchFieldNext),
            KeyCode::Char('k') | KeyCode::Up => Some(Message::ActionsDispatchFieldPrev),
            _ => None,
        };
    }

    // Workflow sidebar keys
    if list.show_workflow_sidebar && list.workflow_sidebar_focused {
        return match key.code {
            KeyCode::Char('j') | KeyCode::Down => Some(Message::ActionsWorkflowSidebarDown),
            KeyCode::Char('k') | KeyCode::Up => Some(Message::ActionsWorkflowSidebarUp),
            KeyCode::Enter => Some(Message::ActionsWorkflowSidebarSelect),
            KeyCode::Char('w') => Some(Message::ActionsToggleWorkflowSidebar),
            KeyCode::Char('d') => Some(Message::ActionsDispatchOpen),
            KeyCode::Tab | KeyCode::BackTab => Some(Message::ActionsToggleWorkflowSidebar),
            KeyCode::Esc => Some(Message::ActionsToggleWorkflowSidebar),
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
        KeyCode::Char('w') => Some(Message::ActionsToggleWorkflowSidebar),
        KeyCode::Char('d') => Some(Message::ActionsDispatchOpen),
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

fn handle_discussions_keys(key: KeyEvent, _state: &AppState) -> Option<Message> {
    match key.code {
        KeyCode::Char('j') | KeyCode::Down => Some(Message::ListSelect(1)),
        KeyCode::Char('k') | KeyCode::Up => Some(Message::ListSelect(usize::MAX)),
        KeyCode::Char('o') | KeyCode::Enter => Some(Message::DiscussionsOpenInBrowser),
        KeyCode::Esc => Some(Message::Back),
        _ => None,
    }
}

fn handle_gists_keys(key: KeyEvent, _state: &AppState) -> Option<Message> {
    match key.code {
        KeyCode::Char('j') | KeyCode::Down => Some(Message::ListSelect(1)),
        KeyCode::Char('k') | KeyCode::Up => Some(Message::ListSelect(usize::MAX)),
        KeyCode::Char('o') | KeyCode::Enter => Some(Message::GistsOpenInBrowser),
        KeyCode::Esc => Some(Message::Back),
        _ => None,
    }
}

fn handle_org_keys(key: KeyEvent, _state: &AppState) -> Option<Message> {
    match key.code {
        KeyCode::Char('j') | KeyCode::Down => Some(Message::ListSelect(1)),
        KeyCode::Char('k') | KeyCode::Up => Some(Message::ListSelect(usize::MAX)),
        KeyCode::Esc => Some(Message::Back),
        _ => None,
    }
}

fn handle_dashboard_keys(key: KeyEvent, state: &AppState) -> Option<Message> {
    if state.recent_repos.is_empty() {
        return handle_list_keys(key);
    }
    match key.code {
        KeyCode::Char('j') | KeyCode::Down => Some(Message::ListSelect(1)),
        KeyCode::Char('k') | KeyCode::Up => Some(Message::ListSelect(usize::MAX)),
        KeyCode::Enter => {
            // Navigate to the selected repo's Code tab
            if let Some(repo) = state.recent_repos.get(state.dashboard_selected) {
                let parts: Vec<&str> = repo.full_name.splitn(2, '/').collect();
                if parts.len() == 2 {
                    let repo_id = ghtui_core::types::common::RepoId::new(parts[0], parts[1]);
                    return Some(Message::Navigate(Route::Code {
                        repo: repo_id,
                        path: String::new(),
                        git_ref: repo.default_branch.clone(),
                    }));
                }
            }
            None
        }
        _ => None,
    }
}
