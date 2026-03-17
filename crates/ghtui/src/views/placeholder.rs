use ghtui_core::AppState;
use ratatui::Frame;
use ratatui::layout::Rect;
use ratatui::style::{Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, Paragraph};

pub fn render(frame: &mut Frame, state: &AppState, area: Rect, title: &str, description: &str) {
    let theme = &state.theme;

    let lines = vec![
        Line::raw(""),
        Line::from(vec![Span::styled(
            format!("  {} ", title),
            Style::default().fg(theme.fg).add_modifier(Modifier::BOLD),
        )]),
        Line::raw(""),
        Line::from(vec![Span::styled(
            format!("  {}", description),
            Style::default().fg(theme.fg_dim),
        )]),
        Line::raw(""),
        Line::from(vec![Span::styled(
            "  Coming soon — this tab is under development.",
            Style::default().fg(theme.fg_muted),
        )]),
    ];

    let paragraph = Paragraph::new(lines)
        .style(Style::default().bg(theme.bg))
        .block(
            Block::default()
                .borders(Borders::ALL)
                .border_style(theme.border_style())
                .title(Span::styled(
                    format!(" {} ", title),
                    Style::default().fg(theme.fg),
                )),
        );

    frame.render_widget(paragraph, area);
}
