use ghtui_core::AppState;
use ratatui::Frame;
use ratatui::layout::{Constraint, Direction, Layout, Rect};
use ratatui::style::{Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, Clear, List, ListItem, ListState, Paragraph, Wrap};

pub fn render(frame: &mut Frame, state: &AppState, area: Rect) {
    let theme = &state.theme;

    if state.is_loading("actions_list") {
        let spinner = ghtui_widgets::Spinner::new(
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_millis() as usize
                / 100,
        );
        let paragraph = Paragraph::new(Line::from(spinner.span()))
            .style(theme.text())
            .block(
                Block::default()
                    .title(" Actions ")
                    .borders(Borders::ALL)
                    .border_style(theme.border_style()),
            );
        frame.render_widget(paragraph, area);
        return;
    }

    let Some(ref list_state) = state.actions_list else {
        let paragraph = Paragraph::new("No data").style(theme.text_dim()).block(
            Block::default()
                .title(" Actions ")
                .borders(Borders::ALL)
                .border_style(theme.border_style()),
        );
        frame.render_widget(paragraph, area);
        return;
    };

    // Workflow sidebar layout
    let content_area = if list_state.show_workflow_sidebar {
        let sidebar_width = 30u16.min(area.width / 3);
        let h_chunks = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([Constraint::Length(sidebar_width), Constraint::Min(0)])
            .split(area);
        render_workflow_sidebar(frame, state, list_state, h_chunks[0]);
        h_chunks[1]
    } else {
        area
    };

    // Layout: filter bar + list
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Length(1), Constraint::Min(0)])
        .split(content_area);

    // Filter bar
    render_filter_bar(frame, state, list_state, chunks[0]);

    // Dispatch modal overlay
    if list_state.dispatch.is_some() {
        render_dispatch_modal(frame, state, list_state, area);
        return;
    }

    // List
    let items: Vec<ListItem> = list_state
        .items
        .iter()
        .enumerate()
        .map(|(i, run)| {
            let is_selected = i == list_state.selected;

            let status_icon = match (&run.status, &run.conclusion) {
                (_, Some(ghtui_core::types::RunConclusion::Success)) => {
                    Span::styled(" ✓ ", Style::default().fg(theme.success))
                }
                (_, Some(ghtui_core::types::RunConclusion::Failure)) => {
                    Span::styled(" ✗ ", Style::default().fg(theme.danger))
                }
                (_, Some(ghtui_core::types::RunConclusion::Cancelled)) => {
                    Span::styled(" ◌ ", Style::default().fg(theme.fg_muted))
                }
                (_, Some(ghtui_core::types::RunConclusion::Skipped)) => {
                    Span::styled(" ◌ ", Style::default().fg(theme.fg_muted))
                }
                (Some(ghtui_core::types::RunStatus::InProgress), _) => {
                    Span::styled(" ● ", Style::default().fg(theme.warning))
                }
                (Some(ghtui_core::types::RunStatus::Queued), _) => {
                    Span::styled(" ○ ", Style::default().fg(theme.warning))
                }
                (Some(ghtui_core::types::RunStatus::Waiting), _) => {
                    Span::styled(" ◎ ", Style::default().fg(theme.warning))
                }
                _ => Span::styled(" · ", Style::default().fg(theme.fg_muted)),
            };

            let name = run.name.as_deref().unwrap_or("Unknown");
            let branch = run.head_branch.as_deref().unwrap_or("");

            let title_style = if is_selected {
                theme.selected()
            } else {
                theme.text()
            };

            // Relative time
            let elapsed = chrono::Utc::now() - run.created_at;
            let time_str = if elapsed.num_days() > 0 {
                format!("{}d ago", elapsed.num_days())
            } else if elapsed.num_hours() > 0 {
                format!("{}h ago", elapsed.num_hours())
            } else {
                format!("{}m ago", elapsed.num_minutes())
            };

            let line = Line::from(vec![
                status_icon,
                Span::styled(name, title_style),
                Span::styled(
                    format!(" #{}", run.run_number),
                    Style::default().fg(theme.fg_muted),
                ),
                Span::raw(" "),
                Span::styled(format!("({})", branch), Style::default().fg(theme.accent)),
                Span::styled(format!(" {}", run.event), Style::default().fg(theme.fg_dim)),
                Span::raw(" "),
                Span::styled(time_str, Style::default().fg(theme.fg_dim)),
            ]);

            ListItem::new(line)
        })
        .collect();

    let total_str = list_state
        .pagination
        .total
        .map(|t| format!("/{}", t))
        .unwrap_or_default();
    let page_str = format!(" p.{}{} ", list_state.pagination.page, total_str);

    let title = format!(
        " Actions ({}) {} w:Workflows d:Dispatch ",
        list_state.items.len(),
        page_str
    );

    let list = List::new(items)
        .block(
            Block::default()
                .title(Span::styled(title, theme.text_bold()))
                .borders(Borders::ALL)
                .border_style(theme.border_style()),
        )
        .highlight_style(theme.selected());

    let mut list_widget_state = ListState::default();
    list_widget_state.select(Some(list_state.selected));
    frame.render_stateful_widget(list, chunks[1], &mut list_widget_state);
}

