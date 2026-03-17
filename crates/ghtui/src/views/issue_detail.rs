use ghtui_core::AppState;
use ghtui_core::state::issue::{
    AssigneePickerState, InlineEditTarget, IssueSection, LabelPickerState,
};
use ghtui_widgets::render_markdown;
use ratatui::Frame;
use ratatui::layout::{Constraint, Direction, Layout, Rect};
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, Clear, List, ListItem, Paragraph, Wrap};

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

    // === Label picker overlay ===
    if let Some(ref picker) = detail_state.label_picker {
        render_label_picker(frame, picker, theme, area);
        return;
    }
    if let Some(ref picker) = detail_state.assignee_picker {
        render_assignee_picker(frame, picker, theme, area);
        return;
    }
    if let Some(ref picker) = detail_state.milestone_picker {
        render_milestone_picker(frame, picker, theme, area);
        return;
    }

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
        let title_focused = detail_state.focus == IssueSection::Title && !detail_state.is_editing();
        let marker = if title_focused { "▸" } else { " " };
        let mut spans = vec![Span::styled(
            format!("{}{}", marker, issue.title),
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

    // Remove labels/assignees from header (moved to body section)
    // Only show milestone in header
    if let Some(ref milestone) = issue.milestone {
        header_lines.push(Line::from(vec![
            Span::styled(" Milestone: ", Style::default().fg(theme.fg_muted)),
            Span::styled(milestone.title.clone(), theme.text()),
        ]));
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
    let focus = &detail_state.focus;
    let is_editing = detail_state.is_editing();

    let mut lines: Vec<Line<'static>> = Vec::new();

    // Helper for focus indicator
    let focus_marker = |section: &IssueSection| -> &'static str {
        if !is_editing && focus == section {
            "▸ "
        } else {
            "  "
        }
    };
    let focus_style = |section: &IssueSection| -> Style {
        if !is_editing && focus == section {
            Style::default()
                .fg(theme.accent)
                .add_modifier(Modifier::BOLD)
        } else {
            Style::default().fg(theme.fg_muted)
        }
    };

    // --- Labels section ---
    let labels_focused = *focus == IssueSection::Labels;
    if !issue.labels.is_empty() || labels_focused {
        let mut spans = vec![Span::styled(
            format!("{}Labels: ", focus_marker(&IssueSection::Labels)),
            focus_style(&IssueSection::Labels),
        )];
        for label in &issue.labels {
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
    let assignees_focused = *focus == IssueSection::Assignees;
    if !issue.assignees.is_empty() || assignees_focused {
        let mut spans = vec![Span::styled(
            format!("{}Assignees: ", focus_marker(&IssueSection::Assignees)),
            focus_style(&IssueSection::Assignees),
        )];
        for assignee in &issue.assignees {
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

    lines.push(Line::styled("─".repeat(50), theme.border_style()));

    // --- Body section ---
    let body_focused = *focus == IssueSection::Body;
    lines.push(Line::styled(
        format!(
            "{}Body{}",
            focus_marker(&IssueSection::Body),
            if body_focused && !is_editing {
                "  (e:Edit)"
            } else {
                ""
            }
        ),
        focus_style(&IssueSection::Body),
    ));

    if let Some(ref body) = issue.body {
        if !body.is_empty() {
            lines.push(Line::raw(""));
            lines.extend(render_markdown(body));
        }
    }
    // Issue reactions
    if let Some(ref reactions) = issue.reactions {
        let summary = reactions.summary();
        if !summary.is_empty() {
            lines.push(Line::styled(
                format!("  {} (+/- to react)", summary),
                Style::default().fg(theme.fg_dim),
            ));
        }
    }
    lines.push(Line::raw(""));
    lines.push(Line::styled("─".repeat(50), theme.border_style()));

    // --- Timeline events ---
    if !detail_state.detail.timeline.is_empty() {
        lines.push(Line::raw(""));
        for event in &detail_state.detail.timeline {
            // Skip "commented" events (shown as comments already)
            if event.event == "commented" {
                continue;
            }
            let time = event
                .created_at
                .map(|t| t.format("%m/%d %H:%M").to_string())
                .unwrap_or_default();
            lines.push(Line::from(vec![
                Span::styled(
                    format!("  {} ", event.icon()),
                    Style::default().fg(theme.fg_dim),
                ),
                Span::styled(event.display(), Style::default().fg(theme.fg_muted)),
                Span::styled(format!("  {}", time), Style::default().fg(theme.fg_dim)),
            ]));
        }
        lines.push(Line::raw(""));
        lines.push(Line::styled("─".repeat(50), theme.border_style()));
    }

    // --- Comments ---
    let comment_count = detail_state.detail.comments.len();
    lines.push(Line::raw(""));
    lines.push(Line::styled(
        format!(
            "  Comments ({})  c:New  x:Close/Reopen  o:Browser",
            comment_count
        ),
        Style::default()
            .fg(theme.accent)
            .add_modifier(Modifier::BOLD),
    ));

    for (i, comment) in detail_state.detail.comments.iter().enumerate() {
        let section = IssueSection::Comment(i);
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
        // Comment reactions
        if let Some(ref reactions) = comment.reactions {
            let summary = reactions.summary();
            if !summary.is_empty() {
                lines.push(Line::styled(
                    format!("  {}", summary),
                    Style::default().fg(theme.fg_dim),
                ));
            }
        }
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
    _theme: &ghtui_core::theme::Theme,
    area: Rect,
    title: &str,
) {
    let widget = ghtui_widgets::EditorView::new(&detail_state.editor, title)
        .status_hint("Ctrl+Enter: Submit  Esc: Cancel  (markdown)");
    frame.render_widget(widget, area);
}

fn render_bottom_editor(
    frame: &mut Frame,
    detail_state: &ghtui_core::state::IssueDetailState,
    _theme: &ghtui_core::theme::Theme,
    area: Rect,
    title: &str,
) {
    let widget = ghtui_widgets::InlineEditorView::new(&detail_state.editor, title);
    frame.render_widget(widget, area);
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

fn render_milestone_picker(
    frame: &mut Frame,
    picker: &ghtui_core::state::issue::MilestonePickerState,
    theme: &ghtui_core::theme::Theme,
    area: Rect,
) {
    let height = (picker.available.len() as u16 + 5).min(area.height.saturating_sub(4));
    let width = 50.min(area.width.saturating_sub(4));
    let x = (area.width.saturating_sub(width)) / 2 + area.x;
    let y = (area.height.saturating_sub(height)) / 2 + area.y;
    let popup_area = Rect::new(x, y, width, height);

    frame.render_widget(Clear, popup_area);

    let items: Vec<ListItem> = picker
        .available
        .iter()
        .enumerate()
        .map(|(i, ms)| {
            let is_cursor = i == picker.cursor;
            let is_selected = picker.selected == Some(ms.number as u64);
            let check = if is_selected { "(●) " } else { "( ) " };
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
                Span::styled(ms.title.clone(), style),
            ]))
        })
        .collect();

    let title = " Milestone — Space:Select  s:Save  0:Clear  Esc:Cancel ".to_string();

    let list = List::new(items).block(
        Block::default()
            .title(title)
            .borders(Borders::ALL)
            .style(Style::default().bg(Color::Black)),
    );
    frame.render_widget(list, popup_area);
}
