use ghtui_core::AppState;
use ratatui::Frame;
use ratatui::layout::{Constraint, Direction, Layout, Rect};
use ratatui::style::{Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, List, ListItem, ListState, Paragraph, Wrap};

pub fn render(frame: &mut Frame, state: &AppState, area: Rect) {
    let theme = &state.theme;

    if state.is_loading("settings") {
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
                    .title(" Settings ")
                    .borders(Borders::ALL)
                    .border_style(theme.border_style()),
            );
        frame.render_widget(paragraph, area);
        return;
    }

    let Some(ref settings) = state.settings else {
        let paragraph = Paragraph::new("No data").style(theme.text_dim()).block(
            Block::default()
                .title(" Settings ")
                .borders(Borders::ALL)
                .border_style(theme.border_style()),
        );
        frame.render_widget(paragraph, area);
        return;
    };

    // Horizontal split: sidebar (30) | content (rest)
    let chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Length(30), Constraint::Min(0)])
        .split(area);

    // Sidebar
    let sidebar_titles = [
        "General".to_string(),
        "Branch Protection".to_string(),
        format!("Collaborators ({})", settings.collaborators.len()),
        format!("Webhooks ({})", settings.webhooks.len()),
        format!("Deploy Keys ({})", settings.deploy_keys.len()),
    ];

    let sidebar_items: Vec<ListItem> = sidebar_titles
        .iter()
        .enumerate()
        .map(|(i, title)| {
            let style = if i == settings.tab {
                if settings.sidebar_focused {
                    Style::default()
                        .fg(theme.tab_active_fg)
                        .add_modifier(Modifier::BOLD)
                        .bg(theme.selection_bg)
                } else {
                    Style::default()
                        .fg(theme.tab_active_fg)
                        .add_modifier(Modifier::BOLD)
                }
            } else {
                Style::default().fg(theme.fg_muted)
            };
            ListItem::new(Line::from(Span::styled(format!("  {} ", title), style)))
        })
        .collect();

    let sidebar_border_style = if settings.sidebar_focused {
        Style::default().fg(theme.accent)
    } else {
        theme.border_style()
    };

    let sidebar = List::new(sidebar_items).block(
        Block::default()
            .title(" Settings ")
            .borders(Borders::ALL)
            .border_style(sidebar_border_style),
    );

    let mut sidebar_state = ListState::default();
    sidebar_state.select(Some(settings.tab));
    frame.render_stateful_widget(sidebar, chunks[0], &mut sidebar_state);

    // Content panel
    let content_area = chunks[1];

    match settings.tab {
        0 => render_general(frame, state, content_area),
        1 => render_branch_protections(frame, state, content_area),
        2 => render_collaborators(frame, state, content_area),
        3 => render_webhooks(frame, state, content_area),
        4 => render_deploy_keys(frame, state, content_area),
        _ => {}
    }
}

