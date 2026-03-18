use ghtui_core::theme::Theme;
use ratatui::Frame;
use ratatui::layout::Rect;
use ratatui::style::{Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, List, ListItem, ListState, Paragraph};

pub fn render_sidebar(
    frame: &mut Frame,
    theme: &Theme,
    title: &str,
    items: &[String],
    selected: usize,
    focused: bool,
    area: Rect,
) {
    // Build ListItems with selection highlighting
    let list_items: Vec<ListItem> = items
        .iter()
        .enumerate()
        .map(|(i, item_title)| {
            let style = if i == selected {
                if focused {
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
            ListItem::new(Line::from(Span::styled(
                format!("  {} ", item_title),
                style,
            )))
        })
        .collect();

    let border_style = if focused {
        Style::default().fg(theme.accent)
    } else {
        theme.border_style()
    };

    let list = List::new(list_items).block(
        Block::default()
            .title(format!(" {} ", title))
            .borders(Borders::ALL)
            .border_style(border_style),
    );

    let mut state = ListState::default();
    state.select(Some(selected));
    frame.render_stateful_widget(list, area, &mut state);
}

pub fn render_loading(frame: &mut Frame, theme: &Theme, area: Rect, title: &str) {
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

pub fn centered_rect(width: u16, height: u16, area: Rect) -> Rect {
    let x = (area.width.saturating_sub(width)) / 2 + area.x;
    let y = (area.height.saturating_sub(height)) / 2 + area.y;
    Rect::new(x, y, width.min(area.width), height.min(area.height))
}

/// Parse a GitHub label hex color (e.g., "d73a4a") and return a styled Span.
/// Uses the label color as background with contrasting text color.
pub fn label_span(name: &str, hex_color: &str) -> Span<'static> {
    let (r, g, b) = parse_hex_color(hex_color);
    // Choose white or black text based on luminance
    let luminance = 0.299 * r as f64 + 0.587 * g as f64 + 0.114 * b as f64;
    let fg = if luminance > 128.0 {
        ratatui::style::Color::Rgb(0, 0, 0)
    } else {
        ratatui::style::Color::Rgb(255, 255, 255)
    };
    let bg = ratatui::style::Color::Rgb(r, g, b);
    Span::styled(format!(" {} ", name), Style::default().fg(fg).bg(bg))
}

/// Format a chrono DateTime as a relative time string (e.g., "3 days ago").
pub fn time_ago(dt: &chrono::DateTime<chrono::Utc>) -> String {
    let now = chrono::Utc::now();
    let duration = now.signed_duration_since(*dt);

    if duration.num_minutes() < 1 {
        "just now".to_string()
    } else if duration.num_minutes() < 60 {
        let m = duration.num_minutes();
        format!("{} minute{} ago", m, if m == 1 { "" } else { "s" })
    } else if duration.num_hours() < 24 {
        let h = duration.num_hours();
        format!("{} hour{} ago", h, if h == 1 { "" } else { "s" })
    } else if duration.num_days() < 30 {
        let d = duration.num_days();
        format!("{} day{} ago", d, if d == 1 { "" } else { "s" })
    } else if duration.num_days() < 365 {
        let m = duration.num_days() / 30;
        format!("{} month{} ago", m, if m == 1 { "" } else { "s" })
    } else {
        let y = duration.num_days() / 365;
        format!("{} year{} ago", y, if y == 1 { "" } else { "s" })
    }
}

/// Parse an RFC3339/ISO8601 date string and return a relative time string.
/// Returns the original string if parsing fails.
pub fn time_ago_rfc3339(s: &str) -> String {
    chrono::DateTime::parse_from_rfc3339(s)
        .map(|dt| time_ago(&dt.with_timezone(&chrono::Utc)))
        .unwrap_or_else(|_| s.to_string())
}

fn parse_hex_color(hex: &str) -> (u8, u8, u8) {
    let hex = hex.trim_start_matches('#');
    if hex.len() >= 6 {
        let r = u8::from_str_radix(&hex[0..2], 16).unwrap_or(128);
        let g = u8::from_str_radix(&hex[2..4], 16).unwrap_or(128);
        let b = u8::from_str_radix(&hex[4..6], 16).unwrap_or(128);
        (r, g, b)
    } else {
        (128, 128, 128) // default gray
    }
}
