use ghtui_core::AppState;
use ratatui::Frame;
use ratatui::layout::{Constraint, Direction, Layout, Rect};
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, List, ListItem, ListState, Paragraph};

pub fn render(frame: &mut Frame, state: &AppState, area: Rect) {
    let theme = &state.theme;

    if state.is_loading("pr_list") {
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
                    .title(" Pull Requests ")
                    .borders(Borders::ALL)
                    .border_style(theme.border_style()),
            );
        frame.render_widget(paragraph, area);
        return;
    }

    let Some(ref list_state) = state.pr_list else {
        let paragraph = Paragraph::new("  No data — check API rate limit or network connection")
            .style(theme.text_dim())
            .block(
                Block::default()
                    .title(" Pull Requests ")
                    .borders(Borders::ALL)
                    .border_style(theme.border_style()),
            );
        frame.render_widget(paragraph, area);
        return;
    };

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(1), // Filter bar
            Constraint::Min(0),    // PR list
            Constraint::Length(1), // Footer
        ])
        .split(area);

    // === Filter bar ===
    render_filter_bar(frame, list_state, theme, chunks[0]);

    // === PR list ===
    render_pr_list(frame, list_state, theme, chunks[1]);

    // === Footer ===
    render_footer(frame, list_state, theme, chunks[2]);
}

fn render_filter_bar(
    frame: &mut Frame,
    list_state: &ghtui_core::state::PrListState,
    theme: &ghtui_core::theme::Theme,
    area: Rect,
) {
    if list_state.search_mode {
        let search_line = Line::from(vec![
            Span::styled(" / ", Style::default().fg(theme.accent)),
            Span::styled(
                list_state.search_query.clone(),
                Style::default()
                    .fg(Color::White)
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
        let bar = Paragraph::new(search_line).style(Style::default().bg(theme.bg_subtle));
        frame.render_widget(bar, area);
    } else {
        let filter_state = list_state
            .filters
            .state
            .map(|s| format!("{}", s))
            .unwrap_or_else(|| "open".to_string());
        let is_open = filter_state == "open";

        let mut line = Line::from(vec![
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
            Span::styled("  │ ", Style::default().fg(theme.fg_dim)),
            Span::styled(
                format!("Sort: {}", list_state.sort_display()),
                Style::default().fg(theme.fg_muted),
            ),
            Span::styled(
                "  (s:state  o:sort  /:search  F:clear)",
                Style::default().fg(theme.fg_dim),
            ),
        ]);

        // Show active filters
        let filters = &list_state.filters;
        let mut filter_parts: Vec<Span> = Vec::new();
        if let Some(ref label) = filters.label {
            filter_parts.push(Span::styled(
                format!("  label:{}", label),
                Style::default().fg(theme.accent),
            ));
        }
        if let Some(ref author) = filters.author {
            filter_parts.push(Span::styled(
                format!("  author:{}", author),
                Style::default().fg(theme.accent),
            ));
        }
        if let Some(ref assignee) = filters.assignee {
            filter_parts.push(Span::styled(
                format!("  assignee:{}", assignee),
                Style::default().fg(theme.accent),
            ));
        }
        if !filter_parts.is_empty() {
            let mut new_spans = line.spans.clone();
            new_spans.extend(filter_parts);
            line = Line::from(new_spans);
        }
        let bar = Paragraph::new(line).style(Style::default().bg(theme.bg_subtle));
        frame.render_widget(bar, area);
    }
}

fn render_pr_list(
    frame: &mut Frame,
    list_state: &ghtui_core::state::PrListState,
    theme: &ghtui_core::theme::Theme,
    area: Rect,
) {
    let items: Vec<ListItem> = list_state
        .items
        .iter()
        .enumerate()
        .map(|(i, pr)| {
            let is_selected = i == list_state.selected;

            let state_icon = match pr.state {
                ghtui_core::types::PrState::Open => {
                    Span::styled("● ", Style::default().fg(theme.success))
                }
                ghtui_core::types::PrState::Closed => {
                    Span::styled("● ", Style::default().fg(theme.danger))
                }
                ghtui_core::types::PrState::Merged => {
                    Span::styled("● ", Style::default().fg(theme.done))
                }
            };

            let title_style = if is_selected {
                theme.selected()
            } else {
                theme.text()
            };

            // Line 1: state icon + title + draft badge
            let mut line1_spans = vec![
                Span::raw("  "),
                state_icon,
                Span::styled(pr.title.clone(), title_style),
            ];

            if pr.draft {
                line1_spans.push(Span::styled(" Draft", Style::default().fg(theme.fg_muted)));
            }

            let mut lines = vec![Line::from(line1_spans)];
            if let Some(ll) = super::components::label_line(&pr.labels) {
                lines.push(ll);
            }

            // Line 2/3: #number · opened time ago by user · +/- · assignees · comments
            let mut meta_spans: Vec<Span> = vec![
                Span::raw("    "),
                Span::styled(
                    format!("#{}", pr.number),
                    Style::default().fg(theme.fg_muted),
                ),
                Span::styled(
                    format!(
                        " opened {} by {}",
                        super::components::time_ago(&pr.created_at),
                        pr.user.login,
                    ),
                    Style::default().fg(theme.fg_dim),
                ),
            ];

            // Show +/- stats if available
            if let (Some(additions), Some(deletions)) = (pr.additions, pr.deletions)
                && (additions > 0 || deletions > 0)
            {
                meta_spans.push(Span::styled(
                    format!("  +{}", additions),
                    Style::default().fg(theme.success),
                ));
                meta_spans.push(Span::styled(
                    format!(" -{}", deletions),
                    Style::default().fg(theme.danger),
                ));
            }

            if !pr.assignees.is_empty() {
                let assignees: Vec<String> = pr.assignees.iter().map(|a| a.login.clone()).collect();
                meta_spans.push(Span::styled(
                    format!("  → {}", assignees.join(", ")),
                    Style::default().fg(theme.fg_dim),
                ));
            }

            if let Some(count) = pr.comments
                && count > 0
            {
                meta_spans.push(Span::styled(
                    format!("  💬 {}", count),
                    Style::default().fg(theme.fg_dim),
                ));
            }

            lines.push(Line::from(meta_spans));
            ListItem::new(lines)
        })
        .collect();

    let title = format!(" Pull Requests ({}) ", list_state.items.len());

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

fn render_footer(
    frame: &mut Frame,
    list_state: &ghtui_core::state::PrListState,
    theme: &ghtui_core::theme::Theme,
    area: Rect,
) {
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
    frame.render_widget(footer, area);
}
