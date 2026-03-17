use ghtui_core::theme::Theme;
use ghtui_core::types::{DiffFile, DiffFileStatus, DiffLineKind};
use ratatui::buffer::Buffer;
use ratatui::layout::Rect;
use ratatui::style::{Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Paragraph, StatefulWidget, Widget};

#[derive(Debug, Default)]
pub struct DiffViewState {
    pub scroll: usize,
    pub current_file: usize,
    pub show_all_files: bool,
}

impl DiffViewState {
    pub fn scroll_down(&mut self, amount: usize) {
        self.scroll = self.scroll.saturating_add(amount);
    }

    pub fn scroll_up(&mut self, amount: usize) {
        self.scroll = self.scroll.saturating_sub(amount);
    }

    pub fn next_file(&mut self, total: usize) {
        if self.current_file < total.saturating_sub(1) {
            self.current_file += 1;
            self.scroll = 0;
        }
    }

    pub fn prev_file(&mut self, total: usize) {
        if total > 0 && self.current_file > 0 {
            self.current_file -= 1;
            self.scroll = 0;
        }
    }
}

pub struct DiffView<'a> {
    files: &'a [DiffFile],
    theme: &'a Theme,
    block: Option<Block<'a>>,
}

impl<'a> DiffView<'a> {
    pub fn new(files: &'a [DiffFile], theme: &'a Theme) -> Self {
        Self {
            files,
            theme,
            block: None,
        }
    }

    pub fn block(mut self, block: Block<'a>) -> Self {
        self.block = block.into();
        self
    }

    fn render_file_summary(&self) -> Vec<Line<'static>> {
        let mut lines = Vec::new();

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
        ]));
        lines.push(Line::raw(""));

        // File list with bar chart
        for file in self.files {
            let status_icon = match file.status {
                DiffFileStatus::Added => Span::styled("A ", Style::default().fg(theme.diff_add_fg)),
                DiffFileStatus::Removed => {
                    Span::styled("D ", Style::default().fg(theme.diff_remove_fg))
                }
                DiffFileStatus::Modified => Span::styled("M ", Style::default().fg(theme.warning)),
                DiffFileStatus::Renamed => Span::styled("R ", Style::default().fg(theme.info)),
            };

            // Change bar: █ blocks for additions (green) and deletions (red)
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
                Span::styled(" ", Style::default()),
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
        }

        lines.push(Line::raw(""));
        lines
    }

    fn render_file_diff(&self, file: &DiffFile) -> Vec<Line<'static>> {
        let mut lines = Vec::new();
        let theme = self.theme;

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

        lines.push(Line::from(vec![
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

        for hunk in &file.hunks {
            // Hunk header
            lines.push(Line::styled(
                format!(" {}", hunk.header),
                Style::default()
                    .fg(theme.diff_hunk)
                    .bg(theme.bg_subtle)
                    .add_modifier(Modifier::ITALIC),
            ));

            for diff_line in &hunk.lines {
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
            }
        }

        lines.push(Line::raw(""));
        lines
    }
}

impl StatefulWidget for DiffView<'_> {
    type State = DiffViewState;

    fn render(self, area: Rect, buf: &mut Buffer, state: &mut Self::State) {
        let mut all_lines = Vec::new();

        if state.show_all_files || self.files.len() <= 1 {
            all_lines.extend(self.render_file_summary());
            for file in self.files {
                all_lines.extend(self.render_file_diff(file));
            }
        } else if let Some(file) = self.files.get(state.current_file) {
            all_lines.extend(self.render_file_summary());
            all_lines.extend(self.render_file_diff(file));
        }

        // Clamp scroll
        let content_height = area.height.saturating_sub(2) as usize; // account for block borders
        let max_scroll = all_lines.len().saturating_sub(content_height);
        state.scroll = state.scroll.min(max_scroll);

        let visible: Vec<Line> = all_lines
            .into_iter()
            .skip(state.scroll)
            .take(area.height as usize)
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
