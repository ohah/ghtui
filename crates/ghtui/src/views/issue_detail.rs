use ghtui_core::AppState;
use ghtui_widgets::render_markdown;
use ratatui::Frame;
use ratatui::layout::{Constraint, Direction, Layout, Rect};
use ratatui::style::{Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, Paragraph, Wrap};

pub fn render(frame: &mut Frame, state: &AppState, area: Rect) {
    let theme = &state.theme;

    if state.is_loading("issue_detail") {
        let spinner = ghtui_widgets::Spinner::new(
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_millis() as usize
                / 100,
        );
        let paragraph = Paragraph::new(Line::from(spinner.span())).block(
            Block::default()
                .title(" Issue Detail ")
                .borders(Borders::ALL)
                .border_style(theme.border_style()),
        );
        frame.render_widget(paragraph, area);
        return;
    }

    let Some(ref detail_state) = state.issue_detail else {
        let paragraph = Paragraph::new("No data").style(theme.text_dim()).block(
            Block::default()
                .title(" Issue Detail ")
                .borders(Borders::ALL)
                .border_style(theme.border_style()),
        );
        frame.render_widget(paragraph, area);
        return;
    };

    let issue = &detail_state.detail.issue;

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Length(5), Constraint::Min(0)])
        .split(area);

    let state_color = match issue.state {
        ghtui_core::types::IssueState::Open => theme.success,
        ghtui_core::types::IssueState::Closed => theme.done,
    };

    // Header with metadata
    let mut header_lines = vec![
        Line::from(vec![
            Span::styled(
                format!(" {} ", issue.state),
                Style::default()
                    .fg(theme.bg)
                    .bg(state_color)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::styled(
                format!(" #{}", issue.number),
                Style::default().fg(theme.fg_muted),
            ),
            Span::styled(
                format!("  by @{}", issue.user.login),
                Style::default().fg(theme.fg_dim),
            ),
            Span::styled(
                format!("  {}", issue.created_at.format("%Y-%m-%d")),
                Style::default().fg(theme.fg_dim),
            ),
        ]),
        Line::styled(format!(" {}", issue.title), theme.text_bold()),
    ];

    // Labels
    if !issue.labels.is_empty() {
        let mut label_spans: Vec<Span> = vec![Span::styled(
            " Labels: ",
            Style::default().fg(theme.fg_muted),
        )];
        for label in &issue.labels {
            label_spans.push(Span::styled(
                format!(" {} ", label.name),
                Style::default().fg(theme.accent),
            ));
            label_spans.push(Span::raw(" "));
        }
        header_lines.push(Line::from(label_spans));
    }

    // Assignees + Milestone on same line
    let mut meta_spans: Vec<Span> = Vec::new();
    if !issue.assignees.is_empty() {
        meta_spans.push(Span::styled(
            " Assignees: ",
            Style::default().fg(theme.fg_muted),
        ));
        let names: Vec<String> = issue.assignees.iter().map(|a| a.login.clone()).collect();
        meta_spans.push(Span::styled(names.join(", "), theme.text()));
    }
    if let Some(ref milestone) = issue.milestone {
        if !meta_spans.is_empty() {
            meta_spans.push(Span::raw("  "));
        }
        meta_spans.push(Span::styled(
            " Milestone: ",
            Style::default().fg(theme.fg_muted),
        ));
        meta_spans.push(Span::styled(milestone.title.clone(), theme.text()));
    }
    if !meta_spans.is_empty() {
        header_lines.push(Line::from(meta_spans));
    }

    let header = Paragraph::new(header_lines)
        .style(Style::default().bg(theme.bg))
        .block(
            Block::default()
                .borders(Borders::BOTTOM)
                .border_style(theme.border_style()),
        );
    frame.render_widget(header, chunks[0]);

    // Body + Comments
    let mut lines: Vec<Line<'static>> = Vec::new();
    let selected_comment = detail_state.selected_comment;

    // Issue body (selected when selected_comment is None)
    let body_selected = selected_comment.is_none();
    if body_selected {
        lines.push(Line::styled(
            "▸ Issue Body  (e:Edit)".to_string(),
            Style::default()
                .fg(theme.accent)
                .add_modifier(Modifier::BOLD),
        ));
    }
    if let Some(ref body) = issue.body {
        if !body.is_empty() {
            lines.push(Line::raw(""));
            lines.extend(render_markdown(body));
            lines.push(Line::raw(""));
            lines.push(Line::styled("─".repeat(40), theme.border_style()));
        }
    }

    let comment_count = detail_state.detail.comments.len();
    lines.push(Line::raw(""));
    lines.push(Line::styled(
        format!(
            "  Comments ({})  j/k:Navigate  c:New  r:Reply  e:Edit",
            comment_count
        ),
        Style::default()
            .fg(theme.accent)
            .add_modifier(Modifier::BOLD),
    ));

    for (i, comment) in detail_state.detail.comments.iter().enumerate() {
        let is_selected = selected_comment == Some(i);
        let marker = if is_selected { "▸ " } else { "  " };

        lines.push(Line::raw(""));
        let mut header_spans = vec![
            Span::styled(
                marker.to_string(),
                if is_selected {
                    Style::default().fg(theme.accent)
                } else {
                    Style::default()
                },
            ),
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
        ];
        if is_selected {
            header_spans.push(Span::styled(
                "  (e:Edit  r:Reply)".to_string(),
                Style::default().fg(theme.fg_dim),
            ));
        }
        lines.push(Line::from(header_spans));
        lines.extend(render_markdown(&comment.body));
        lines.push(Line::styled("─".repeat(40), theme.border_style()));
    }

    if comment_count == 0 {
        lines.push(Line::styled(
            "  No comments yet. Press 'c' to add one.".to_string(),
            theme.text_dim(),
        ));
    }

    let paragraph = Paragraph::new(lines)
        .style(theme.text())
        .block(
            Block::default()
                .borders(Borders::ALL)
                .border_style(theme.border_style())
                .title(" Body & Comments "),
        )
        .wrap(Wrap { trim: false })
        .scroll((detail_state.scroll as u16, 0));

    frame.render_widget(paragraph, chunks[1]);
}
