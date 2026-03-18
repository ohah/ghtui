use ghtui_core::AppState;
use ghtui_core::types::SearchResultItem;
use ratatui::Frame;
use ratatui::layout::{Constraint, Direction, Layout, Rect};
use ratatui::style::{Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, List, ListItem, ListState, Paragraph};

pub fn render(frame: &mut Frame, state: &AppState, area: Rect) {
    let theme = &state.theme;

    let Some(ref search) = state.search else {
        let paragraph = Paragraph::new("  Press / or Ctrl+K to search")
            .style(theme.text_dim())
            .block(
                Block::default()
                    .title(" Search ")
                    .borders(Borders::ALL)
                    .border_style(theme.border_style()),
            );
        frame.render_widget(paragraph, area);
        return;
    };

    // Layout: search bar(3) + results
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Length(3), Constraint::Min(0)])
        .split(area);

    // Search bar
    render_search_bar(frame, search, theme, chunks[0]);

    // Results
    if state.is_loading("search") {
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
                    .title(" Results ")
                    .borders(Borders::ALL)
                    .border_style(theme.border_style()),
            );
        frame.render_widget(paragraph, chunks[1]);
        return;
    }

    match &search.results {
        Some(results) if !results.items.is_empty() => {
            let items: Vec<ListItem> = results
                .items
                .iter()
                .enumerate()
                .map(|(i, item)| render_result_item(item, i == search.selected, theme))
                .collect();

            let title = format!(
                " {} — {} results ",
                search.kind_display(),
                results.total_count
            );

            let list = List::new(items).block(
                Block::default()
                    .title(Span::styled(title, theme.text_bold()))
                    .borders(Borders::ALL)
                    .border_style(theme.border_style()),
            );

            let mut list_state = ListState::default();
            list_state.select(Some(search.selected));
            frame.render_stateful_widget(list, chunks[1], &mut list_state);
        }
        Some(results) if results.items.is_empty() => {
            let paragraph = Paragraph::new(format!(
                "  No {} found for '{}'",
                search.kind_display(),
                search.query
            ))
            .style(theme.text_dim())
            .block(
                Block::default()
                    .title(" Results ")
                    .borders(Borders::ALL)
                    .border_style(theme.border_style()),
            );
            frame.render_widget(paragraph, chunks[1]);
        }
        _ => {
            let hint = "  Enter a query and press Enter to search";
            let paragraph = Paragraph::new(hint).style(theme.text_dim()).block(
                Block::default()
                    .title(" Results ")
                    .borders(Borders::ALL)
                    .border_style(theme.border_style()),
            );
            frame.render_widget(paragraph, chunks[1]);
        }
    }
}

fn render_search_bar(
    frame: &mut Frame,
    search: &ghtui_core::state::SearchViewState,
    theme: &ghtui_core::Theme,
    area: Rect,
) {
    let kind_style = Style::default()
        .fg(theme.accent)
        .add_modifier(Modifier::BOLD);

    let mut spans = vec![
        Span::styled(format!(" {} ", search.kind_display()), kind_style),
        Span::styled(" │ ", Style::default().fg(theme.fg_dim)),
    ];

    if search.input_mode {
        spans.push(Span::styled("/", Style::default().fg(theme.accent)));
        spans.push(Span::styled(&search.input_query, theme.text()));
        spans.push(Span::styled("█", Style::default().fg(theme.accent)));
    } else if search.query.is_empty() {
        spans.push(Span::styled(
            "Press / to search...",
            Style::default().fg(theme.fg_muted),
        ));
    } else {
        spans.push(Span::styled(&search.query, theme.text()));
    }

    let history_hint = if search.input_mode && !search.history.is_empty() {
        "  [↑/↓]:History"
    } else {
        ""
    };
    spans.push(Span::styled(
        format!("  [Tab]:Kind{}", history_hint),
        Style::default().fg(theme.fg_dim),
    ));

    let search_line = Line::from(spans);
    let paragraph = Paragraph::new(search_line)
        .style(Style::default().bg(theme.bg))
        .block(
            Block::default()
                .title(" Search ")
                .borders(Borders::ALL)
                .border_style(if search.input_mode {
                    Style::default().fg(theme.accent)
                } else {
                    theme.border_style()
                }),
        );
    frame.render_widget(paragraph, area);
}

fn render_result_item<'a>(
    item: &'a SearchResultItem,
    is_selected: bool,
    theme: &ghtui_core::Theme,
) -> ListItem<'a> {
    let title_style = if is_selected {
        theme.selected()
    } else {
        theme.text()
    };

    let line = match item {
        SearchResultItem::Repo {
            full_name,
            description,
            stars,
            language,
        } => {
            let mut spans = vec![
                Span::styled("  ", Style::default()),
                Span::styled(full_name.as_str(), title_style),
                Span::styled(format!(" ★{}", stars), Style::default().fg(theme.warning)),
            ];
            if let Some(lang) = language {
                spans.push(Span::styled(
                    format!(" [{}]", lang),
                    Style::default().fg(theme.accent),
                ));
            }
            if let Some(desc) = description {
                let short = if desc.chars().count() > 60 {
                    let truncated: String = desc.chars().take(57).collect();
                    format!(" — {}...", truncated)
                } else {
                    format!(" — {}", desc)
                };
                spans.push(Span::styled(short, Style::default().fg(theme.fg_dim)));
            }
            Line::from(spans)
        }
        SearchResultItem::Issue {
            repo,
            number,
            title,
            state,
            is_pr,
        } => {
            let type_icon = if *is_pr { "PR" } else { "IS" };
            let state_color = if state == "open" {
                theme.success
            } else {
                theme.danger
            };
            Line::from(vec![
                Span::styled(
                    format!("  {} ", type_icon),
                    Style::default().fg(theme.success),
                ),
                Span::styled(title.as_str(), title_style),
                Span::styled(format!(" #{}", number), Style::default().fg(theme.fg_muted)),
                Span::styled(format!(" {}", state), Style::default().fg(state_color)),
                Span::styled(format!(" {}", repo), Style::default().fg(theme.fg_dim)),
            ])
        }
        SearchResultItem::Code {
            repo,
            path,
            fragment,
        } => {
            let short_fragment = if fragment.chars().count() > 80 {
                let truncated: String = fragment.chars().take(77).collect();
                format!("{}...", truncated)
            } else {
                fragment.clone()
            };
            Line::from(vec![
                Span::styled("  ", Style::default()),
                Span::styled(path.as_str(), title_style),
                Span::styled(format!(" ({})", repo), Style::default().fg(theme.fg_dim)),
                Span::raw("  "),
                Span::styled(short_fragment, Style::default().fg(theme.fg_muted)),
            ])
        }
    };

    ListItem::new(line)
}
