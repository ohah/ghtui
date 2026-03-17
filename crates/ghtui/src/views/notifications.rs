use ghtui_core::AppState;
use ratatui::Frame;
use ratatui::layout::Rect;
use ratatui::style::Style;
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, List, ListItem, ListState, Paragraph};

pub fn render(frame: &mut Frame, state: &AppState, area: Rect) {
    let theme = &state.theme;

    if state.is_loading("notifications") {
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
                    .title(" Notifications ")
                    .borders(Borders::ALL)
                    .border_style(theme.border_style()),
            );
        frame.render_widget(paragraph, area);
        return;
    }

    let Some(ref notif_state) = state.notifications else {
        let paragraph = Paragraph::new("No data").style(theme.text_dim()).block(
            Block::default()
                .title(" Notifications ")
                .borders(Borders::ALL)
                .border_style(theme.border_style()),
        );
        frame.render_widget(paragraph, area);
        return;
    };

    if notif_state.items.is_empty() {
        let paragraph = Paragraph::new("  No notifications — all caught up!")
            .style(theme.text_dim())
            .block(
                Block::default()
                    .title(" Notifications ")
                    .borders(Borders::ALL)
                    .border_style(theme.border_style()),
            );
        frame.render_widget(paragraph, area);
        return;
    }

    let items: Vec<ListItem> = notif_state
        .items
        .iter()
        .enumerate()
        .map(|(i, notif)| {
            let is_selected = i == notif_state.selected;

            let unread = if notif.unread {
                Span::styled(" ● ", Style::default().fg(theme.accent))
            } else {
                Span::raw("   ")
            };

            let type_icon = match notif.subject.subject_type.as_str() {
                "PullRequest" => Span::styled("PR ", Style::default().fg(theme.success)),
                "Issue" => Span::styled("IS ", Style::default().fg(theme.success)),
                "Release" => Span::styled("RE ", Style::default().fg(theme.warning)),
                _ => Span::styled("   ", Style::default().fg(theme.fg_muted)),
            };

            let title_style = if is_selected {
                theme.selected()
            } else {
                theme.text()
            };

            let line = Line::from(vec![
                unread,
                type_icon,
                Span::styled(&notif.subject.title, title_style),
                Span::raw(" "),
                Span::styled(
                    &notif.repository.full_name,
                    Style::default().fg(theme.fg_dim),
                ),
            ]);

            ListItem::new(line)
        })
        .collect();

    let title = format!(" Notifications ({}) ", notif_state.items.len());

    let list = List::new(items)
        .block(
            Block::default()
                .title(Span::styled(title, theme.text_bold()))
                .borders(Borders::ALL)
                .border_style(theme.border_style()),
        )
        .highlight_style(theme.selected());

    let mut list_widget_state = ListState::default();
    list_widget_state.select(Some(notif_state.selected));
    frame.render_stateful_widget(list, area, &mut list_widget_state);
}
