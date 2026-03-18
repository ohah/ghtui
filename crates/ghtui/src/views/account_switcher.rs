use ghtui_core::AppState;
use ratatui::Frame;
use ratatui::layout::Rect;
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, Clear, Paragraph, Wrap};

pub fn render(frame: &mut Frame, state: &AppState, area: Rect) {
    let accounts = &state.accounts;

    let height = (accounts.len() as u16 + 4).min(area.height.saturating_sub(4));
    let width = 50.min(area.width.saturating_sub(4));
    let popup_area = super::components::centered_rect(width, height, area);

    frame.render_widget(Clear, popup_area);

    let mut lines = Vec::new();

    if accounts.is_empty() {
        lines.push(Line::styled(
            " No accounts found",
            Style::default().fg(Color::DarkGray),
        ));
        lines.push(Line::styled(
            " Run `gh auth login` to add accounts",
            Style::default().fg(Color::DarkGray),
        ));
    } else {
        for (i, account) in accounts.iter().enumerate() {
            let is_selected = i == state.account_selected;
            let is_current = state.current_account.as_ref().is_some_and(|c| c == account);

            let marker = if is_current { "● " } else { "  " };
            let cursor = if is_selected { "▸ " } else { "  " };

            let style = if is_selected {
                Style::default()
                    .fg(Color::White)
                    .add_modifier(Modifier::BOLD)
            } else if is_current {
                Style::default().fg(Color::Green)
            } else {
                Style::default().fg(Color::Gray)
            };

            lines.push(Line::from(vec![
                Span::styled(cursor.to_string(), style),
                Span::styled(
                    marker.to_string(),
                    Style::default().fg(if is_current {
                        Color::Green
                    } else {
                        Color::DarkGray
                    }),
                ),
                Span::styled(account.display_name(), style),
                Span::styled(
                    format!("  ({})", account.host),
                    Style::default().fg(Color::DarkGray),
                ),
            ]));
        }
    }

    let title = format!(" Switch Account ({}) ", accounts.len());
    let paragraph = Paragraph::new(lines)
        .block(
            Block::default()
                .title(title)
                .borders(Borders::ALL)
                .style(Style::default().bg(Color::Black)),
        )
        .wrap(Wrap { trim: false });

    frame.render_widget(paragraph, popup_area);
}