fn render_filter_bar(
    frame: &mut Frame,
    _state: &AppState,
    list_state: &ghtui_core::state::ActionsListState,
    area: Rect,
) {
    let theme = &_state.theme;

    if list_state.search_mode {
        let search_line = Line::from(vec![
            Span::styled(" /", Style::default().fg(theme.accent)),
            Span::styled(&list_state.search_query, theme.text()),
            Span::styled("█", Style::default().fg(theme.accent)),
        ]);
        let paragraph = Paragraph::new(search_line).style(Style::default().bg(theme.bg));
        frame.render_widget(paragraph, area);
        return;
    }

    let mut spans = vec![Span::styled(" ", Style::default())];

    // Status filter
    let status_style = if list_state.filters.status.is_some() {
        Style::default().fg(theme.accent)
    } else {
        Style::default().fg(theme.fg_dim)
    };
    spans.push(Span::styled(
        format!("[s]:{} ", list_state.filters.status_display()),
        status_style,
    ));

    // Event filter
    let event_style = if list_state.filters.event.is_some() {
        Style::default().fg(theme.accent)
    } else {
        Style::default().fg(theme.fg_dim)
    };
    spans.push(Span::styled(
        format!("[e]:{} ", list_state.filters.event_display()),
        event_style,
    ));

    // Branch filter
    if let Some(ref branch) = list_state.filters.branch {
        spans.push(Span::styled(
            format!("branch:{} ", branch),
            Style::default().fg(theme.accent),
        ));
    }

    // Workflow filter
    if let Some(wf_id) = list_state.filters.workflow_id {
        let wf_name = list_state
            .workflows
            .iter()
            .find(|w| w.id == wf_id)
            .map(|w| w.name.as_str())
            .unwrap_or("?");
        spans.push(Span::styled(
            format!("workflow:{} ", wf_name),
            Style::default().fg(theme.accent),
        ));
    }

    // Active filter indicator
    if list_state.filters.has_active_filters() {
        spans.push(Span::styled(
            "[F]:Clear ",
            Style::default().fg(theme.fg_muted),
        ));
    }

    let filter_line = Line::from(spans);
    let paragraph = Paragraph::new(filter_line).style(Style::default().bg(theme.bg));
    frame.render_widget(paragraph, area);
}

fn render_workflow_sidebar(
    frame: &mut Frame,
    state: &AppState,
    list_state: &ghtui_core::state::ActionsListState,
    area: Rect,
) {
    let theme = &state.theme;
    let focused = list_state.workflow_sidebar_focused;
    let border_color = if focused { theme.accent } else { theme.border };

    let mut items: Vec<ListItem> = Vec::new();

    // "All workflows" option
    let all_style = if list_state.workflow_sidebar_selected == 0 && focused {
        theme.selected()
    } else if list_state.workflow_sidebar_selected == 0 {
        Style::default().fg(theme.fg).add_modifier(Modifier::BOLD)
    } else {
        Style::default().fg(theme.fg_dim)
    };
    let all_marker = if list_state.filters.workflow_id.is_none() {
        "● "
    } else {
        "  "
    };
    items.push(ListItem::new(Line::from(vec![
        Span::styled(all_marker, Style::default().fg(theme.accent)),
        Span::styled("All workflows", all_style),
    ])));

    for (i, wf) in list_state.workflows.iter().enumerate() {
        let idx = i + 1;
        let is_selected = idx == list_state.workflow_sidebar_selected;
        let is_active = list_state.filters.workflow_id == Some(wf.id);

        let name_style = if is_selected && focused {
            theme.selected()
        } else if is_selected {
            Style::default().fg(theme.fg).add_modifier(Modifier::BOLD)
        } else {
            Style::default().fg(theme.fg_dim)
        };

        let marker = if is_active { "● " } else { "  " };

        items.push(ListItem::new(Line::from(vec![
            Span::styled(marker, Style::default().fg(theme.accent)),
            Span::styled(&wf.name, name_style),
        ])));
    }

    let title = if focused {
        " Workflows (Enter:select  d:dispatch) "
    } else {
        " Workflows (w:toggle) "
    };

    let list = List::new(items).block(
        Block::default()
            .borders(Borders::ALL)
            .border_style(Style::default().fg(border_color))
            .title(title),
    );

    let mut sidebar_state = ListState::default();
    sidebar_state.select(Some(list_state.workflow_sidebar_selected));
    frame.render_stateful_widget(list, area, &mut sidebar_state);
}

