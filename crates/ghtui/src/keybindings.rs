use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use ghtui_core::config::KeybindingConfig;
use ghtui_core::message::ModalKind;
use ghtui_core::router::Route;
use ghtui_core::state::InputMode;
use ghtui_core::state::issue::InlineEditTarget;
use ghtui_core::state::pr::PrInlineEditTarget;
use ghtui_core::{AppState, Message};

/// Convert a KeyEvent to a human-readable string like "Ctrl+s", "Shift+Tab", "a", "Enter"
pub fn key_event_to_string(key: &KeyEvent) -> String {
    let mut parts = Vec::new();
    if key.modifiers.contains(KeyModifiers::SUPER) {
        parts.push("Cmd");
    }
    if key.modifiers.contains(KeyModifiers::CONTROL) {
        parts.push("Ctrl");
    }
    if key.modifiers.contains(KeyModifiers::ALT) {
        parts.push("Alt");
    }
    if key.modifiers.contains(KeyModifiers::SHIFT)
        && !matches!(key.code, KeyCode::Char(_) | KeyCode::BackTab)
    {
        parts.push("Shift");
    }

    let key_name = match key.code {
        KeyCode::Char(c) => c.to_string(),
        KeyCode::Enter => "Enter".to_string(),
        KeyCode::Esc => "Esc".to_string(),
        KeyCode::Backspace => "Backspace".to_string(),
        KeyCode::Tab => "Tab".to_string(),
        KeyCode::BackTab => "Shift+Tab".to_string(),
        KeyCode::Delete => "Delete".to_string(),
        KeyCode::Insert => "Insert".to_string(),
        KeyCode::Home => "Home".to_string(),
        KeyCode::End => "End".to_string(),
        KeyCode::PageUp => "PageUp".to_string(),
        KeyCode::PageDown => "PageDown".to_string(),
        KeyCode::Up => "Up".to_string(),
        KeyCode::Down => "Down".to_string(),
        KeyCode::Left => "Left".to_string(),
        KeyCode::Right => "Right".to_string(),
        KeyCode::F(n) => format!("F{n}"),
        _ => format!("{:?}", key.code),
    };

    parts.push(&key_name);
    parts.join("+")
}

pub fn handle_key(key: KeyEvent, state: &AppState) -> Option<Message> {
    // Global: Ctrl-C always quits
    if key.modifiers.contains(KeyModifiers::CONTROL) && key.code == KeyCode::Char('c') {
        return Some(Message::Quit);
    }

    // Keymap settings intercept: when capturing, grab the next keypress
    if let Some(ref ks) = state.keymap_settings {
        if ks.capturing {
            // Esc cancels capture mode without assigning
            if key.code == KeyCode::Esc && key.modifiers.is_empty() {
                return Some(Message::KeymapSettingsEdit); // toggle capturing off
            }
            let key_str = key_event_to_string(&key);
            return Some(Message::KeymapSettingsCapture(key_str));
        }
        // When keymap settings is open (but not capturing), handle its own keys
        return handle_keymap_settings_keys(key);
    }

    match state.input_mode {
        InputMode::Insert => handle_insert_mode(key, state),
        InputMode::Normal => handle_normal_mode(key, state),
    }
}

fn handle_keymap_settings_keys(key: KeyEvent) -> Option<Message> {
    match key.code {
        KeyCode::Esc | KeyCode::Char('q') => Some(Message::KeymapSettingsClose),
        KeyCode::Char('j') | KeyCode::Down => Some(Message::KeymapSettingsDown),
        KeyCode::Char('k') | KeyCode::Up => Some(Message::KeymapSettingsUp),
        KeyCode::Enter => Some(Message::KeymapSettingsEdit),
        KeyCode::Char('R') => Some(Message::KeymapSettingsReset),
        _ => None,
    }
}

fn handle_insert_mode(key: KeyEvent, state: &AppState) -> Option<Message> {
    let ks = key_event_to_string(&key);
    let kb = &state.config.keybindings;
    let ctrl = key.modifiers.contains(KeyModifiers::CONTROL);
    let alt = key.modifiers.contains(KeyModifiers::ALT);

    if ks == kb.edit_cancel {
        Some(Message::ModalClose)
    } else if ks == kb.edit_submit {
        Some(Message::ModalSubmit)
    } else if key.code == KeyCode::Enter && (ctrl || alt) {
        // Ctrl+Enter / Alt+Enter: always submit as alternate
        Some(Message::ModalSubmit)
    } else {
        match key.code {
            KeyCode::Enter => Some(Message::InputChanged("\n".to_string())),
            KeyCode::Char(c) => Some(Message::InputChanged(c.to_string())),
            KeyCode::Backspace => Some(Message::InputChanged("\x08".to_string())),
            _ => None,
        }
    }
}

