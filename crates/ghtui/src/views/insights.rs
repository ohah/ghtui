use ghtui_core::AppState;
use ratatui::Frame;
use ratatui::layout::{Constraint, Direction, Layout, Rect};
use ratatui::style::{Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, List, ListItem, Paragraph, Tabs, Wrap};

pub fn render(frame: &mut Frame, state: &AppState, area: Rect) {
    let theme = &state.theme;

    if state.is_loading("insights") {
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
                    .title(" Insights ")
                    .borders(Borders::ALL)
                    .border_style(theme.border_style()),
            );
        frame.render_widget(paragraph, area);
        return;
    }

    let Some(ref insights) = state.insights else {
        let paragraph = Paragraph::new("No data").style(theme.text_dim()).block(
            Block::default()
                .title(" Insights ")
                .borders(Borders::ALL)
                .border_style(theme.border_style()),
        );
        frame.render_widget(paragraph, area);
        return;
    };

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Length(2), Constraint::Min(0)])
        .split(area);

    let tab_titles = vec![
        format!("Contributors ({})", insights.contributors.len()),
        "Commit Activity".to_string(),
        "Traffic".to_string(),
    ];
    let tabs = Tabs::new(tab_titles)
        .select(insights.tab)
        .style(Style::default().fg(theme.fg_muted))
        .highlight_style(
            Style::default()
                .fg(theme.tab_active_fg)
                .add_modifier(Modifier::BOLD | Modifier::UNDERLINED),
        )
        .divider(" │ ")
        .block(
            Block::default()
                .borders(Borders::BOTTOM)
                .border_style(theme.border_style()),
        );
    frame.render_widget(tabs, chunks[0]);

    match insights.tab {
        0 => render_contributors(frame, state, chunks[1]),
        1 => render_commit_activity(frame, state, chunks[1]),
        2 => render_traffic(frame, state, chunks[1]),
        _ => {}
    }
}

fn render_contributors(frame: &mut Frame, state: &AppState, area: Rect) {
    let theme = &state.theme;
    let insights = state.insights.as_ref().unwrap();

    if insights.contributors.is_empty() {
        let paragraph = Paragraph::new("  No contributor data available (may still be computing)")
            .style(theme.text_dim())
            .block(
                Block::default()
                    .title(" Contributors ")
                    .borders(Borders::ALL)
                    .border_style(theme.border_style()),
            );
        frame.render_widget(paragraph, area);
        return;
    }

    // Sort by total commits desc
    let mut sorted: Vec<_> = insights.contributors.iter().collect();
    sorted.sort_by(|a, b| b.total.cmp(&a.total));

    let items: Vec<ListItem> = sorted
        .iter()
        .enumerate()
        .map(|(i, contrib)| {
            let login = contrib
                .author
                .as_ref()
                .map(|a| a.login.as_str())
                .unwrap_or("unknown");

            let total_additions: u64 = contrib.weeks.iter().map(|w| w.additions).sum();
            let total_deletions: u64 = contrib.weeks.iter().map(|w| w.deletions).sum();

            // Simple bar chart
            let max_commits = sorted.first().map(|c| c.total).unwrap_or(1).max(1);
            let bar_width = (contrib.total as f64 / max_commits as f64 * 20.0) as usize;
            let bar: String = "█".repeat(bar_width);

            ListItem::new(Line::from(vec![
                Span::styled(
                    format!("  {:<3}", i + 1),
                    Style::default().fg(theme.fg_muted),
                ),
                Span::styled(format!("{:<20}", login), theme.text()),
                Span::styled(
                    format!("{:>6} commits  ", contrib.total),
                    Style::default().fg(theme.accent),
                ),
                Span::styled(
                    format!("+{:<8}", total_additions),
                    Style::default().fg(theme.success),
                ),
                Span::styled(
                    format!("-{:<8}", total_deletions),
                    Style::default().fg(theme.danger),
                ),
                Span::styled(bar, Style::default().fg(theme.accent)),
            ]))
        })
        .collect();

    let list = List::new(items).block(
        Block::default()
            .title(format!(" Contributors ({}) ", insights.contributors.len()))
            .borders(Borders::ALL)
            .border_style(theme.border_style()),
    );
    frame.render_widget(list, area);
}

