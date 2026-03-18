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
            let visibility = if g.public { "Public" } else { "Secret" };
            let vis_color = if g.public {
                theme.success
            } else {
                theme.warning
            };
            let desc = g.description.as_deref().unwrap_or("(no description)");
            let line = Line::from(vec![
                Span::styled(format!("[{}] ", visibility), Style::default().fg(vis_color)),
                Span::styled(
                    desc.to_string(),
                    if i == gists_state.selected {
                        Style::default()
                            .fg(theme.accent)
                            .add_modifier(Modifier::BOLD)
                    } else {
                        Style::default().fg(theme.fg)
                    },
                ),
                Span::styled(
                    format!("  {} files", g.files_count),
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
                " Gists ",
                Style::default().fg(theme.fg).add_modifier(Modifier::BOLD),
            )),
    );

    frame.render_widget(list, area);
}
