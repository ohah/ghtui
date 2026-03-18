use super::components;
use ghtui_core::AppState;
use ratatui::Frame;
use ratatui::layout::{Constraint, Direction, Layout, Rect};
use ratatui::style::{Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, List, ListItem, Paragraph, Wrap};

pub fn render(frame: &mut Frame, state: &AppState, area: Rect) {
    let theme = &state.theme;

    if state.is_loading("insights") {
        components::render_loading(frame, theme, area, "Insights");
        return;
    }

    let Some(ref insights) = state.insights else {
        let paragraph = Paragraph::new("  No data — check API rate limit or network")
            .style(theme.text_dim())
            .block(
                Block::default()
                    .title(" Insights ")
                    .borders(Borders::ALL)
                    .border_style(theme.border_style()),
            );
        frame.render_widget(paragraph, area);
        return;
    };

    let show_sidebar = area.width >= 80;

    let content_area = if show_sidebar {
        // Horizontal split: sidebar (30) | content (rest)
        let chunks = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([Constraint::Length(30), Constraint::Min(0)])
            .split(area);

        // Sidebar
        let sidebar_titles = [
            format!("Contributors ({})", insights.contributors.len()),
            "Commit Activity".to_string(),
            "Traffic".to_string(),
            "Code Frequency".to_string(),
            format!("Forks ({})", insights.forks.len()),
            format!("Dependencies ({})", insights.dependencies.len()),
        ];

        components::render_sidebar(
            frame,
            theme,
            "Insights",
            &sidebar_titles,
            insights.tab,
            insights.sidebar_focused,
            chunks[0],
        );

        chunks[1]
    } else {
        area
    };

    // Content
    match insights.tab {
        0 => render_contributors(frame, state, content_area),
        1 => render_commit_activity(frame, state, content_area),
        2 => render_traffic(frame, state, content_area),
        3 => render_code_frequency(frame, state, content_area),
        4 => render_forks(frame, state, content_area),
        5 => render_dependencies(frame, state, content_area),
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

fn render_code_frequency(frame: &mut Frame, state: &AppState, area: Rect) {
    let theme = &state.theme;
    let insights = state.insights.as_ref().unwrap();

    if state.is_loading("code_frequency") || insights.code_frequency.is_empty() {
        let msg = if state.is_loading("code_frequency") {
            "  Loading code frequency..."
        } else {
            "  No code frequency data available"
        };
        let paragraph = Paragraph::new(msg).style(theme.text_dim()).block(
            Block::default()
                .title(" Code Frequency ")
                .borders(Borders::ALL)
                .border_style(theme.border_style()),
        );
        frame.render_widget(paragraph, area);
        return;
    }

    let recent: Vec<_> = insights
        .code_frequency
        .iter()
        .rev()
        .take(26)
        .collect::<Vec<_>>()
        .into_iter()
        .rev()
        .collect();

    let max_val = recent
        .iter()
        .map(|cf| cf.1.unsigned_abs().max(cf.2.unsigned_abs()))
        .max()
        .unwrap_or(1)
        .max(1);

    let mut lines: Vec<Line<'static>> = Vec::new();
    lines.push(Line::raw(""));
    lines.push(Line::styled(
        "  Additions/Deletions (last 26 weeks)".to_string(),
        Style::default()
            .fg(theme.accent)
            .add_modifier(Modifier::BOLD),
    ));
    lines.push(Line::raw(""));

    for cf in &recent {
        let ts = chrono::DateTime::from_timestamp(cf.0, 0)
            .map(|dt| dt.format("%m/%d").to_string())
            .unwrap_or_default();

        let add_bar_len = ((cf.1.unsigned_abs() as f64 / max_val as f64) * 30.0) as usize;
        let del_bar_len = ((cf.2.unsigned_abs() as f64 / max_val as f64) * 30.0) as usize;

        lines.push(Line::from(vec![
            Span::styled(format!("  {} ", ts), Style::default().fg(theme.fg_dim)),
            Span::styled(format!("+{:<6}", cf.1), Style::default().fg(theme.success)),
            Span::styled("█".repeat(add_bar_len), Style::default().fg(theme.success)),
        ]));
        lines.push(Line::from(vec![
            Span::styled("        ".to_string(), Style::default()),
            Span::styled(
                format!("-{:<6}", cf.2.unsigned_abs()),
                Style::default().fg(theme.danger),
            ),
            Span::styled("█".repeat(del_bar_len), Style::default().fg(theme.danger)),
        ]));
    }

    let paragraph = Paragraph::new(lines)
        .block(
            Block::default()
                .title(" Code Frequency ")
                .borders(Borders::ALL)
                .border_style(theme.border_style()),
        )
        .wrap(Wrap { trim: false })
        .scroll((insights.scroll as u16, 0));
    frame.render_widget(paragraph, area);
}

fn render_forks(frame: &mut Frame, state: &AppState, area: Rect) {
    let theme = &state.theme;
    let insights = state.insights.as_ref().unwrap();

    if state.is_loading("forks") {
        components::render_loading(frame, theme, area, "Forks");
        return;
    }

    if insights.forks.is_empty() {
        let paragraph = Paragraph::new("  No forks").style(theme.text_dim()).block(
            Block::default()
                .title(" Forks ")
                .borders(Borders::ALL)
                .border_style(theme.border_style()),
        );
        frame.render_widget(paragraph, area);
        return;
    }

    let items: Vec<ListItem> = insights
        .forks
        .iter()
        .map(|fork| {
            ListItem::new(Line::from(vec![
                Span::styled("  ", Style::default()),
                Span::styled(fork.full_name.clone(), theme.text()),
                Span::styled(
                    format!(" ★{}", fork.stargazers_count),
                    Style::default().fg(theme.warning),
                ),
                Span::styled(
                    format!(" @{}", fork.owner.login),
                    Style::default().fg(theme.fg_dim),
                ),
            ]))
        })
        .collect();

    let list = List::new(items).block(
        Block::default()
            .title(format!(" Forks ({}) ", insights.forks.len()))
            .borders(Borders::ALL)
            .border_style(theme.border_style()),
    );
    frame.render_widget(list, area);
}

fn render_dependencies(frame: &mut Frame, state: &AppState, area: Rect) {
    let theme = &state.theme;
    let insights = state.insights.as_ref().unwrap();

    if insights.dependencies.is_empty() {
        let hint = if state.is_loading("dependency_graph") {
            "  Loading dependency graph..."
        } else {
            "  No dependencies found (SBOM not available)"
        };
        let paragraph = Paragraph::new(hint).style(theme.text_dim()).block(
            Block::default()
                .title(" Dependencies ")
                .borders(Borders::ALL)
                .border_style(theme.border_style()),
        );
        frame.render_widget(paragraph, area);
        return;
    }

    let items: Vec<ListItem> = insights
        .dependencies
        .iter()
        .map(|dep| {
            let version = dep.version.as_deref().unwrap_or("?");
            ListItem::new(Line::from(vec![
                Span::styled("  ", Style::default()),
                Span::styled(&dep.name, theme.text()),
                Span::styled(
                    format!("  {}", version),
                    Style::default().fg(theme.fg_muted),
                ),
            ]))
        })
        .collect();

    let list = List::new(items).block(
        Block::default()
            .title(format!(" Dependencies ({}) ", insights.dependencies.len()))
            .borders(Borders::ALL)
            .border_style(theme.border_style()),
    );
    frame.render_widget(list, area);
}
