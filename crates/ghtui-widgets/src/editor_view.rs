use ghtui_core::editor::TextEditor;
use ratatui::buffer::Buffer;
use ratatui::layout::Rect;
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, Paragraph, Widget, Wrap};

/// Configuration for the editor view appearance.
#[derive(Debug, Clone)]
pub struct EditorTheme {
    pub text: Color,
    pub line_number: Color,
    pub line_number_active: Color,
    pub cursor: Color,
    pub separator: Color,
    pub bg: Color,
    pub border: Color,
    pub status_bg: Color,
    pub status_fg: Color,
    pub selection_bg: Color,
    pub selection_fg: Color,
}

impl Default for EditorTheme {
    fn default() -> Self {
        Self {
            text: Color::Rgb(230, 237, 243),        // #e6edf3
            line_number: Color::Rgb(110, 118, 129), // #6e7681
            line_number_active: Color::Rgb(230, 237, 243),
            cursor: Color::Rgb(88, 166, 255),  // #58a6ff
            separator: Color::Rgb(48, 54, 61), // #30363d
            bg: Color::Rgb(22, 27, 34),        // #161b22
            border: Color::Rgb(88, 166, 255),
            status_bg: Color::Rgb(22, 27, 34),
            status_fg: Color::Rgb(125, 133, 144),
            selection_bg: Color::Rgb(56, 89, 138), // #38598a — GitHub-like selection blue
            selection_fg: Color::Rgb(230, 237, 243),
        }
    }
}

/// Fullscreen editor widget that renders a TextEditor with line numbers and cursor.
pub struct EditorView<'a> {
    editor: &'a TextEditor,
    title: &'a str,
    theme: EditorTheme,
    show_line_numbers: bool,
    show_status_bar: bool,
    status_hint: &'a str,
    /// Optional syntax-highlighted spans per line (absolute line index → spans).
    /// If provided, uses these colors instead of default text color.
    highlighted: Option<&'a [Vec<Span<'a>>]>,
}

impl<'a> EditorView<'a> {
    pub fn new(editor: &'a TextEditor, title: &'a str) -> Self {
        Self {
            editor,
            title,
            theme: EditorTheme::default(),
            show_line_numbers: true,
            show_status_bar: true,
            status_hint: "Ctrl+S: Submit  Esc: Cancel",
            highlighted: None,
        }
    }

    pub fn highlighted(mut self, spans: &'a [Vec<Span<'a>>]) -> Self {
        self.highlighted = Some(spans);
        self
    }

    pub fn theme(mut self, theme: EditorTheme) -> Self {
        self.theme = theme;
        self
    }

    pub fn line_numbers(mut self, show: bool) -> Self {
        self.show_line_numbers = show;
        self
    }

    pub fn status_bar(mut self, show: bool) -> Self {
        self.show_status_bar = show;
        self
    }

    pub fn status_hint(mut self, hint: &'a str) -> Self {
        self.status_hint = hint;
        self
    }
}