fn render_general(frame: &mut Frame, state: &AppState, area: Rect) {
    let theme = &state.theme;
    let settings = state.settings.as_ref().unwrap();
    let repo = &settings.repo;

    let label_style = Style::default().fg(theme.fg_muted);
    let value_style = theme.text();
    let accent_bold = Style::default()
        .fg(theme.accent)
        .add_modifier(Modifier::BOLD);

    let mut lines: Vec<Line<'static>> = Vec::new();

    // Editing indicator
    if let Some(ref field) = settings.editing {
        let field_name = match field {
            ghtui_core::state::settings::SettingsEditField::Description => "Description",
            ghtui_core::state::settings::SettingsEditField::DefaultBranch => "Default branch",
            ghtui_core::state::settings::SettingsEditField::Topics => "Topics",
        };
        lines.push(Line::from(vec![
            Span::styled(
                format!("  Editing: {} ", field_name),
                Style::default().fg(theme.accent),
            ),
            Span::styled(settings.edit_buffer.clone(), theme.text()),
            Span::styled("█", Style::default().fg(theme.accent)),
            Span::styled(
                "  (Enter:Save Esc:Cancel)",
                Style::default().fg(theme.fg_dim),
            ),
        ]));
        lines.push(Line::raw(""));
    }

    lines.push(Line::raw(""));
    lines.push(kv("Name", &repo.full_name, label_style, value_style));
    lines.push(Line::from(vec![
        Span::styled(format!("    {:<18}", "Description"), label_style),
        Span::styled(
            repo.description.as_deref().unwrap_or("(none)").to_string(),
            value_style,
        ),
        Span::styled(" [d:edit]", Style::default().fg(theme.fg_dim)),
    ]));
    lines.push(Line::from(vec![
        Span::styled(format!("    {:<18}", "Visibility"), label_style),
        Span::styled(
            repo.visibility
                .as_deref()
                .unwrap_or(if repo.private { "private" } else { "public" })
                .to_string(),
            value_style,
        ),
        Span::styled(" [V:toggle]", Style::default().fg(theme.fg_dim)),
    ]));
    lines.push(kv(
        "Default branch",
        &repo.default_branch,
        label_style,
        value_style,
    ));
    lines.push(kv(
        "Language",
        repo.language.as_deref().unwrap_or("(none)"),
        label_style,
        value_style,
    ));
    lines.push(kv(
        "License",
        repo.license
            .as_ref()
            .map(|l| l.name.as_str())
            .unwrap_or("(none)"),
        label_style,
        value_style,
    ));
    lines.push(Line::raw(""));

    // Stats
    lines.push(Line::styled("  Statistics".to_string(), accent_bold));
    lines.push(kv(
        "Stars",
        &repo.stargazers_count.to_string(),
        label_style,
        value_style,
    ));
    lines.push(kv(
        "Forks",
        &repo.forks_count.to_string(),
        label_style,
        value_style,
    ));
    lines.push(kv(
        "Open issues",
        &repo.open_issues_count.to_string(),
        label_style,
        value_style,
    ));
    lines.push(kv(
        "Watchers",
        &repo.watchers_count.to_string(),
        label_style,
        value_style,
    ));
    lines.push(kv(
        "Size",
        &format!("{} KB", repo.size),
        label_style,
        value_style,
    ));
    lines.push(Line::raw(""));

    // Features
    lines.push(Line::styled("  Features".to_string(), accent_bold));
    lines.push(flag("Issues", repo.has_issues, theme));
    lines.push(flag("Projects", repo.has_projects, theme));
    lines.push(flag("Wiki", repo.has_wiki, theme));
    lines.push(flag(
        "Discussions",
        repo.has_discussions.unwrap_or(false),
        theme,
    ));
    lines.push(flag(
        "Allow forking",
        repo.allow_forking.unwrap_or(false),
        theme,
    ));
    lines.push(Line::raw(""));

    // Flags
    lines.push(Line::styled("  Status".to_string(), accent_bold));
    lines.push(flag("Fork", repo.fork, theme));
    lines.push(flag("Archived", repo.archived, theme));
    lines.push(flag("Disabled", repo.disabled, theme));
    lines.push(Line::raw(""));

    // Topics
    if let Some(ref topics) = repo.topics {
        if !topics.is_empty() {
            lines.push(Line::styled("  Topics".to_string(), accent_bold));
            let topic_spans: Vec<Span> = topics
                .iter()
                .map(|t| Span::styled(format!(" {} ", t), Style::default().fg(theme.accent)))
                .collect();
            lines.push(Line::from(topic_spans));
            lines.push(Line::raw(""));
        }
    }

    // Dates
    lines.push(Line::styled("  Dates".to_string(), accent_bold));
    lines.push(kv("Created", &repo.created_at, label_style, value_style));
    lines.push(kv("Updated", &repo.updated_at, label_style, value_style));
    if let Some(ref pushed) = repo.pushed_at {
        lines.push(kv("Last push", pushed, label_style, value_style));
    }

    let paragraph = Paragraph::new(lines)
        .block(
            Block::default()
                .title(" General ")
                .borders(Borders::ALL)
                .border_style(theme.border_style()),
        )
        .wrap(Wrap { trim: false })
        .scroll((settings.scroll as u16, 0));
    frame.render_widget(paragraph, area);
}

