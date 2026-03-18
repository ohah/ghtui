use super::components;
use ghtui_core::AppState;
use ghtui_core::types::code::FileEntryType;
use ratatui::Frame;
use ratatui::layout::{Constraint, Direction, Layout, Rect};
use ratatui::style::{Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, List, ListItem, ListState, Paragraph, Wrap};

pub fn render(frame: &mut Frame, state: &AppState, area: Rect) {
    let theme = &state.theme;

    if state.is_loading("code_contents") && state.code.is_none() {
        components::render_loading(frame, theme, area, "Code");
        return;
    }

    let Some(ref code) = state.code else {
        let paragraph = Paragraph::new("  No repository selected")
            .style(theme.text_dim())
            .block(
                Block::default()
                    .title(" Code ")
                    .borders(Borders::ALL)
                    .border_style(theme.border_style()),
            );
        frame.render_widget(paragraph, area);
        return;
    };

    // Horizontal split: file tree (35) | content (rest)
    let chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Length(35), Constraint::Min(0)])
        .split(area);

    render_file_tree(frame, state, code, chunks[0]);
    render_content(frame, state, code, chunks[1]);
}

fn render_file_tree(
    frame: &mut Frame,
    state: &AppState,
    code: &ghtui_core::state::CodeViewState,
    area: Rect,
) {
    let theme = &state.theme;

    let title = if code.current_path.is_empty() {
        " / ".to_string()
    } else {
        format!(" /{} ", code.current_path)
    };

    let loading = state.is_loading("code_contents");

    let list_items: Vec<ListItem> = if loading {
        vec![ListItem::new(Line::from(Span::styled(
            "  Loading...",
            Style::default().fg(theme.fg_dim),
        )))]
    } else if code.entries.is_empty() {
        vec![ListItem::new(Line::from(Span::styled(
            "  (empty directory)",
            Style::default().fg(theme.fg_dim),
        )))]
    } else {
        code.entries
            .iter()
            .enumerate()
            .map(|(i, entry)| {
                let icon = match entry.entry_type {
                    FileEntryType::Dir => "\u{1F4C1} ",
                    FileEntryType::File => "\u{1F4C4} ",
                };

                let size_str = match (&entry.entry_type, entry.size) {
                    (FileEntryType::File, Some(s)) => format_size(s),
                    _ => String::new(),
                };

                let is_selected = i == code.selected;
                let style = if is_selected {
                    if code.sidebar_focused {
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
                    match entry.entry_type {
                        FileEntryType::Dir => Style::default().fg(theme.accent),
                        FileEntryType::File => Style::default().fg(theme.fg),
                    }
                };

                let spans = vec![
                    Span::styled(format!(" {}", icon), style),
                    Span::styled(entry.name.clone(), style),
                    Span::styled(
                        if size_str.is_empty() {
                            String::new()
                        } else {
                            format!(" ({})", size_str)
                        },
                        Style::default().fg(theme.fg_dim),
                    ),
                ];

                ListItem::new(Line::from(spans))
            })
            .collect()
    };

    let border_style = if code.sidebar_focused {
        Style::default().fg(theme.accent)
    } else {
        theme.border_style()
    };

    let list = List::new(list_items).block(
        Block::default()
            .title(Span::styled(title, Style::default().fg(theme.fg)))
            .borders(Borders::ALL)
            .border_style(border_style),
    );

    let mut list_state = ListState::default();
    if !code.entries.is_empty() {
        list_state.select(Some(code.selected));
    }
    frame.render_stateful_widget(list, area, &mut list_state);
}

fn render_content(
    frame: &mut Frame,
    state: &AppState,
    code: &ghtui_core::state::CodeViewState,
    area: Rect,
) {
    let theme = &state.theme;

    let border_style = if !code.sidebar_focused {
        Style::default().fg(theme.accent)
    } else {
        theme.border_style()
    };

    if state.is_loading("code_file") {
        let paragraph = Paragraph::new("  Loading file...")
            .style(theme.text_dim())
            .block(
                Block::default()
                    .title(" File ")
                    .borders(Borders::ALL)
                    .border_style(border_style),
            );
        frame.render_widget(paragraph, area);
        return;
    }

    // If viewing a file
    if let (Some(content), Some(filename)) = (&code.file_content, &code.file_name) {
        render_file_content(
            frame,
            theme,
            content,
            filename,
            code.scroll,
            border_style,
            area,
        );
        return;
    }

    // If README is available and no file selected
    if let Some(ref readme) = code.readme_content {
        let lines = ghtui_widgets::render_markdown(readme);

        let paragraph = Paragraph::new(lines)
            .scroll((code.scroll.min(u16::MAX as usize) as u16, 0))
            .style(Style::default().bg(theme.bg))
            .block(
                Block::default()
                    .title(" README.md ")
                    .borders(Borders::ALL)
                    .border_style(border_style),
            )
            .wrap(Wrap { trim: false });

        frame.render_widget(paragraph, area);
        return;
    }

    // Default: no content
    let paragraph = Paragraph::new("  Select a file to view")
        .style(theme.text_dim())
        .block(
            Block::default()
                .title(" Code ")
                .borders(Borders::ALL)
                .border_style(border_style),
        );
    frame.render_widget(paragraph, area);
}

fn render_file_content(
    frame: &mut Frame,
    theme: &ghtui_core::theme::Theme,
    content: &str,
    filename: &str,
    scroll: usize,
    border_style: Style,
    area: Rect,
) {
    let total_lines = content.lines().count();
    let gutter_width = format!("{}", total_lines).len();

    let lines: Vec<Line> = content
        .lines()
        .enumerate()
        .map(|(i, line)| {
            let line_num = format!("{:>width$} ", i + 1, width = gutter_width);
            Line::from(vec![
                Span::styled(line_num, Style::default().fg(theme.fg_muted)),
                Span::styled(line.to_string(), Style::default().fg(theme.fg)),
            ])
        })
        .collect();

    let title = format!(" {} ({} lines) ", filename, total_lines);

    let paragraph = Paragraph::new(lines)
        .scroll((scroll as u16, 0))
        .style(Style::default().bg(theme.bg))
        .block(
            Block::default()
                .title(Span::styled(title, Style::default().fg(theme.fg)))
                .borders(Borders::ALL)
                .border_style(border_style),
        );

    frame.render_widget(paragraph, area);
}

fn format_size(bytes: u64) -> String {
    if bytes < 1024 {
        format!("{} B", bytes)
    } else if bytes < 1024 * 1024 {
        format!("{:.1} KB", bytes as f64 / 1024.0)
    } else {
        format!("{:.1} MB", bytes as f64 / (1024.0 * 1024.0))
    }
}
