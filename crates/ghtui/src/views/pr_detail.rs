use ghtui_core::AppState;
use ghtui_widgets::{TabBar, render_markdown};
use ratatui::Frame;
use ratatui::layout::{Constraint, Direction, Layout, Rect};
use ratatui::style::{Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, Paragraph, Wrap};

pub fn render(frame: &mut Frame, state: &AppState, area: Rect) {
    let theme = &state.theme;

    if state.is_loading("pr_detail") {
        let spinner = ghtui_widgets::Spinner::new(
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_millis() as usize
                / 100,
        );
        let paragraph = Paragraph::new(Line::from(spinner.span())).block(
            Block::default()
                .title(" PR Detail ")
                .borders(Borders::ALL)
                .border_style(theme.border_style()),
        );
        frame.render_widget(paragraph, area);
        return;
    }

    let Some(ref detail_state) = state.pr_detail else {
        let paragraph = Paragraph::new("No data").style(theme.text_dim()).block(
            Block::default()
                .title(" PR Detail ")
                .borders(Borders::ALL)
                .border_style(theme.border_style()),
        );
        frame.render_widget(paragraph, area);
        return;
    };

    let pr = &detail_state.detail.pr;

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(4),
            Constraint::Length(1),
            Constraint::Min(0),
        ])
        .split(area);

    // Title bar
    let state_color = match pr.state {
        ghtui_core::types::PrState::Open => theme.success,
        ghtui_core::types::PrState::Closed => theme.danger,
        ghtui_core::types::PrState::Merged => theme.done,
    };

    let title_lines = vec![
        Line::from(vec![
            Span::styled(
                format!(" {} ", pr.state),
                Style::default()
                    .fg(theme.bg)
                    .bg(state_color)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::styled(
                format!(" #{}", pr.number),
                Style::default().fg(theme.fg_muted),
            ),
        ]),
        Line::styled(format!(" {}", pr.title), theme.text_bold()),
        Line::from(vec![
            Span::styled(
                format!(" @{}", pr.user.login),
                Style::default().fg(theme.accent),
            ),
            Span::styled(
                format!(" wants to merge {} into {}", pr.head_ref, pr.base_ref),
                Style::default().fg(theme.fg_dim),
            ),
        ]),
    ];

    let title = Paragraph::new(title_lines)
        .style(Style::default().bg(theme.bg))
        .block(
            Block::default()
                .borders(Borders::BOTTOM)
                .border_style(theme.border_style()),
        );
    frame.render_widget(title, chunks[0]);

    // Tab bar
    let tabs = ["Conversation", "Diff", "Checks"];
    let tab_bar = TabBar::new(&tabs, detail_state.tab);
    frame.render_widget(tab_bar, chunks[1]);

    // Content
    match detail_state.tab {
        0 => render_conversation(frame, state, detail_state, chunks[2]),
        1 => render_diff_tab(frame, state, detail_state, chunks[2]),
        2 => render_checks(frame, state, detail_state, chunks[2]),
        _ => {}
    }
}

fn render_conversation(
    frame: &mut Frame,
    state: &AppState,
    detail: &ghtui_core::state::PrDetailState,
    area: Rect,
) {
    let theme = &state.theme;
    let mut lines = Vec::new();

    if let Some(ref body) = detail.detail.pr.body {
        lines.extend(render_markdown(body));
        lines.push(Line::raw(""));
        lines.push(Line::styled("─".repeat(40), theme.border_style()));
    }

    for comment in &detail.detail.comments {
        lines.push(Line::raw(""));
        lines.push(Line::from(vec![
            Span::styled(
                format!("@{}", comment.user.login),
                Style::default()
                    .fg(theme.accent)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::styled(
                format!(" · {}", comment.created_at.format("%Y-%m-%d %H:%M")),
                Style::default().fg(theme.fg_muted),
            ),
        ]));
        lines.extend(render_markdown(&comment.body));
        lines.push(Line::styled("─".repeat(40), theme.border_style()));
    }

    for review in &detail.detail.reviews {
        lines.push(Line::raw(""));
        let state_color = match review.state {
            ghtui_core::types::ReviewState::Approved => theme.success,
            ghtui_core::types::ReviewState::ChangesRequested => theme.danger,
            _ => theme.warning,
        };
        lines.push(Line::from(vec![
            Span::styled(
                format!("@{}", review.user.login),
                Style::default()
                    .fg(theme.accent)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::styled(
                format!(" {}", review.state),
                Style::default()
                    .fg(state_color)
                    .add_modifier(Modifier::BOLD),
            ),
        ]));
        if let Some(ref body) = review.body {
            if !body.is_empty() {
                lines.extend(render_markdown(body));
            }
        }
    }

    if lines.is_empty() {
        lines.push(Line::styled("  No comments yet", theme.text_dim()));
    }

    let paragraph = Paragraph::new(lines)
        .style(theme.text())
        .block(
            Block::default()
                .borders(Borders::ALL)
                .border_style(theme.border_style())
                .title(" Conversation "),
        )
        .wrap(Wrap { trim: false })
        .scroll((detail.scroll as u16, 0));

    frame.render_widget(paragraph, area);
}

fn render_diff_tab(
    frame: &mut Frame,
    state: &AppState,
    detail: &ghtui_core::state::PrDetailState,
    area: Rect,
) {
    let theme = &state.theme;

    if let Some(ref files) = detail.diff {
        let mut diff_state = ghtui_widgets::DiffViewState {
            show_all_files: true,
            ..Default::default()
        };
        let diff_view = ghtui_widgets::DiffView::new(files).block(
            Block::default()
                .borders(Borders::ALL)
                .border_style(theme.border_style())
                .title(" Diff "),
        );
        frame.render_stateful_widget(diff_view, area, &mut diff_state);
    } else {
        let spinner = ghtui_widgets::Spinner::new(0);
        let paragraph = Paragraph::new(Line::from(spinner.span_with_message("Loading diff...")))
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .border_style(theme.border_style())
                    .title(" Diff "),
            );
        frame.render_widget(paragraph, area);
    }
}

fn render_checks(
    frame: &mut Frame,
    state: &AppState,
    detail: &ghtui_core::state::PrDetailState,
    area: Rect,
) {
    let theme = &state.theme;

    let lines: Vec<Line> = if detail.detail.checks.is_empty() {
        vec![Line::styled("  No checks found", theme.text_dim())]
    } else {
        detail
            .detail
            .checks
            .iter()
            .map(|check| {
                let icon = match check.conclusion.as_deref() {
                    Some("success") => Span::styled("  ✓ ", Style::default().fg(theme.success)),
                    Some("failure") => Span::styled("  ✗ ", Style::default().fg(theme.danger)),
                    _ => Span::styled("  ● ", Style::default().fg(theme.warning)),
                };
                Line::from(vec![icon, Span::styled(&check.name, theme.text())])
            })
            .collect()
    };

    let paragraph = Paragraph::new(lines).block(
        Block::default()
            .borders(Borders::ALL)
            .border_style(theme.border_style())
            .title(" Checks "),
    );
    frame.render_widget(paragraph, area);
}
