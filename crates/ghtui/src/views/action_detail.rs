use ghtui_core::AppState;
use ghtui_core::ansi::parse_ansi_line;
use ghtui_core::state::ActionDetailFocus;
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

    // Layout: header(3) + jobs(40%) + log(rest) + action_bar(1)
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3),
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

    let header_lines = vec![
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
            let icon = match &job.conclusion {
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
            };

            // Duration
            let duration = match (job.started_at, job.completed_at) {
                (Some(start), Some(end)) => {
                    let secs = (end - start).num_seconds();
                    if secs >= 60 {
                        format!(" {}m{}s", secs / 60, secs % 60)
                    } else {
                        format!(" {}s", secs)
                    }
                }
                (Some(_), None) => " running...".to_string(),
                _ => String::new(),
            };

            let job_style = if is_selected_job && jobs_focused {
                theme.selected()
            } else if is_selected_job {
                Style::default()
                    .fg(theme.accent)
                    .add_modifier(Modifier::BOLD)
            } else {
                theme.text()
            };

            items.push(ListItem::new(Line::from(vec![
                icon,
                Span::styled(&job.name, job_style),
                Span::styled(duration, Style::default().fg(theme.fg_dim)),
            ])));

            // Steps (shown for selected job)
            if is_selected_job {
                for step in &job.steps {
                    let collapsed = detail.is_step_collapsed(step.number);
                    let fold_icon = if collapsed { "▸" } else { "▾" };

                    let step_icon = match step.conclusion.as_deref() {
                        Some("success") => Span::styled("  ✓ ", Style::default().fg(theme.success)),
                        Some("failure") => Span::styled("  ✗ ", Style::default().fg(theme.danger)),
                        Some("skipped") => {
                            Span::styled("  ◌ ", Style::default().fg(theme.fg_muted))
                        }
                        _ => Span::styled("  ● ", Style::default().fg(theme.warning)),
                    };

                    let step_duration = match (step.started_at, step.completed_at) {
                        (Some(start), Some(end)) => {
                            let secs = (end - start).num_seconds();
                            if secs >= 60 {
                                format!(" {}m{}s", secs / 60, secs % 60)
                            } else if secs > 0 {
                                format!(" {}s", secs)
                            } else {
                                String::new()
                            }
                        }
                        _ => String::new(),
                    };

                    items.push(ListItem::new(Line::from(vec![
                        Span::styled(
                            format!("  {} ", fold_icon),
                            Style::default().fg(theme.fg_dim),
                        ),
                        step_icon,
                        Span::styled(&step.name, Style::default().fg(theme.fg_muted)),
                        Span::styled(step_duration, Style::default().fg(theme.fg_dim)),
                    ])));
                }
            }
            items
        })
        .collect();

    let jobs_border = if jobs_focused {
        Style::default().fg(theme.accent)
    } else {
        theme.border_style()
    };

    let jobs_list = List::new(job_items)
        .block(
            Block::default()
                .title(Span::styled(
                    format!(" Jobs ({}) ", detail.detail.jobs.len()),
                    theme.text_bold(),
                ))
                .borders(Borders::ALL)
                .border_style(jobs_border),
        )
        .highlight_style(theme.selected());

    let mut jobs_state = ListState::default();
    jobs_state.select(Some(detail.selected_job));
    frame.render_stateful_widget(jobs_list, chunks[1], &mut jobs_state);

    // Log with ANSI color
    let log_focused = detail.focus == ActionDetailFocus::Log;
    let log_border = if log_focused {
        Style::default().fg(theme.accent)
    } else {
        theme.border_style()
    };

    if let Some(ref log) = detail.log {
        let log_lines: Vec<Line> = log
            .iter()
            .map(|line| parse_ansi_line(&line.content))
            .collect();

        let total_lines = log_lines.len();
        let log_title = format!(" Log ({} lines) ", total_lines);

        let log_paragraph = Paragraph::new(log_lines)
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

fn render_action_bar(
    frame: &mut Frame,
    detail: &ghtui_core::state::ActionDetailState,
    theme: &ghtui_core::Theme,
    area: Rect,
) {
    let items = detail.action_bar_items();
    let mut spans = vec![Span::styled(" ", Style::default())];

    for (i, item) in items.iter().enumerate() {
        let is_selected = detail.action_bar_focused && i == detail.action_bar_selected;
        let style = if is_selected {
            Style::default()
                .fg(theme.bg)
                .bg(theme.accent)
                .add_modifier(Modifier::BOLD)
        } else {
            Style::default().fg(theme.fg_dim)
        };

        spans.push(Span::styled(format!(" {} ", item), style));
        if i < items.len() - 1 {
            spans.push(Span::styled(" │ ", Style::default().fg(theme.fg_dim)));
        }
    }

    let line = Line::from(spans);
    let paragraph = Paragraph::new(line).style(Style::default().bg(theme.footer_bg));
    frame.render_widget(paragraph, area);
}
