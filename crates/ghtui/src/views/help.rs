use ratatui::Frame;
use ratatui::layout::{Alignment, Rect};
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, Clear, Paragraph, Wrap};

pub fn render(frame: &mut Frame, area: Rect) {
    // Center the help popup
    let width = 60.min(area.width.saturating_sub(4));
    let height = 50.min(area.height.saturating_sub(4));
    let popup_area = super::components::centered_rect(width, height, area);

    frame.render_widget(Clear, popup_area);

    let help_lines = vec![
        Line::styled("Keybindings", Style::default().add_modifier(Modifier::BOLD)),
        Line::raw(""),
        section("Global"),
        key_line("q", "Quit"),
        key_line("H", "Home (Dashboard)"),
        key_line("?", "Toggle help"),
        key_line("t", "Toggle theme (Dark/Light)"),
        key_line("Ctrl+P", "Command palette"),
        key_line("Ctrl+K", "Global search"),
        key_line("S", "Switch account"),
        key_line("1-7", "Switch tabs"),
        key_line("Esc", "Go back / cancel"),
        Line::raw(""),
        section("Navigation"),
        key_line("j / k", "Move down / up"),
        key_line("Enter", "Open / select"),
        key_line("n / p", "Next / previous page"),
        key_line("s", "Toggle state filter"),
        key_line("o", "Sort cycle / open in browser"),
        key_line("r", "Refresh"),
        key_line("F", "Clear filters"),
        Line::raw(""),
        section("Editing"),
        key_line("e", "Edit focused section"),
        key_line("c", "New comment"),
        key_line("r", "Reply to comment"),
        key_line("d", "Delete"),
        key_line("l", "Edit labels"),
        key_line("a", "Edit assignees"),
        key_line("m / M", "Set milestone"),
        key_line("Ctrl+S", "Submit editor"),
        key_line("Ctrl+Z / Ctrl+Y", "Undo / Redo"),
        Line::raw(""),
        section("PR Detail"),
        key_line("Tab", "Switch sub-tabs"),
        key_line("A", "Approve"),
        key_line("R", "Request changes"),
        key_line("D", "Toggle draft"),
        key_line("G", "Toggle auto-merge"),
        key_line("v", "Add reviewer"),
        key_line("b", "Change base branch"),
        key_line("x", "Close / reopen"),
        Line::raw(""),
        section("Files Changed (Diff)"),
        key_line("J / K", "Block select"),
        key_line("Enter", "Fold / open comment editor"),
        key_line("e", "Expand context"),
        key_line("s", "Toggle side-by-side"),
        key_line("f", "Toggle file tree"),
        key_line("V", "Mark file as viewed"),
        key_line("z", "Resolve/unresolve thread"),
        key_line("Ctrl+G", "Insert suggestion"),
        Line::raw(""),
        section("Markdown Links"),
        key_line("o", "Open link (Body/Comment)"),
        key_line("", "1 link: opens directly"),
        key_line("", "2+ links: URL picker modal"),
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
