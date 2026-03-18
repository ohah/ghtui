use ghtui_core::AppState;
use ratatui::Frame;
use ratatui::layout::{Constraint, Direction, Layout, Rect};
use ratatui::style::{Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, List, ListItem, Paragraph, Tabs, Wrap};

pub fn render(frame: &mut Frame, state: &AppState, area: Rect) {
    let theme = &state.theme;

    if state.is_loading("security") {
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
                    .title(" Security ")
                    .borders(Borders::ALL)
                    .border_style(theme.border_style()),
            );
        frame.render_widget(paragraph, area);
        return;
    }

    let Some(ref security) = state.security else {
        let paragraph = Paragraph::new("No data").style(theme.text_dim()).block(
            Block::default()
                .title(" Security ")
                .borders(Borders::ALL)
                .border_style(theme.border_style()),
        );
        frame.render_widget(paragraph, area);
        return;
    };

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Length(2), Constraint::Min(0)])
        .split(area);

    let dep_count = security.dependabot_alerts.len();
    let cs_count = security.code_scanning_alerts.len();
    let ss_count = security.secret_scanning_alerts.len();

    let adv_count = security.advisories.len();

    let tab_titles = vec![
        format!("Dependabot ({})", dep_count),
        format!("Code Scanning ({})", cs_count),
        format!("Secret Scanning ({})", ss_count),
        format!("Advisories ({})", adv_count),
    ];
    let tabs = Tabs::new(tab_titles)
        .select(security.tab)
        .style(Style::default().fg(theme.fg_muted))
        .highlight_style(
            Style::default()
                .fg(theme.tab_active_fg)
                .add_modifier(Modifier::BOLD | Modifier::UNDERLINED),
        )
        .divider(" │ ")
        .block(
            Block::default()
                .borders(Borders::BOTTOM)
                .border_style(theme.border_style()),
        );
    frame.render_widget(tabs, chunks[0]);

    if security.detail_open {
        // Split: list (left 40%) + detail (right 60%)
        let split = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([Constraint::Percentage(40), Constraint::Percentage(60)])
            .split(chunks[1]);

        match security.tab {
            0 => render_dependabot(frame, state, split[0]),
            1 => render_code_scanning(frame, state, split[0]),
            2 => render_secret_scanning(frame, state, split[0]),
            3 => render_advisories(frame, state, split[0]),
            _ => {}
        }
        render_detail_panel(frame, state, split[1]);
    } else {
        match security.tab {
            0 => render_dependabot(frame, state, chunks[1]),
            1 => render_code_scanning(frame, state, chunks[1]),
            2 => render_secret_scanning(frame, state, chunks[1]),
            3 => render_advisories(frame, state, chunks[1]),
            _ => {}
        }
    }
}

fn severity_color(severity: &str, theme: &ghtui_core::theme::Theme) -> ratatui::style::Color {
    match severity.to_lowercase().as_str() {
        "critical" | "high" => theme.danger,
        "medium" => theme.warning,
        "low" => theme.info,
        _ => theme.fg_muted,
    }
}

fn render_dependabot(frame: &mut Frame, state: &AppState, area: Rect) {
    let theme = &state.theme;
    let security = state.security.as_ref().unwrap();

    if state.is_loading("dependabot") {
        render_loading(frame, theme, area, "Dependabot Alerts");
        return;
    }

    if security.dependabot_alerts.is_empty() {
        let paragraph = Paragraph::new("  No open Dependabot alerts")
            .style(theme.text_dim())
            .block(
                Block::default()
                    .title(" Dependabot Alerts ")
                    .borders(Borders::ALL)
                    .border_style(theme.border_style()),
            );
        frame.render_widget(paragraph, area);
        return;
    }

    let items: Vec<ListItem> = security
        .dependabot_alerts
        .iter()
        .map(|alert| {
            let sev = &alert.security_vulnerability.severity;
            let color = severity_color(sev, theme);

            ListItem::new(Line::from(vec![
                Span::styled(
                    format!("  {:<9}", sev.to_uppercase()),
                    Style::default().fg(color),
                ),
                Span::styled(alert.security_advisory.summary.clone(), theme.text()),
                Span::styled(
                    format!("  ({})", alert.dependency.package.name),
                    theme.text_dim(),
                ),
            ]))
        })
        .collect();

    let list = List::new(items).block(
        Block::default()
            .title(format!(
                " Dependabot Alerts ({}) ",
                security.dependabot_alerts.len()
            ))
            .borders(Borders::ALL)
            .border_style(theme.border_style()),
    );
    frame.render_widget(list, area);
}