fn render_commit_activity(frame: &mut Frame, state: &AppState, area: Rect) {
    let theme = &state.theme;
    let insights = state.insights.as_ref().unwrap();

    if insights.commit_activity.is_empty() {
        let paragraph = Paragraph::new("  No commit activity data available")
            .style(theme.text_dim())
            .block(
                Block::default()
                    .title(" Commit Activity (last 52 weeks) ")
                    .borders(Borders::ALL)
                    .border_style(theme.border_style()),
            );
        frame.render_widget(paragraph, area);
        return;
    }

    let mut lines: Vec<Line<'static>> = Vec::new();
    lines.push(Line::raw(""));

    // Show last 26 weeks as ASCII chart
    let recent: Vec<_> = insights
        .commit_activity
        .iter()
        .rev()
        .take(26)
        .collect::<Vec<_>>()
        .into_iter()
        .rev()
        .collect();

    let max_total = recent.iter().map(|w| w.total).max().unwrap_or(1).max(1);

    for week in &recent {
        let bar_width = (week.total as f64 / max_total as f64 * 40.0) as usize;
        let bar: String = "█".repeat(bar_width);

        let ts = chrono::DateTime::from_timestamp(week.week, 0)
            .map(|dt| dt.format("%m/%d").to_string())
            .unwrap_or_default();

        lines.push(Line::from(vec![
            Span::styled(format!("  {:<7}", ts), Style::default().fg(theme.fg_muted)),
            Span::styled(format!("{:>4} ", week.total), theme.text()),
            Span::styled(bar, Style::default().fg(theme.success)),
        ]));
    }

    let total_commits: u64 = insights.commit_activity.iter().map(|w| w.total).sum();
    lines.push(Line::raw(""));
    lines.push(Line::from(vec![Span::styled(
        format!("  Total commits (52 weeks): {}", total_commits),
        Style::default()
            .fg(theme.accent)
            .add_modifier(Modifier::BOLD),
    )]));

    let paragraph = Paragraph::new(lines)
        .block(
            Block::default()
                .title(" Commit Activity (last 26 weeks) ")
                .borders(Borders::ALL)
                .border_style(theme.border_style()),
        )
        .wrap(Wrap { trim: false })
        .scroll((insights.scroll as u16, 0));
    frame.render_widget(paragraph, area);
}

fn render_traffic(frame: &mut Frame, state: &AppState, area: Rect) {
    let theme = &state.theme;
    let insights = state.insights.as_ref().unwrap();

    let mut lines: Vec<Line<'static>> = Vec::new();
    lines.push(Line::raw(""));

    // Views
    lines.push(Line::styled(
        "  Views (last 14 days)".to_string(),
        Style::default()
            .fg(theme.accent)
            .add_modifier(Modifier::BOLD),
    ));
    if let Some(ref views) = insights.traffic_views {
        lines.push(Line::from(vec![
            Span::styled("    Total: ", Style::default().fg(theme.fg_muted)),
            Span::styled(views.count.to_string(), theme.text()),
            Span::styled("  Unique: ", Style::default().fg(theme.fg_muted)),
            Span::styled(views.uniques.to_string(), theme.text()),
        ]));
        for entry in &views.views {
            let date = entry.timestamp.get(..10).unwrap_or(&entry.timestamp);
            lines.push(Line::from(vec![
                Span::styled(
                    format!("    {:<12}", date),
                    Style::default().fg(theme.fg_muted),
                ),
                Span::styled(format!("{:>5} views", entry.count), theme.text()),
                Span::styled(format!("  ({} unique)", entry.uniques), theme.text_dim()),
            ]));
        }
    } else {
        lines.push(Line::styled(
            "    No data (push access required)".to_string(),
            theme.text_dim(),
        ));
    }

    lines.push(Line::raw(""));

    // Clones
    lines.push(Line::styled(
        "  Clones (last 14 days)".to_string(),
        Style::default()
            .fg(theme.accent)
            .add_modifier(Modifier::BOLD),
    ));
    if let Some(ref clones) = insights.traffic_clones {
        lines.push(Line::from(vec![
            Span::styled("    Total: ", Style::default().fg(theme.fg_muted)),
            Span::styled(clones.count.to_string(), theme.text()),
            Span::styled("  Unique: ", Style::default().fg(theme.fg_muted)),
            Span::styled(clones.uniques.to_string(), theme.text()),
        ]));
        for entry in &clones.clones {
            let date = entry.timestamp.get(..10).unwrap_or(&entry.timestamp);
            lines.push(Line::from(vec![
                Span::styled(
                    format!("    {:<12}", date),
                    Style::default().fg(theme.fg_muted),
                ),
                Span::styled(format!("{:>5} clones", entry.count), theme.text()),
                Span::styled(format!("  ({} unique)", entry.uniques), theme.text_dim()),
            ]));
        }
    } else {
        lines.push(Line::styled(
            "    No data (push access required)".to_string(),
            theme.text_dim(),
        ));
    }

    let paragraph = Paragraph::new(lines)
        .block(
            Block::default()
                .title(" Traffic ")
                .borders(Borders::ALL)
                .border_style(theme.border_style()),
        )
        .wrap(Wrap { trim: false })
        .scroll((insights.scroll as u16, 0));
    frame.render_widget(paragraph, area);
}
