use ghtui_core::theme::Theme;
use ratatui::Frame;
use ratatui::layout::Rect;
use ratatui::style::{Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, List, ListItem, ListState, Paragraph};

pub fn render_sidebar(
    frame: &mut Frame,
    theme: &Theme,
    title: &str,
    items: &[String],
    selected: usize,
    focused: bool,
    area: Rect,
) {
    // Build ListItems with selection highlighting
    let list_items: Vec<ListItem> = items
        .iter()
        .enumerate()
        .map(|(i, item_title)| {
            let style = if i == selected {
                if focused {
                    Style::default()
                        .fg(theme.tab_active_fg)
                        .add_modifier(Modifier::BOLD)
                        .bg(theme.selection_bg)
                } else {
                    Style::default()
                        .fg(theme.tab_active_fg)
                        .add_modifier(Modifier::BOLD)
                }
            } else {
                Style::default().fg(theme.fg_muted)
            };
            ListItem::new(Line::from(Span::styled(
                format!("  {} ", item_title),
                style,
            )))
        })
        .collect();

    let border_style = if focused {
        Style::default().fg(theme.accent)
    } else {
        theme.border_style()
    };

    let list = List::new(list_items).block(
        Block::default()
            .title(format!(" {} ", title))
            .borders(Borders::ALL)
            .border_style(border_style),
    );

    let mut state = ListState::default();
    state.select(Some(selected));
    frame.render_stateful_widget(list, area, &mut state);
}

pub fn render_loading(frame: &mut Frame, theme: &Theme, area: Rect, title: &str) {
    let spinner = ghtui_widgets::Spinner::new(
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_millis() as usize
            / 100,
    );
    let paragraph = Paragraph::new(Line::from(spinner.span()))
        .style(theme.text())
        .block(
            Block::default()
                .title(format!(" {} ", title))
                .borders(Borders::ALL)
                .border_style(theme.border_style()),
        );
    frame.render_widget(paragraph, area);
}

#[allow(dead_code)]
pub fn centered_rect(width: u16, height: u16, area: Rect) -> Rect {
    let x = (area.width.saturating_sub(width)) / 2 + area.x;
    let y = (area.height.saturating_sub(height)) / 2 + area.y;
    Rect::new(x, y, width.min(area.width), height.min(area.height))
}
