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
            // Line 1: title + category badge + answered indicator
            let title_style = if i == disc_state.selected {
                Style::default()
                    .fg(theme.accent)
                    .add_modifier(Modifier::BOLD)
            } else {
                Style::default().fg(theme.fg)
            };

            let mut line1_spans = vec![
                Span::styled(format!("  {} ", d.title), title_style),
                Span::styled(
                    format!("[{}]", d.category),
                    Style::default().fg(theme.fg_dim),
                ),
            ];
            if d.is_answered {
                line1_spans.push(Span::styled(
                    " [Answered]".to_string(),
                    Style::default().fg(theme.success),
                ));
            }
            let line1 = Line::from(line1_spans);

            // Line 2: #number + author + comment count + relative time
            let time_str = super::components::time_ago_rfc3339(&d.created_at);

            let line2 = Line::from(vec![
                Span::styled(
                    format!("  #{} ", d.number),
                    Style::default().fg(theme.fg_muted),
                ),
                Span::styled(
                    format!("@{}", d.author),
                    Style::default().fg(theme.fg_muted),
                ),
                Span::styled(
                    format!("  {} comments", d.comments_count),
                    Style::default().fg(theme.fg_muted),
                ),
                Span::styled(
                    format!("  {}", time_str),
                    Style::default().fg(theme.fg_muted),
                ),
            ]);

            ListItem::new(vec![line1, line2])
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
