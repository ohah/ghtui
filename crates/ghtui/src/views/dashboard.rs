use ghtui_core::AppState;
use ratatui::Frame;
use ratatui::layout::{Constraint, Direction, Layout, Rect};
use ratatui::style::{Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, List, ListItem, Paragraph, Wrap};

pub fn render(frame: &mut Frame, state: &AppState, area: Rect) {
    let theme = &state.theme;

    let repo_name = state
        .current_repo
        .as_ref()
        .map(|r| r.full_name())
        .unwrap_or_else(|| "No repository selected".to_string());

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(5), // Repo info
            Constraint::Min(0),    // Recent repos or README placeholder
        ])
        .split(area);

    // Repo info section
    let info_lines = vec![
        Line::raw(""),
        Line::from(vec![Span::styled(
            "  About",
            Style::default().fg(theme.fg).add_modifier(Modifier::BOLD),
        )]),
        Line::from(vec![Span::styled(
            "  A comprehensive GitHub TUI built with Rust and ratatui",
            Style::default().fg(theme.fg_dim),
        )]),
        Line::raw(""),
    ];

    let info = Paragraph::new(info_lines)
        .style(Style::default().bg(theme.bg))
        .block(
            Block::default()
                .borders(Borders::BOTTOM)
                .border_style(Style::default().fg(theme.border)),
        );
    frame.render_widget(info, chunks[0]);

    // If we have recent repos, show them; otherwise show the static help
    if !state.recent_repos.is_empty() {
        render_recent_repos(frame, state, chunks[1]);
    } else if state.is_loading("recent_repos") {
        let loading = Paragraph::new("  Loading recent repositories...")
            .style(Style::default().fg(theme.fg_dim))
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .border_style(Style::default().fg(theme.border))
                    .title(Span::styled(
                        " Recent Repositories ",
                        Style::default().fg(theme.fg),
                    )),
            );
        frame.render_widget(loading, chunks[1]);
    } else {
        render_quick_nav(frame, state, &repo_name, chunks[1]);
    }
}

fn render_recent_repos(frame: &mut Frame, state: &AppState, area: Rect) {
    let theme = &state.theme;

    let items: Vec<ListItem> = state
        .recent_repos
        .iter()
        .enumerate()
        .map(|(i, repo)| {
            let visibility = if repo.private { "Private" } else { "Public" };
            let vis_color = if repo.private {
                theme.warning
            } else {
                theme.success
            };
            let desc = repo.description.as_deref().unwrap_or("");
            let lang = repo.language.as_deref().unwrap_or("");
            let stars = repo.stargazers_count;

            let line = Line::from(vec![
                Span::styled(
                    format!("  {} ", repo.full_name),
                    if i == state.dashboard_selected {
                        Style::default()
                            .fg(theme.accent)
                            .add_modifier(Modifier::BOLD)
                    } else {
                        Style::default().fg(theme.fg)
                    },
                ),
                Span::styled(format!("[{}] ", visibility), Style::default().fg(vis_color)),
                Span::styled(
                    if !lang.is_empty() {
                        format!("{} ", lang)
                    } else {
                        String::new()
                    },
                    Style::default().fg(theme.fg_muted),
                ),
                Span::styled(
                    if stars > 0 {
                        format!("* {} ", stars)
                    } else {
                        String::new()
                    },
                    Style::default().fg(theme.warning),
                ),
                Span::styled(
                    if !desc.is_empty() {
                        format!("- {}", desc)
                    } else {
                        String::new()
                    },
                    Style::default().fg(theme.fg_dim),
                ),
            ]);
            ListItem::new(line)
        })
        .collect();

    let list = List::new(items).block(
        Block::default()
            .borders(Borders::ALL)
            .border_style(Style::default().fg(theme.border))
            .title(Span::styled(
                " Recent Repositories (Enter to open) ",
                Style::default().fg(theme.fg).add_modifier(Modifier::BOLD),
            )),
    );

    frame.render_widget(list, area);
}

fn render_quick_nav(frame: &mut Frame, state: &AppState, repo_name: &str, area: Rect) {
    let theme = &state.theme;

    let readme_lines = vec![
        Line::raw(""),
        Line::from(vec![Span::styled(
            "  README.md",
            Style::default().fg(theme.fg).add_modifier(Modifier::BOLD),
        )]),
        Line::raw(""),
        Line::from(vec![
            Span::styled("  # ", Style::default().fg(theme.fg_muted)),
            Span::styled(
                repo_name.to_string(),
                Style::default().fg(theme.fg).add_modifier(Modifier::BOLD),
            ),
        ]),
        Line::raw(""),
        Line::from(vec![Span::styled(
            "  A comprehensive GitHub TUI that aims to cover everything",
            Style::default().fg(theme.fg),
        )]),
        Line::from(vec![Span::styled(
            "  you can do on the GitHub web interface.",
            Style::default().fg(theme.fg),
        )]),
        Line::raw(""),
        Line::from(vec![
            Span::styled("  ## ", Style::default().fg(theme.fg_muted)),
            Span::styled(
                "Quick Navigation",
                Style::default().fg(theme.fg).add_modifier(Modifier::BOLD),
            ),
        ]),
        Line::raw(""),
        Line::from(vec![
            Span::styled(
                "  1 ",
                Style::default()
                    .fg(theme.accent)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::styled("Code        ", Style::default().fg(theme.fg)),
            Span::styled(
                "  2 ",
                Style::default()
                    .fg(theme.accent)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::styled("Issues      ", Style::default().fg(theme.fg)),
            Span::styled(
                "  3 ",
                Style::default()
                    .fg(theme.accent)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::styled("Pull requests", Style::default().fg(theme.fg)),
        ]),
        Line::from(vec![
            Span::styled(
                "  4 ",
                Style::default()
                    .fg(theme.accent)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::styled("Actions     ", Style::default().fg(theme.fg)),
            Span::styled(
                "  5 ",
                Style::default()
                    .fg(theme.accent)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::styled("Notifications", Style::default().fg(theme.fg)),
            Span::styled(
                "  6 ",
                Style::default()
                    .fg(theme.accent)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::styled("Search", Style::default().fg(theme.fg)),
        ]),
        Line::raw(""),
        Line::from(vec![
            Span::styled(
                "  t ",
                Style::default()
                    .fg(theme.warning)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::styled("Toggle theme   ", Style::default().fg(theme.fg)),
            Span::styled(
                "  ? ",
                Style::default()
                    .fg(theme.warning)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::styled("Help   ", Style::default().fg(theme.fg)),
            Span::styled(
                "  q ",
                Style::default()
                    .fg(theme.danger)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::styled("Quit", Style::default().fg(theme.fg)),
        ]),
    ];

    let readme = Paragraph::new(readme_lines)
        .style(Style::default().bg(theme.bg))
        .block(
            Block::default()
                .borders(Borders::ALL)
                .border_style(Style::default().fg(theme.border))
                .title(Span::styled(" Code ", Style::default().fg(theme.fg))),
        )
        .wrap(Wrap { trim: false });

    frame.render_widget(readme, area);
}
