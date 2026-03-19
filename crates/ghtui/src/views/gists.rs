use ghtui_core::AppState;
use ratatui::Frame;
use ratatui::layout::Rect;
use ratatui::style::{Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, List, ListItem, Paragraph};

pub fn render(frame: &mut Frame, state: &AppState, area: Rect) {
    let theme = &state.theme;

    if state.is_loading("gists") {
        let loading = Paragraph::new("  Loading gists...").style(Style::default().fg(theme.fg_dim));
        frame.render_widget(loading, area);
        return;
    }

    let Some(ref gists_state) = state.gists else {
        let empty = Paragraph::new("  No gists loaded.").style(Style::default().fg(theme.fg_dim));
        frame.render_widget(empty, area);
        return;
    };

    if gists_state.items.is_empty() {
        let empty = Paragraph::new("  No gists found.").style(Style::default().fg(theme.fg_dim));
        frame.render_widget(empty, area);
        return;
    }

    let items: Vec<ListItem> = gists_state
        .items
        .iter()
        .enumerate()
        .map(|(i, g)| {
            // Line 1: visibility badge + description
            let visibility = if g.public { "Public" } else { "Secret" };
            let vis_color = if g.public {
                theme.success
            } else {
                theme.warning
            };
            let desc = g.description.as_deref().unwrap_or("(no description)");
            let desc_style = if i == gists_state.selected {
                Style::default()
                    .fg(theme.accent)
                    .add_modifier(Modifier::BOLD)
            } else {
                Style::default().fg(theme.fg)
            };

            let line1 = Line::from(vec![
                Span::styled(
                    format!("  [{}] ", visibility),
                    Style::default().fg(vis_color),
                ),
                Span::styled(desc.to_string(), desc_style),
            ]);

            // Line 2: file count + created relative time
            let time_str = format!(
                "created {}",
                super::components::time_ago_rfc3339(&g.created_at)
            );

            let line2 = Line::from(vec![
                Span::styled(
                    format!(
                        "  {} file{}",
                        g.files_count,
                        if g.files_count == 1 { "" } else { "s" }
                    ),
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
                " Gists ",
                Style::default().fg(theme.fg).add_modifier(Modifier::BOLD),
            )),
    );

    frame.render_widget(list, area);
}
