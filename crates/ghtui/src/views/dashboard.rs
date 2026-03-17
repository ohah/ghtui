use ghtui_core::AppState;
use ratatui::Frame;
use ratatui::layout::{Constraint, Direction, Layout, Rect};
use ratatui::style::{Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, Paragraph, Wrap};

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
            Constraint::Min(0),    // README placeholder
        ])
        .split(area);

    // Repo info section (like GitHub's repo header area)
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

    // README section
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
                repo_name,
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

    frame.render_widget(readme, chunks[1]);
}