impl Widget for EditorView<'_> {
    fn render(self, area: Rect, buf: &mut Buffer) {
        if area.height < 2 {
            return;
        }

        let status_height: u16 = if self.show_status_bar { 1 } else { 0 };
        let editor_area = Rect::new(area.x, area.y, area.width, area.height - status_height);
        let status_area = if self.show_status_bar {
            Rect::new(area.x, area.y + area.height - 1, area.width, 1)
        } else {
            Rect::ZERO
        };

        // Render editor content
        let mut lines: Vec<Line<'static>> = Vec::new();

        for (i, line) in self.editor.visible_lines() {
            let is_cursor_line = self.editor.is_cursor_line(i);

            let line_num_style = if is_cursor_line {
                Style::default().fg(self.theme.line_number_active)
            } else {
                Style::default().fg(self.theme.line_number)
            };

            let mut spans: Vec<Span<'static>> = Vec::new();

            if self.show_line_numbers {
                spans.push(Span::styled(format!(" {:>3} ", i + 1), line_num_style));
                spans.push(Span::styled(
                    "│ ",
                    Style::default().fg(self.theme.separator),
                ));
            }

            let sel_range = self.editor.selection_byte_range_for_line(i);
            let has_selection = sel_range.is_some();

            if has_selection {
                // Line has selection — render with highlight
                let (sel_start, sel_end) = sel_range.unwrap();
                let sel_style = Style::default()
                    .fg(self.theme.selection_fg)
                    .bg(self.theme.selection_bg);
                let normal_style = Style::default().fg(self.theme.text);

                if is_cursor_line {
                    let byte_col = self.editor.cursor_byte_col();
                    // We need to render: before_sel | selected | after_sel, with cursor overlay
                    // Split into segments: [0..sel_start] [sel_start..sel_end] [sel_end..]
                    // The cursor block goes at byte_col position
                    let before_sel = &line[..sel_start];
                    let selected = &line[sel_start..sel_end];
                    let after_sel = &line[sel_end..];

                    if byte_col <= sel_start {
                        // Cursor is before or at selection start
                        let before_cursor = &line[..byte_col];
                        let between = &line[byte_col..sel_start];
                        spans.push(Span::styled(before_cursor.to_string(), normal_style));
                        spans.push(Span::styled(
                            "\u{2588}",
                            Style::default()
                                .fg(self.theme.cursor)
                                .add_modifier(Modifier::SLOW_BLINK),
                        ));
                        spans.push(Span::styled(between.to_string(), normal_style));
                        spans.push(Span::styled(selected.to_string(), sel_style));
                        spans.push(Span::styled(after_sel.to_string(), normal_style));
                    } else if byte_col >= sel_end {
                        // Cursor is at or after selection end
                        spans.push(Span::styled(before_sel.to_string(), normal_style));
                        spans.push(Span::styled(selected.to_string(), sel_style));
                        let between = &line[sel_end..byte_col];
                        let after_cursor = &line[byte_col..];
                        spans.push(Span::styled(between.to_string(), normal_style));
                        spans.push(Span::styled(
                            "\u{2588}",
                            Style::default()
                                .fg(self.theme.cursor)
                                .add_modifier(Modifier::SLOW_BLINK),
                        ));
                        spans.push(Span::styled(after_cursor.to_string(), normal_style));
                    } else {
                        // Cursor is inside selection
                        spans.push(Span::styled(before_sel.to_string(), normal_style));
                        let sel_before_cursor = &line[sel_start..byte_col];
                        let sel_after_cursor = &line[byte_col..sel_end];
                        spans.push(Span::styled(sel_before_cursor.to_string(), sel_style));
                        spans.push(Span::styled(
                            "\u{2588}",
                            Style::default()
                                .fg(self.theme.cursor)
                                .bg(self.theme.selection_bg)
                                .add_modifier(Modifier::SLOW_BLINK),
                        ));
                        spans.push(Span::styled(sel_after_cursor.to_string(), sel_style));
                        spans.push(Span::styled(after_sel.to_string(), normal_style));
                    }
                } else {
                    // Non-cursor line with selection
                    let before_sel = &line[..sel_start];
                    let selected = &line[sel_start..sel_end];
                    let after_sel = &line[sel_end..];
                    spans.push(Span::styled(before_sel.to_string(), normal_style));
                    spans.push(Span::styled(selected.to_string(), sel_style));
                    spans.push(Span::styled(after_sel.to_string(), normal_style));
                }
            } else if is_cursor_line {
                let byte_col = self.editor.cursor_byte_col();
                let before = &line[..byte_col];
                let after = &line[byte_col..];
                spans.push(Span::styled(
                    before.to_string(),
                    Style::default().fg(self.theme.text),
                ));
                spans.push(Span::styled(
                    "\u{2588}",
                    Style::default()
                        .fg(self.theme.cursor)
                        .add_modifier(Modifier::SLOW_BLINK),
                ));
                spans.push(Span::styled(
                    after.to_string(),
                    Style::default().fg(self.theme.text),
                ));
            } else if let Some(hl) = self.highlighted.and_then(|h| h.get(i)) {
                // Use syntax-highlighted spans (clone to 'static)
                for s in hl {
                    spans.push(Span::styled(s.content.to_string(), s.style));
                }
            } else {
                spans.push(Span::styled(
                    line.clone(),
                    Style::default().fg(self.theme.text),
                ));
            }

            lines.push(Line::from(spans));
        }

        let editor_widget = Paragraph::new(lines)
            .block(
                Block::default()
                    .title(self.title)
                    .borders(Borders::ALL)
                    .border_style(Style::default().fg(self.theme.border))
                    .style(Style::default().bg(self.theme.bg)),
            )
            .wrap(Wrap { trim: false });
        editor_widget.render(editor_area, buf);

        // Render status bar
        if self.show_status_bar {
            let total = self.editor.line_count();
            let row = self.editor.cursor_row + 1;
            let col = self.editor.cursor_col + 1;

            let status = Line::from(vec![
                Span::styled(
                    format!(" Ln {}, Col {} ", row, col),
                    Style::default().fg(self.theme.status_fg),
                ),
                Span::styled(" │ ", Style::default().fg(self.theme.separator)),
                Span::styled(
                    format!("{} lines ", total),
                    Style::default().fg(self.theme.status_fg),
                ),
                Span::styled(" │ ", Style::default().fg(self.theme.separator)),
                Span::styled(
                    self.status_hint.to_string(),
                    Style::default().fg(self.theme.status_fg),
                ),
            ]);
            let status_widget =
                Paragraph::new(status).style(Style::default().bg(self.theme.status_bg));
            status_widget.render(status_area, buf);
        }
    }
}

/// Compact inline editor (for bottom panels, comment editing).
pub struct InlineEditorView<'a> {
    editor: &'a TextEditor,
    title: &'a str,
    theme: EditorTheme,
}

