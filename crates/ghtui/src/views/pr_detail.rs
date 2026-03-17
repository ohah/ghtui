use ghtui_core::AppState;
use ghtui_core::state::issue::{AssigneePickerState, LabelPickerState};
use ghtui_core::state::pr::{PrInlineEditTarget, PrSection};
use ghtui_widgets::{TabBar, render_markdown};
use ratatui::Frame;
use ratatui::layout::{Constraint, Direction, Layout, Rect};
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, Clear, List, ListItem, Paragraph, Wrap};

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

    let edit_target = &detail_state.edit_target;

    // Body editing → fullscreen editor
    if matches!(edit_target, Some(PrInlineEditTarget::PrBody)) {
        let widget = ghtui_widgets::EditorView::new(&detail_state.editor, " Edit Body (markdown) ")
            .status_hint("Ctrl+Enter: Submit  Esc: Cancel  (markdown)");
        frame.render_widget(widget, area);
        return;
    }

    let is_comment_editing = matches!(
        edit_target,
        Some(
            PrInlineEditTarget::Comment(_)
                | PrInlineEditTarget::NewComment
                | PrInlineEditTarget::QuoteReply(_)
        )
    );
    let is_title_editing = matches!(edit_target, Some(PrInlineEditTarget::PrTitle));

    let header_height = if is_title_editing { 6 } else { 5 };
    let editor_height: u16 = if is_comment_editing { 10 } else { 0 };

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(header_height),
            Constraint::Length(1),
            Constraint::Min(0),
            Constraint::Length(editor_height),
        ])
        .split(area);

    // === Header ===
    render_header(frame, detail_state, theme, chunks[0], is_title_editing);

    // === Tab bar ===
    let tabs = ["Conversation", "Diff", "Checks"];
    let tab_bar = TabBar::new(&tabs, detail_state.tab);
    frame.render_widget(tab_bar, chunks[1]);

    // === Content ===
    match detail_state.tab {
        0 => render_conversation(frame, state, detail_state, chunks[2]),
        1 => render_diff_tab(frame, state, detail_state, chunks[2]),
        2 => render_checks(frame, state, detail_state, chunks[2]),
        _ => {}
    }

    // === Label picker overlay ===
    if let Some(ref picker) = detail_state.label_picker {
        render_label_picker(frame, picker, theme, area);
        return;
    }
    if let Some(ref picker) = detail_state.assignee_picker {
        render_assignee_picker(frame, picker, theme, area);
        return;
    }

    // === Bottom editor for comments ===
    if is_comment_editing {
        let title = match edit_target {
            Some(PrInlineEditTarget::Comment(_)) => " Edit Comment ",
            Some(PrInlineEditTarget::NewComment) => " New Comment ",
            Some(PrInlineEditTarget::QuoteReply(_)) => " Reply ",
            _ => "",
        };
        let widget = ghtui_widgets::InlineEditorView::new(&detail_state.editor, title);
        frame.render_widget(widget, chunks[3]);
    }
}

fn render_header(
    frame: &mut Frame,
    detail_state: &ghtui_core::state::PrDetailState,
    theme: &ghtui_core::theme::Theme,
    area: Rect,
    is_title_editing: bool,
) {
    let pr = &detail_state.detail.pr;
    let state_color = match pr.state {
        ghtui_core::types::PrState::Open => theme.success,
        ghtui_core::types::PrState::Closed => theme.danger,
        ghtui_core::types::PrState::Merged => theme.done,
    };

    let mut header_lines = vec![Line::from(vec![
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
        Span::styled(
            format!("  by @{}", pr.user.login),
            Style::default().fg(theme.fg_dim),
        ),
        Span::styled(
            format!("  {}", pr.created_at.format("%Y-%m-%d")),
            Style::default().fg(theme.fg_dim),
        ),
        if pr.draft {
            Span::styled("  Draft", Style::default().fg(theme.warning))
        } else {
            Span::raw("")
        },
    ])];

    // Title
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
            " Enter: Save  Esc: Cancel",
            Style::default().fg(theme.fg_dim),
        ));
    } else {
        let title_focused = detail_state.focus == PrSection::Title && !detail_state.is_editing();
        let marker = if title_focused { "▸" } else { " " };
        let mut spans = vec![Span::styled(
            format!("{}{}", marker, pr.title),
            theme.text_bold(),
        )];
        if title_focused {
            spans.push(Span::styled(
                "  (e:Edit)",
                Style::default().fg(theme.fg_dim),
            ));
        }
        header_lines.push(Line::from(spans));
    }

    // Branch info
    header_lines.push(Line::from(vec![
        Span::styled(
            format!(" {} ", pr.head_ref),
            Style::default().fg(theme.accent),
        ),
        Span::styled(" → ", Style::default().fg(theme.fg_dim)),
        Span::styled(
            format!("{} ", pr.base_ref),
            Style::default().fg(theme.accent),
        ),
    ]));

    let header = Paragraph::new(header_lines)
        .style(Style::default().bg(theme.bg))
        .block(
            Block::default()
                .borders(Borders::BOTTOM)
                .border_style(theme.border_style()),
        );
    frame.render_widget(header, area);
}

