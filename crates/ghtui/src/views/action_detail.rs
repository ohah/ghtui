use ghtui_core::AppState;
use ghtui_core::state::ActionDetailFocus;
use ghtui_core::state::actions::format_duration;
use ratatui::Frame;
use ratatui::layout::{Constraint, Direction, Layout, Rect};
use ratatui::style::{Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, List, ListItem, ListState, Paragraph, Wrap};

pub fn render(frame: &mut Frame, state: &AppState, area: Rect) {
    let theme = &state.theme;

    if state.is_loading("action_detail") {
        let spinner = ghtui_widgets::Spinner::new(
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_millis() as usize
                / 100,
        );
        let paragraph = Paragraph::new(Line::from(spinner.span())).block(
            Block::default()
                .title(" Run Detail ")
                .borders(Borders::ALL)
                .border_style(theme.border_style()),
        );
        frame.render_widget(paragraph, area);
        return;
    }

    let Some(ref detail) = state.action_detail else {
        let paragraph = Paragraph::new("No data").style(theme.text_dim()).block(
            Block::default()
                .title(" Run Detail ")
                .borders(Borders::ALL)
                .border_style(theme.border_style()),
        );
        frame.render_widget(paragraph, area);
        return;
    };

    // Dynamic header height: 3 base + 1 per artifact line + 1 for deployments
    let has_artifacts = !detail.artifacts.is_empty();
    let has_deployments = !detail.pending_deployments.is_empty();
    let extra_lines = if has_artifacts { 1 } else { 0 } + if has_deployments { 1 } else { 0 };
    let header_height = 3 + extra_lines as u16;

    // Layout: header + jobs(40%) + log(rest) + action_bar(1)
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(header_height),
            Constraint::Percentage(40),
            Constraint::Min(0),
            Constraint::Length(1),
        ])
        .split(area);

    let run = &detail.detail.run;

    // Header
    let status_color = match &run.conclusion {
        Some(ghtui_core::types::RunConclusion::Success) => theme.success,
        Some(ghtui_core::types::RunConclusion::Failure) => theme.danger,
        Some(ghtui_core::types::RunConclusion::Cancelled) => theme.fg_muted,
        _ => theme.warning,
    };

    let status_text = match (&run.status, &run.conclusion) {
        (_, Some(c)) => format!("{}", c),
        (Some(s), None) => format!("{}", s),
        _ => "unknown".to_string(),
    };

    let mut header_lines = vec![
        Line::from(vec![
            Span::styled(" ● ", Style::default().fg(status_color)),
            Span::styled(run.name.as_deref().unwrap_or("Unknown"), theme.text_bold()),
            Span::styled(
                format!(" #{}", run.run_number),
                Style::default().fg(theme.fg_muted),
            ),
            Span::styled(
                format!(" ({})", status_text),
                Style::default().fg(status_color),
            ),
        ]),
        Line::from(vec![
            Span::styled(" Branch: ", Style::default().fg(theme.fg_dim)),
            Span::styled(
                run.head_branch.as_deref().unwrap_or("?"),
                Style::default().fg(theme.accent),
            ),
            Span::raw("  "),
            Span::styled(" Event: ", Style::default().fg(theme.fg_dim)),
            Span::styled(&run.event, theme.text()),
            Span::raw("  "),
            Span::styled(" SHA: ", Style::default().fg(theme.fg_dim)),
            Span::styled(
                &run.head_sha[..7.min(run.head_sha.len())],
                Style::default().fg(theme.fg_muted),
            ),
        ]),
    ];

    // Artifacts line
    if has_artifacts {
        let artifact_names: Vec<&str> = detail.artifacts.iter().map(|a| a.name.as_str()).collect();
        let total_size: u64 = detail.artifacts.iter().map(|a| a.size_in_bytes).sum();
        let size_str = format_size(total_size);
        let mut spans = vec![
            Span::styled(
                format!(" Artifacts ({}): ", detail.artifacts.len()),
                Style::default().fg(theme.fg_dim),
            ),
            Span::styled(artifact_names.join(", "), Style::default().fg(theme.accent)),
            Span::styled(
                format!(" ({})", size_str),
                Style::default().fg(theme.fg_muted),
            ),
        ];
        // Download progress indicator
        if let Some(ref downloading) = detail.downloading_artifact {
            spans.push(Span::styled(
                format!("  [downloading: {}...]", downloading),
                Style::default().fg(theme.warning),
            ));
        }
        header_lines.push(Line::from(spans));
    }

    // Pending deployments line
    if has_deployments {
        let env_names: Vec<&str> = detail
            .pending_deployments
            .iter()
            .map(|d| d.environment.name.as_str())
            .collect();
        header_lines.push(Line::from(vec![
            Span::styled(" ⏳ Pending: ", Style::default().fg(theme.warning)),
            Span::styled(env_names.join(", "), Style::default().fg(theme.accent)),
        ]));
    }

    let header = Paragraph::new(header_lines)
        .style(Style::default().bg(theme.bg))
        .block(
            Block::default()
                .borders(Borders::BOTTOM)
                .border_style(theme.border_style()),
        );
    frame.render_widget(header, chunks[0]);

    // Jobs with steps
    let jobs_focused = detail.focus == ActionDetailFocus::Jobs;
    let job_items: Vec<ListItem> = detail
        .detail
        .jobs
        .iter()
        .enumerate()
        .flat_map(|(job_idx, job)| {
            let mut items = Vec::new();
            let is_selected_job = job_idx == detail.selected_job;

            // Job header
            let icon = conclusion_icon(&job.conclusion, theme);
            let duration = format_duration(job.started_at, job.completed_at);

            let job_style = if is_selected_job && jobs_focused {
                theme.selected()
            } else if is_selected_job {
                Style::default()
                    .fg(theme.accent)
                    .add_modifier(Modifier::BOLD)
            } else {
                theme.text()
            };

            let mut job_spans = vec![icon, Span::styled(&job.name, job_style)];
            if !duration.is_empty() {
                job_spans.push(Span::styled(
                    format!(" {}", duration),
                    Style::default().fg(theme.fg_dim),
                ));
            }
            items.push(ListItem::new(Line::from(job_spans)));

            // Steps (shown for selected job, unless collapsed)
            if is_selected_job && !detail.steps_collapsed {
                for step in &job.steps {
                    let step_icon = match step.conclusion.as_deref() {
                        Some("success") => Span::styled("  ✓ ", Style::default().fg(theme.success)),
                        Some("failure") => Span::styled("  ✗ ", Style::default().fg(theme.danger)),
                        Some("skipped") => {
                            Span::styled("  ◌ ", Style::default().fg(theme.fg_muted))
                        }
                        _ => Span::styled("  ● ", Style::default().fg(theme.warning)),
                    };

                    let step_dur = format_duration(step.started_at, step.completed_at);
                    let mut step_spans = vec![
                        Span::styled("    ", Style::default()),
                        step_icon,
                        Span::styled(&step.name, Style::default().fg(theme.fg_muted)),
                    ];
                    if !step_dur.is_empty() {
                        step_spans.push(Span::styled(
                            format!(" {}", step_dur),
                            Style::default().fg(theme.fg_dim),
                        ));
                    }
                    items.push(ListItem::new(Line::from(step_spans)));
                }
            } else if is_selected_job && detail.steps_collapsed {
                items.push(ListItem::new(Line::from(vec![Span::styled(
                    format!("    ▸ {} steps collapsed", job.steps.len()),
                    Style::default().fg(theme.fg_dim),
                )])));
            }
            items
        })
        .collect();

    let jobs_border = if jobs_focused {
        Style::default().fg(theme.accent)
    } else {
        theme.border_style()
    };

    let fold_hint = if detail.steps_collapsed {
        " [l:expand]"
    } else {
        " [h:fold]"
    };
    let jobs_title = format!(" Jobs ({}){} ", detail.detail.jobs.len(), fold_hint);

    let jobs_list = List::new(job_items)
        .block(
            Block::default()
                .title(Span::styled(jobs_title, theme.text_bold()))
                .borders(Borders::ALL)
                .border_style(jobs_border),
        )
        .highlight_style(theme.selected());

    let mut jobs_state = ListState::default();
    jobs_state.select(Some(detail.selected_job));
    frame.render_stateful_widget(jobs_list, chunks[1], &mut jobs_state);

    // Log — use pre-parsed ANSI lines (no per-frame parsing)
    let log_focused = detail.focus == ActionDetailFocus::Log;
    let log_border = if log_focused {
        Style::default().fg(theme.accent)
    } else {
        theme.border_style()
    };

    if let Some(ref parsed) = detail.parsed_log {
        let total_lines = parsed.len();
        let live_indicator = if detail.log_streaming { " LIVE " } else { "" };
        let log_title = format!(" Log ({} lines){} ", total_lines, live_indicator);

        let log_paragraph = Paragraph::new(parsed.clone())
            .style(theme.text())
            .block(
                Block::default()
                    .title(Span::styled(log_title, theme.text_bold()))
                    .borders(Borders::ALL)
                    .border_style(log_border),
            )
            .wrap(Wrap { trim: false })
            .scroll((detail.log_scroll as u16, 0));

        frame.render_widget(log_paragraph, chunks[2]);
    } else {
        let hint = if state.is_loading("job_log") {
            "  Loading logs..."
        } else {
            "  Select a job and press Enter to view logs"
        };
        let paragraph = Paragraph::new(hint).style(theme.text_dim()).block(
            Block::default()
                .title(" Log ")
                .borders(Borders::ALL)
                .border_style(log_border),
        );
        frame.render_widget(paragraph, chunks[2]);
    }

    // Action bar
    render_action_bar(frame, detail, theme, chunks[3]);
}

