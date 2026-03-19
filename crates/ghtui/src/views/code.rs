use super::components;
use ghtui_core::AppState;
use ghtui_core::types::code::CommitDetail;
use ratatui::Frame;
use ratatui::layout::{Constraint, Direction, Layout, Rect};
use ratatui::style::{Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, Clear, List, ListItem, ListState, Paragraph, Wrap};

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

    // Fullscreen editor mode
    if code.editing {
        let title = format!(" Edit: {} ", code.file_name.as_deref().unwrap_or("file"));
        let filename = code.file_name.as_deref().unwrap_or("");
        let is_dark = theme.bg == ratatui::style::Color::Rgb(13, 17, 23);
        let content = code.editor.content();
        let hl_spans = crate::highlighter::highlight_file(&content, filename, is_dark);
        let widget = ghtui_widgets::EditorView::new(&code.editor, &title)
            .highlighted(&hl_spans)
            .status_hint("Ctrl+S:Commit  Ctrl+Z:Undo  Ctrl+Y:Redo  Ctrl+C/V/X:Copy/Paste/Cut  Shift+Arrow:Select  Esc:Cancel");
        frame.render_widget(widget, area);
        return;
    }

    let show_sidebar = area.width >= 80;

    if show_sidebar {
        // Horizontal split: sidebar (35) | content (rest)
        let chunks = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([Constraint::Length(35), Constraint::Min(0)])
            .split(area);

        if code.show_commits {
            render_commit_list(frame, state, code, chunks[0]);
        } else {
            render_file_tree(frame, state, code, chunks[0]);
        }

        if let Some(ref detail) = code.commit_detail {
            render_commit_detail(frame, state, detail, code.commit_scroll, chunks[1]);
        } else {
            render_content(frame, state, code, chunks[1]);
        }
    } else {
        // Narrow: show only content (or commit detail)
        if let Some(ref detail) = code.commit_detail {
            render_commit_detail(frame, state, detail, code.commit_scroll, area);
        } else {
            render_content(frame, state, code, area);
        }
    }

    // Ref picker overlay
    if code.ref_picker_open {
        render_ref_picker(frame, state, code, area);
    }
}