fn render_code_scanning(frame: &mut Frame, state: &AppState, area: Rect) {
    let theme = &state.theme;
    let security = state.security.as_ref().unwrap();

    if state.is_loading("code_scanning") {
        render_loading(frame, theme, area, "Code Scanning Alerts");
        return;
    }

    if security.code_scanning_alerts.is_empty() {
        let paragraph = Paragraph::new("  No open code scanning alerts")
            .style(theme.text_dim())
            .block(
                Block::default()
                    .title(" Code Scanning Alerts ")
                    .borders(Borders::ALL)
                    .border_style(theme.border_style()),
            );
        frame.render_widget(paragraph, area);
        return;
    }

    let items: Vec<ListItem> = security
        .code_scanning_alerts
        .iter()
        .map(|alert| {
            let sev = alert
                .rule
                .security_severity_level
                .as_deref()
                .or(alert.rule.severity.as_deref())
                .unwrap_or("unknown");
            let color = severity_color(sev, theme);

            let desc = alert
                .rule
                .description
                .as_deref()
                .or(alert.rule.name.as_deref())
                .unwrap_or("Unknown rule");

            let location = alert
                .most_recent_instance
                .as_ref()
                .and_then(|i| i.location.as_ref())
                .and_then(|l| l.path.as_deref())
                .unwrap_or("");

            ListItem::new(Line::from(vec![
                Span::styled(
                    format!("  {:<9}", sev.to_uppercase()),
                    Style::default().fg(color),
                ),
                Span::styled(desc.to_string(), theme.text()),
                Span::styled(format!("  {}", location), theme.text_dim()),
            ]))
        })
        .collect();

    let list = List::new(items).block(
        Block::default()
            .title(format!(
                " Code Scanning Alerts ({}) ",
                security.code_scanning_alerts.len()
            ))
            .borders(Borders::ALL)
            .border_style(theme.border_style()),
    );
    frame.render_widget(list, area);
}

fn render_secret_scanning(frame: &mut Frame, state: &AppState, area: Rect) {
    let theme = &state.theme;
    let security = state.security.as_ref().unwrap();

    if state.is_loading("secret_scanning") {
        render_loading(frame, theme, area, "Secret Scanning Alerts");
        return;
    }

    if security.secret_scanning_alerts.is_empty() {
        let paragraph = Paragraph::new("  No open secret scanning alerts")
            .style(theme.text_dim())
            .block(
                Block::default()
                    .title(" Secret Scanning Alerts ")
                    .borders(Borders::ALL)
                    .border_style(theme.border_style()),
            );
        frame.render_widget(paragraph, area);
        return;
    }

    let items: Vec<ListItem> = security
        .secret_scanning_alerts
        .iter()
        .map(|alert| {
            let secret_type = alert
                .secret_type_display_name
                .as_deref()
                .or(alert.secret_type.as_deref())
                .unwrap_or("Unknown");

            ListItem::new(Line::from(vec![
                Span::styled("  ⚠ ", Style::default().fg(theme.warning)),
                Span::styled(secret_type.to_string(), theme.text()),
                Span::styled(
                    format!("  #{} ({})", alert.number, alert.state),
                    theme.text_dim(),
                ),
            ]))
        })
        .collect();

    let list = List::new(items).block(
        Block::default()
            .title(format!(
                " Secret Scanning Alerts ({}) ",
                security.secret_scanning_alerts.len()
            ))
            .borders(Borders::ALL)
            .border_style(theme.border_style()),
    );
    frame.render_widget(list, area);
}

