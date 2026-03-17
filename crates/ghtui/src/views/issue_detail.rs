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

    let edit_target = &detail_state.edit_target;

    // Body editing → fullscreen editor
    if matches!(edit_target, Some(InlineEditTarget::IssueBody)) {
        render_fullscreen_editor(frame, detail_state, theme, area, " Edit Body (markdown) ");
        return;
    }

    // Determine layout based on editing state
    let is_comment_editing = matches!(
        edit_target,
        Some(
            InlineEditTarget::Comment(_)
                | InlineEditTarget::NewComment
                | InlineEditTarget::QuoteReply(_)
        )
    );
    let is_title_editing = matches!(edit_target, Some(InlineEditTarget::IssueTitle));

    let header_height = if is_title_editing { 6 } else { 5 };
    let editor_height: u16 = if is_comment_editing { 10 } else { 0 };

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(header_height),
            Constraint::Min(0),
            Constraint::Length(editor_height),
        ])
        .split(area);

    // === Header ===
    render_header(frame, detail_state, theme, chunks[0], is_title_editing);

    // === Body + Comments ===
    render_body_comments(frame, state, theme, chunks[1]);

    // === Bottom editor for comments ===
    if is_comment_editing {
        let title = match edit_target {
            Some(InlineEditTarget::Comment(_)) => " Edit Comment ",
            Some(InlineEditTarget::NewComment) => " New Comment ",
            Some(InlineEditTarget::QuoteReply(_)) => " Reply ",
            _ => "",
        };
        render_bottom_editor(frame, detail_state, theme, chunks[2], title);
    }
}

fn render_header(
    frame: &mut Frame,
    detail_state: &ghtui_core::state::IssueDetailState,
    theme: &ghtui_core::theme::Theme,
    area: Rect,
    is_title_editing: bool,
) {
    let issue = &detail_state.detail.issue;
    let state_color = match issue.state {
        ghtui_core::types::IssueState::Open => theme.success,
        ghtui_core::types::IssueState::Closed => theme.done,
    };

    let mut header_lines = vec![Line::from(vec![
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
    ])];

    // Title: inline editable or normal
    if is_title_editing {
        let editor = &detail_state.editor;
        let line = editor.lines.first().map(|s| s.as_str()).unwrap_or("");
        let col = editor.cursor_byte_col();
        let before = &line[..col];
        let after = &line[col..];

        header_lines.push(Line::from(vec![
            Span::styled(" ✎ ", Style::default().fg(theme.warning)),
            Span::styled(
                before.to_string(),
                Style::default()
                    .fg(Color::White)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::styled(
                "█",
                Style::default()
                    .fg(Color::Rgb(88, 166, 255))
                    .add_modifier(Modifier::SLOW_BLINK),
            ),
            Span::styled(
                after.to_string(),
                Style::default()
                    .fg(Color::White)
                    .add_modifier(Modifier::BOLD),
            ),
        ]));
        header_lines.push(Line::styled(
            " Enter: Save  Esc: Cancel".to_string(),
            Style::default().fg(theme.fg_dim),
        ));
    } else {
        header_lines.push(Line::from(vec![
            Span::styled(format!(" {}", issue.title), theme.text_bold()),
            Span::styled("  (T:Edit title)", Style::default().fg(theme.fg_dim)),
        ]));
    }

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
    frame.render_widget(header, area);
}

fn render_body_comments(
    frame: &mut Frame,
    state: &AppState,
    theme: &ghtui_core::theme::Theme,
    area: Rect,
) {
    let detail_state = state.issue_detail.as_ref().unwrap();
    let issue = &detail_state.detail.issue;
    let selected_comment = detail_state.selected_comment;
    let is_editing = detail_state.is_editing();

    let mut lines: Vec<Line<'static>> = Vec::new();

    // Issue body
    let body_selected = selected_comment.is_none();
    if body_selected && !is_editing {
        lines.push(Line::styled(
            "▸ Issue Body  (e:Edit body  c:Comment)".to_string(),
            Style::default()
                .fg(theme.accent)
                .add_modifier(Modifier::BOLD),
        ));
    }

    if let Some(ref body) = issue.body {
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
            "  Comments ({})  j/k:Select  c:New  e:Edit  r:Reply",
            comment_count
        ),
        Style::default()
            .fg(theme.accent)
            .add_modifier(Modifier::BOLD),
    ));

    for (i, comment) in detail_state.detail.comments.iter().enumerate() {
        let is_selected = selected_comment == Some(i);
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
        lines.extend(render_markdown(&comment.body));
        lines.push(Line::styled("─".repeat(50), theme.border_style()));
    }

    if comment_count == 0 && !is_editing {
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
    frame.render_widget(paragraph, area);
}

