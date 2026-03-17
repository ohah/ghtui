use std::collections::HashSet;

use ghtui_core::editor::TextEditor;
use ghtui_core::theme::Theme;
use ghtui_core::types::{DiffFile, DiffFileStatus, DiffLineKind, ReviewComment};
use ratatui::buffer::Buffer;
use ratatui::layout::Rect;
use ratatui::style::{Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Paragraph, StatefulWidget, Widget};

const BOX_PAD: &str = "          ";
const BOX_INDENT: usize = 12; // BOX_PAD(10) + "│ "(2)

/// ╭─ label ─────────╮  (border_style for box chars, label keeps its own style)
fn box_top(label_spans: Vec<Span<'static>>, border: Style, width: usize) -> Line<'static> {
    let content_width = width.saturating_sub(BOX_INDENT + 1); // -1 for ╮
    let label_len: usize = label_spans.iter().map(|s| s.content.chars().count()).sum();
    let fill = content_width.saturating_sub(label_len + 1);
    let mut spans = vec![Span::styled(format!("{}╭─", BOX_PAD), border)];
    spans.extend(label_spans);
    spans.push(Span::styled("─".repeat(fill), border));
    spans.push(Span::styled("╮", border));
    Line::from(spans)
}

/// │ content          │  (padded to width, border chars use border_style)
fn box_mid(content: Vec<Span<'static>>, border: Style, width: usize) -> Line<'static> {
    let content_width = width.saturating_sub(BOX_INDENT + 2); // -2 for " │"
    let actual: usize = content.iter().map(|s| s.content.chars().count()).sum();
    let padding = content_width.saturating_sub(actual);
    let mut result = vec![Span::styled(format!("{}│ ", BOX_PAD), border)];
    result.extend(content);
    if padding > 0 {
        result.push(Span::raw(" ".repeat(padding)));
    }
    result.push(Span::styled("│", border));
    Line::from(result)
}

/// ╰─────────────────╯
fn box_btm(border: Style, width: usize) -> Line<'static> {
    let content_width = width.saturating_sub(BOX_INDENT + 1);
    Line::from(vec![
        Span::styled(format!("{}╰─", BOX_PAD), border),
        Span::styled("─".repeat(content_width), border),
        Span::styled("╯", border),
    ])
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DiffLineId {
    Summary,
    FileHeader(usize),
    HunkHeader(usize, usize),
    Code(usize, usize, usize),
}

#[derive(Debug, Default)]
pub struct DiffViewState {
    pub scroll: usize,
    pub cursor: usize,
    pub total_lines: usize,
    pub show_all_files: bool,
    pub collapsed_files: HashSet<usize>,
    pub select_anchor: Option<usize>,
    pub line_ids: Vec<DiffLineId>,
}

impl DiffViewState {
    pub fn cursor_down(&mut self) {
        if self.cursor < self.total_lines.saturating_sub(1) {
            self.cursor += 1;
        }
    }
    pub fn cursor_up(&mut self) {
        self.cursor = self.cursor.saturating_sub(1);
    }
    pub fn cursor_down_select(&mut self) {
        if self.select_anchor.is_none() {
            self.select_anchor = Some(self.cursor);
        }
        self.cursor_down();
    }
    pub fn cursor_up_select(&mut self) {
        if self.select_anchor.is_none() {
            self.select_anchor = Some(self.cursor);
        }
        self.cursor_up();
    }
    pub fn clear_selection(&mut self) {
        self.select_anchor = None;
    }
    pub fn selection_range(&self) -> Option<(usize, usize)> {
        self.select_anchor
            .map(|anchor| (anchor.min(self.cursor), anchor.max(self.cursor)))
    }
    pub fn toggle_file_collapse(&mut self) {
        if let Some(id) = self.line_ids.get(self.cursor) {
            let file_idx = match id {
                DiffLineId::FileHeader(f)
                | DiffLineId::HunkHeader(f, _)
                | DiffLineId::Code(f, _, _) => Some(*f),
                _ => None,
            };
            if let Some(idx) = file_idx {
                if !self.collapsed_files.remove(&idx) {
                    self.collapsed_files.insert(idx);
                }
            }
        }
    }
    pub fn page_down(&mut self, page_size: usize) {
        self.cursor = (self.cursor + page_size).min(self.total_lines.saturating_sub(1));
    }
    pub fn page_up(&mut self, page_size: usize) {
        self.cursor = self.cursor.saturating_sub(page_size);
    }
}

pub struct DiffView<'a> {
    files: &'a [DiffFile],
    review_comments: &'a [ReviewComment],
    theme: &'a Theme,
    block: Option<Block<'a>>,
    comment_editor: Option<(&'a str, u32, &'a TextEditor)>,
}

impl<'a> DiffView<'a> {
    pub fn new(files: &'a [DiffFile], theme: &'a Theme) -> Self {
        Self {
            files,
            review_comments: &[],
            theme,
            block: None,
            comment_editor: None,
        }
    }
    pub fn review_comments(mut self, comments: &'a [ReviewComment]) -> Self {
        self.review_comments = comments;
        self
    }
    pub fn comment_editor(mut self, path: &'a str, line: u32, editor: &'a TextEditor) -> Self {
        self.comment_editor = Some((path, line, editor));
        self
    }
    pub fn block(mut self, block: Block<'a>) -> Self {
        self.block = block.into();
        self
    }

    fn render_comment_body(
        body: &str,
        theme: &Theme,
        border_style: Style,
        width: usize,
        lines: &mut Vec<Line<'static>>,
        line_ids: &mut Vec<DiffLineId>,
    ) {
        let mut in_suggestion = false;
        let mut in_code_block = false;

        for body_line in body.lines() {
            if body_line.trim() == "```suggestion" {
                in_suggestion = true;
                lines.push(box_mid(
                    vec![Span::styled(
                        " 💡 Suggested change:",
                        Style::default()
                            .fg(theme.diff_add_fg)
                            .add_modifier(Modifier::BOLD),
                    )],
                    border_style,
                    width,
                ));
                line_ids.push(DiffLineId::Summary);
                continue;
            }
            if in_suggestion && body_line.trim() == "```" {
                in_suggestion = false;
                continue;
            }
            if !in_suggestion && body_line.starts_with("```") {
                in_code_block = !in_code_block;
                lines.push(box_mid(
                    vec![Span::styled(
                        body_line.to_string(),
                        Style::default().fg(theme.fg_muted),
                    )],
                    border_style,
                    width,
                ));
                line_ids.push(DiffLineId::Summary);
                continue;
            }

            if in_suggestion {
                lines.push(box_mid(
                    vec![Span::styled(
                        format!(" +{}", body_line),
                        Style::default().fg(theme.diff_add_fg).bg(theme.diff_add_bg),
                    )],
                    border_style,
                    width,
                ));
            } else if in_code_block {
                lines.push(box_mid(
                    vec![Span::styled(
                        format!(" {}", body_line),
                        Style::default().fg(theme.fg).bg(theme.bg_subtle),
                    )],
                    border_style,
                    width,
                ));
            } else {
                lines.push(box_mid(
                    vec![Span::styled(
                        body_line.to_string(),
                        Style::default().fg(theme.fg),
                    )],
                    border_style,
                    width,
                ));
            }
            line_ids.push(DiffLineId::Summary);
        }
    }

    fn render_file_summary(
        &self,
        lines: &mut Vec<Line<'static>>,
        line_ids: &mut Vec<DiffLineId>,
        state: &DiffViewState,
    ) {
        let total_adds: u32 = self.files.iter().map(|f| f.additions).sum();
        let total_dels: u32 = self.files.iter().map(|f| f.deletions).sum();
        let theme = self.theme;

        lines.push(Line::from(vec![
            Span::styled(
                format!(" {} files changed", self.files.len()),
                Style::default().fg(theme.fg).add_modifier(Modifier::BOLD),
            ),
            Span::styled(
                format!("  +{}", total_adds),
                Style::default().fg(theme.diff_add_fg),
            ),
            Span::styled(
                format!("  -{}", total_dels),
                Style::default().fg(theme.diff_remove_fg),
            ),
            Span::styled(
                "  (Enter:comment/fold  h/l:fold  J/K:select)",
                Style::default().fg(theme.fg_muted),
            ),
        ]));
        line_ids.push(DiffLineId::Summary);
        lines.push(Line::raw(""));
        line_ids.push(DiffLineId::Summary);

        for (fi, file) in self.files.iter().enumerate() {
            let collapsed = state.collapsed_files.contains(&fi);
            let fold_icon = if collapsed { "▸ " } else { "▾ " };
            let status_icon = match file.status {
                DiffFileStatus::Added => Span::styled("A ", Style::default().fg(theme.diff_add_fg)),
                DiffFileStatus::Removed => {
                    Span::styled("D ", Style::default().fg(theme.diff_remove_fg))
                }
                DiffFileStatus::Modified => Span::styled("M ", Style::default().fg(theme.warning)),
                DiffFileStatus::Renamed => Span::styled("R ", Style::default().fg(theme.info)),
            };
            let total_changes = file.additions + file.deletions;
            let bar_width = 20u32;
            let (add_blocks, del_blocks) = if total_changes > 0 {
                let a = ((file.additions as f64 / total_changes as f64) * bar_width as f64).round()
                    as u32;
                (a.max(if file.additions > 0 { 1 } else { 0 }), bar_width - a)
            } else {
                (0, 0)
            };
            let mut spans = vec![
                Span::styled(fold_icon, Style::default().fg(theme.fg_dim)),
                status_icon,
                Span::styled(
                    format!("{:<40} ", file.filename),
                    Style::default().fg(theme.fg),
                ),
                Span::styled(
                    format!("{:>3} ", total_changes),
                    Style::default().fg(theme.fg_dim),
                ),
            ];
            if add_blocks > 0 {
                spans.push(Span::styled(
                    "█".repeat(add_blocks as usize),
                    Style::default().fg(theme.diff_add_fg),
                ));
            }
            if del_blocks > 0 {
                spans.push(Span::styled(
                    "█".repeat(del_blocks as usize),
                    Style::default().fg(theme.diff_remove_fg),
                ));
            }
            lines.push(Line::from(spans));
            line_ids.push(DiffLineId::Summary);
        }
        lines.push(Line::raw(""));
        line_ids.push(DiffLineId::Summary);
    }

    fn render_file_diff(
        &self,
        file: &DiffFile,
        file_idx: usize,
        lines: &mut Vec<Line<'static>>,
        line_ids: &mut Vec<DiffLineId>,
        state: &DiffViewState,
        width: usize,
    ) {
        let theme = self.theme;
        let collapsed = state.collapsed_files.contains(&file_idx);
        let status_label = match file.status {
            DiffFileStatus::Added => " ADDED ",
            DiffFileStatus::Removed => " DELETED ",
            DiffFileStatus::Modified => " MODIFIED ",
            DiffFileStatus::Renamed => " RENAMED ",
        };
        let status_color = match file.status {
            DiffFileStatus::Added => theme.diff_add_fg,
            DiffFileStatus::Removed => theme.diff_remove_fg,
            DiffFileStatus::Modified => theme.warning,
            DiffFileStatus::Renamed => theme.info,
        };
        let fold_icon = if collapsed { "▸" } else { "▾" };

        lines.push(Line::from(vec![
            Span::styled(
                format!(" {} ", fold_icon),
                Style::default().fg(theme.fg_dim),
            ),
            Span::styled(
                status_label,
                Style::default()
                    .fg(theme.bg)
                    .bg(status_color)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::styled(
                format!(" {} ", file.filename),
                Style::default()
                    .fg(theme.fg)
                    .bg(theme.bg_subtle)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::styled(
                format!(" +{} -{} ", file.additions, file.deletions),
                Style::default().fg(theme.fg_dim).bg(theme.bg_subtle),
            ),
        ]));
        line_ids.push(DiffLineId::FileHeader(file_idx));

        if collapsed {
            return;
        }

        let border_style = Style::default().fg(theme.border);
        let accent_style = Style::default().fg(theme.accent);

        for (hi, hunk) in file.hunks.iter().enumerate() {
            lines.push(Line::styled(
                format!(" {}", hunk.header),
                Style::default()
                    .fg(theme.diff_hunk)
                    .bg(theme.bg_subtle)
                    .add_modifier(Modifier::ITALIC),
            ));
            line_ids.push(DiffLineId::HunkHeader(file_idx, hi));

            for (li, diff_line) in hunk.lines.iter().enumerate() {
                let old_ln = diff_line
                    .old_line
                    .map(|n| format!("{:>4}", n))
                    .unwrap_or_else(|| "    ".to_string());
                let new_ln = diff_line
                    .new_line
                    .map(|n| format!("{:>4}", n))
                    .unwrap_or_else(|| "    ".to_string());

                match diff_line.kind {
                    DiffLineKind::Add => {
                        lines.push(Line::from(vec![
                            Span::styled(
                                format!("{}  {} ", old_ln, new_ln),
                                Style::default().fg(theme.diff_add_fg).bg(theme.diff_add_bg),
                            ),
                            Span::styled(
                                format!("+{}", diff_line.content),
                                Style::default().fg(theme.diff_add_fg).bg(theme.diff_add_bg),
                            ),
                        ]));
                    }
                    DiffLineKind::Remove => {
                        lines.push(Line::from(vec![
                            Span::styled(
                                format!("{}  {} ", old_ln, new_ln),
                                Style::default()
                                    .fg(theme.diff_remove_fg)
                                    .bg(theme.diff_remove_bg),
                            ),
                            Span::styled(
                                format!("-{}", diff_line.content),
                                Style::default()
                                    .fg(theme.diff_remove_fg)
                                    .bg(theme.diff_remove_bg),
                            ),
                        ]));
                    }
                    DiffLineKind::Context => {
                        lines.push(Line::from(vec![
                            Span::styled(
                                format!("{}  {} ", old_ln, new_ln),
                                Style::default().fg(theme.fg_muted),
                            ),
                            Span::styled(
                                format!(" {}", diff_line.content),
                                Style::default().fg(theme.fg_dim),
                            ),
                        ]));
                    }
                    DiffLineKind::Header => {
                        lines.push(Line::styled(
                            format!("{}{}", BOX_PAD, diff_line.content),
                            Style::default().fg(theme.diff_hunk),
                        ));
                    }
                }
                line_ids.push(DiffLineId::Code(file_idx, hi, li));

                // Review comments on this line
                let target_line = diff_line.new_line.or(diff_line.old_line);
                if let Some(ln) = target_line {
                    let matching: Vec<&ReviewComment> = self
                        .review_comments
                        .iter()
                        .filter(|rc| {
                            rc.path == file.filename
                                && (rc.line == Some(ln) || rc.original_line == Some(ln))
                        })
                        .collect();

                    let roots: Vec<&&ReviewComment> = matching
                        .iter()
                        .filter(|rc| rc.in_reply_to_id.is_none())
                        .collect();

                    for root in &roots {
                        lines.push(box_top(
                            vec![
                                Span::styled(
                                    format!(" @{}", root.user.login),
                                    Style::default()
                                        .fg(theme.accent)
                                        .add_modifier(Modifier::BOLD),
                                ),
                                Span::styled(
                                    format!("  {} ", root.created_at.format("%m/%d %H:%M")),
                                    Style::default().fg(theme.fg_muted),
                                ),
                            ],
                            border_style,
                            width,
                        ));
                        line_ids.push(DiffLineId::Summary);

                        Self::render_comment_body(
                            &root.body,
                            theme,
                            border_style,
                            width,
                            lines,
                            line_ids,
                        );

                        let replies: Vec<&&ReviewComment> = matching
                            .iter()
                            .filter(|rc| rc.in_reply_to_id == Some(root.id))
                            .collect();
                        for reply in &replies {
                            lines.push(box_mid(
                                vec![
                                    Span::styled(
                                        format!("↳ @{} ", reply.user.login),
                                        Style::default().fg(theme.accent),
                                    ),
                                    Span::styled(
                                        reply.body.lines().next().unwrap_or("").to_string(),
                                        Style::default().fg(theme.fg_dim),
                                    ),
                                ],
                                border_style,
                                width,
                            ));
                            line_ids.push(DiffLineId::Summary);
                        }

                        lines.push(box_btm(border_style, width));
                        line_ids.push(DiffLineId::Summary);
                    }

                    // Orphan comments
                    for orphan in matching.iter().filter(|rc| {
                        rc.in_reply_to_id.is_some()
                            && !roots.iter().any(|r| Some(r.id) == rc.in_reply_to_id)
                    }) {
                        lines.push(box_top(
                            vec![Span::styled(
                                format!(" @{} ", orphan.user.login),
                                Style::default().fg(theme.accent),
                            )],
                            border_style,
                            width,
                        ));
                        line_ids.push(DiffLineId::Summary);
                        Self::render_comment_body(
                            &orphan.body,
                            theme,
                            border_style,
                            width,
                            lines,
                            line_ids,
                        );
                        lines.push(box_btm(border_style, width));
                        line_ids.push(DiffLineId::Summary);
                    }

                    // Inline comment editor
                    if let Some((editor_path, editor_line, editor)) = &self.comment_editor {
                        if *editor_path == file.filename && *editor_line == ln {
                            lines.push(box_top(
                                vec![Span::styled(
                                    " Review comment  Ctrl+Enter:submit  Esc:cancel  Ctrl+S:suggestion ",
                                    Style::default().fg(theme.accent).add_modifier(Modifier::BOLD),
                                )],
                                accent_style,
                                width,
                            ));
                            line_ids.push(DiffLineId::Summary);

                            for (ei, editor_line_text) in editor.lines.iter().enumerate() {
                                let is_cursor_line = ei == editor.cursor_row;
                                if is_cursor_line {
                                    let col = editor.cursor_byte_col().min(editor_line_text.len());
                                    let before = &editor_line_text[..col];
                                    let after = &editor_line_text[col..];
                                    lines.push(box_mid(
                                        vec![
                                            Span::styled(
                                                before.to_string(),
                                                Style::default().fg(theme.fg),
                                            ),
                                            Span::styled(
                                                "█",
                                                Style::default()
                                                    .fg(theme.accent)
                                                    .add_modifier(Modifier::SLOW_BLINK),
                                            ),
                                            Span::styled(
                                                after.to_string(),
                                                Style::default().fg(theme.fg),
                                            ),
                                        ],
                                        accent_style,
                                        width,
                                    ));
                                } else {
                                    lines.push(box_mid(
                                        vec![Span::styled(
                                            editor_line_text.to_string(),
                                            Style::default().fg(theme.fg),
                                        )],
                                        accent_style,
                                        width,
                                    ));
                                }
                                line_ids.push(DiffLineId::Summary);
                            }

                            lines.push(box_btm(accent_style, width));
                            line_ids.push(DiffLineId::Summary);
                        }
                    }
                }
            }
        }

        lines.push(Line::raw(""));
        line_ids.push(DiffLineId::Summary);
    }
}

impl StatefulWidget for DiffView<'_> {
    type State = DiffViewState;

    fn render(self, area: Rect, buf: &mut Buffer, state: &mut Self::State) {
        let theme = self.theme;
        let width = area.width as usize;
        let mut all_lines: Vec<Line<'static>> = Vec::new();
        let mut line_ids: Vec<DiffLineId> = Vec::new();

        self.render_file_summary(&mut all_lines, &mut line_ids, state);
        for (fi, file) in self.files.iter().enumerate() {
            let inner_width = width.saturating_sub(2); // account for block borders
            self.render_file_diff(file, fi, &mut all_lines, &mut line_ids, state, inner_width);
        }

        state.total_lines = all_lines.len();
        state.line_ids = line_ids;

        if state.cursor >= state.total_lines {
            state.cursor = state.total_lines.saturating_sub(1);
        }

        let content_height = area.height.saturating_sub(2) as usize;
        if content_height > 0 {
            if state.cursor < state.scroll {
                state.scroll = state.cursor;
            } else if state.cursor >= state.scroll + content_height {
                state.scroll = state.cursor - content_height + 1;
            }
        }
        let max_scroll = all_lines.len().saturating_sub(content_height);
        state.scroll = state.scroll.min(max_scroll);

        let selection = state.selection_range();

        let visible: Vec<Line> = all_lines
            .into_iter()
            .enumerate()
            .skip(state.scroll)
            .take(area.height as usize)
            .map(|(idx, line)| {
                let is_cursor = idx == state.cursor;
                let is_selected = selection
                    .map(|(s, e)| idx >= s && idx <= e)
                    .unwrap_or(false);
                if is_cursor {
                    let mut spans = vec![Span::styled("▌", Style::default().fg(theme.accent))];
                    spans.extend(line.spans);
                    Line::from(spans).style(Style::default().bg(theme.selection_bg))
                } else if is_selected {
                    let mut spans = vec![Span::styled("│", Style::default().fg(theme.accent))];
                    spans.extend(line.spans);
                    Line::from(spans).style(Style::default().bg(theme.bg_overlay))
                } else {
                    line
                }
            })
            .collect();

        let paragraph = Paragraph::new(visible);
        let paragraph = if let Some(block) = self.block {
            paragraph.block(block)
        } else {
            paragraph
        };
        paragraph.render(area, buf);
    }
}