fn render_file_tree(
    frame: &mut Frame,
    state: &AppState,
    code: &ghtui_core::state::CodeViewState,
    area: Rect,
) {
    let theme = &state.theme;

    let ref_label = if code.git_ref.is_empty() {
        String::new()
    } else {
        format!(" [{}]", code.git_ref)
    };

    let title = format!(" /{} ", ref_label);

    let loading = state.is_loading("code_contents") || state.is_loading("code_tree");

    let list_items: Vec<ListItem> = if loading {
        vec![ListItem::new(Line::from(Span::styled(
            "  Loading...",
            Style::default().fg(theme.fg_dim),
        )))]
    } else if code.tree_loaded {
        // Tree view mode
        if code.tree_visible.is_empty() {
            vec![ListItem::new(Line::from(Span::styled(
                "  (empty repository)",
                Style::default().fg(theme.fg_dim),
            )))]
        } else {
            code.tree_visible
                .iter()
                .enumerate()
                .map(|(vi, &ti)| {
                    let node = &code.tree[ti];
                    let indent = "  ".repeat(node.depth);

                    let (arrow, icon) = if node.is_dir {
                        if code.expanded_dirs.contains(&node.path) {
                            ("▾ ", "")
                        } else {
                            ("▸ ", "")
                        }
                    } else {
                        ("  ", "")
                    };

                    let size_str = if !node.is_dir {
                        node.size.map(format_size).unwrap_or_default()
                    } else {
                        String::new()
                    };

                    let is_selected = vi == code.selected;
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
                    } else if node.is_dir {
                        Style::default().fg(theme.accent)
                    } else {
                        Style::default().fg(theme.fg)
                    };

                    let arrow_style = Style::default().fg(theme.fg_muted);

                    let spans = vec![
                        Span::styled(
                            format!(" {}{}{}", indent, arrow, icon),
                            if is_selected { style } else { arrow_style },
                        ),
                        Span::styled(node.name.clone(), style),
                        Span::styled(
                            if size_str.is_empty() {
                                String::new()
                            } else {
                                format!(" {}", size_str)
                            },
                            Style::default().fg(theme.fg_dim),
                        ),
                    ];

                    ListItem::new(Line::from(spans))
                })
                .collect()
        }
    } else if code.entries.is_empty() {
        vec![ListItem::new(Line::from(Span::styled(
            "  (empty directory)",
            Style::default().fg(theme.fg_dim),
        )))]
    } else {
        // Flat list fallback
        code.entries
            .iter()
            .enumerate()
            .map(|(i, entry)| {
                let icon = if entry.entry_type == ghtui_core::types::code::FileEntryType::Dir {
                    "▸ "
                } else {
                    "  "
                };

                let size_str = if entry.entry_type == ghtui_core::types::code::FileEntryType::File {
                    entry.size.map(format_size).unwrap_or_default()
                } else {
                    String::new()
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
                } else if entry.entry_type == ghtui_core::types::code::FileEntryType::Dir {
                    Style::default().fg(theme.accent)
                } else {
                    Style::default().fg(theme.fg)
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

    let border_style = if code.sidebar_focused && !code.show_commits {
        Style::default().fg(theme.accent)
    } else {
        theme.border_style()
    };

    let item_count = if code.tree_loaded {
        code.tree_visible.len()
    } else {
        code.entries.len()
    };

    let list = List::new(list_items).block(
        Block::default()
            .title(Span::styled(title, Style::default().fg(theme.fg)))
            .borders(Borders::ALL)
            .border_style(border_style),
    );

    let mut list_state = ListState::default();
    if item_count > 0 {
        list_state.select(Some(code.selected));
    }
    frame.render_stateful_widget(list, area, &mut list_state);
}

fn render_commit_list(
    frame: &mut Frame,
    state: &AppState,
    code: &ghtui_core::state::CodeViewState,
    area: Rect,
) {
    let theme = &state.theme;
    let loading = state.is_loading("code_commits");

    let title = format!(" Commits [{}] ", code.git_ref);

    let list_items: Vec<ListItem> = if loading {
        vec![ListItem::new(Line::from(Span::styled(
            "  Loading commits...",
            Style::default().fg(theme.fg_dim),
        )))]
    } else if code.commits.is_empty() {
        vec![ListItem::new(Line::from(Span::styled(
            "  No commits",
            Style::default().fg(theme.fg_dim),
        )))]
    } else {
        code.commits
            .iter()
            .enumerate()
            .map(|(i, commit)| {
                let is_selected = i == code.commit_selected;
                let style = if is_selected {
                    Style::default()
                        .fg(theme.tab_active_fg)
                        .add_modifier(Modifier::BOLD)
                        .bg(theme.selection_bg)
                } else {
                    Style::default().fg(theme.fg)
                };
                let sha_style = if is_selected {
                    style
                } else {
                    Style::default().fg(theme.accent)
                };
                let dim_style = if is_selected {
                    style
                } else {
                    Style::default().fg(theme.fg_dim)
                };

                let short_sha = if commit.sha.len() > 7 {
                    &commit.sha[..7]
                } else {
                    &commit.sha
                };

                // Truncate message to fit
                let max_msg = area.width.saturating_sub(12) as usize;
                let msg = if commit.message.len() > max_msg {
                    format!("{}...", &commit.message[..max_msg.saturating_sub(3)])
                } else {
                    commit.message.clone()
                };

                let relative = super::components::time_ago_rfc3339(&commit.date);

                ListItem::new(vec![
                    Line::from(vec![
                        Span::styled(format!(" {} ", short_sha), sha_style),
                        Span::styled(msg, style),
                    ]),
                    Line::from(vec![Span::styled(
                        format!("   {} - {}", commit.author, relative),
                        dim_style,
                    )]),
                ])
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
    if !code.commits.is_empty() {
        list_state.select(Some(code.commit_selected));
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

    // If viewing an image file
    if let (Some(image_bytes), Some(filename)) = (&code.image_data, &code.file_name) {
        render_image_preview(frame, theme, image_bytes, filename, border_style, area);
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

fn render_commit_detail(
    frame: &mut Frame,
    state: &AppState,
    detail: &CommitDetail,
    scroll: usize,
    area: Rect,
) {
    let theme = &state.theme;
    let border_style = Style::default().fg(theme.accent);

    let short_sha = if detail.sha.len() > 10 {
        &detail.sha[..10]
    } else {
        &detail.sha
    };

    let title = format!(" Commit {} ", short_sha);

    let mut lines: Vec<Line> = vec![
        Line::from(vec![
            Span::styled("  SHA: ", Style::default().fg(theme.fg_dim)),
            Span::styled(&detail.sha, Style::default().fg(theme.accent)),
        ]),
        Line::from(vec![
            Span::styled("  Author: ", Style::default().fg(theme.fg_dim)),
            Span::styled(&detail.author, Style::default().fg(theme.fg)),
        ]),
        Line::from(vec![
            Span::styled("  Date: ", Style::default().fg(theme.fg_dim)),
            Span::styled(
                super::components::time_ago_rfc3339(&detail.date),
                Style::default().fg(theme.fg),
            ),
        ]),
        Line::from(""),
    ];

    // Full message
    for msg_line in detail.message.lines() {
        lines.push(Line::from(Span::styled(
            format!("  {}", msg_line),
            Style::default().fg(theme.fg),
        )));
    }
    lines.push(Line::from(""));

    // Stats
    lines.push(Line::from(vec![
        Span::styled("  ", Style::default()),
        Span::styled(
            format!("+{}", detail.additions),
            Style::default().fg(theme.accent),
        ),
        Span::styled(" / ", Style::default().fg(theme.fg_dim)),
        Span::styled(
            format!("-{}", detail.deletions),
            Style::default().fg(theme.danger),
        ),
        Span::styled(
            format!("  ({} files changed)", detail.files.len()),
            Style::default().fg(theme.fg_dim),
        ),
    ]));
    lines.push(Line::from(""));

    // Files
    for file in &detail.files {
        use ghtui_core::types::code::FileChangeStatus;
        let status_style = match &file.status {
            FileChangeStatus::Added => Style::default().fg(theme.accent),
            FileChangeStatus::Removed => Style::default().fg(theme.danger),
            _ => Style::default().fg(theme.warning),
        };
        let status_char = file.status.label();

        lines.push(Line::from(vec![
            Span::styled(format!("  {} ", status_char), status_style),
            Span::styled(&file.filename, Style::default().fg(theme.fg)),
            Span::styled(
                format!("  +{} -{}", file.additions, file.deletions),
                Style::default().fg(theme.fg_dim),
            ),
        ]));
    }

    let paragraph = Paragraph::new(lines)
        .scroll((scroll.min(u16::MAX as usize) as u16, 0))
        .style(Style::default().bg(theme.bg))
        .block(
            Block::default()
                .title(Span::styled(title, Style::default().fg(theme.fg)))
                .borders(Borders::ALL)
                .border_style(border_style),
        )
        .wrap(Wrap { trim: false });

    frame.render_widget(paragraph, area);
}

fn render_ref_picker(
    frame: &mut Frame,
    state: &AppState,
    code: &ghtui_core::state::CodeViewState,
    area: Rect,
) {
    let theme = &state.theme;

    // Center the picker
    let width = 50u16.min(area.width.saturating_sub(4));
    let height = 20u16.min(area.height.saturating_sub(4));
    let x = area.x + (area.width.saturating_sub(width)) / 2;
    let y = area.y + (area.height.saturating_sub(height)) / 2;
    let picker_area = Rect::new(x, y, width, height);

    frame.render_widget(Clear, picker_area);

    let items: Vec<ListItem> = code
        .ref_picker_items
        .iter()
        .enumerate()
        .map(|(i, (name, is_branch))| {
            let is_selected = i == code.ref_picker_selected;
            let prefix = if *is_branch { "branch" } else { "tag" };
            let is_current = name == &code.git_ref;

            let style = if is_selected {
                Style::default()
                    .fg(theme.tab_active_fg)
                    .add_modifier(Modifier::BOLD)
                    .bg(theme.selection_bg)
            } else if is_current {
                Style::default()
                    .fg(theme.accent)
                    .add_modifier(Modifier::BOLD)
            } else {
                Style::default().fg(theme.fg)
            };
            let dim = if is_selected {
                style
            } else {
                Style::default().fg(theme.fg_dim)
            };

            let current_marker = if is_current { " *" } else { "" };

            ListItem::new(Line::from(vec![
                Span::styled(format!(" {} ", prefix), dim),
                Span::styled(format!("{}{}", name, current_marker), style),
            ]))
        })
        .collect();

    let border_style = Style::default().fg(theme.accent);
    let title = format!(" Switch branch/tag ({}) ", code.ref_picker_items.len());

    let list = List::new(items).block(
        Block::default()
            .title(Span::styled(title, Style::default().fg(theme.fg)))
            .borders(Borders::ALL)
            .border_style(border_style),
    );

    let mut list_state = ListState::default();
    if !code.ref_picker_items.is_empty() {
        list_state.select(Some(code.ref_picker_selected));
    }
    frame.render_stateful_widget(list, picker_area, &mut list_state);
}

fn render_image_preview(
    frame: &mut Frame,
    theme: &ghtui_core::theme::Theme,
    image_bytes: &[u8],
    filename: &str,
    border_style: Style,
    area: Rect,
) {
    let block = Block::default()
        .title(Span::styled(
            format!(" {} (image) ", filename),
            Style::default().fg(theme.fg),
        ))
        .borders(Borders::ALL)
        .border_style(border_style);

    let inner = block.inner(area);
    frame.render_widget(block, area);

    // Cache the decoded DynamicImage to avoid re-decoding every frame.
    // Protocol creation is cheap (halfblocks), image decoding is expensive.
    use std::cell::RefCell;
    type ImgCache = Option<(usize, image::DynamicImage)>;
    thread_local! {
        static DECODED_IMG: RefCell<ImgCache> = const { RefCell::new(None) };
    }

    let bytes_len = image_bytes.len();

    // Get or decode image
    let decoded = DECODED_IMG.with(|c| {
        let borrow = c.borrow();
        if let Some((len, ref img)) = *borrow {
            if len == bytes_len {
                return Some(img.clone());
            }
        }
        None
    });

    let dyn_img = if let Some(img) = decoded {
        Some(img)
    } else if let Ok(img) = image::load_from_memory(image_bytes) {
        DECODED_IMG.with(|c| {
            *c.borrow_mut() = Some((bytes_len, img.clone()));
        });
        Some(img)
    } else {
        None
    };

    if let Some(img) = dyn_img {
        let picker = ratatui_image::picker::Picker::halfblocks();
        let resize = ratatui_image::Resize::Fit(None);
        match picker.new_protocol(img, inner, resize) {
            Ok(protocol) => {
                let image_widget = ratatui_image::Image::new(&protocol);
                frame.render_widget(image_widget, inner);
            }
            Err(_) => {
                let fallback =
                    Paragraph::new(format!("  [Image: {} - unable to render]", filename))
                        .style(Style::default().fg(theme.fg_dim));
                frame.render_widget(fallback, inner);
            }
        }
    } else {
        let info = format!(
            "  [Image: {} ({} bytes) - unable to decode]",
            filename,
            image_bytes.len()
        );
        let fallback = Paragraph::new(info).style(Style::default().fg(theme.fg_dim));
        frame.render_widget(fallback, inner);
    }
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

    // Syntax highlighting (Tree-sitter for 26 languages)
    let is_dark = theme.bg == ratatui::style::Color::Rgb(13, 17, 23);
    let highlighted = crate::highlighter::highlight_file(content, filename, is_dark);

    // Viewport-aware lazy rendering: only create Line objects for visible range
    let visible_height = area.height.saturating_sub(2) as usize; // minus top/bottom borders
    let buffer = 20;
    let start = scroll;
    let end = (scroll + visible_height + buffer).min(total_lines);

    let lines: Vec<Line> = content
        .lines()
        .enumerate()
        .skip(start)
        .take(end - start)
        .map(|(i, line)| {
            let line_num = format!("{:>width$} ", i + 1, width = gutter_width);
            let mut spans = vec![Span::styled(line_num, Style::default().fg(theme.fg_muted))];

            if let Some(hl_spans) = highlighted.get(i) {
                spans.extend(hl_spans.iter().cloned());
            } else {
                spans.push(Span::styled(
                    line.to_string(),
                    Style::default().fg(theme.fg),
                ));
            }

            Line::from(spans)
        })
        .collect();

    let title = format!(" {} ({} lines) ", filename, total_lines);

    // No .scroll() needed — we already sliced to the visible window
    let paragraph = Paragraph::new(lines)
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