fn render_fullscreen_editor(
    frame: &mut Frame,
    detail_state: &ghtui_core::state::IssueDetailState,
    theme: &ghtui_core::theme::Theme,
    area: Rect,
    title: &str,
) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Min(0), Constraint::Length(1)])
        .split(area);

    let editor = &detail_state.editor;
    let total_lines = editor.line_count();

    let mut lines: Vec<Line<'static>> = Vec::new();
    for (i, line) in editor.lines.iter().enumerate() {
        let is_cursor_line = i == editor.cursor_row;
        let line_num_style = if is_cursor_line {
            Style::default().fg(Color::Rgb(230, 237, 243))
        } else {
            Style::default().fg(Color::Rgb(110, 118, 129))
        };

        if is_cursor_line {
            // Split line at cursor byte position
            let col = editor.cursor_byte_col();
            let before = &line[..col];
            let after = &line[col..];
            lines.push(Line::from(vec![
                Span::styled(format!(" {:>3} ", i + 1), line_num_style),
                Span::styled("│ ", Style::default().fg(Color::Rgb(48, 54, 61))),
                Span::styled(
                    before.to_string(),
                    Style::default().fg(Color::Rgb(230, 237, 243)),
                ),
                Span::styled(
                    "█",
                    Style::default()
                        .fg(Color::Rgb(88, 166, 255))
                        .add_modifier(Modifier::SLOW_BLINK),
                ),
                Span::styled(
                    after.to_string(),
                    Style::default().fg(Color::Rgb(230, 237, 243)),
                ),
            ]));
        } else {
            lines.push(Line::from(vec![
                Span::styled(format!(" {:>3} ", i + 1), line_num_style),
                Span::styled("│ ", Style::default().fg(Color::Rgb(48, 54, 61))),
                Span::styled(line.clone(), Style::default().fg(Color::Rgb(230, 237, 243))),
            ]));
        }
    }

    let editor = Paragraph::new(lines)
        .block(
            Block::default()
                .title(title)
                .borders(Borders::ALL)
                .border_style(Style::default().fg(theme.accent))
                .style(Style::default().bg(Color::Rgb(22, 27, 34))),
        )
        .wrap(Wrap { trim: false });
    frame.render_widget(editor, chunks[0]);

    // Status bar
    let status = Line::from(vec![
        Span::styled(
            " Ctrl+Enter ",
            Style::default()
                .fg(Color::Rgb(13, 17, 23))
                .bg(theme.success)
                .add_modifier(Modifier::BOLD),
        ),
        Span::styled(" Submit  ", Style::default().fg(theme.fg_dim)),
        Span::styled(
            " Esc ",
            Style::default()
                .fg(Color::Rgb(13, 17, 23))
                .bg(theme.warning)
                .add_modifier(Modifier::BOLD),
        ),
        Span::styled(" Cancel  ", Style::default().fg(theme.fg_dim)),
        Span::styled(
            format!(" {} lines  (markdown) ", total_lines),
            Style::default().fg(theme.fg_muted),
        ),
    ]);
    let status_bar = Paragraph::new(status).style(Style::default().bg(theme.footer_bg));
    frame.render_widget(status_bar, chunks[1]);
}

fn render_bottom_editor(
    frame: &mut Frame,
    detail_state: &ghtui_core::state::IssueDetailState,
    theme: &ghtui_core::theme::Theme,
    area: Rect,
    title: &str,
) {
    let mut lines: Vec<Line<'static>> = Vec::new();
    let editor = &detail_state.editor;

    for (i, line) in editor.lines.iter().enumerate() {
        let is_cursor_line = i == editor.cursor_row;
        if is_cursor_line {
            let col = editor.cursor_byte_col();
            let before = &line[..col];
            let after = &line[col..];
            lines.push(Line::from(vec![
                Span::styled(
                    format!("  {}", before),
                    Style::default().fg(Color::Rgb(230, 237, 243)),
                ),
                Span::styled(
                    "█",
                    Style::default()
                        .fg(Color::Rgb(88, 166, 255))
                        .add_modifier(Modifier::SLOW_BLINK),
                ),
                Span::styled(
                    after.to_string(),
                    Style::default().fg(Color::Rgb(230, 237, 243)),
                ),
            ]));
        } else {
            lines.push(Line::styled(
                format!("  {}", line),
                Style::default().fg(Color::Rgb(230, 237, 243)),
            ));
        }
    }

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

    let paragraph = Paragraph::new(lines)
        .block(
            Block::default()
                .title(title)
                .borders(Borders::ALL)
                .border_style(Style::default().fg(theme.accent))
                .style(Style::default().bg(Color::Rgb(22, 27, 34))),
        )
        .wrap(Wrap { trim: false });
    frame.render_widget(paragraph, area);
}
