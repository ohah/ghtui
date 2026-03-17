use ratatui::Frame;
use ratatui::layout::{Alignment, Rect};
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, Clear, Paragraph, Wrap};

pub fn render(frame: &mut Frame, area: Rect) {
    // Center the help popup
    let width = 60.min(area.width.saturating_sub(4));
    let height = 24.min(area.height.saturating_sub(4));
    let x = (area.width.saturating_sub(width)) / 2 + area.x;
    let y = (area.height.saturating_sub(height)) / 2 + area.y;
    let popup_area = Rect::new(x, y, width, height);

    frame.render_widget(Clear, popup_area);

    let help_lines = vec![
        Line::styled("Keybindings", Style::default().add_modifier(Modifier::BOLD)),
        Line::raw(""),
        section("Global"),
        key_line("q / Ctrl-C", "Quit"),
        key_line("?", "Toggle help"),
        key_line("S", "Switch account"),
        key_line("Esc", "Go back"),
        key_line("1/p", "Pull Requests"),
        key_line("2/i", "Issues"),
        key_line("3/a", "Actions"),
        key_line("4/n", "Notifications"),
        key_line("5/s", "Search"),
        Line::raw(""),
        section("List Views"),
        key_line("j/k / Up/Down", "Navigate"),
        key_line("Enter", "Open selected"),
        key_line("r", "Refresh"),
        Line::raw(""),
        section("PR Detail"),
        key_line("Tab / Shift-Tab", "Switch tabs"),
        key_line("c", "Add comment"),
        key_line("m", "Merge PR"),
        Line::raw(""),
        section("Issue Detail"),
        key_line("c", "Add comment"),
    ];

    let paragraph = Paragraph::new(help_lines)
        .block(
            Block::default()
                .title(" Help ")
                .borders(Borders::ALL)
                .style(Style::default().bg(Color::Black)),
        )
        .alignment(Alignment::Left)
        .wrap(Wrap { trim: false });

    frame.render_widget(paragraph, popup_area);
}

fn section(title: &str) -> Line<'static> {
    Line::styled(
        title.to_string(),
        Style::default()
            .fg(Color::Cyan)
            .add_modifier(Modifier::BOLD),
    )
}

fn key_line(key: &str, desc: &str) -> Line<'static> {
    Line::from(vec![
        Span::styled(format!("  {:<20}", key), Style::default().fg(Color::Yellow)),
        Span::raw(desc.to_string()),
    ])
}