fn render_loading(frame: &mut Frame, theme: &ghtui_core::theme::Theme, area: Rect, title: &str) {
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

fn render_detail_panel(frame: &mut Frame, state: &AppState, area: Rect) {
    let theme = &state.theme;
    let security = state.security.as_ref().unwrap();

    let mut lines: Vec<Line<'static>> = Vec::new();
    let label = Style::default().fg(theme.fg_muted);
    let value = theme.text();
    let accent = Style::default()
        .fg(theme.accent)
        .add_modifier(Modifier::BOLD);

    match security.tab {
        0 => {
            // Dependabot detail
            if let Some(alert) = security.dependabot_alerts.get(security.selected) {
                let sev_color = severity_color(&alert.security_vulnerability.severity, theme);

                lines.push(Line::raw(""));
                lines.push(Line::styled(
                    format!("  {}", alert.security_advisory.summary),
                    accent,
                ));
                lines.push(Line::raw(""));

                lines.push(detail_kv(
                    "Severity",
                    &alert.security_vulnerability.severity,
                    label,
                    Style::default().fg(sev_color),
                ));
                lines.push(detail_kv(
                    "GHSA",
                    &alert.security_advisory.ghsa_id,
                    label,
                    value,
                ));
                if let Some(ref cve) = alert.security_advisory.cve_id {
                    lines.push(detail_kv("CVE", cve, label, value));
                }
                lines.push(detail_kv("State", &alert.state, label, value));
                lines.push(detail_kv(
                    "Package",
                    &alert.dependency.package.name,
                    label,
                    value,
                ));
                lines.push(detail_kv(
                    "Ecosystem",
                    &alert.dependency.package.ecosystem,
                    label,
                    value,
                ));

                if let Some(ref manifest) = alert.dependency.manifest_path {
                    lines.push(detail_kv("Manifest", manifest, label, value));
                }
                if let Some(ref range) = alert.security_vulnerability.vulnerable_version_range {
                    lines.push(detail_kv(
                        "Vulnerable",
                        range,
                        label,
                        Style::default().fg(theme.danger),
                    ));
                }
                if let Some(ref patched) = alert.security_vulnerability.first_patched_version {
                    lines.push(detail_kv(
                        "Patched",
                        &patched.identifier,
                        label,
                        Style::default().fg(theme.success),
                    ));
                }

                lines.push(Line::raw(""));
                lines.push(Line::styled("  Description", accent));
                lines.push(Line::raw(""));
                for desc_line in alert.security_advisory.description.lines() {
                    lines.push(Line::styled(
                        format!("  {}", desc_line),
                        Style::default().fg(theme.fg_dim),
                    ));
                }
            }
        }
        1 => {
            // Code Scanning detail
            if let Some(alert) = security.code_scanning_alerts.get(security.selected) {
                lines.push(Line::raw(""));
                let name = alert.rule.name.as_deref().unwrap_or("Unknown");
                lines.push(Line::styled(format!("  {}", name), accent));
                lines.push(Line::raw(""));

                if let Some(ref sev) = alert.rule.severity {
                    let sev_color = severity_color(sev, theme);
                    lines.push(detail_kv(
                        "Severity",
                        sev,
                        label,
                        Style::default().fg(sev_color),
                    ));
                }
                if let Some(ref sec_sev) = alert.rule.security_severity_level {
                    lines.push(detail_kv("Security level", sec_sev, label, value));
                }
                lines.push(detail_kv("State", &alert.state, label, value));
                lines.push(detail_kv("Tool", &alert.tool.name, label, value));
                if let Some(ref ver) = alert.tool.version {
                    lines.push(detail_kv("Version", ver, label, value));
                }
                if let Some(ref rule_id) = alert.rule.id {
                    lines.push(detail_kv("Rule ID", rule_id, label, value));
                }

                if let Some(ref instance) = alert.most_recent_instance {
                    if let Some(ref loc) = instance.location {
                        lines.push(Line::raw(""));
                        lines.push(Line::styled("  Location", accent));
                        if let Some(ref path) = loc.path {
                            let loc_str = match (loc.start_line, loc.end_line) {
                                (Some(s), Some(e)) if s == e => format!("{}:{}", path, s),
                                (Some(s), Some(e)) => format!("{}:{}-{}", path, s, e),
                                (Some(s), None) => format!("{}:{}", path, s),
                                _ => path.clone(),
                            };
                            lines.push(detail_kv("File", &loc_str, label, value));
                        }
                    }
                }

                if let Some(ref desc) = alert.rule.description {
                    lines.push(Line::raw(""));
                    lines.push(Line::styled("  Description", accent));
                    lines.push(Line::raw(""));
                    for desc_line in desc.lines() {
                        lines.push(Line::styled(
                            format!("  {}", desc_line),
                            Style::default().fg(theme.fg_dim),
                        ));
                    }
                }
            }
        }
        2 => {
            // Secret Scanning detail
            if let Some(alert) = security.secret_scanning_alerts.get(security.selected) {
                let display_name = alert
                    .secret_type_display_name
                    .as_deref()
                    .or(alert.secret_type.as_deref())
                    .unwrap_or("Unknown");

                lines.push(Line::raw(""));
                lines.push(Line::styled(format!("  {}", display_name), accent));
                lines.push(Line::raw(""));

                lines.push(detail_kv(
                    "Alert #",
                    &alert.number.to_string(),
                    label,
                    value,
                ));
                lines.push(detail_kv("State", &alert.state, label, value));
                if let Some(ref secret_type) = alert.secret_type {
                    lines.push(detail_kv("Type", secret_type, label, value));
                }
                if let Some(ref resolution) = alert.resolution {
                    lines.push(detail_kv("Resolution", resolution, label, value));
                }
                lines.push(detail_kv("Created", &alert.created_at, label, value));
            }
        }
        3 => {
            // Advisory detail
            if let Some(adv) = security.advisories.get(security.selected) {
                lines.push(Line::raw(""));
                lines.push(Line::styled(format!("  {}", adv.summary), accent));
                lines.push(Line::raw(""));
                lines.push(detail_kv("GHSA", &adv.ghsa_id, label, value));
                if let Some(ref cve) = adv.cve_id {
                    lines.push(detail_kv("CVE", cve, label, value));
                }
                if let Some(ref sev) = adv.severity {
                    lines.push(detail_kv("Severity", sev, label, value));
                }
                lines.push(detail_kv("State", &adv.state, label, value));
                if let Some(ref desc) = adv.description {
                    lines.push(Line::raw(""));
                    for line in desc.lines().take(20) {
                        lines.push(Line::styled(
                            format!("    {}", line),
                            Style::default().fg(theme.fg_muted),
                        ));
                    }
                }
            }
        }
        _ => {}
    }

    if lines.is_empty() {
        lines.push(Line::styled(
            "  No alert selected",
            Style::default().fg(theme.fg_dim),
        ));
    }

    let paragraph = Paragraph::new(lines)
        .block(
            Block::default()
                .title(" Detail (Esc:Close o:Open) ")
                .borders(Borders::ALL)
                .border_style(Style::default().fg(theme.accent)),
        )
        .wrap(Wrap { trim: false })
        .scroll((security.detail_scroll as u16, 0));
    frame.render_widget(paragraph, area);
}

fn detail_kv(label: &str, value: &str, label_style: Style, value_style: Style) -> Line<'static> {
    Line::from(vec![
        Span::styled(format!("    {:<18}", label), label_style),
        Span::styled(value.to_string(), value_style),
    ])
}

