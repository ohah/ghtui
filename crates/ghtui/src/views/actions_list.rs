use ghtui_core::AppState;
use ratatui::Frame;
use ratatui::layout::Rect;
use ratatui::style::Style;
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, List, ListItem, ListState, Paragraph};

pub fn render(frame: &mut Frame, state: &AppState, area: Rect) {
    let theme = &state.theme;

    if state.is_loading("actions_list") {
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
                    .title(" Actions ")
                    .borders(Borders::ALL)
                    .border_style(theme.border_style()),
            );
        frame.render_widget(paragraph, area);
        return;
    }

    let Some(ref list_state) = state.actions_list else {
        let paragraph = Paragraph::new("No data")
            .style(theme.text_dim())
            .block(
                Block::default()
                    .title(" Actions ")
                    .borders(Borders::ALL)
                    .border_style(theme.border_style()),
            );
        frame.render_widget(paragraph, area);
        return;
    };

    let items: Vec<ListItem> = list_state
        .items
        .iter()
        .enumerate()
        .map(|(i, run)| {
            let is_selected = i == list_state.selected;

            let status_icon = match (&run.status, &run.conclusion) {
                (_, Some(ghtui_core::types::RunConclusion::Success)) => {
                    Span::styled(" ✓ ", Style::default().fg(theme.success))
                }
                (_, Some(ghtui_core::types::RunConclusion::Failure)) => {
                    Span::styled(" ✗ ", Style::default().fg(theme.danger))
                }
                (_, Some(ghtui_core::types::RunConclusion::Cancelled)) => {
                    Span::styled(" ◌ ", Style::default().fg(theme.fg_muted))
                }
                (Some(ghtui_core::types::RunStatus::InProgress), _) => {
                    Span::styled(" ● ", Style::default().fg(theme.warning))
                }
                (Some(ghtui_core::types::RunStatus::Queued), _) => {
                    Span::styled(" ○ ", Style::default().fg(theme.warning))
                }
                _ => Span::styled(" · ", Style::default().fg(theme.fg_muted)),
            };

            let name = run.name.as_deref().unwrap_or("Unknown");
            let branch = run.head_branch.as_deref().unwrap_or("");

            let title_style = if is_selected {
                theme.selected()
            } else {
                theme.text()
            };

            let line = Line::from(vec![
                status_icon,
                Span::styled(name, title_style),
                Span::styled(
                    format!(" #{}", run.run_number),
                    Style::default().fg(theme.fg_muted),
                ),
                Span::raw(" "),
                Span::styled(
                    format!("({})", branch),
                    Style::default().fg(theme.accent),
                ),
                Span::styled(
                    format!(" {}", run.event),
                    Style::default().fg(theme.fg_dim),
                ),
            ]);

            ListItem::new(line)
        })
        .collect();

    let title = format!(" Actions ({}) ", list_state.items.len());

    let list = List::new(items)
        .block(
            Block::default()
                .title(Span::styled(title, theme.text_bold()))
                .borders(Borders::ALL)
                .border_style(theme.border_style()),
        )
        .highlight_style(theme.selected());

    let mut list_widget_state = ListState::default();
    list_widget_state.select(Some(list_state.selected));
    frame.render_stateful_widget(list, area, &mut list_widget_state);
}
