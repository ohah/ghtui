use ghtui_core::AppState;
use ratatui::Frame;
use ratatui::layout::{Constraint, Direction, Layout, Rect};
use ratatui::style::{Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, List, ListItem, Paragraph};

pub fn render(frame: &mut Frame, state: &AppState, area: Rect) {
    let theme = &state.theme;

    if state.is_loading("orgs") {
        let loading =
            Paragraph::new("  Loading organizations...").style(Style::default().fg(theme.fg_dim));
        frame.render_widget(loading, area);
        return;
    }

    let Some(ref org_state) = state.org else {
        let empty =
            Paragraph::new("  No organizations loaded.").style(Style::default().fg(theme.fg_dim));
        frame.render_widget(empty, area);
        return;
    };

    if org_state.orgs.is_empty() {
        let empty = Paragraph::new("  Not a member of any organizations.")
            .style(Style::default().fg(theme.fg_dim));
        frame.render_widget(empty, area);
        return;
    }

    let chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(40), Constraint::Percentage(60)])
        .split(area);

    // Left: org list
    let org_items: Vec<ListItem> = org_state
        .orgs
        .iter()
        .enumerate()
        .map(|(i, org)| {
            let name = org.name.as_deref().unwrap_or(&org.login);
            let line = Line::from(vec![Span::styled(
                format!("  {} ", name),
                if i == org_state.selected_org {
                    Style::default()
                        .fg(theme.accent)
                        .add_modifier(Modifier::BOLD)
                } else {
                    Style::default().fg(theme.fg)
                },
            )]);
            ListItem::new(line)
        })
        .collect();

    let org_list = List::new(org_items).block(
        Block::default()
            .borders(Borders::ALL)
            .border_style(Style::default().fg(theme.border))
            .title(Span::styled(
                " Organizations ",
                Style::default().fg(theme.fg).add_modifier(Modifier::BOLD),
            )),
    );
    frame.render_widget(org_list, chunks[0]);

    // Right: members
    if state.is_loading("org_members") {
        let loading = Paragraph::new("  Loading members...")
            .style(Style::default().fg(theme.fg_dim))
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .border_style(Style::default().fg(theme.border))
                    .title(Span::styled(" Members ", Style::default().fg(theme.fg))),
            );
        frame.render_widget(loading, chunks[1]);
    } else {
        let member_items: Vec<ListItem> = org_state
            .members
            .iter()
            .map(|m| {
                let role = m.role.as_deref().unwrap_or("member");
                let line = Line::from(vec![
                    Span::styled(format!("  @{} ", m.login), Style::default().fg(theme.fg)),
                    Span::styled(format!("({})", role), Style::default().fg(theme.fg_muted)),
                ]);
                ListItem::new(line)
            })
            .collect();

        let member_list = List::new(member_items).block(
            Block::default()
                .borders(Borders::ALL)
                .border_style(Style::default().fg(theme.border))
                .title(Span::styled(
                    " Members ",
                    Style::default().fg(theme.fg).add_modifier(Modifier::BOLD),
                )),
        );
        frame.render_widget(member_list, chunks[1]);
    }
}
