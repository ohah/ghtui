use std::collections::HashSet;

use ghtui_core::editor::TextEditor;
use ghtui_core::theme::Theme;
use ghtui_core::types::{DiffFile, DiffFileStatus, DiffLineKind, ReviewComment};
use ratatui::buffer::Buffer;
use ratatui::layout::Rect;
use ratatui::style::{Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Paragraph, StatefulWidget, Widget};

/// Identifies a renderable line in the diff view
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DiffLineId {
    Summary,                   // summary header / file list
    FileHeader(usize),         // file index
    HunkHeader(usize, usize),  // file, hunk
    Code(usize, usize, usize), // file, hunk, line
}

#[derive(Debug, Default)]
pub struct DiffViewState {
    pub scroll: usize,
    pub cursor: usize,      // cursor position in rendered line list
    pub total_lines: usize, // total rendered lines (set during render)
    pub show_all_files: bool,
    pub collapsed_files: HashSet<usize>, // file indices that are collapsed
    pub select_anchor: Option<usize>,    // anchor for shift+move selection
    // Mapping from rendered line index to DiffLineId (populated during render)
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

    /// Returns (start, end) inclusive range of selected lines
    pub fn selection_range(&self) -> Option<(usize, usize)> {
        self.select_anchor.map(|anchor| {
            let start = anchor.min(self.cursor);
            let end = anchor.max(self.cursor);
            (start, end)
        })
    }

    pub fn toggle_file_collapse(&mut self) {
        if let Some(id) = self.line_ids.get(self.cursor) {
            let file_idx = match id {
                DiffLineId::FileHeader(f) => Some(*f),
                DiffLineId::HunkHeader(f, _) => Some(*f),
                DiffLineId::Code(f, _, _) => Some(*f),
                _ => None,
            };
            if let Some(idx) = file_idx {
                if self.collapsed_files.contains(&idx) {
                    self.collapsed_files.remove(&idx);
                } else {
                    self.collapsed_files.insert(idx);
                }
            }
        }
    }