fn render_advisories(frame: &mut Frame, state: &AppState, area: Rect) {
    let theme = &state.theme;
    let security = state.security.as_ref().unwrap();

    if security.advisories.is_empty() {
        let paragraph = Paragraph::new("  No security advisories")
            .style(theme.text_dim())
            .block(
                Block::default()
                    .title(" Advisories ")
                    .borders(Borders::ALL)
                    .border_style(theme.border_style()),
            );
        frame.render_widget(paragraph, area);
        return;
    }

    let items: Vec<ListItem> = security
        .advisories
        .iter()
        .enumerate()
        .map(|(i, adv)| {
            let is_selected = i == security.selected;
            let severity_text = adv.severity.as_deref().unwrap_or("unknown");
            let sev_color = severity_color(severity_text, theme);
            let title_style = if is_selected {
                theme.selected()
            } else {
                theme.text()
            };

            ListItem::new(Line::from(vec![
                Span::styled(
                    format!(
                        " {} ",
                        severity_text
                            .chars()
                            .next()
                            .unwrap_or('?')
                            .to_ascii_uppercase()
                    ),
                    Style::default().fg(sev_color),
                ),
                Span::styled(&adv.ghsa_id, Style::default().fg(theme.fg_muted)),
                Span::styled(format!(" {} ", adv.summary), title_style),
                Span::styled(
                    format!("({})", adv.state),
                    Style::default().fg(theme.fg_dim),
                ),
            ]))
        })
        .collect();

    let list = List::new(items).block(
        Block::default()
            .title(format!(" Advisories ({}) ", security.advisories.len()))
            .borders(Borders::ALL)
            .border_style(theme.border_style()),
    );
    frame.render_widget(list, area);
}
