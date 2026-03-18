use ghtui_core::AppState;
use ratatui::Frame;
use ratatui::layout::{Constraint, Direction, Layout, Rect};
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
        let paragraph = Paragraph::new("  No data — check API rate limit or network")
            .style(theme.text_dim())
            .block(
                Block::default()
                    .title(" Notifications ")
                    .borders(Borders::ALL)
                    .border_style(theme.border_style()),
            );
        frame.render_widget(paragraph, area);
        return;
    };

    // Layout: filter bar + list
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Length(1), Constraint::Min(0)])
        .split(area);

    // Filter bar
    render_filter_bar(frame, notif_state, theme, chunks[0]);

    let filtered = notif_state.filtered_items();

    if filtered.is_empty() {
        let msg = if notif_state.filters.has_active_filters() {
            "  No notifications matching filters"
        } else {
            "  No notifications — all caught up!"
        };
        let paragraph = Paragraph::new(msg).style(theme.text_dim()).block(
            Block::default()
                .title(" Notifications ")
                .borders(Borders::ALL)
                .border_style(theme.border_style()),
        );
        frame.render_widget(paragraph, chunks[1]);
        return;
    }

    // Build list items (with optional repo grouping)
    // display_selected tracks the actual index in the Vec<ListItem> for ListState
    let (items, display_selected): (Vec<ListItem>, usize) = if notif_state.grouped {
        let groups = notif_state.repo_groups();
        let mut list_items = Vec::new();
        let mut flat_idx = 0usize;
        let mut display_idx = 0usize;
        let mut selected_display = 0usize;

        for repo_name in &groups {
            // Repo group header
            list_items.push(ListItem::new(Line::from(vec![
                Span::styled("  ", Style::default()),
                Span::styled(repo_name.clone(), theme.text_bold()),
            ])));
            display_idx += 1;

            for notif in &filtered {
                if notif.repository.full_name != *repo_name {
                    continue;
                }
                let is_selected = flat_idx == notif_state.selected;
                if is_selected {
                    selected_display = display_idx;
                }
                list_items.push(render_notification_item(notif, is_selected, false, theme));
                flat_idx += 1;
                display_idx += 1;
            }
        }
        (list_items, selected_display)
    } else {
        let items: Vec<ListItem> = filtered
            .iter()
            .enumerate()
            .map(|(i, notif)| {
                let is_selected = i == notif_state.selected;
                render_notification_item(notif, is_selected, true, theme)
            })
            .collect();
        let sel = notif_state.selected;
        (items, sel)
    };

    let unread_count = filtered.iter().filter(|n| n.unread).count();
    let title = format!(
        " Notifications ({}/{} unread) ",
        unread_count,
        filtered.len()
    );

    let list = List::new(items)
        .block(
            Block::default()
                .title(Span::styled(title, theme.text_bold()))
                .borders(Borders::ALL)
                .border_style(theme.border_style()),
        )
        .highlight_style(theme.selected());

    let mut list_widget_state = ListState::default();
    list_widget_state.select(Some(display_selected));
    frame.render_stateful_widget(list, chunks[1], &mut list_widget_state);
}

fn render_notification_item<'a>(
    notif: &'a ghtui_core::types::Notification,
    is_selected: bool,
    show_repo: bool,
    theme: &ghtui_core::Theme,
) -> ListItem<'a> {
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

    // Line 1: unread_dot + type_icon + title + repo_name
    let mut line1_spans = vec![
        unread,
        type_icon,
        Span::styled(&notif.subject.title, title_style),
    ];

    if show_repo {
        line1_spans.push(Span::raw(" "));
        line1_spans.push(Span::styled(
            &notif.repository.full_name,
            Style::default().fg(theme.fg_dim),
        ));
    }

    // Line 2: (indented) reason_badge + relative_time
    let reason_badge = match notif.reason.as_str() {
        "review_requested" => Span::styled(" review ", Style::default().fg(theme.warning)),
        "assign" => Span::styled(" assign ", Style::default().fg(theme.accent)),
        "mention" => Span::styled(" @mention ", Style::default().fg(theme.accent)),
        "ci_activity" => Span::styled(" CI ", Style::default().fg(theme.fg_muted)),
        "subscribed" => Span::styled(" subscribed ", Style::default().fg(theme.fg_muted)),
        _ => Span::styled("", Style::default()),
    };

    let relative_time = super::components::time_ago(&notif.updated_at);

    let line2_spans = vec![
        Span::raw("      "),
        reason_badge,
        Span::raw(" "),
        Span::styled(relative_time, Style::default().fg(theme.fg_dim)),
    ];

    ListItem::new(vec![Line::from(line1_spans), Line::from(line2_spans)])
}

fn render_filter_bar(
    frame: &mut Frame,
    notif_state: &ghtui_core::state::NotificationListState,
    theme: &ghtui_core::Theme,
    area: Rect,
) {
    let mut spans = vec![Span::styled(" ", Style::default())];

    // Reason filter
    let reason_style = if notif_state.filters.reason.is_some() {
        Style::default().fg(theme.accent)
    } else {
        Style::default().fg(theme.fg_dim)
    };
    spans.push(Span::styled(
        format!("[s]:{} ", notif_state.filters.reason_display()),
        reason_style,
    ));

    // Type filter
    let type_style = if notif_state.filters.subject_type.is_some() {
        Style::default().fg(theme.accent)
    } else {
        Style::default().fg(theme.fg_dim)
    };
    spans.push(Span::styled(
        format!("[e]:{} ", notif_state.filters.type_display()),
        type_style,
    ));

    // Grouped toggle
    let group_style = if notif_state.grouped {
        Style::default().fg(theme.accent)
    } else {
        Style::default().fg(theme.fg_dim)
    };
    spans.push(Span::styled(
        format!(
            "[g]:{} ",
            if notif_state.grouped {
                "Grouped"
            } else {
                "Flat"
            }
        ),
        group_style,
    ));

    let line = Line::from(spans);
    let paragraph = Paragraph::new(line).style(Style::default().bg(theme.bg));
    frame.render_widget(paragraph, area);
}
