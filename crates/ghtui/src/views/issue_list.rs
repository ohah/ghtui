use ghtui_core::AppState;
use ratatui::Frame;
use ratatui::layout::Rect;
use ratatui::style::Style;
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, List, ListItem, ListState, Paragraph};

pub fn render(frame: &mut Frame, state: &AppState, area: Rect) {
    let theme = &state.theme;

    if state.is_loading("issue_list") {
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
                    .title(" Issues ")
                    .borders(Borders::ALL)
                    .border_style(theme.border_style()),
            );
        frame.render_widget(paragraph, area);
        return;
    }

    let Some(ref list_state) = state.issue_list else {
        let paragraph = Paragraph::new("No data")
            .style(theme.text_dim())
            .block(
                Block::default()
                    .title(" Issues ")
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
        .map(|(i, issue)| {
            let is_selected = i == list_state.selected;

            let state_icon = match issue.state {
                ghtui_core::types::IssueState::Open => Span::styled(
                    " ● ",
                    Style::default().fg(theme.success),
                ),
                ghtui_core::types::IssueState::Closed => Span::styled(
                    " ● ",
                    Style::default().fg(theme.done),
                ),
            };

            let title_style = if is_selected {
                theme.selected()
            } else {
                theme.text()
            };

            let labels: Vec<Span> = issue
                .labels
                .iter()
                .map(|l| {
                    Span::styled(
                        format!(" {} ", l.name),
                        Style::default().fg(theme.accent),
                    )
                })
                .collect();

            let mut spans = vec![
                state_icon,
                Span::styled(&issue.title, title_style),
                Span::styled(
                    format!(" #{}", issue.number),
                    Style::default().fg(theme.fg_muted),
                ),
            ];
            spans.extend(labels);

            ListItem::new(Line::from(spans))
        })
        .collect();

    let title = format!(" Issues ({}) ", list_state.items.len());

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
