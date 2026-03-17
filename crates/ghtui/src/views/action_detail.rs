use ghtui_core::AppState;
use ratatui::Frame;
use ratatui::layout::{Constraint, Direction, Layout, Rect};
use ratatui::style::Style;
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
        let paragraph = Paragraph::new(Line::from(spinner.span()))
            .block(
                Block::default()
                    .title(" Run Detail ")
                    .borders(Borders::ALL)
                    .border_style(theme.border_style()),
            );
        frame.render_widget(paragraph, area);
        return;
    }

    let Some(ref detail) = state.action_detail else {
        let paragraph = Paragraph::new("No data")
            .style(theme.text_dim())
            .block(
                Block::default()
                    .title(" Run Detail ")
                    .borders(Borders::ALL)
                    .border_style(theme.border_style()),
            );
        frame.render_widget(paragraph, area);
        return;
    };

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3),
            Constraint::Percentage(40),
            Constraint::Min(0),
        ])
        .split(area);

    let run = &detail.detail.run;

    let status_color = match &run.conclusion {
        Some(ghtui_core::types::RunConclusion::Success) => theme.success,
        Some(ghtui_core::types::RunConclusion::Failure) => theme.danger,
        _ => theme.warning,
    };

    let header_lines = vec![
        Line::from(vec![
            Span::styled(" ● ", Style::default().fg(status_color)),
            Span::styled(
                run.name.as_deref().unwrap_or("Unknown"),
                theme.text_bold(),
            ),
            Span::styled(
                format!(" #{}", run.run_number),
                Style::default().fg(theme.fg_muted),
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
        ]),
    ];

    let header = Paragraph::new(header_lines)
        .style(Style::default().bg(theme.bg))
        .block(Block::default().borders(Borders::BOTTOM).border_style(theme.border_style()));
    frame.render_widget(header, chunks[0]);

    // Jobs
    let job_items: Vec<ListItem> = detail
        .detail
        .jobs
        .iter()
        .map(|job| {
            let icon = match &job.conclusion {
                Some(ghtui_core::types::RunConclusion::Success) => {
                    Span::styled(" ✓ ", Style::default().fg(theme.success))
                }
                Some(ghtui_core::types::RunConclusion::Failure) => {
                    Span::styled(" ✗ ", Style::default().fg(theme.danger))
                }
                _ => Span::styled(" ● ", Style::default().fg(theme.warning)),
            };
            ListItem::new(Line::from(vec![icon, Span::styled(&job.name, theme.text())]))
        })
        .collect();

    let jobs_list = List::new(job_items)
        .block(
            Block::default()
                .title(" Jobs ")
                .borders(Borders::ALL)
                .border_style(theme.border_style()),
        )
        .highlight_style(theme.selected());

    let mut jobs_state = ListState::default();
    jobs_state.select(Some(detail.selected_job));
    frame.render_stateful_widget(jobs_list, chunks[1], &mut jobs_state);

    // Log
    if let Some(ref log) = detail.log {
        let log_lines: Vec<Line> = log
            .iter()
            .map(|line| Line::styled(&line.content, theme.text()))
            .collect();

        let log_paragraph = Paragraph::new(log_lines)
            .style(theme.text())
            .block(
                Block::default()
                    .title(" Log ")
                    .borders(Borders::ALL)
                    .border_style(theme.border_style()),
            )
            .wrap(Wrap { trim: false })
            .scroll((detail.log_scroll as u16, 0));

        frame.render_widget(log_paragraph, chunks[2]);
    } else {
        let paragraph = Paragraph::new("  Select a job to view logs")
            .style(theme.text_dim())
            .block(
                Block::default()
                    .title(" Log ")
                    .borders(Borders::ALL)
                    .border_style(theme.border_style()),
            );
        frame.render_widget(paragraph, chunks[2]);
    }
}
