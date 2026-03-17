use ghtui_core::AppState;
use ratatui::Frame;
use ratatui::layout::Rect;
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::Line;
use ratatui::widgets::{Block, Borders, Clear, Paragraph, Wrap};

pub fn render(frame: &mut Frame, state: &AppState, area: Rect, title: &str, hint: &str) {
    let width = 60.min(area.width.saturating_sub(4));
    let height = 15.min(area.height.saturating_sub(4));
    let x = (area.width.saturating_sub(width)) / 2 + area.x;
    let y = (area.height.saturating_sub(height)) / 2 + area.y;
    let popup_area = Rect::new(x, y, width, height);

    frame.render_widget(Clear, popup_area);

    let mut lines = Vec::new();

    // Hint
    lines.push(Line::styled(
        hint.to_string(),
        Style::default().fg(Color::DarkGray),
    ));
    lines.push(Line::raw(""));

    // Input content
    let input = &state.input_buffer;
    for line in input.split('\n') {
        lines.push(Line::raw(line.to_string()));
    }

    // Cursor
    lines.push(Line::styled(
        "█".to_string(),
        Style::default()
            .fg(Color::White)
            .add_modifier(Modifier::SLOW_BLINK),
    ));

    lines.push(Line::raw(""));
    lines.push(Line::styled(
        " Ctrl+Enter: Submit  |  Esc: Cancel ".to_string(),
        Style::default().fg(Color::DarkGray),
    ));

    let paragraph = Paragraph::new(lines)
        .block(
            Block::default()
                .title(format!(" {} ", title))
                .borders(Borders::ALL)
                .style(Style::default().bg(Color::Black)),
        )
        .wrap(Wrap { trim: false });

    frame.render_widget(paragraph, popup_area);
}