fn handle_normal_mode(key: KeyEvent, state: &AppState) -> Option<Message> {
    let ks = key_event_to_string(&key);
    let kb = &state.config.keybindings;

    // Command palette intercepts all keys when open
    if state.command_palette.is_some() {
        return handle_palette_keys(key, state);
    }

    // Modal-specific keys
    if state.modal.is_some() {
        return handle_modal_keys(key, state);
    }

    // Command palette open
    if ks == kb.palette {
        return Some(Message::PaletteOpen);
    }

    // Skip global keys when editing inline or in diff comment
    let is_inline_editing = state.issue_detail.as_ref().is_some_and(|d| d.is_editing())
        || state.pr_detail.as_ref().is_some_and(|d| {
            d.is_editing() || d.diff_comment_target.is_some() || d.action_bar_focused
        })
        || state.code.as_ref().is_some_and(|c| c.editing);

    if !is_inline_editing {
        // Global keys (configurable)
        if ks == kb.quit {
            return Some(Message::Quit);
        }
        if ks == kb.home {
            return Some(Message::GoHome);
        }
        if ks == kb.help {
            return Some(Message::ModalOpen(ModalKind::Help));
        }
        if ks == kb.theme_toggle {
            return Some(Message::ToggleTheme);
        }
        if ks == kb.account_switch {
            return Some(Message::ModalOpen(ModalKind::AccountSwitcher));
        }
        if ks == kb.search {
            return Some(Message::SearchOpen);
        }
        // Tab navigation: 1-9 for global tabs (matching GitHub web)
        match key.code {
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
        Route::Notifications => handle_notification_keys(key, state),
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
                let ks = key_event_to_string(&key);
                let kb = &state.config.keybindings;
                if nav_down(&key, &ks, kb) {
                    Some(Message::ListSelect(1))
                } else if nav_up(&key, &ks, kb) {
                    Some(Message::ListSelect(usize::MAX))
                } else if key.code == KeyCode::Enter {
                    state
                        .accounts
                        .get(state.account_selected)
                        .map(|account| Message::AccountSwitch(account.clone()))
                } else {
                    None
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

    let ks = key_event_to_string(&key);
    let kb = &state.config.keybindings;

    if nav_down(&key, &ks, kb) {
        Some(Message::ListSelect(1))
    } else if nav_up(&key, &ks, kb) {
        Some(Message::ListSelect(usize::MAX))
    } else if key.code == KeyCode::Enter {
        Some(Message::SearchNavigate)
    } else if ks == kb.search_start {
        Some(Message::SearchOpen)
    } else if key.code == KeyCode::Tab {
        Some(Message::SearchCycleKind)
    } else {
        None
    }
}

fn handle_notification_keys(key: KeyEvent, state: &AppState) -> Option<Message> {
    let ks = key_event_to_string(&key);
    let kb = &state.config.keybindings;

    if nav_down(&key, &ks, kb) {
        Some(Message::ListSelect(1))
    } else if nav_up(&key, &ks, kb) {
        Some(Message::ListSelect(usize::MAX))
    } else if key.code == KeyCode::Enter {
        Some(Message::NotificationNavigate)
    } else if ks == kb.nav_refresh {
        Some(Message::Tick)
    } else {
        match key.code {
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
}

fn nav_down(key: &KeyEvent, ks: &str, kb: &KeybindingConfig) -> bool {
    ks == kb.nav_down || key.code == KeyCode::Down
}

fn nav_up(key: &KeyEvent, ks: &str, kb: &KeybindingConfig) -> bool {
    ks == kb.nav_up || key.code == KeyCode::Up
}

fn handle_list_keys(key: KeyEvent, state: &AppState) -> Option<Message> {
    let ks = key_event_to_string(&key);
    let kb = &state.config.keybindings;

    if nav_down(&key, &ks, kb) {
        Some(Message::ListSelect(1))
    } else if nav_up(&key, &ks, kb) {
        Some(Message::ListSelect(usize::MAX))
    } else if key.code == KeyCode::Enter {
        Some(Message::ListSelect(0))
    } else if ks == kb.nav_refresh {
        Some(Message::Tick)
    } else {
        None
    }
}

fn handle_insights_keys(key: KeyEvent, state: &AppState) -> Option<Message> {
    let sidebar_focused = state.insights.as_ref().is_some_and(|s| s.sidebar_focused);
    let ks = key_event_to_string(&key);
    let kb = &state.config.keybindings;

    if sidebar_focused {
        if nav_down(&key, &ks, kb) {
            Some(Message::TabChanged(1))
        } else if nav_up(&key, &ks, kb) {
            Some(Message::TabChanged(usize::MAX))
        } else {
            match key.code {
                KeyCode::Enter | KeyCode::Tab => Some(Message::InsightsSidebarFocus), // → content
                KeyCode::BackTab => Some(Message::GlobalTabPrev),                     // ← 상위 탭
                _ => None,
            }
        }
    } else {
        // Content focused
        if nav_down(&key, &ks, kb) {
            Some(Message::ListSelect(1))
        } else if nav_up(&key, &ks, kb) {
            Some(Message::ListSelect(usize::MAX))
        } else {
            match key.code {
                KeyCode::Tab => Some(Message::GlobalTabNext),
                KeyCode::BackTab | KeyCode::Esc => Some(Message::InsightsSidebarFocus),
                KeyCode::PageDown => Some(Message::ScrollDown),
                KeyCode::PageUp => Some(Message::ScrollUp),
                _ => None,
            }
        }
    }
}

fn handle_security_keys(key: KeyEvent, state: &AppState) -> Option<Message> {
    let sidebar_focused = state.security.as_ref().is_some_and(|s| s.sidebar_focused);
    let detail_open = state.security.as_ref().is_some_and(|s| s.detail_open);
    let ks = key_event_to_string(&key);
    let kb = &state.config.keybindings;

    if detail_open {
        if nav_down(&key, &ks, kb) || key.code == KeyCode::PageDown {
            Some(Message::ScrollDown)
        } else if nav_up(&key, &ks, kb) || key.code == KeyCode::PageUp {
            Some(Message::ScrollUp)
        } else if ks == kb.open_browser {
            Some(Message::SecurityOpenInBrowser)
        } else {
            match key.code {
                KeyCode::Esc | KeyCode::Enter => Some(Message::SecurityToggleDetail),
                _ => None,
            }
        }
    } else if sidebar_focused {
        if nav_down(&key, &ks, kb) {
            Some(Message::TabChanged(1))
        } else if nav_up(&key, &ks, kb) {
            Some(Message::TabChanged(usize::MAX))
        } else {
            match key.code {
                KeyCode::Enter | KeyCode::Tab => Some(Message::SecuritySidebarFocus),
                KeyCode::BackTab => Some(Message::GlobalTabPrev),
                _ => None,
            }
        }
    } else {
        // Content focused
        if nav_down(&key, &ks, kb) {
            Some(Message::ListSelect(1))
        } else if nav_up(&key, &ks, kb) {
            Some(Message::ListSelect(usize::MAX))
        } else if ks == kb.open_browser {
            Some(Message::SecurityOpenInBrowser)
        } else if ks == kb.delete {
            Some(Message::SecurityDismissAlert)
        } else {
            match key.code {
                KeyCode::Tab => Some(Message::GlobalTabNext),
                KeyCode::BackTab | KeyCode::Esc => Some(Message::SecuritySidebarFocus),
                KeyCode::Enter => Some(Message::SecurityToggleDetail),
                KeyCode::Char('r') => Some(Message::SecurityReopenAlert),
                _ => None,
            }
        }
    }
}

fn handle_code_keys(key: KeyEvent, state: &AppState) -> Option<Message> {
    let code = state.code.as_ref();
    let sidebar_focused = code.map(|c| c.sidebar_focused).unwrap_or(true);
    let ref_picker_open = code.is_some_and(|c| c.ref_picker_open);
    let show_commits = code.is_some_and(|c| c.show_commits);
    let has_commit_detail = code.is_some_and(|c| c.commit_detail.is_some());
    let is_editing = code.is_some_and(|c| c.editing);
    let ks = key_event_to_string(&key);
    let kb = &state.config.keybindings;

    // File editing mode — fullscreen editor
    if is_editing {
        let ctrl = key.modifiers.contains(KeyModifiers::CONTROL);
        let super_key = key.modifiers.contains(KeyModifiers::SUPER);
        let shift = key.modifiers.contains(KeyModifiers::SHIFT);
        let alt = key.modifiers.contains(KeyModifiers::ALT);
        let cmd = ctrl || super_key; // Cmd on macOS, Ctrl on Linux/Windows

        // Configurable editor bindings
        if ks == kb.edit_cancel {
            return Some(Message::CodeEditCancel);
        }
        if ks == kb.edit_submit {
            return Some(Message::CodeEditSubmit);
        }
        if ks == kb.edit_undo {
            return Some(Message::CodeEditUndo);
        }
        if ks == kb.edit_redo {
            return Some(Message::CodeEditRedo);
        }

        return match key.code {
            // Cmd shortcuts (non-configurable editor commands)
            KeyCode::Char('a') if cmd => Some(Message::CodeEditSelectAll),
            KeyCode::Char('x') if cmd => Some(Message::CodeEditCut),
            KeyCode::Char('c') if cmd => Some(Message::CodeEditCopy),
            KeyCode::Char('v') if cmd => {
                // Read clipboard and send as paste message
                let text = arboard::Clipboard::new()
                    .ok()
                    .and_then(|mut cb| cb.get_text().ok())
                    .unwrap_or_default();
                Some(Message::CodeEditPaste(text))
            }
            // Cmd+Shift+Arrow — select to line start/end, doc top/bottom
            KeyCode::Left if cmd && shift => Some(Message::CodeEditSelectToLineStart),
            KeyCode::Right if cmd && shift => Some(Message::CodeEditSelectToLineEnd),
            KeyCode::Up if cmd && shift => Some(Message::CodeEditSelectToDocTop),
            KeyCode::Down if cmd && shift => Some(Message::CodeEditSelectToDocBottom),
            // Alt+Shift+Arrow — select word
            KeyCode::Left if alt && shift => Some(Message::CodeEditSelectWordLeft),
            KeyCode::Right if alt && shift => Some(Message::CodeEditSelectWordRight),
            // Cmd+Arrow — line/doc navigation (no selection)
            KeyCode::Left if cmd => Some(Message::CodeEditMoveLineStart),
            KeyCode::Right if cmd => Some(Message::CodeEditMoveLineEnd),
            KeyCode::Up if cmd => Some(Message::CodeEditMoveDocTop),
            KeyCode::Down if cmd => Some(Message::CodeEditMoveDocBottom),
            // Alt/Option+Arrow — word movement
            KeyCode::Left if alt => Some(Message::CodeEditWordLeft),
            KeyCode::Right if alt => Some(Message::CodeEditWordRight),
            // Shift+Arrow — char selection
            KeyCode::Left if shift => Some(Message::CodeEditSelectLeft),
            KeyCode::Right if shift => Some(Message::CodeEditSelectRight),
            KeyCode::Up if shift => Some(Message::CodeEditSelectUp),
            KeyCode::Down if shift => Some(Message::CodeEditSelectDown),
            // Regular keys
            KeyCode::Char(c) => Some(Message::CodeEditChar(c)),
            KeyCode::Enter => Some(Message::CodeEditNewline),
            KeyCode::Backspace => Some(Message::CodeEditBackspace),
            KeyCode::Delete => Some(Message::CodeEditDelete),
            KeyCode::Tab => Some(Message::CodeEditTab),
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
        if nav_down(&key, &ks, kb) {
            return Some(Message::ListSelect(1));
        } else if nav_up(&key, &ks, kb) {
            return Some(Message::ListSelect(usize::MAX));
        }
        return match key.code {
            KeyCode::Enter => Some(Message::CodeSelectRef),
            KeyCode::Esc => Some(Message::CodeCloseRefPicker),
            _ => None,
        };
    }

    // Commit detail view
    if has_commit_detail {
        if nav_down(&key, &ks, kb) || key.code == KeyCode::PageDown {
            return Some(Message::ScrollDown);
        } else if nav_up(&key, &ks, kb) || key.code == KeyCode::PageUp {
            return Some(Message::ScrollUp);
        }
        return match key.code {
            KeyCode::Esc | KeyCode::Backspace => Some(Message::CodeCloseCommitDetail),
            _ => None,
        };
    }

    // Commit list mode
    if show_commits && sidebar_focused {
        if nav_down(&key, &ks, kb) {
            return Some(Message::ListSelect(1));
        } else if nav_up(&key, &ks, kb) {
            return Some(Message::ListSelect(usize::MAX));
        } else if ks == kb.code_commits {
            return Some(Message::CodeToggleCommits);
        } else if ks == kb.code_branch {
            return Some(Message::CodeOpenRefPicker);
        }
        return match key.code {
            KeyCode::Enter => Some(Message::CodeOpenCommitDetail),
            KeyCode::Esc => Some(Message::CodeToggleCommits),
            KeyCode::Tab => Some(Message::CodeSidebarFocus), // → content
            KeyCode::BackTab => Some(Message::GlobalTabPrev), // ← 상위 탭
            _ => None,
        };
    }

    if sidebar_focused {
        // File tree focused
        if nav_down(&key, &ks, kb) {
            return Some(Message::ListSelect(1));
        } else if nav_up(&key, &ks, kb) {
            return Some(Message::ListSelect(usize::MAX));
        } else if ks == kb.code_branch {
            return Some(Message::CodeOpenRefPicker);
        } else if ks == kb.code_commits {
            return Some(Message::CodeToggleCommits);
        }
        return match key.code {
            KeyCode::Enter | KeyCode::Char('l') | KeyCode::Right => Some(Message::CodeNavigateInto),
            KeyCode::Backspace | KeyCode::Char('h') | KeyCode::Left => {
                Some(Message::CodeNavigateBack)
            }
            KeyCode::Tab => Some(Message::CodeSidebarFocus), // → content
            KeyCode::BackTab => Some(Message::GlobalTabPrev), // ← 상위 탭
            _ => None,
        };
    }

    // Content focused: scroll + edit
    if nav_down(&key, &ks, kb) {
        Some(Message::ListSelect(1))
    } else if nav_up(&key, &ks, kb) {
        Some(Message::ListSelect(usize::MAX))
    } else if ks == kb.code_branch {
        Some(Message::CodeOpenRefPicker)
    } else if ks == kb.code_commits {
        Some(Message::CodeToggleCommits)
    } else if ks == kb.code_edit {
        Some(Message::CodeStartEdit)
    } else {
        match key.code {
            KeyCode::Tab => Some(Message::GlobalTabNext),
            KeyCode::BackTab | KeyCode::Esc => Some(Message::CodeSidebarFocus),
            KeyCode::PageDown => Some(Message::ScrollDown),
            KeyCode::PageUp => Some(Message::ScrollUp),
            KeyCode::Backspace => Some(Message::CodeNavigateBack),
            _ => None,
        }
    }
}

fn handle_settings_keys(key: KeyEvent, state: &AppState) -> Option<Message> {
    let is_editing = state.settings.as_ref().is_some_and(|s| s.is_editing());
    let ks = key_event_to_string(&key);
    let kb = &state.config.keybindings;

    if is_editing {
        if ks == kb.edit_cancel {
            return Some(Message::SettingsEditCancel);
        }
        if ks == kb.edit_submit {
            return Some(Message::SettingsEditSubmit);
        }
        return match key.code {
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
        if nav_down(&key, &ks, kb) {
            return Some(Message::TabChanged(1));
        } else if nav_up(&key, &ks, kb) {
            return Some(Message::TabChanged(usize::MAX));
        }
        return match key.code {
            KeyCode::Enter | KeyCode::Tab => Some(Message::SettingsSidebarFocus), // → content
            KeyCode::BackTab => Some(Message::GlobalTabPrev),                     // ← 상위 탭
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

    if nav_down(&key, &ks, kb) {
        Some(Message::ListSelect(1))
    } else if nav_up(&key, &ks, kb) {
        Some(Message::ListSelect(usize::MAX))
    } else {
        match key.code {
            KeyCode::Tab => Some(Message::GlobalTabNext),
            KeyCode::BackTab | KeyCode::Esc => Some(Message::SettingsSidebarFocus),
            KeyCode::PageDown => Some(Message::ScrollDown),
            KeyCode::PageUp => Some(Message::ScrollUp),
            // Edit keys (only on General tab)
            KeyCode::Char('d') if on_general => {
                Some(Message::SettingsStartEdit("description".to_string()))
            }
            KeyCode::Char('b') if on_general => {
                Some(Message::SettingsStartEdit("default_branch".to_string()))
            }
            KeyCode::Char('T') if on_general => {
                Some(Message::SettingsStartEdit("topics".to_string()))
            }
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
            KeyCode::Char('d') if on_branch_protection => {
                Some(Message::SettingsDeleteBranchProtection)
            }
            KeyCode::Char('e') if on_branch_protection => {
                Some(Message::SettingsToggleBranchEnforceAdmins)
            }
            _ => None,
        }
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

    let ks = key_event_to_string(&key);
    let kb = &state.config.keybindings;

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
        if ks == kb.edit_submit {
            return Some(Message::ActionsDispatchSubmit);
        }
        if nav_down(&key, &ks, kb) {
            return Some(Message::ActionsDispatchFieldNext);
        }
        if nav_up(&key, &ks, kb) {
            return Some(Message::ActionsDispatchFieldPrev);
        }
        return match key.code {
            KeyCode::Esc => Some(Message::ActionsDispatchClose),
            KeyCode::Enter => Some(Message::ActionsDispatchEditStart),
            _ => None,
        };
    }

    // Workflow sidebar keys
    if list.show_workflow_sidebar && list.workflow_sidebar_focused {
        if nav_down(&key, &ks, kb) {
            return Some(Message::ActionsWorkflowSidebarDown);
        } else if nav_up(&key, &ks, kb) {
            return Some(Message::ActionsWorkflowSidebarUp);
        }
        return match key.code {
            KeyCode::Enter => Some(Message::ActionsWorkflowSidebarSelect),
            KeyCode::Char('w') => Some(Message::ActionsToggleWorkflowSidebar),
            KeyCode::Char('d') => Some(Message::ActionsDispatchOpen),
            KeyCode::Tab => Some(Message::ActionsToggleWorkflowSidebar), // → run list
            KeyCode::Esc => Some(Message::ActionsToggleWorkflowSidebar),
            _ => None,
        };
    }

    if nav_down(&key, &ks, kb) {
        Some(Message::ListSelect(1))
    } else if nav_up(&key, &ks, kb) {
        Some(Message::ListSelect(usize::MAX))
    } else if ks == kb.nav_open {
        Some(Message::ListSelect(0))
    } else if ks == kb.nav_refresh {
        Some(Message::Tick)
    } else if ks == kb.filter_toggle {
        Some(Message::ActionsToggleStatus)
    } else if ks == kb.nav_next_page {
        Some(Message::ActionsNextPage)
    } else if ks == kb.nav_prev_page {
        Some(Message::ActionsPrevPage)
    } else if ks == kb.search_start {
        Some(Message::ActionsSearchStart)
    } else if ks == kb.open_browser {
        Some(Message::ActionsOpenInBrowser)
    } else if ks == kb.filter_clear {
        Some(Message::ActionsFilterClear)
    } else {
        match key.code {
            KeyCode::Char('e') => Some(Message::ActionsCycleEvent),
            KeyCode::Char('x') => Some(Message::ActionsCancelRun),
            KeyCode::Char('R') => Some(Message::ActionsRerunRun),
            KeyCode::Char('w') => Some(Message::ActionsToggleWorkflowSidebar),
            KeyCode::Char('d') => Some(Message::ActionsDispatchOpen),
            _ => None,
        }
    }
}

fn handle_action_detail_keys(key: KeyEvent, state: &AppState) -> Option<Message> {
    use ghtui_core::state::ActionDetailFocus;

    let focus = state
        .action_detail
        .as_ref()
        .map(|d| d.focus)
        .unwrap_or(ActionDetailFocus::Jobs);
    let ks = key_event_to_string(&key);
    let kb = &state.config.keybindings;

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
        ActionDetailFocus::Log => {
            if nav_down(&key, &ks, kb) || key.code == KeyCode::PageDown {
                Some(Message::ScrollDown)
            } else if nav_up(&key, &ks, kb) || key.code == KeyCode::PageUp {
                Some(Message::ScrollUp)
            } else if ks == kb.open_browser {
                Some(Message::ActionDetailOpenInBrowser)
            } else {
                match key.code {
                    KeyCode::Char('d') if key.modifiers.contains(KeyModifiers::CONTROL) => {
                        Some(Message::ScrollDown)
                    }
                    KeyCode::Char('u') if key.modifiers.contains(KeyModifiers::CONTROL) => {
                        Some(Message::ScrollUp)
                    }
                    KeyCode::Tab => Some(Message::ActionDetailFocusJobs),
                    KeyCode::BackTab => Some(Message::ActionDetailActionBarFocus),
                    KeyCode::Char('x') => Some(Message::ActionDetailActionBarFocus),
                    _ => None,
                }
            }
        }
        ActionDetailFocus::Jobs => {
            if nav_down(&key, &ks, kb) {
                Some(Message::ListSelect(1))
            } else if nav_up(&key, &ks, kb) {
                Some(Message::ListSelect(usize::MAX))
            } else if ks == kb.open_browser {
                Some(Message::ActionDetailOpenInBrowser)
            } else {
                match key.code {
                    KeyCode::Enter => Some(Message::ListSelect(0)),
                    KeyCode::Tab => Some(Message::ActionDetailFocusLog),
                    KeyCode::BackTab => Some(Message::ActionDetailActionBarFocus),
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
                    KeyCode::Char('x') => Some(Message::ActionDetailActionBarFocus),
                    _ => None,
                }
            }
        }
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

    let ks = key_event_to_string(&key);
    let kb = &state.config.keybindings;

    if nav_down(&key, &ks, kb) {
        Some(Message::ListSelect(1))
    } else if nav_up(&key, &ks, kb) {
        Some(Message::ListSelect(usize::MAX))
    } else if ks == kb.nav_open {
        Some(Message::ListSelect(0))
    } else if ks == kb.nav_refresh {
        Some(Message::Tick)
    } else if ks == kb.filter_toggle {
        Some(Message::PrToggleStateFilter)
    } else if ks == kb.nav_next_page {
        Some(Message::PrNextPage)
    } else if ks == kb.nav_prev_page {
        Some(Message::PrPrevPage)
    } else if ks == kb.create {
        Some(Message::ModalOpen(ModalKind::CreatePr))
    } else if ks == kb.search_start {
        Some(Message::PrSearchStart)
    } else if ks == kb.sort_cycle {
        Some(Message::PrSortCycle)
    } else if ks == kb.filter_clear {
        Some(Message::PrFilterClear)
    } else {
        None
    }
}

fn handle_pr_detail_keys(key: KeyEvent, state: &AppState) -> Option<Message> {
    let is_editing = state.pr_detail.as_ref().is_some_and(|d| d.is_editing());
    let ks = key_event_to_string(&key);
    let kb = &state.config.keybindings;

    if is_editing {
        let is_title_edit = state
            .pr_detail
            .as_ref()
            .is_some_and(|d| matches!(d.edit_target, Some(PrInlineEditTarget::PrTitle)));

        let ctrl = key.modifiers.contains(KeyModifiers::CONTROL);
        let alt = key.modifiers.contains(KeyModifiers::ALT);

        // Configurable editor bindings
        if ks == kb.edit_cancel {
            return Some(Message::PrEditCancel);
        }
        if ks == kb.edit_submit {
            return Some(Message::PrEditSubmit);
        }
        if ks == kb.edit_undo {
            return Some(Message::PrEditUndo);
        }
        if ks == kb.edit_redo {
            return Some(Message::PrEditRedo);
        }

        return match key.code {
            KeyCode::Enter if ctrl || alt || is_title_edit => Some(Message::PrEditSubmit),
            KeyCode::Enter => Some(Message::PrEditNewline),
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
                if nav_down(&key, &ks, kb) {
                    return Some(Message::ListSelect(1));
                } else if nav_up(&key, &ks, kb) {
                    return Some(Message::ListSelect(usize::MAX));
                }
                return match key.code {
                    KeyCode::Esc => Some(Message::PrLabelCancel),
                    KeyCode::Enter | KeyCode::Char(' ') => detail
                        .label_picker
                        .as_ref()
                        .map(|p| Message::PrLabelSelect(p.cursor)),
                    KeyCode::Char('s') => Some(Message::PrLabelApply),
                    _ => None,
                };
            }
            if detail.assignee_picker.is_some() {
                if nav_down(&key, &ks, kb) {
                    return Some(Message::ListSelect(1));
                } else if nav_up(&key, &ks, kb) {
                    return Some(Message::ListSelect(usize::MAX));
                }
                return match key.code {
                    KeyCode::Esc => Some(Message::PrAssigneeCancel),
                    KeyCode::Enter | KeyCode::Char(' ') => detail
                        .assignee_picker
                        .as_ref()
                        .map(|p| Message::PrAssigneeSelect(p.cursor)),
                    KeyCode::Char('s') => Some(Message::PrAssigneeApply),
                    _ => None,
                };
            }
            if detail.milestone_picker.is_some() {
                if nav_down(&key, &ks, kb) {
                    return Some(Message::ListSelect(1));
                } else if nav_up(&key, &ks, kb) {
                    return Some(Message::ListSelect(usize::MAX));
                }
                return match key.code {
                    KeyCode::Esc => Some(Message::PrMilestoneCancel),
                    KeyCode::Enter | KeyCode::Char(' ') => detail
                        .milestone_picker
                        .as_ref()
                        .map(|p| Message::PrMilestoneSelect(p.cursor)),
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

            if ks == kb.edit_cancel {
                return Some(Message::PrDiffCommentCancel);
            }
            if ks == kb.edit_submit {
                return Some(Message::PrDiffCommentSubmit);
            }

            return match key.code {
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
            if nav_down(&key, &ks, kb) {
                return Some(Message::PrDiffTreeDown);
            } else if nav_up(&key, &ks, kb) {
                return Some(Message::PrDiffTreeUp);
            } else if ks == kb.open_browser {
                return Some(Message::PrOpenInBrowser);
            }
            return match key.code {
                KeyCode::Enter => Some(Message::PrDiffTreeSelect),
                KeyCode::Char('f') => Some(Message::PrDiffToggleTree),
                KeyCode::Char('V') => Some(Message::PrDiffMarkViewed),
                KeyCode::Tab => Some(Message::PrDiffTreeFocus), // switch to diff
                KeyCode::BackTab => Some(Message::TabChanged(usize::MAX)),
                KeyCode::Esc => Some(Message::PrDiffTreeFocus), // unfocus tree
                _ => None,
            };
        }

        // Diff focused
        let shift = key.modifiers.contains(KeyModifiers::SHIFT);

        if shift && nav_down(&key, &ks, kb) {
            return Some(Message::PrDiffSelectDown);
        } else if shift && nav_up(&key, &ks, kb) {
            return Some(Message::PrDiffSelectUp);
        }

        if nav_down(&key, &ks, kb) {
            return Some(Message::PrDiffCursorDown);
        } else if nav_up(&key, &ks, kb) {
            return Some(Message::PrDiffCursorUp);
        } else if ks == kb.open_browser {
            return Some(Message::PrOpenInBrowser);
        }

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
            KeyCode::Char('J') => Some(Message::PrDiffSelectDown),
            KeyCode::Char('K') => Some(Message::PrDiffSelectUp),
            KeyCode::Enter => Some(Message::PrDiffToggleCollapse),
            KeyCode::Char('l') | KeyCode::Right => Some(Message::PrDiffExpand),
            KeyCode::Char('h') | KeyCode::Left => Some(Message::PrDiffCollapse),
            KeyCode::Char('V') => Some(Message::PrDiffMarkViewed),
            KeyCode::Esc => Some(Message::PrDiffClearSelection),
            KeyCode::PageDown => Some(Message::ScrollDown),
            KeyCode::PageUp => Some(Message::ScrollUp),
            KeyCode::Char('s') => Some(Message::PrDiffToggleSideBySide),
            KeyCode::Char('z') => Some(Message::PrReviewThreadToggleResolve),
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
    if nav_down(&key, &ks, kb) {
        Some(Message::ListSelect(1))
    } else if nav_up(&key, &ks, kb) {
        Some(Message::ListSelect(usize::MAX))
    } else if ks == kb.open_browser {
        Some(Message::PrOpenInBrowser)
    } else if ks == kb.delete {
        Some(Message::PrDeleteComment)
    } else {
        match key.code {
            KeyCode::Tab => Some(Message::TabChanged(1)),
            KeyCode::BackTab => Some(Message::TabChanged(usize::MAX)),
            KeyCode::Char('e') => Some(Message::PrStartEditBody),
            KeyCode::Char('c') => Some(Message::PrStartComment),
            KeyCode::Char('r') => Some(Message::PrStartReply),
            KeyCode::Char('l') => Some(Message::PrLabelToggle),
            KeyCode::Char('a') => Some(Message::PrAssigneeToggle),
            KeyCode::Char('v') => Some(Message::PrReviewerToggle),
            KeyCode::Char('m') => Some(Message::ModalOpen(ModalKind::MergePr)),
            KeyCode::Char('x') => Some(Message::PrToggleState),
            KeyCode::Char('A') => Some(Message::PrApprove),
            KeyCode::Char('R') => Some(Message::PrRequestChanges),
            KeyCode::Char('D') => Some(Message::PrDraftToggle),
            KeyCode::Char('G') => Some(Message::PrAutoMergeToggle),
            KeyCode::Char('M') => Some(Message::PrMilestoneToggle),
            KeyCode::Char('b') => Some(Message::PrChangeBase),
            KeyCode::Char('+') => Some(Message::PrAddReaction("+1".to_string())),
            KeyCode::Char('-') => Some(Message::PrAddReaction("-1".to_string())),
            KeyCode::PageDown => Some(Message::ScrollDown),
            KeyCode::PageUp => Some(Message::ScrollUp),
            _ => None,
        }
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

    let ks = key_event_to_string(&key);
    let kb = &state.config.keybindings;

    if nav_down(&key, &ks, kb) {
        Some(Message::ListSelect(1))
    } else if nav_up(&key, &ks, kb) {
        Some(Message::ListSelect(usize::MAX))
    } else if ks == kb.nav_open {
        Some(Message::ListSelect(0))
    } else if ks == kb.nav_refresh {
        Some(Message::Tick)
    } else if ks == kb.filter_toggle {
        Some(Message::IssueToggleStateFilter)
    } else if ks == kb.nav_next_page {
        Some(Message::IssueNextPage)
    } else if ks == kb.nav_prev_page {
        Some(Message::IssuePrevPage)
    } else if ks == kb.create {
        Some(Message::ModalOpen(ModalKind::CreateIssue))
    } else if ks == kb.search_start {
        Some(Message::IssueSearchStart)
    } else if ks == kb.sort_cycle {
        Some(Message::IssueSortCycle)
    } else if ks == kb.filter_clear {
        Some(Message::IssueFilterClear)
    } else {
        None
    }
}

fn handle_issue_detail_keys(key: KeyEvent, state: &AppState) -> Option<Message> {
    let is_editing = state.issue_detail.as_ref().is_some_and(|d| d.is_editing());
    let ks = key_event_to_string(&key);
    let kb = &state.config.keybindings;

    if is_editing {
        // Inline editing mode
        let is_title_edit = state
            .issue_detail
            .as_ref()
            .is_some_and(|d| matches!(d.edit_target, Some(InlineEditTarget::IssueTitle)));

        let ctrl = key.modifiers.contains(KeyModifiers::CONTROL);
        let alt = key.modifiers.contains(KeyModifiers::ALT);

        // Configurable editor bindings
        if ks == kb.edit_cancel {
            return Some(Message::IssueEditCancel);
        }
        if ks == kb.edit_submit {
            return Some(Message::IssueEditSubmit);
        }
        if ks == kb.edit_undo {
            return Some(Message::IssueEditUndo);
        }
        if ks == kb.edit_redo {
            return Some(Message::IssueEditRedo);
        }

        return match key.code {
            KeyCode::Enter if ctrl || alt || is_title_edit => Some(Message::IssueEditSubmit),
            KeyCode::Enter => Some(Message::IssueEditNewline),
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
                if nav_down(&key, &ks, kb) {
                    return Some(Message::ListSelect(1));
                } else if nav_up(&key, &ks, kb) {
                    return Some(Message::ListSelect(usize::MAX));
                }
                return match key.code {
                    KeyCode::Esc => Some(Message::IssueLabelCancel),
                    KeyCode::Enter | KeyCode::Char(' ') => detail
                        .label_picker
                        .as_ref()
                        .map(|p| Message::IssueLabelSelect(p.cursor)),
                    KeyCode::Char('s') => Some(Message::IssueLabelApply),
                    _ => None,
                };
            }
            if detail.assignee_picker.is_some() {
                if nav_down(&key, &ks, kb) {
                    return Some(Message::ListSelect(1));
                } else if nav_up(&key, &ks, kb) {
                    return Some(Message::ListSelect(usize::MAX));
                }
                return match key.code {
                    KeyCode::Esc => Some(Message::IssueAssigneeCancel),
                    KeyCode::Enter | KeyCode::Char(' ') => detail
                        .assignee_picker
                        .as_ref()
                        .map(|p| Message::IssueAssigneeSelect(p.cursor)),
                    KeyCode::Char('s') => Some(Message::IssueAssigneeApply),
                    _ => None,
                };
            }
            if detail.milestone_picker.is_some() {
                if nav_down(&key, &ks, kb) {
                    return Some(Message::ListSelect(1));
                } else if nav_up(&key, &ks, kb) {
                    return Some(Message::ListSelect(usize::MAX));
                }
                return match key.code {
                    KeyCode::Esc => Some(Message::IssueMilestoneCancel),
                    KeyCode::Enter | KeyCode::Char(' ') => detail
                        .milestone_picker
                        .as_ref()
                        .map(|p| Message::IssueMilestoneSelect(p.cursor)),
                    KeyCode::Char('s') => Some(Message::IssueMilestoneApply),
                    KeyCode::Char('0') => Some(Message::IssueMilestoneClear),
                    _ => None,
                };
            }
        }
        return None;
    }

    // Normal section navigation mode
    if nav_down(&key, &ks, kb) {
        Some(Message::ListSelect(1))
    } else if nav_up(&key, &ks, kb) {
        Some(Message::ListSelect(usize::MAX))
    } else if ks == kb.open_browser {
        Some(Message::IssueOpenInBrowser)
    } else if ks == kb.delete {
        Some(Message::IssueDeleteComment)
    } else {
        match key.code {
            KeyCode::Char('e') => Some(Message::IssueStartEditBody),
            KeyCode::Char('c') => Some(Message::IssueStartComment),
            KeyCode::Char('r') => Some(Message::IssueStartReply),
            KeyCode::Char('l') => Some(Message::IssueLabelToggle),
            KeyCode::Char('a') => Some(Message::IssueAssigneeToggle),
            KeyCode::Char('m') => Some(Message::IssueMilestoneToggle),
            KeyCode::Char('x') => Some(Message::IssueToggleState),
            KeyCode::Char('L') => Some(Message::IssueLockToggle),
            KeyCode::Char('P') => Some(Message::IssuePinToggle),
            KeyCode::Char('X') => Some(Message::IssueTransfer), // Shift+X: transfer
            // Quick reactions
            KeyCode::Char('+') => Some(Message::IssueAddReaction("+1".to_string())),
            KeyCode::Char('-') => Some(Message::IssueAddReaction("-1".to_string())),
            KeyCode::PageDown => Some(Message::ScrollDown),
            KeyCode::PageUp => Some(Message::ScrollUp),
            _ => None,
        }
    }
}

fn handle_discussions_keys(key: KeyEvent, state: &AppState) -> Option<Message> {
    let ks = key_event_to_string(&key);
    let kb = &state.config.keybindings;

    if nav_down(&key, &ks, kb) {
        Some(Message::ListSelect(1))
    } else if nav_up(&key, &ks, kb) {
        Some(Message::ListSelect(usize::MAX))
    } else if ks == kb.open_browser || key.code == KeyCode::Enter {
        Some(Message::DiscussionsOpenInBrowser)
    } else if key.code == KeyCode::Esc {
        Some(Message::Back)
    } else {
        None
    }
}

fn handle_gists_keys(key: KeyEvent, state: &AppState) -> Option<Message> {
    let ks = key_event_to_string(&key);
    let kb = &state.config.keybindings;

    if nav_down(&key, &ks, kb) {
        Some(Message::ListSelect(1))
    } else if nav_up(&key, &ks, kb) {
        Some(Message::ListSelect(usize::MAX))
    } else if ks == kb.open_browser || key.code == KeyCode::Enter {
        Some(Message::GistsOpenInBrowser)
    } else if key.code == KeyCode::Esc {
        Some(Message::Back)
    } else {
        None
    }
}

fn handle_org_keys(key: KeyEvent, state: &AppState) -> Option<Message> {
    let ks = key_event_to_string(&key);
    let kb = &state.config.keybindings;

    if nav_down(&key, &ks, kb) {
        Some(Message::ListSelect(1))
    } else if nav_up(&key, &ks, kb) {
        Some(Message::ListSelect(usize::MAX))
    } else if key.code == KeyCode::Esc {
        Some(Message::Back)
    } else {
        None
    }
}

fn handle_dashboard_keys(key: KeyEvent, state: &AppState) -> Option<Message> {
    if state.recent_repos.is_empty() {
        return handle_list_keys(key, state);
    }

    let ks = key_event_to_string(&key);
    let kb = &state.config.keybindings;

    if nav_down(&key, &ks, kb) {
        Some(Message::ListSelect(1))
    } else if nav_up(&key, &ks, kb) {
        Some(Message::ListSelect(usize::MAX))
    } else if key.code == KeyCode::Enter {
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
    } else {
        None
    }
}
