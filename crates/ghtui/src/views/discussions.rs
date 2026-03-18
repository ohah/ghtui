use ghtui_core::AppState;
use ratatui::Frame;
use ratatui::layout::Rect;
use ratatui::style::{Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, List, ListItem, Paragraph};

pub fn render(frame: &mut Frame, state: &AppState, area: Rect) {
    let theme = &state.theme;

    if state.is_loading("discussions") {
        let loading =
            Paragraph::new("  Loading discussions...").style(Style::default().fg(theme.fg_dim));
        frame.render_widget(loading, area);
        return;
    }

    let Some(ref disc_state) = state.discussions else {
        let empty = Paragraph::new("  No discussions loaded. Navigate to a repo first.")
            .style(Style::default().fg(theme.fg_dim));
        frame.render_widget(empty, area);
        return;
    };

    if disc_state.items.is_empty() {
        let empty =
            Paragraph::new("  No discussions found.").style(Style::default().fg(theme.fg_dim));
        frame.render_widget(empty, area);
        return;
    }

    let items: Vec<ListItem> = disc_state
        .items
        .iter()
        .enumerate()
        .map(|(i, d)| {
            let answered = if d.is_answered { " [Answered]" } else { "" };
            let line = Line::from(vec![
                Span::styled(
                    format!("#{:<5} ", d.number),
                    Style::default().fg(theme.fg_muted),
                ),
                Span::styled(
                    d.title.clone(),
                    if i == disc_state.selected {
                        Style::default()
                            .fg(theme.accent)
                            .add_modifier(Modifier::BOLD)
                    } else {
                        Style::default().fg(theme.fg)
                    },
                ),
                Span::styled(
                    format!("  [{}]", d.category),
                    Style::default().fg(theme.fg_dim),
                ),
                Span::styled(answered.to_string(), Style::default().fg(theme.success)),
                Span::styled(
                    format!("  {} comments  @{}", d.comments_count, d.author),
                    Style::default().fg(theme.fg_muted),
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
                " Discussions ",
                Style::default().fg(theme.fg).add_modifier(Modifier::BOLD),
            )),
    );

    frame.render_widget(list, area);
}
