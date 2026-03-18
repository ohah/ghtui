use ghtui_core::AppState;
use ratatui::Frame;
use ratatui::layout::{Constraint, Direction, Layout, Rect};
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, List, ListItem, ListState, Paragraph, Wrap};

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
        let paragraph = Paragraph::new("  No data — check API rate limit or network")
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

    // Separate pinned and unpinned issues
    let pinned_count = list_state
        .items
        .iter()
        .filter(|i| list_state.pinned_numbers.contains(&i.number))
        .count();

    // Card height: 4 lines per card (border top, title+labels, meta, border bottom)
    // Cards laid out horizontally, 2 per row
    let card_rows = pinned_count.div_ceil(2); // ceil div
    let pinned_height = if pinned_count > 0 {
        (card_rows as u16 * 4) + 1 // +1 for "Pinned" header
    } else {
        0
    };

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(1),             // Filter bar
            Constraint::Length(pinned_height), // Pinned cards
            Constraint::Min(0),                // Issue list
            Constraint::Length(1),             // Footer
        ])
        .split(area);

    // === Filter bar ===
    render_filter_bar(frame, list_state, theme, chunks[0]);

    // === Pinned cards ===
    if pinned_count > 0 {
        render_pinned_cards(frame, list_state, theme, chunks[1]);
    }

    // === Issue list (non-pinned only in list) ===
    render_issue_list(frame, list_state, theme, chunks[2]);

    // === Footer ===
    render_footer(frame, list_state, theme, chunks[3]);
}