impl<'a> InlineEditorView<'a> {
    pub fn new(editor: &'a TextEditor, title: &'a str) -> Self {
        Self {
            editor,
            title,
            theme: EditorTheme::default(),
        }
    }

    pub fn theme(mut self, theme: EditorTheme) -> Self {
        self.theme = theme;
        self
    }
}

impl Widget for InlineEditorView<'_> {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let mut lines: Vec<Line<'static>> = Vec::new();

        for (i, line) in self.editor.lines.iter().enumerate() {
            let is_cursor_line = i == self.editor.cursor_row;
            let sel_range = self.editor.selection_byte_range_for_line(i);
            let has_selection = sel_range.is_some();
            let sel_style = Style::default()
                .fg(self.theme.selection_fg)
                .bg(self.theme.selection_bg);
            let normal_style = Style::default().fg(self.theme.text);

            if has_selection {
                let (sel_start, sel_end) = sel_range.unwrap();
                let mut spans: Vec<Span<'static>> = Vec::new();
                spans.push(Span::styled("  ".to_string(), normal_style));

                if is_cursor_line {
                    let byte_col = self.editor.cursor_byte_col();
                    let before_sel = &line[..sel_start];
                    let selected = &line[sel_start..sel_end];
                    let after_sel = &line[sel_end..];

                    if byte_col <= sel_start {
                        let before_cursor = &line[..byte_col];
                        let between = &line[byte_col..sel_start];
                        spans.push(Span::styled(before_cursor.to_string(), normal_style));
                        spans.push(Span::styled(
                            "\u{2588}",
                            Style::default()
                                .fg(self.theme.cursor)
                                .add_modifier(Modifier::SLOW_BLINK),
                        ));
                        spans.push(Span::styled(between.to_string(), normal_style));
                        spans.push(Span::styled(selected.to_string(), sel_style));
                        spans.push(Span::styled(after_sel.to_string(), normal_style));
                    } else if byte_col >= sel_end {
                        spans.push(Span::styled(before_sel.to_string(), normal_style));
                        spans.push(Span::styled(selected.to_string(), sel_style));
                        let between = &line[sel_end..byte_col];
                        let after_cursor = &line[byte_col..];
                        spans.push(Span::styled(between.to_string(), normal_style));
                        spans.push(Span::styled(
                            "\u{2588}",
                            Style::default()
                                .fg(self.theme.cursor)
                                .add_modifier(Modifier::SLOW_BLINK),
                        ));
                        spans.push(Span::styled(after_cursor.to_string(), normal_style));
                    } else {
                        spans.push(Span::styled(before_sel.to_string(), normal_style));
                        let sel_before_cursor = &line[sel_start..byte_col];
                        let sel_after_cursor = &line[byte_col..sel_end];
                        spans.push(Span::styled(sel_before_cursor.to_string(), sel_style));
                        spans.push(Span::styled(
                            "\u{2588}",
                            Style::default()
                                .fg(self.theme.cursor)
                                .bg(self.theme.selection_bg)
                                .add_modifier(Modifier::SLOW_BLINK),
                        ));
                        spans.push(Span::styled(sel_after_cursor.to_string(), sel_style));
                        spans.push(Span::styled(after_sel.to_string(), normal_style));
                    }
                } else {
                    let before_sel = &line[..sel_start];
                    let selected = &line[sel_start..sel_end];
                    let after_sel = &line[sel_end..];
                    spans.push(Span::styled(before_sel.to_string(), normal_style));
                    spans.push(Span::styled(selected.to_string(), sel_style));
                    spans.push(Span::styled(after_sel.to_string(), normal_style));
                }
                lines.push(Line::from(spans));
            } else if is_cursor_line {
                let byte_col = self.editor.cursor_byte_col();
                let before = &line[..byte_col];
                let after = &line[byte_col..];
                lines.push(Line::from(vec![
                    Span::styled(
                        format!("  {}", before),
                        Style::default().fg(self.theme.text),
                    ),
                    Span::styled(
                        "\u{2588}",
                        Style::default()
                            .fg(self.theme.cursor)
                            .add_modifier(Modifier::SLOW_BLINK),
                    ),
                    Span::styled(after.to_string(), Style::default().fg(self.theme.text)),
                ]));
            } else {
                lines.push(Line::styled(
                    format!("  {}", line),
                    Style::default().fg(self.theme.text),
                ));
            }
        }

        lines.push(Line::raw(""));
        lines.push(Line::from(vec![Span::styled(
            "  Ctrl+S: Submit  Esc: Cancel",
            Style::default().fg(self.theme.status_fg),
        )]));

        let widget = Paragraph::new(lines)
            .block(
                Block::default()
                    .title(self.title)
                    .borders(Borders::ALL)
                    .border_style(Style::default().fg(self.theme.border))
                    .style(Style::default().bg(self.theme.bg)),
            )
            .wrap(Wrap { trim: false });
        widget.render(area, buf);
    }
}
