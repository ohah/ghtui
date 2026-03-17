use ghtui_core::AppState;
use ghtui_core::message::ModalKind;
use ghtui_core::router::{Route, TAB_LABELS};
use ghtui_core::theme::Theme;
use ratatui::Frame;
use ratatui::layout::{Constraint, Direction, Layout, Rect};
use ratatui::style::{Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Paragraph};

use crate::views;

pub fn render(frame: &mut Frame, state: &AppState, _tick: usize) {
    let theme = &state.theme;
    let size = frame.area();

    // Set background
    let bg_block = Block::default().style(Style::default().bg(theme.bg));
    frame.render_widget(bg_block, size);

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(1), // Repo header
            Constraint::Length(1), // Tab bar
            Constraint::Min(0),    // Main content
            Constraint::Length(1), // Footer
        ])
        .split(size);

    // Repo header (like GitHub's top bar)
    render_repo_header(frame, state, theme, chunks[0]);

    // Global tab bar
    render_global_tabs(frame, state, theme, chunks[1]);

    // Main content
    let content_area = chunks[2];
    match &state.route {
        Route::Dashboard | Route::Code { .. } => {
            views::dashboard::render(frame, state, content_area)
        }
        Route::PrList { .. } => views::pr_list::render(frame, state, content_area),
        Route::PrDetail { .. } => views::pr_detail::render(frame, state, content_area),
        Route::IssueList { .. } => views::issue_list::render(frame, state, content_area),
        Route::IssueDetail { .. } => views::issue_detail::render(frame, state, content_area),
        Route::ActionsList { .. } => views::actions_list::render(frame, state, content_area),
        Route::ActionDetail { .. } | Route::JobLog { .. } => {
            views::action_detail::render(frame, state, content_area)
        }
        Route::Notifications => views::notifications::render(frame, state, content_area),
        Route::Security { .. } => views::security::render(frame, state, content_area),
        Route::Insights { .. } => views::insights::render(frame, state, content_area),
        Route::Settings { .. } => views::settings::render(frame, state, content_area),
        Route::Search { .. } => views::placeholder::render(
            frame,
            state,
            content_area,
            "Search",
            "Search code, issues, pull requests, and more",
        ),
    }

    // Footer
    render_footer(frame, state, theme, chunks[3]);

    // Toast overlay
    render_toasts(frame, state, theme, size);

    // Modal overlay
    if let Some(ref modal) = state.modal {
        match modal {
            ModalKind::Help => views::help::render(frame, size),
            ModalKind::AccountSwitcher => views::account_switcher::render(frame, state, size),
            ModalKind::AddComment => {
                views::input_modal::render(frame, state, size, "Add Comment", "Enter your comment:")
            }
            ModalKind::CreateIssue => views::input_modal::render(
                frame,
                state,
                size,
                "Create Issue",
                "First line = Title, rest = Body",
            ),
            _ => {}
        }
    }
}

fn render_repo_header(frame: &mut Frame, state: &AppState, theme: &Theme, area: Rect) {
    let repo_name = state
        .current_repo
        .as_ref()
        .map(|r| r.full_name())
        .unwrap_or_else(|| "No repository".to_string());

    let (owner, name) = repo_name.split_once('/').unwrap_or(("", &repo_name));

    let loading_indicator = if !state.loading.is_empty() {
        Span::styled(" ● ", Style::default().fg(theme.warning))
    } else {
        Span::raw("")
    };

    let line = Line::from(vec![
        Span::styled("  ", Style::default()),
        Span::styled(owner.to_string(), Style::default().fg(theme.accent)),
        Span::styled(" / ", Style::default().fg(theme.fg_dim)),
        Span::styled(
            name.to_string(),
            Style::default()
                .fg(theme.accent)
                .add_modifier(Modifier::BOLD),
        ),
        loading_indicator,
    ]);

    let paragraph = Paragraph::new(line).style(Style::default().bg(theme.header_bg));
    frame.render_widget(paragraph, area);
}