fn render_branch_protections(frame: &mut Frame, state: &AppState, area: Rect) {
    let theme = &state.theme;
    let settings = state.settings.as_ref().unwrap();

    if state.is_loading("branch_protections") {
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
                    .title(" Branch Protection Rules ")
                    .borders(Borders::ALL)
                    .border_style(theme.border_style()),
            );
        frame.render_widget(paragraph, area);
        return;
    }

    if settings.branch_protections.is_empty() {
        let paragraph = Paragraph::new("  No branch protection rules configured")
            .style(theme.text_dim())
            .block(
                Block::default()
                    .title(" Branch Protection Rules ")
                    .borders(Borders::ALL)
                    .border_style(theme.border_style()),
            );
        frame.render_widget(paragraph, area);
        return;
    }

    let items: Vec<ListItem> = settings
        .branch_protections
        .iter()
        .map(|bp| {
            let mut details = Vec::new();

            if let Some(ref checks) = bp.required_status_checks {
                if checks.strict {
                    details.push("strict");
                }
                if !checks.contexts.is_empty() {
                    details.push("status checks");
                }
            }
            if let Some(ref enforce) = bp.enforce_admins {
                if enforce.enabled {
                    details.push("enforce admins");
                }
            }
            if let Some(ref reviews) = bp.required_pull_request_reviews {
                if let Some(count) = reviews.required_approving_review_count {
                    if count > 0 {
                        details.push("reviews required");
                    }
                }
            }

            let detail_str = if details.is_empty() {
                "basic protection".to_string()
            } else {
                details.join(", ")
            };

            ListItem::new(Line::from(vec![
                Span::styled("  🔒 ", Style::default().fg(theme.warning)),
                Span::styled(bp.pattern.clone(), theme.text_bold()),
                Span::styled(format!("  ({})", detail_str), theme.text_dim()),
            ]))
        })
        .collect();

    let list = List::new(items).block(
        Block::default()
            .title(format!(
                " Branch Protection Rules ({}) ",
                settings.branch_protections.len()
            ))
            .borders(Borders::ALL)
            .border_style(theme.border_style()),
    );
    frame.render_widget(list, area);
}

fn render_collaborators(frame: &mut Frame, state: &AppState, area: Rect) {
    let theme = &state.theme;
    let settings = state.settings.as_ref().unwrap();

    if state.is_loading("collaborators") {
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
                    .title(" Collaborators ")
                    .borders(Borders::ALL)
                    .border_style(theme.border_style()),
            );
        frame.render_widget(paragraph, area);
        return;
    }

    if settings.collaborators.is_empty() {
        let paragraph = Paragraph::new("  No collaborators found")
            .style(theme.text_dim())
            .block(
                Block::default()
                    .title(" Collaborators ")
                    .borders(Borders::ALL)
                    .border_style(theme.border_style()),
            );
        frame.render_widget(paragraph, area);
        return;
    }

    let selected = settings.selected;
    let content_focused = !settings.sidebar_focused;

    let items: Vec<ListItem> = settings
        .collaborators
        .iter()
        .enumerate()
        .map(|(i, collab)| {
            let role = collab
                .role_name
                .as_deref()
                .or_else(|| {
                    collab.permissions.as_ref().map(|p| {
                        if p.admin {
                            "admin"
                        } else if p.maintain {
                            "maintain"
                        } else if p.push {
                            "write"
                        } else if p.triage {
                            "triage"
                        } else {
                            "read"
                        }
                    })
                })
                .unwrap_or("unknown");

            let role_color = match role {
                "admin" => theme.danger,
                "maintain" => theme.warning,
                "write" => theme.success,
                _ => theme.fg_muted,
            };

            let prefix = if content_focused && i == selected {
                "▶ @"
            } else {
                "  @"
            };

            let mut item = ListItem::new(Line::from(vec![
                Span::styled(prefix, Style::default().fg(theme.fg_muted)),
                Span::styled(collab.login.clone(), theme.text()),
                Span::raw("  "),
                Span::styled(role.to_string(), Style::default().fg(role_color)),
            ]));
            if content_focused && i == selected {
                item = item.style(Style::default().bg(theme.selection_bg));
            }
            item
        })
        .collect();

    let border_style = if content_focused {
        Style::default().fg(theme.accent)
    } else {
        theme.border_style()
    };

    let list = List::new(items).block(
        Block::default()
            .title(format!(
                " Collaborators ({}) [d:remove] ",
                settings.collaborators.len()
            ))
            .borders(Borders::ALL)
            .border_style(border_style),
    );
    frame.render_widget(list, area);
}

/// Key-value line with owned strings to avoid lifetime issues.
fn kv(label: &str, value: &str, label_style: Style, value_style: Style) -> Line<'static> {
    Line::from(vec![
        Span::styled(format!("    {:<18}", label), label_style),
        Span::styled(value.to_string(), value_style),
    ])
}

/// Boolean flag line.
fn flag(label: &str, enabled: bool, theme: &ghtui_core::theme::Theme) -> Line<'static> {
    let (icon, color) = if enabled {
        ("✓", theme.success)
    } else {
        ("✗", theme.fg_muted)
    };
    Line::from(vec![
        Span::styled(
            format!("    {:<18}", label),
            Style::default().fg(theme.fg_muted),
        ),
        Span::styled(icon, Style::default().fg(color)),
    ])
}