fn render_conversation(
    frame: &mut Frame,
    state: &AppState,
    detail: &ghtui_core::state::PrDetailState,
    area: Rect,
) {
    let theme = &state.theme;
    let pr = &detail.detail.pr;
    let focus = &detail.focus;
    let is_editing = detail.is_editing();

    let mut lines: Vec<Line<'static>> = Vec::new();

    let focus_marker = |section: &PrSection| -> &'static str {
        if !is_editing && focus == section {
            "▸ "
        } else {
            "  "
        }
    };
    let focus_style = |section: &PrSection| -> Style {
        if !is_editing && focus == section {
            Style::default()
                .fg(theme.accent)
                .add_modifier(Modifier::BOLD)
        } else {
            Style::default().fg(theme.fg_dim)
        }
    };

    // --- Labels section ---
    let labels_focused = *focus == PrSection::Labels;
    if !pr.labels.is_empty() || labels_focused {
        let mut spans = vec![Span::styled(
            format!("{}Labels: ", focus_marker(&PrSection::Labels)),
            focus_style(&PrSection::Labels),
        )];
        for label in &pr.labels {
            spans.push(Span::styled(
                format!(" {} ", label.name),
                Style::default().fg(theme.accent),
            ));
        }
        if labels_focused && !is_editing {
            spans.push(Span::styled(
                "  (l:Edit)",
                Style::default().fg(theme.fg_dim),
            ));
        }
        lines.push(Line::from(spans));
    }

    // --- Assignees section ---
    let assignees_focused = *focus == PrSection::Assignees;
    if !pr.assignees.is_empty() || assignees_focused {
        let mut spans = vec![Span::styled(
            format!("{}Assignees: ", focus_marker(&PrSection::Assignees)),
            focus_style(&PrSection::Assignees),
        )];
        for assignee in &pr.assignees {
            spans.push(Span::styled(format!("@{} ", assignee.login), theme.text()));
        }
        if assignees_focused && !is_editing {
            spans.push(Span::styled(
                "  (a:Edit)",
                Style::default().fg(theme.fg_dim),
            ));
        }
        lines.push(Line::from(spans));
    }

    // --- Reviewers section ---
    if !detail.detail.reviews.is_empty() {
        let mut spans = vec![Span::styled(
            "  Reviewers: ",
            Style::default().fg(theme.fg_dim),
        )];
        for review in &detail.detail.reviews {
            let (icon, color) = match review.state {
                ghtui_core::types::ReviewState::Approved => ("✓", theme.success),
                ghtui_core::types::ReviewState::ChangesRequested => ("✗", theme.danger),
                ghtui_core::types::ReviewState::Commented => ("●", theme.warning),
                _ => ("○", theme.fg_dim),
            };
            spans.push(Span::styled(
                format!("{} @{} ", icon, review.user.login),
                Style::default().fg(color),
            ));
        }
        lines.push(Line::from(spans));
    }

    lines.push(Line::styled("─".repeat(50), theme.border_style()));

    // --- Body section ---
    let body_focused = *focus == PrSection::Body;
    lines.push(Line::styled(
        format!(
            "{}Body{}",
            focus_marker(&PrSection::Body),
            if body_focused && !is_editing {
                "  (e:Edit)"
            } else {
                ""
            }
        ),
        focus_style(&PrSection::Body),
    ));

    if let Some(ref body) = pr.body {
        if !body.is_empty() {
            lines.push(Line::raw(""));
            lines.extend(render_markdown(body));
        }
    }
    lines.push(Line::raw(""));
    lines.push(Line::styled("─".repeat(50), theme.border_style()));

    // --- Comments ---
    let comment_count = detail.detail.comments.len();
    lines.push(Line::raw(""));
    lines.push(Line::styled(
        format!(
            "  Comments ({})  c:New  m:Merge  x:Close/Reopen  o:Browser",
            comment_count
        ),
        Style::default()
            .fg(theme.accent)
            .add_modifier(Modifier::BOLD),
    ));

    for (i, comment) in detail.detail.comments.iter().enumerate() {
        let section = PrSection::Comment(i);
        let is_focused = *focus == section;
        let marker = focus_marker(&section);

        lines.push(Line::raw(""));
        let mut hdr = vec![
            Span::styled(
                marker.to_string(),
                if is_focused {
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
        if is_focused && !is_editing {
            hdr.push(Span::styled(
                format!("  ({})", section.action_hint()),
                Style::default().fg(theme.fg_dim),
            ));
        }
        lines.push(Line::from(hdr));
        lines.extend(render_markdown(&comment.body));
        lines.push(Line::styled("─".repeat(50), theme.border_style()));
    }

    // Reviews with bodies
    for review in &detail.detail.reviews {
        if let Some(ref body) = review.body {
            if !body.is_empty() {
                lines.push(Line::raw(""));
                let state_color = match review.state {
                    ghtui_core::types::ReviewState::Approved => theme.success,
                    ghtui_core::types::ReviewState::ChangesRequested => theme.danger,
                    _ => theme.warning,
                };
                lines.push(Line::from(vec![
                    Span::styled(
                        format!("  @{}", review.user.login),
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
                lines.extend(render_markdown(body));
                lines.push(Line::styled("─".repeat(50), theme.border_style()));
            }
        }
    }

    if comment_count == 0 && detail.detail.reviews.is_empty() && !is_editing {
        lines.push(Line::styled(
            "  No comments yet. Press 'c' to add one.",
            theme.text_dim(),
        ));
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
            scroll: detail.diff_scroll,
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

    let (passed, failed, pending) = {
        let mut p = 0usize;
        let mut f = 0usize;
        let mut w = 0usize;
        for c in &detail.detail.checks {
            match c.conclusion.as_deref() {
                Some("success") => p += 1,
                Some("failure") | Some("cancelled") | Some("timed_out") => f += 1,
                _ => w += 1,
            }
        }
        (p, f, w)
    };

    let mut lines: Vec<Line> = Vec::new();

    if detail.detail.checks.is_empty() {
        lines.push(Line::styled("  No checks found", theme.text_dim()));
    } else {
        // Summary line
        let mut summary_spans = vec![Span::styled("  ", Style::default())];
        if passed > 0 {
            summary_spans.push(Span::styled(
                format!("✓ {} passed", passed),
                Style::default().fg(theme.success),
            ));
            summary_spans.push(Span::styled("  ", Style::default()));
        }
        if failed > 0 {
            summary_spans.push(Span::styled(
                format!("✗ {} failed", failed),
                Style::default().fg(theme.danger),
            ));
            summary_spans.push(Span::styled("  ", Style::default()));
        }
        if pending > 0 {
            summary_spans.push(Span::styled(
                format!("● {} pending", pending),
                Style::default().fg(theme.warning),
            ));
        }
        lines.push(Line::from(summary_spans));
        lines.push(Line::styled(
            "  ─".to_string() + &"─".repeat(40),
            theme.border_style(),
        ));

        for check in &detail.detail.checks {
            let icon = match check.conclusion.as_deref() {
                Some("success") => Span::styled("  ✓ ", Style::default().fg(theme.success)),
                Some("failure") => Span::styled("  ✗ ", Style::default().fg(theme.danger)),
                Some("cancelled") => Span::styled("  ⊘ ", Style::default().fg(theme.fg_dim)),
                _ => {
                    if check.status == "in_progress" {
                        Span::styled("  ◎ ", Style::default().fg(theme.warning))
                    } else {
                        Span::styled("  ● ", Style::default().fg(theme.warning))
                    }
                }
            };
            let status_text = match check.conclusion.as_deref() {
                Some(c) => c.to_string(),
                None => check.status.clone(),
            };
            lines.push(Line::from(vec![
                icon,
                Span::styled(&check.name, theme.text()),
                Span::styled(
                    format!("  ({})", status_text),
                    Style::default().fg(theme.fg_dim),
                ),
            ]));
        }
    }

    let paragraph = Paragraph::new(lines).block(
        Block::default()
            .borders(Borders::ALL)
            .border_style(theme.border_style())
            .title(" Checks "),
    );
    frame.render_widget(paragraph, area);
}

fn render_label_picker(
    frame: &mut Frame,
    picker: &LabelPickerState,
    theme: &ghtui_core::theme::Theme,
    area: Rect,
) {
    let height = (picker.available.len() as u16 + 4).min(area.height.saturating_sub(4));
    let width = 50.min(area.width.saturating_sub(4));
    let x = (area.width.saturating_sub(width)) / 2 + area.x;
    let y = (area.height.saturating_sub(height)) / 2 + area.y;
    let popup_area = Rect::new(x, y, width, height);

    frame.render_widget(Clear, popup_area);

    let items: Vec<ListItem> = picker
        .available
        .iter()
        .enumerate()
        .map(|(i, label)| {
            let is_cursor = i == picker.cursor;
            let is_selected = picker.selected_names.contains(&label.name);
            let check = if is_selected { "[x] " } else { "[ ] " };
            let cursor = if is_cursor { "▸ " } else { "  " };

            let style = if is_cursor {
                Style::default()
                    .fg(Color::White)
                    .add_modifier(Modifier::BOLD)
            } else if is_selected {
                Style::default().fg(theme.success)
            } else {
                Style::default().fg(Color::Gray)
            };

            ListItem::new(Line::from(vec![
                Span::styled(cursor.to_string(), style),
                Span::styled(
                    check.to_string(),
                    if is_selected {
                        Style::default().fg(theme.success)
                    } else {
                        Style::default().fg(Color::DarkGray)
                    },
                ),
                Span::styled(label.name.clone(), style),
            ]))
        })
        .collect();

    let title = format!(
        " Labels ({}/{}) — Space:Toggle  s:Save  Esc:Cancel ",
        picker.selected_names.len(),
        picker.available.len()
    );

    let list = List::new(items).block(
        Block::default()
            .title(title)
            .borders(Borders::ALL)
            .style(Style::default().bg(Color::Black)),
    );
    frame.render_widget(list, popup_area);
}

fn render_assignee_picker(
    frame: &mut Frame,
    picker: &AssigneePickerState,
    theme: &ghtui_core::theme::Theme,
    area: Rect,
) {
    let height = (picker.available.len() as u16 + 4).min(area.height.saturating_sub(4));
    let width = 45.min(area.width.saturating_sub(4));
    let x = (area.width.saturating_sub(width)) / 2 + area.x;
    let y = (area.height.saturating_sub(height)) / 2 + area.y;
    let popup_area = Rect::new(x, y, width, height);

    frame.render_widget(Clear, popup_area);

    let items: Vec<ListItem> = picker
        .available
        .iter()
        .enumerate()
        .map(|(i, login)| {
            let is_cursor = i == picker.cursor;
            let is_selected = picker.selected_names.contains(login);
            let check = if is_selected { "[x] " } else { "[ ] " };
            let cursor = if is_cursor { "▸ " } else { "  " };

            let style = if is_cursor {
                Style::default()
                    .fg(Color::White)
                    .add_modifier(Modifier::BOLD)
            } else if is_selected {
                Style::default().fg(theme.success)
            } else {
                Style::default().fg(Color::Gray)
            };

            ListItem::new(Line::from(vec![
                Span::styled(cursor.to_string(), style),
                Span::styled(
                    check.to_string(),
                    if is_selected {
                        Style::default().fg(theme.success)
                    } else {
                        Style::default().fg(Color::DarkGray)
                    },
                ),
                Span::styled(format!("@{}", login), style),
            ]))
        })
        .collect();

    let title = format!(
        " Assignees ({}/{}) — Space:Toggle  s:Save  Esc:Cancel ",
        picker.selected_names.len(),
        picker.available.len()
    );

    let list = List::new(items).block(
        Block::default()
            .title(title)
            .borders(Borders::ALL)
            .style(Style::default().bg(Color::Black)),
    );
    frame.render_widget(list, popup_area);
}