fn render_filter_bar(
    frame: &mut Frame,
    list_state: &ghtui_core::state::IssueListState,
    theme: &ghtui_core::theme::Theme,
    area: Rect,
) {
    if list_state.search_mode {
        let search_line = Line::from(vec![
            Span::styled(" 🔍 ", Style::default().fg(theme.accent)),
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

fn render_pinned_cards(
    frame: &mut Frame,
    list_state: &ghtui_core::state::IssueListState,
    theme: &ghtui_core::theme::Theme,
    area: Rect,
) {
    let pinned: Vec<_> = list_state
        .items
        .iter()
        .enumerate()
        .filter(|(_, i)| list_state.pinned_numbers.contains(&i.number))
        .collect();

    if pinned.is_empty() {
        return;
    }

    // Header
    let header_area = Rect::new(area.x, area.y, area.width, 1);
    let header = Paragraph::new(Line::from(vec![
        Span::styled(
            " 📌 Pinned ",
            Style::default()
                .fg(theme.warning)
                .add_modifier(Modifier::BOLD),
        ),
        Span::styled(
            format!("({})", pinned.len()),
            Style::default().fg(theme.fg_muted),
        ),
    ]));
    frame.render_widget(header, header_area);

    // Cards area (below header)
    let cards_area = Rect::new(
        area.x,
        area.y + 1,
        area.width,
        area.height.saturating_sub(1),
    );

    // Layout cards in rows of 2
    let card_width = (cards_area.width / 2).max(20);
    let mut y = cards_area.y;

    for row_cards in pinned.chunks(2) {
        if y + 3 > cards_area.y + cards_area.height {
            break;
        }

        for (col, (list_idx, issue)) in row_cards.iter().enumerate() {
            let x = cards_area.x + (col as u16 * card_width);
            let w = if col == 0 && row_cards.len() == 1 {
                cards_area.width // Full width if only 1 card in row
            } else {
                card_width.min(cards_area.width - (col as u16 * card_width))
            };
            let card_area = Rect::new(x, y, w, 3);

            let is_selected = *list_idx == list_state.selected;

            // Card content
            let state_icon = match issue.state {
                ghtui_core::types::IssueState::Open => "● ",
                ghtui_core::types::IssueState::Closed => "● ",
            };
            let state_color = match issue.state {
                ghtui_core::types::IssueState::Open => theme.success,
                ghtui_core::types::IssueState::Closed => theme.done,
            };

            let title_spans = vec![
                Span::styled(state_icon, Style::default().fg(state_color)),
                Span::styled(
                    issue.title.clone(),
                    if is_selected {
                        theme.selected()
                    } else {
                        theme.text_bold()
                    },
                ),
                Span::styled(
                    format!(" #{}", issue.number),
                    Style::default().fg(theme.fg_muted),
                ),
            ];

            let mut meta_spans: Vec<Span> = Vec::new();
            for label in issue.labels.iter().take(3) {
                meta_spans.push(super::components::label_span(&label.name, &label.color));
                meta_spans.push(Span::raw(" "));
            }
            if let Some(count) = issue.comments {
                if count > 0 {
                    meta_spans.push(Span::styled(
                        format!(" 💬{}", count),
                        Style::default().fg(theme.fg_dim),
                    ));
                }
            }

            let lines = vec![Line::from(title_spans), Line::from(meta_spans)];

            let border_color = if is_selected {
                theme.accent
            } else {
                theme.border
            };
            let card = Paragraph::new(lines)
                .block(
                    Block::default()
                        .borders(Borders::ALL)
                        .border_style(Style::default().fg(border_color))
                        .style(Style::default().bg(theme.bg_subtle)),
                )
                .wrap(Wrap { trim: true });
            frame.render_widget(card, card_area);
        }

        y += 3;
    }
}

fn render_issue_list(
    frame: &mut Frame,
    list_state: &ghtui_core::state::IssueListState,
    theme: &ghtui_core::theme::Theme,
    area: Rect,
) {
    // Show all issues (pinned ones are also in list but already shown as cards)
    let items: Vec<ListItem> = list_state
        .items
        .iter()
        .enumerate()
        .map(|(i, issue)| {
            let is_selected = i == list_state.selected;
            let is_pinned = list_state.pinned_numbers.contains(&issue.number);

            let state_icon = match issue.state {
                ghtui_core::types::IssueState::Open => {
                    Span::styled("● ", Style::default().fg(theme.success))
                }
                ghtui_core::types::IssueState::Closed => {
                    Span::styled("● ", Style::default().fg(theme.done))
                }
            };

            let title_style = if is_selected {
                theme.selected()
            } else {
                theme.text()
            };

            // Line 1: pin icon + state icon + title + labels
            let mut line1_spans = vec![
                if is_pinned {
                    Span::styled("📌", Style::default().fg(theme.warning))
                } else {
                    Span::raw("  ")
                },
                state_icon,
                Span::styled(issue.title.clone(), title_style),
            ];

            for label in &issue.labels {
                line1_spans.push(Span::raw(" "));
                line1_spans.push(super::components::label_span(&label.name, &label.color));
            }

            // Line 2: #number · opened time ago by user · assignees · comment count
            let mut line2_spans: Vec<Span> = vec![
                Span::raw("    "),
                Span::styled(
                    format!("#{}", issue.number),
                    Style::default().fg(theme.fg_muted),
                ),
                Span::styled(
                    format!(
                        " opened {} by {}",
                        super::components::time_ago(&issue.created_at),
                        issue.user.login,
                    ),
                    Style::default().fg(theme.fg_dim),
                ),
            ];

            if !issue.assignees.is_empty() {
                let assignees: Vec<String> =
                    issue.assignees.iter().map(|a| a.login.clone()).collect();
                line2_spans.push(Span::styled(
                    format!("  → {}", assignees.join(", ")),
                    Style::default().fg(theme.fg_dim),
                ));
            }

            if let Some(count) = issue.comments {
                if count > 0 {
                    line2_spans.push(Span::styled(
                        format!("  💬{}", count),
                        Style::default().fg(theme.fg_dim),
                    ));
                }
            }

            ListItem::new(vec![Line::from(line1_spans), Line::from(line2_spans)])
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

fn render_footer(
    frame: &mut Frame,
    list_state: &ghtui_core::state::IssueListState,
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
