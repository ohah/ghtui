use ghtui_core::AppState;
use ghtui_core::state::issue::InlineEditTarget;
use ghtui_widgets::render_markdown;
use ratatui::Frame;
use ratatui::layout::{Constraint, Direction, Layout, Rect};
use ratatui::style::{Color, Modifier, Style};
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
    let is_editing = detail_state.is_editing();

    // Layout: header + body/comments + (optional) editor at bottom
    let constraints = if is_editing {
        vec![
            Constraint::Length(5),
            Constraint::Min(8),
            Constraint::Length(10),
        ]
    } else {
        vec![
            Constraint::Length(5),
            Constraint::Min(0),
            Constraint::Length(0),
        ]
    };

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints(constraints)
        .split(area);

    // === Header ===
    let state_color = match issue.state {
        ghtui_core::types::IssueState::Open => theme.success,
        ghtui_core::types::IssueState::Closed => theme.done,
    };

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

    // Assignees + Milestone
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

    // === Body + Comments (scrollable) ===
    let mut lines: Vec<Line<'static>> = Vec::new();
    let selected_comment = detail_state.selected_comment;
    let edit_target = &detail_state.edit_target;

    // Issue body
    let body_selected = selected_comment.is_none();
    let editing_body = matches!(edit_target, Some(InlineEditTarget::IssueBody));

    if body_selected && !is_editing {
        lines.push(Line::styled(
            "▸ Issue Body  (e:Edit  c:Comment  r:Reply)".to_string(),
            Style::default()
                .fg(theme.accent)
                .add_modifier(Modifier::BOLD),
        ));
    }

    if editing_body {
        // Show "editing..." indicator instead of body
        lines.push(Line::styled(
            "  ✎ Editing issue below...".to_string(),
            Style::default().fg(theme.warning),
        ));
    } else if let Some(ref body) = issue.body {
        if !body.is_empty() {
            lines.push(Line::raw(""));
            lines.extend(render_markdown(body));
        }
    }
    lines.push(Line::raw(""));
    lines.push(Line::styled("─".repeat(50), theme.border_style()));

    // Comments
    let comment_count = detail_state.detail.comments.len();
    lines.push(Line::raw(""));
    lines.push(Line::styled(
        format!(
            "  Comments ({})  j/k:Navigate  c:Comment  e:Edit  r:Reply",
            comment_count
        ),
        Style::default()
            .fg(theme.accent)
            .add_modifier(Modifier::BOLD),
    ));

    for (i, comment) in detail_state.detail.comments.iter().enumerate() {
        let is_selected = selected_comment == Some(i);
        let editing_this = matches!(edit_target, Some(InlineEditTarget::Comment(idx)) if *idx == i);
        let marker = if is_selected && !is_editing {
            "▸ "
        } else {
            "  "
        };

        lines.push(Line::raw(""));
        let mut hdr = vec![
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
        if is_selected && !is_editing {
            hdr.push(Span::styled(
                "  (e:Edit  r:Reply)".to_string(),
                Style::default().fg(theme.fg_dim),
            ));
        }
        lines.push(Line::from(hdr));

        if editing_this {
            lines.push(Line::styled(
                "  ✎ Editing comment below...".to_string(),
                Style::default().fg(theme.warning),
            ));
        } else {
            lines.extend(render_markdown(&comment.body));
        }
        lines.push(Line::styled("─".repeat(50), theme.border_style()));
    }

    if comment_count == 0 && !is_editing {
        lines.push(Line::styled(
            "  No comments yet. Press 'c' to add one.".to_string(),
            theme.text_dim(),
        ));
    }

    let body_block_title = if is_editing {
        " Body & Comments (editing...) "
    } else {
        " Body & Comments "
    };

    let paragraph = Paragraph::new(lines)
        .style(theme.text())
        .block(
            Block::default()
                .borders(Borders::ALL)
                .border_style(theme.border_style())
                .title(body_block_title),
        )
        .wrap(Wrap { trim: false })
        .scroll((detail_state.scroll as u16, 0));
    frame.render_widget(paragraph, chunks[1]);

    // === Inline Editor (bottom panel) ===
    if is_editing {
        render_inline_editor(frame, detail_state, theme, chunks[2]);
    }
}

fn render_inline_editor(
    frame: &mut Frame,
    detail_state: &ghtui_core::state::IssueDetailState,
    theme: &ghtui_core::theme::Theme,
    area: Rect,
) {
    let edit_target = detail_state.edit_target.as_ref().unwrap();
    let title = match edit_target {
        InlineEditTarget::IssueBody => " Edit Issue (first line=Title, rest=Body) ",
        InlineEditTarget::Comment(_) => " Edit Comment ",
        InlineEditTarget::NewComment => " New Comment ",
        InlineEditTarget::QuoteReply(_) => " Reply ",
    };

    let mut lines: Vec<Line<'static>> = Vec::new();

    // Editor content
    let buffer = &detail_state.edit_buffer;
    for line in buffer.split('\n') {
        lines.push(Line::styled(
            format!("  {}", line),
            Style::default().fg(Color::Rgb(230, 237, 243)),
        ));
    }

    // Cursor
    lines.push(Line::styled(
        "  █".to_string(),
        Style::default()
            .fg(Color::White)
            .add_modifier(Modifier::SLOW_BLINK),
    ));

    // Hint bar
    lines.push(Line::raw(""));
    lines.push(Line::from(vec![
        Span::styled(
            "  Ctrl+Enter",
            Style::default()
                .fg(theme.success)
                .add_modifier(Modifier::BOLD),
        ),
        Span::styled(": Submit  ", Style::default().fg(theme.fg_dim)),
        Span::styled(
            "Esc",
            Style::default()
                .fg(theme.warning)
                .add_modifier(Modifier::BOLD),
        ),
        Span::styled(": Cancel", Style::default().fg(theme.fg_dim)),
    ]));

    let border_color = if detail_state.is_editing() {
        theme.accent
    } else {
        theme.border
    };

    let paragraph = Paragraph::new(lines)
        .block(
            Block::default()
                .title(title)
                .borders(Borders::ALL)
                .border_style(Style::default().fg(border_color))
                .style(Style::default().bg(Color::Rgb(22, 27, 34))), // bg_subtle
        )
        .wrap(Wrap { trim: false });

    frame.render_widget(paragraph, area);
}