fn render_dispatch_modal(
    frame: &mut Frame,
    state: &AppState,
    list_state: &ghtui_core::state::ActionsListState,
    area: Rect,
) {
    let theme = &state.theme;
    let Some(ref dispatch) = list_state.dispatch else {
        return;
    };

    let height = (dispatch.inputs.len() as u16 * 2 + 8).min(area.height.saturating_sub(4));
    let width = 60u16.min(area.width.saturating_sub(4));
    let x = (area.width.saturating_sub(width)) / 2 + area.x;
    let y = (area.height.saturating_sub(height)) / 2 + area.y;
    let popup_area = Rect::new(x, y, width, height);

    frame.render_widget(Clear, popup_area);

    let mut lines: Vec<Line> = Vec::new();

    // Header
    lines.push(Line::styled(
        format!("  Dispatch: {}", dispatch.workflow_name),
        Style::default()
            .fg(theme.accent)
            .add_modifier(Modifier::BOLD),
    ));
    lines.push(Line::raw(""));

    // Ref field
    let ref_focused = dispatch.focused_field == 0;
    let ref_marker = if ref_focused { "▸ " } else { "  " };
    let ref_value = if dispatch.editing && ref_focused {
        format!("{}█", dispatch.edit_buffer)
    } else {
        dispatch.git_ref.clone()
    };
    lines.push(Line::from(vec![
        Span::styled(
            format!("{}Branch/Tag: ", ref_marker),
            if ref_focused {
                Style::default()
                    .fg(theme.accent)
                    .add_modifier(Modifier::BOLD)
            } else {
                Style::default().fg(theme.fg_dim)
            },
        ),
        Span::styled(ref_value, theme.text()),
    ]));

    // Input fields
    for (i, field) in dispatch.inputs.iter().enumerate() {
        let field_idx = i + 1;
        let is_focused = dispatch.focused_field == field_idx;
        let marker = if is_focused { "▸ " } else { "  " };

        let value = if dispatch.editing && is_focused {
            format!("{}█", dispatch.edit_buffer)
        } else if field.value.is_empty() {
            "(empty)".to_string()
        } else {
            field.value.clone()
        };

        let required_tag = if field.required { " *" } else { "" };

        lines.push(Line::from(vec![
            Span::styled(
                format!("{}{}{}: ", marker, field.name, required_tag),
                if is_focused {
                    Style::default()
                        .fg(theme.accent)
                        .add_modifier(Modifier::BOLD)
                } else {
                    Style::default().fg(theme.fg_dim)
                },
            ),
            Span::styled(value, theme.text()),
        ]));

        // Description hint
        if is_focused {
            if let Some(ref desc) = field.description {
                lines.push(Line::styled(
                    format!("    {}", desc),
                    Style::default().fg(theme.fg_muted),
                ));
            }
            if !field.options.is_empty() {
                lines.push(Line::styled(
                    format!("    Options: {}", field.options.join(", ")),
                    Style::default().fg(theme.fg_muted),
                ));
            }
        }
    }

    lines.push(Line::raw(""));
    lines.push(Line::styled(
        "  j/k:navigate  Enter:edit  Ctrl+S:submit  Esc:cancel",
        Style::default().fg(theme.fg_dim),
    ));

    let paragraph = Paragraph::new(lines).wrap(Wrap { trim: false }).block(
        Block::default()
            .title(" Workflow Dispatch ")
            .borders(Borders::ALL)
            .style(Style::default().bg(theme.bg)),
    );
    frame.render_widget(paragraph, popup_area);
}
