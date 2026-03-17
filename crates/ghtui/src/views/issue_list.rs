use ghtui_core::AppState;
use ratatui::Frame;
use ratatui::layout::{Constraint, Direction, Layout, Rect};
use ratatui::style::{Modifier, Style};
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
        let paragraph = Paragraph::new("No data").style(theme.text_dim()).block(
            Block::default()
                .title(" Issues ")
                .borders(Borders::ALL)
                .border_style(theme.border_style()),
        );
        frame.render_widget(paragraph, area);
        return;
    };

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(1),
            Constraint::Min(0),
            Constraint::Length(1),
        ])
        .split(area);

    // Filter bar
    let filter_state = list_state
        .filters
        .state
        .map(|s| format!("{}", s))
        .unwrap_or_else(|| "open".to_string());
    let is_open = filter_state == "open";

    let filter_line = Line::from(vec![
        Span::styled(" Filter: ", Style::default().fg(theme.fg_muted)),
        Span::styled(
            if is_open { " ● Open " } else { "   Open " },
            if is_open {
                Style::default()
                    .fg(theme.success)
                    .add_modifier(Modifier::BOLD)
            } else {
                Style::default().fg(theme.fg_muted)
            },
        ),
        Span::styled(" / ", Style::default().fg(theme.fg_dim)),
        Span::styled(
            if !is_open {
                " ● Closed "
            } else {
                "   Closed "
            },
            if !is_open {
                Style::default().fg(theme.done).add_modifier(Modifier::BOLD)
            } else {
                Style::default().fg(theme.fg_muted)
            },
        ),
        Span::styled("  (s:toggle  /:search)", Style::default().fg(theme.fg_dim)),
    ]);

    // If in search mode, show search bar instead of filter
    if list_state.search_mode {
        let search_line = Line::from(vec![
            Span::styled(" 🔍 ", Style::default().fg(theme.accent)),
            Span::styled(
                list_state.search_query.clone(),
                Style::default()
                    .fg(ratatui::style::Color::White)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::styled(
                "█",
                Style::default()
                    .fg(theme.accent)
                    .add_modifier(Modifier::SLOW_BLINK),
            ),
            Span::styled(
                "  Enter:Search  Esc:Cancel",
                Style::default().fg(theme.fg_dim),
            ),
        ]);
        let search_bar = Paragraph::new(search_line).style(Style::default().bg(theme.bg_subtle));
        frame.render_widget(search_bar, chunks[0]);
    } else {
        let filter_bar = Paragraph::new(filter_line).style(Style::default().bg(theme.bg_subtle));
        frame.render_widget(filter_bar, chunks[0]);
    }

    // Issue list
    let items: Vec<ListItem> = list_state
        .items
        .iter()
        .enumerate()
        .map(|(i, issue)| {
            let is_selected = i == list_state.selected;

            let state_icon = match issue.state {
                ghtui_core::types::IssueState::Open => {
                    Span::styled(" ● ", Style::default().fg(theme.success))
                }
                ghtui_core::types::IssueState::Closed => {
                    Span::styled(" ● ", Style::default().fg(theme.done))
                }
            };

            let title_style = if is_selected {
                theme.selected()
            } else {
                theme.text()
            };

            let mut spans = vec![
                state_icon,
                Span::styled(issue.title.clone(), title_style),
                Span::styled(
                    format!(" #{}", issue.number),
                    Style::default().fg(theme.fg_muted),
                ),
            ];

            // Labels
            for label in &issue.labels {
                spans.push(Span::styled(
                    format!(" {} ", label.name),
                    Style::default().fg(theme.accent),
                ));
            }

            // Comment count
            if let Some(count) = issue.comments {
                if count > 0 {
                    spans.push(Span::styled(
                        format!("  💬{}", count),
                        Style::default().fg(theme.fg_dim),
                    ));
                }
            }

            // Assignees
            if !issue.assignees.is_empty() {
                let assignees: Vec<String> =
                    issue.assignees.iter().map(|a| a.login.clone()).collect();
                spans.push(Span::styled(
                    format!("  → {}", assignees.join(", ")),
                    Style::default().fg(theme.fg_dim),
                ));
            }

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
    frame.render_stateful_widget(list, chunks[1], &mut list_widget_state);

    // Pagination footer
    let page = list_state.pagination.page;
    let has_next = list_state.pagination.has_next;
    let page_info = format!(" Page {} ", page);
    let nav_hint = match (page > 1, has_next) {
        (true, true) => "p:Prev n:Next",
        (true, false) => "p:Prev",
        (false, true) => "n:Next",
        (false, false) => "",
    };

    let footer_line = Line::from(vec![
        Span::styled(page_info, Style::default().fg(theme.fg_muted)),
        Span::styled(format!(" {} ", nav_hint), Style::default().fg(theme.fg_dim)),
        Span::styled(
            " c:Create  s:Filter  Enter:Open ",
            Style::default().fg(theme.fg_dim),
        ),
    ]);
    let footer = Paragraph::new(footer_line).style(Style::default().bg(theme.bg_subtle));
    frame.render_widget(footer, chunks[2]);
}