    pub fn page_down(&mut self, page_size: usize) {
        let target = self.cursor + page_size;
        self.cursor = target.min(self.total_lines.saturating_sub(1));
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
    /// Inline comment editor: (file_path, line_number, editor)
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

    fn render_file_summary(
        &self,
        lines: &mut Vec<Line<'static>>,
        line_ids: &mut Vec<DiffLineId>,
        state: &DiffViewState,
    ) {
        let total_adds: u32 = self.files.iter().map(|f| f.additions).sum();
        let total_dels: u32 = self.files.iter().map(|f| f.deletions).sum();
        let theme = self.theme;

        // Summary header
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
                "  (Enter:fold  j/k:move  J/K:select)",
                Style::default().fg(theme.fg_muted),
            ),
        ]));
        line_ids.push(DiffLineId::Summary);

        lines.push(Line::raw(""));
        line_ids.push(DiffLineId::Summary);

        // File list with bar chart
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
    ) {
        let theme = self.theme;
        let collapsed = state.collapsed_files.contains(&file_idx);

        // File header bar
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

        for (hi, hunk) in file.hunks.iter().enumerate() {
            // Hunk header
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
                            format!("          {}", diff_line.content),
                            Style::default().fg(theme.diff_hunk),
                        ));
                    }
                }
                line_ids.push(DiffLineId::Code(file_idx, hi, li));

                // Show review comments attached to this line
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

                    // Group threaded comments (in_reply_to_id)
                    let roots: Vec<&&ReviewComment> = matching
                        .iter()
                        .filter(|rc| rc.in_reply_to_id.is_none())
                        .collect();

                    for root in &roots {
                        // Comment box
                        lines.push(Line::from(vec![
                            Span::styled("          ┌─ ", Style::default().fg(theme.border)),
                            Span::styled(
                                format!("@{}", root.user.login),
                                Style::default()
                                    .fg(theme.accent)
                                    .add_modifier(Modifier::BOLD),
                            ),
                            Span::styled(
                                format!("  {}", root.created_at.format("%m/%d %H:%M")),
                                Style::default().fg(theme.fg_muted),
                            ),
                        ]));
                        line_ids.push(DiffLineId::Summary);

                        for body_line in root.body.lines() {
                            lines.push(Line::from(vec![
                                Span::styled("          │ ", Style::default().fg(theme.border)),
                                Span::styled(body_line.to_string(), Style::default().fg(theme.fg)),
                            ]));
                            line_ids.push(DiffLineId::Summary);
                        }

                        // Replies
                        let replies: Vec<&&ReviewComment> = matching
                            .iter()
                            .filter(|rc| rc.in_reply_to_id == Some(root.id))
                            .collect();
                        for reply in &replies {
                            lines.push(Line::from(vec![
                                Span::styled("          │  ↳ ", Style::default().fg(theme.border)),
                                Span::styled(
                                    format!("@{}: ", reply.user.login),
                                    Style::default().fg(theme.accent),
                                ),
                                Span::styled(
                                    reply.body.lines().next().unwrap_or("").to_string(),
                                    Style::default().fg(theme.fg_dim),
                                ),
                            ]));
                            line_ids.push(DiffLineId::Summary);
                        }

                        lines.push(Line::styled(
                            "          └───────────",
                            Style::default().fg(theme.border),
                        ));
                        line_ids.push(DiffLineId::Summary);
                    }

                    // Show non-root comments that aren't replies to any root in matching
                    let orphans: Vec<&&ReviewComment> = matching
                        .iter()
                        .filter(|rc| {
                            rc.in_reply_to_id.is_some()
                                && !roots.iter().any(|r| Some(r.id) == rc.in_reply_to_id)
                        })
                        .collect();
                    for orphan in &orphans {
                        lines.push(Line::from(vec![
                            Span::styled("          ┌─ ", Style::default().fg(theme.border)),
                            Span::styled(
                                format!("@{}: ", orphan.user.login),
                                Style::default().fg(theme.accent),
                            ),
                            Span::styled(
                                orphan.body.lines().next().unwrap_or("").to_string(),
                                Style::default().fg(theme.fg),
                            ),
                        ]));
                        line_ids.push(DiffLineId::Summary);
                        lines.push(Line::styled(
                            "          └───────────",
                            Style::default().fg(theme.border),
                        ));
                        line_ids.push(DiffLineId::Summary);
                    }

                    // Show inline comment editor if targeting this line
                    if let Some((editor_path, editor_line, editor)) = &self.comment_editor {
                        if *editor_path == file.filename && *editor_line == ln {
                            lines.push(Line::from(vec![
                                Span::styled("          ┌─ ", Style::default().fg(theme.accent)),
                                Span::styled(
                                    "Review comment (Ctrl+Enter:submit  Esc:cancel)",
                                    Style::default()
                                        .fg(theme.accent)
                                        .add_modifier(Modifier::BOLD),
                                ),
                            ]));
                            line_ids.push(DiffLineId::Summary);

                            let content = editor.content();
                            if content.is_empty() {
                                lines.push(Line::from(vec![
                                    Span::styled("          │ ", Style::default().fg(theme.accent)),
                                    Span::styled(
                                        "█",
                                        Style::default()
                                            .fg(theme.accent)
                                            .add_modifier(Modifier::SLOW_BLINK),
                                    ),
                                ]));
                            } else {
                                for (ei, editor_line_text) in content.lines().enumerate() {
                                    let is_cursor_line = ei == editor.cursor_row;
                                    let mut spans = vec![Span::styled(
                                        "          │ ",
                                        Style::default().fg(theme.accent),
                                    )];
                                    if is_cursor_line {
                                        let col = editor.cursor_byte_col();
                                        let before =
                                            &editor_line_text[..col.min(editor_line_text.len())];
                                        let after =
                                            &editor_line_text[col.min(editor_line_text.len())..];
                                        spans.push(Span::styled(
                                            before.to_string(),
                                            Style::default().fg(theme.fg),
                                        ));
                                        spans.push(Span::styled(
                                            "█",
                                            Style::default()
                                                .fg(theme.accent)
                                                .add_modifier(Modifier::SLOW_BLINK),
                                        ));
                                        spans.push(Span::styled(
                                            after.to_string(),
                                            Style::default().fg(theme.fg),
                                        ));
                                    } else {
                                        spans.push(Span::styled(
                                            editor_line_text.to_string(),
                                            Style::default().fg(theme.fg),
                                        ));
                                    }
                                    lines.push(Line::from(spans));
                                }
                            }
                            line_ids.push(DiffLineId::Summary);

                            lines.push(Line::styled(
                                "          └───────────",
                                Style::default().fg(theme.accent),
                            ));
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
        let mut all_lines: Vec<Line<'static>> = Vec::new();
        let mut line_ids: Vec<DiffLineId> = Vec::new();

        // Build all lines
        self.render_file_summary(&mut all_lines, &mut line_ids, state);
        for (fi, file) in self.files.iter().enumerate() {
            self.render_file_diff(file, fi, &mut all_lines, &mut line_ids, state);
        }

        state.total_lines = all_lines.len();
        state.line_ids = line_ids;

        // Clamp cursor
        if state.cursor >= state.total_lines {
            state.cursor = state.total_lines.saturating_sub(1);
        }

        // Auto-scroll to keep cursor visible
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

        // Get selection range
        let selection = state.selection_range();

        // Apply cursor and selection highlighting
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
                    // Cursor line: highlight with selection bg + bold marker
                    let mut spans = vec![Span::styled("▌", Style::default().fg(theme.accent))];
                    spans.extend(line.spans);
                    Line::from(spans).style(Style::default().bg(theme.selection_bg))
                } else if is_selected {
                    // Selected range: subtle highlight
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
