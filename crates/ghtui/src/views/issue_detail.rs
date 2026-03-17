use ghtui_core::AppState;
use ghtui_widgets::render_markdown;
use ratatui::Frame;
use ratatui::layout::{Constraint, Direction, Layout, Rect};
use ratatui::style::{Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, Paragraph, Wrap};

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

    let issue = &detail_state.detail.issue;

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Length(3), Constraint::Min(0)])
        .split(area);

    let state_color = match issue.state {
        ghtui_core::types::IssueState::Open => theme.success,
        ghtui_core::types::IssueState::Closed => theme.done,
    };

    let title_lines = vec![
        Line::from(vec![
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
        ]),
        Line::styled(format!(" {}", issue.title), theme.text_bold()),
    ];

    let title = Paragraph::new(title_lines)
        .style(Style::default().bg(theme.bg))
        .block(
            Block::default()
                .borders(Borders::BOTTOM)
                .border_style(theme.border_style()),
        );
    frame.render_widget(title, chunks[0]);

    let mut lines = Vec::new();

    if let Some(ref body) = issue.body {
        lines.extend(render_markdown(body));
        lines.push(Line::raw(""));
        lines.push(Line::styled("─".repeat(40), theme.border_style()));
    }

    for comment in &detail_state.detail.comments {
        lines.push(Line::raw(""));
        lines.push(Line::from(vec![
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
        ]));
        lines.extend(render_markdown(&comment.body));
        lines.push(Line::styled("─".repeat(40), theme.border_style()));
    }

    let paragraph = Paragraph::new(lines)
        .style(theme.text())
        .block(
            Block::default()
                .borders(Borders::ALL)
                .border_style(theme.border_style())
                .title(" Comments "),
        )
        .wrap(Wrap { trim: false })
        .scroll((detail_state.scroll as u16, 0));

    frame.render_widget(paragraph, chunks[1]);
}