fn render_global_tabs(frame: &mut Frame, state: &AppState, theme: &Theme, area: Rect) {
    // Build spans and track each tab's x position and width for underline
    let mut spans: Vec<Span> = Vec::new();
    let mut tab_positions: Vec<(u16, u16)> = Vec::new(); // (label_start_x, label_width)
    let mut x: u16 = 0;

    for (i, label) in TAB_LABELS.iter().enumerate() {
        let is_active = i == state.active_tab;

        // Key: " N "
        let key_text = format!(" {} ", i + 1);
        let key_width = key_text.len() as u16;
        let key_style = if is_active {
            Style::default()
                .fg(theme.tab_active_fg)
                .bg(theme.header_bg)
                .add_modifier(Modifier::DIM)
        } else {
            Style::default()
                .fg(theme.fg_muted)
                .bg(theme.header_bg)
                .add_modifier(Modifier::DIM)
        };
        spans.push(Span::styled(key_text, key_style));
        x += key_width;

        // Label
        let label_start = x;
        let label_width = label.len() as u16;
        let label_style = if is_active {
            Style::default()
                .fg(theme.tab_active_fg)
                .bg(theme.header_bg)
                .add_modifier(Modifier::BOLD)
        } else {
            Style::default()
                .fg(theme.tab_inactive_fg)
                .bg(theme.header_bg)
        };
        spans.push(Span::styled(label.to_string(), label_style));
        x += label_width;

        tab_positions.push((label_start, label_width));

        // Separator between tabs
        if i < TAB_LABELS.len() - 1 {
            let sep = " ";
            spans.push(Span::styled(sep, Style::default().bg(theme.header_bg)));
            x += sep.len() as u16;
        }
    }

    let line = Line::from(spans);
    let paragraph = Paragraph::new(line).style(Style::default().bg(theme.header_bg));
    frame.render_widget(paragraph, area);

    // Draw underline on active tab label
    if let Some(&(label_start, label_width)) = tab_positions.get(state.active_tab) {
        for lx in label_start..label_start + label_width {
            let abs_x = area.x + lx;
            if abs_x < area.x + area.width {
                if let Some(cell) = frame.buffer_mut().cell_mut((abs_x, area.y)) {
                    cell.set_style(
                        Style::default()
                            .fg(theme.tab_active_border)
                            .bg(theme.header_bg)
                            .add_modifier(Modifier::UNDERLINED),
                    );
                }
            }
        }
    }
}

fn render_footer(frame: &mut Frame, state: &AppState, theme: &Theme, area: Rect) {
    let mode_indicator = match state.input_mode {
        ghtui_core::state::InputMode::Normal => Span::styled(
            " NORMAL ",
            Style::default()
                .fg(theme.bg)
                .bg(theme.success)
                .add_modifier(Modifier::BOLD),
        ),
        ghtui_core::state::InputMode::Insert => Span::styled(
            " INSERT ",
            Style::default()
                .fg(theme.bg)
                .bg(theme.warning)
                .add_modifier(Modifier::BOLD),
        ),
    };

    let shortcuts = match &state.route {
        Route::PrList { .. } | Route::IssueList { .. } | Route::ActionsList { .. } => {
            "j/k:Navigate Enter:Open r:Refresh"
        }
        Route::PrDetail { .. } => "Tab:Switch c:Comment m:Merge Esc:Back",
        Route::IssueDetail { .. } => "c:Comment Esc:Back",
        Route::Notifications => "j/k:Navigate Enter:Open",
        _ => "1-6:Tabs t:Theme ?:Help q:Quit",
    };

    let theme_mode = match theme.mode {
        ghtui_core::ThemeMode::Dark => "Dark",
        ghtui_core::ThemeMode::Light => "Light",
    };

    let account_display = state
        .current_account
        .as_ref()
        .map(|a| format!(" @{} ", a.display_name()))
        .unwrap_or_default();

    let line = Line::from(vec![
        mode_indicator,
        Span::raw(" "),
        Span::styled(shortcuts.to_string(), Style::default().fg(theme.fg_dim)),
        Span::raw("  "),
        Span::styled(
            format!("[{}]", theme_mode),
            Style::default().fg(theme.fg_muted),
        ),
        Span::raw("  "),
        Span::styled(account_display, Style::default().fg(theme.accent)),
    ]);

    let paragraph = Paragraph::new(line).style(Style::default().bg(theme.footer_bg));
    frame.render_widget(paragraph, area);
}

fn render_toasts(frame: &mut Frame, state: &AppState, _theme: &Theme, area: Rect) {
    if state.toasts.is_empty() {
        return;
    }

    let toast = state.toasts.front().unwrap();
    let width = (toast.message.len() as u16 + 6).min(area.width);
    let x = area.width.saturating_sub(width).saturating_sub(1);
    let y = area.height.saturating_sub(3);
    let toast_area = Rect::new(x, y, width, 1);

    let widget = ghtui_widgets::ToastWidget::new(toast);
    frame.render_widget(widget, toast_area);
}