fn conclusion_icon<'a>(
    conclusion: &Option<ghtui_core::types::RunConclusion>,
    theme: &ghtui_core::Theme,
) -> Span<'a> {
    match conclusion {
        Some(ghtui_core::types::RunConclusion::Success) => {
            Span::styled(" ✓ ", Style::default().fg(theme.success))
        }
        Some(ghtui_core::types::RunConclusion::Failure) => {
            Span::styled(" ✗ ", Style::default().fg(theme.danger))
        }
        Some(ghtui_core::types::RunConclusion::Cancelled) => {
            Span::styled(" ◌ ", Style::default().fg(theme.fg_muted))
        }
        _ => Span::styled(" ● ", Style::default().fg(theme.warning)),
    }
}

fn render_action_bar(
    frame: &mut Frame,
    detail: &ghtui_core::state::ActionDetailState,
    theme: &ghtui_core::Theme,
    area: Rect,
) {
    let items = &detail.action_bar_items;
    let is_focused = detail.focus == ActionDetailFocus::ActionBar;
    let mut spans = vec![Span::styled(" ", Style::default())];

    for (i, item) in items.iter().enumerate() {
        let is_selected = is_focused && i == detail.action_bar_selected;
        let style = if is_selected {
            Style::default()
                .fg(theme.bg)
                .bg(theme.accent)
                .add_modifier(Modifier::BOLD)
        } else {
            Style::default().fg(theme.fg_dim)
        };

        spans.push(Span::styled(format!(" {} ", item.label()), style));
        if i < items.len() - 1 {
            spans.push(Span::styled(" │ ", Style::default().fg(theme.fg_dim)));
        }
    }

    let line = Line::from(spans);
    let paragraph = Paragraph::new(line).style(Style::default().bg(theme.footer_bg));
    frame.render_widget(paragraph, area);
}

fn format_size(bytes: u64) -> String {
    if bytes >= 1_073_741_824 {
        format!("{:.1} GB", bytes as f64 / 1_073_741_824.0)
    } else if bytes >= 1_048_576 {
        format!("{:.1} MB", bytes as f64 / 1_048_576.0)
    } else if bytes >= 1024 {
        format!("{:.1} KB", bytes as f64 / 1024.0)
    } else {
        format!("{} B", bytes)
    }
}