fn render_webhooks(frame: &mut Frame, state: &AppState, area: Rect) {
    let theme = &state.theme;
    let settings = state.settings.as_ref().unwrap();

    if state.is_loading("webhooks") {
        render_loading_spinner(frame, theme, area, "Webhooks");
        return;
    }

    if settings.webhooks.is_empty() {
        let paragraph = Paragraph::new("  No webhooks configured")
            .style(theme.text_dim())
            .block(
                Block::default()
                    .title(" Webhooks ")
                    .borders(Borders::ALL)
                    .border_style(theme.border_style()),
            );
        frame.render_widget(paragraph, area);
        return;
    }

    let selected = settings.selected;
    let content_focused = !settings.sidebar_focused;

    let items: Vec<ListItem> = settings
        .webhooks
        .iter()
        .enumerate()
        .map(|(i, hook)| {
            let prefix = if content_focused && i == selected {
                "▶"
            } else {
                " "
            };
            let status_icon = if hook.active {
                Span::styled(" ● ", Style::default().fg(theme.success))
            } else {
                Span::styled(" ○ ", Style::default().fg(theme.fg_muted))
            };
            let url = hook.config.url.as_deref().unwrap_or("(no url)");
            let events = if hook.events.len() > 3 {
                format!(
                    "{} + {} more",
                    hook.events[..3].join(", "),
                    hook.events.len() - 3
                )
            } else {
                hook.events.join(", ")
            };

            let mut item = ListItem::new(Line::from(vec![
                Span::styled(prefix, Style::default().fg(theme.accent)),
                status_icon,
                Span::styled(url.to_string(), theme.text()),
                Span::styled(format!("  ({})", events), Style::default().fg(theme.fg_dim)),
            ]));
            if content_focused && i == selected {
                item = item.style(Style::default().bg(theme.selection_bg));
            }
            item
        })
        .collect();

    let border_style = if content_focused {
        Style::default().fg(theme.accent)
    } else {
        theme.border_style()
    };

    let list = List::new(items).block(
        Block::default()
            .title(format!(
                " Webhooks ({}) [d:delete a:toggle] ",
                settings.webhooks.len()
            ))
            .borders(Borders::ALL)
            .border_style(border_style),
    );
    frame.render_widget(list, area);
}

fn render_deploy_keys(frame: &mut Frame, state: &AppState, area: Rect) {
    let theme = &state.theme;
    let settings = state.settings.as_ref().unwrap();

    if state.is_loading("deploy_keys") {
        render_loading_spinner(frame, theme, area, "Deploy Keys");
        return;
    }

    if settings.deploy_keys.is_empty() {
        let paragraph = Paragraph::new("  No deploy keys configured")
            .style(theme.text_dim())
            .block(
                Block::default()
                    .title(" Deploy Keys ")
                    .borders(Borders::ALL)
                    .border_style(theme.border_style()),
            );
        frame.render_widget(paragraph, area);
        return;
    }

    let selected = settings.selected;
    let content_focused = !settings.sidebar_focused;

    let items: Vec<ListItem> = settings
        .deploy_keys
        .iter()
        .enumerate()
        .map(|(i, key)| {
            let prefix = if content_focused && i == selected {
                "▶ 🔑 "
            } else {
                "  🔑 "
            };
            let read_only = if key.read_only {
                Span::styled(" read-only", Style::default().fg(theme.fg_muted))
            } else {
                Span::styled(" read-write", Style::default().fg(theme.warning))
            };
            let verified = if key.verified {
                Span::styled(" ✓", Style::default().fg(theme.success))
            } else {
                Span::styled("", Style::default())
            };

            let mut item = ListItem::new(Line::from(vec![
                Span::styled(prefix, Style::default().fg(theme.accent)),
                Span::styled(key.title.clone(), theme.text()),
                read_only,
                verified,
                Span::styled(
                    format!("  {}", key.created_at),
                    Style::default().fg(theme.fg_dim),
                ),
            ]));
            if content_focused && i == selected {
                item = item.style(Style::default().bg(theme.selection_bg));
            }
            item
        })
        .collect();

    let border_style = if content_focused {
        Style::default().fg(theme.accent)
    } else {
        theme.border_style()
    };

    let list = List::new(items).block(
        Block::default()
            .title(format!(
                " Deploy Keys ({}) [d:delete] ",
                settings.deploy_keys.len()
            ))
            .borders(Borders::ALL)
            .border_style(border_style),
    );
    frame.render_widget(list, area);
}

fn render_loading_spinner(
    frame: &mut Frame,
    theme: &ghtui_core::theme::Theme,
    area: Rect,
    title: &str,
) {
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
                .title(format!(" {} ", title))
                .borders(Borders::ALL)
                .border_style(theme.border_style()),
        );
    frame.render_widget(paragraph, area);
}
