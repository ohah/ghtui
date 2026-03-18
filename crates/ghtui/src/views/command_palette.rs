use ghtui_core::AppState;
use ratatui::Frame;
use ratatui::layout::Rect;
use ratatui::style::{Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, Clear, Paragraph};

pub fn render(frame: &mut Frame, state: &AppState, area: Rect) {
    let Some(ref palette) = state.command_palette else {
        return;
    };

    let width = 60u16.min(area.width.saturating_sub(4));
    // query line + blank + items + border (2)
    let content_height = 2 + palette.filtered.len().max(1) + 2;
    let height = (content_height as u16).min(area.height.saturating_sub(4));
    let popup_area = super::components::centered_rect(width, height, area);

    frame.render_widget(Clear, popup_area);

    let theme = &state.theme;

    let mut lines: Vec<Line<'static>> = Vec::new();

    // Query input line
    lines.push(Line::from(vec![
        Span::styled(
            " > ",
            Style::default()
                .fg(theme.accent)
                .add_modifier(Modifier::BOLD),
        ),
        Span::styled(palette.query.clone(), Style::default().fg(theme.fg)),
        Span::styled("_", Style::default().fg(theme.accent)),
    ]));
    lines.push(Line::raw(""));

    // Filtered items
    for (display_idx, &item_idx) in palette.filtered.iter().enumerate() {
        let item = &palette.items[item_idx];
        let is_selected = display_idx == palette.selected;

        let style = if is_selected {
            Style::default()
                .fg(theme.tab_active_fg)
                .bg(theme.selection_bg)
                .add_modifier(Modifier::BOLD)
        } else {
            Style::default().fg(theme.fg)
        };

        let cat_style = if is_selected {
            style
        } else {
            Style::default().fg(theme.fg_muted)
        };

        let prefix = if is_selected { ">" } else { " " };

        lines.push(Line::from(vec![
            Span::styled(format!("{} ", prefix), style),
            Span::styled(format!("{:<12}", item.category), cat_style),
            Span::styled(item.label.clone(), style),
        ]));
    }

    if palette.filtered.is_empty() {
        lines.push(Line::styled(
            "  No matching commands".to_string(),
            Style::default().fg(theme.fg_dim),
        ));
    }

    let paragraph = Paragraph::new(lines).block(
        Block::default()
            .title(" Command Palette (Ctrl+P) ")
            .borders(Borders::ALL)
            .border_style(Style::default().fg(theme.accent))
            .style(Style::default().bg(theme.bg)),
    );

    frame.render_widget(paragraph, popup_area);
}
