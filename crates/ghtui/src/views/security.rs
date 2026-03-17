use ghtui_core::AppState;
use ratatui::Frame;
use ratatui::layout::{Constraint, Direction, Layout, Rect};
use ratatui::style::{Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, List, ListItem, Paragraph, Tabs};

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

    let tab_titles = vec![
        format!("Dependabot ({})", dep_count),
        format!("Code Scanning ({})", cs_count),
        format!("Secret Scanning ({})", ss_count),
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

    match security.tab {
        0 => render_dependabot(frame, state, chunks[1]),
        1 => render_code_scanning(frame, state, chunks[1]),
        2 => render_secret_scanning(frame, state, chunks[1]),
        _ => {}
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
