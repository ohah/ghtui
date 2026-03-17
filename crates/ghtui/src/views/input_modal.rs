use ghtui_core::AppState;
use ratatui::Frame;
use ratatui::layout::{Constraint, Direction, Layout, Rect};
use ratatui::style::{Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, Paragraph, Wrap};

pub fn render(frame: &mut Frame, state: &AppState, area: Rect, title: &str, hint: &str) {
    let theme = &state.theme;

    // Full-width layout: hint bar + editor + status bar
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(2), // hint
            Constraint::Min(0),    // editor content
            Constraint::Length(1), // status bar
        ])
        .split(area);

    // Hint bar
    let hint_line = Line::from(vec![
        Span::styled(
            format!(" {} ", title),
            Style::default()
                .fg(theme.bg)
                .bg(theme.accent)
                .add_modifier(Modifier::BOLD),
        ),
        Span::styled(format!("  {}", hint), Style::default().fg(theme.fg_dim)),
    ]);
    let hint_bar = Paragraph::new(hint_line).style(Style::default().bg(theme.bg_subtle));
    frame.render_widget(hint_bar, chunks[0]);

    // Editor content
    let input = &state.input_buffer;
    let input_lines: Vec<&str> = input.split('\n').collect();
    let mut lines: Vec<Line> = Vec::new();

    for (i, line_text) in input_lines.iter().enumerate() {
        let line_num = format!(" {:>3} │ ", i + 1);
        lines.push(Line::from(vec![
            Span::styled(line_num, Style::default().fg(theme.fg_muted)),
            Span::styled(line_text.to_string(), Style::default().fg(theme.fg)),
        ]));
    }

    // Cursor line
    let cursor_line_num = format!(" {:>3} │ ", input_lines.len() + 1);
    lines.push(Line::from(vec![
        Span::styled(cursor_line_num, Style::default().fg(theme.fg_muted)),
        Span::styled(
            "█",
            Style::default()
                .fg(theme.accent)
                .add_modifier(Modifier::SLOW_BLINK),
        ),
    ]));

    let editor = Paragraph::new(lines)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .border_style(Style::default().fg(theme.border))
                .style(Style::default().bg(theme.bg)),
        )
        .wrap(Wrap { trim: false });
    frame.render_widget(editor, chunks[1]);

    // Status bar
    let status = Line::from(vec![
        Span::styled(
            " Ctrl+S ",
            Style::default()
                .fg(theme.bg)
                .bg(theme.success)
                .add_modifier(Modifier::BOLD),
        ),
        Span::styled(" Submit  ", Style::default().fg(theme.fg_dim)),
        Span::styled(
            " Esc ",
            Style::default()
                .fg(theme.bg)
                .bg(theme.danger)
                .add_modifier(Modifier::BOLD),
        ),
        Span::styled(" Cancel  ", Style::default().fg(theme.fg_dim)),
        Span::styled(" Enter ", Style::default().fg(theme.fg).bg(theme.bg_subtle)),
        Span::styled(" New line", Style::default().fg(theme.fg_dim)),
    ]);
    let status_bar = Paragraph::new(status).style(Style::default().bg(theme.bg_subtle));
    frame.render_widget(status_bar, chunks[2]);
}
