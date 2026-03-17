use ghtui_core::types::{DiffFile, DiffLineKind};
use ratatui::buffer::Buffer;
use ratatui::layout::Rect;
use ratatui::style::{Color, Modifier, Style};
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
    block: Option<Block<'a>>,
}

impl<'a> DiffView<'a> {
    pub fn new(files: &'a [DiffFile]) -> Self {
        Self { files, block: None }
    }

    pub fn block(mut self, block: Block<'a>) -> Self {
        self.block = block.into();
        self
    }

    fn render_file_list(&self) -> Vec<Line<'static>> {
        let mut lines = Vec::new();
        lines.push(Line::styled(
            "Files changed:",
            Style::default().add_modifier(Modifier::BOLD),
        ));

        for (i, file) in self.files.iter().enumerate() {
            let status_char = match file.status {
                ghtui_core::types::DiffFileStatus::Added => 'A',
                ghtui_core::types::DiffFileStatus::Removed => 'D',
                ghtui_core::types::DiffFileStatus::Modified => 'M',
                ghtui_core::types::DiffFileStatus::Renamed => 'R',
            };
            let status_color = match file.status {
                ghtui_core::types::DiffFileStatus::Added => Color::Green,
                ghtui_core::types::DiffFileStatus::Removed => Color::Red,
                ghtui_core::types::DiffFileStatus::Modified => Color::Yellow,
                ghtui_core::types::DiffFileStatus::Renamed => Color::Cyan,
            };

            lines.push(Line::from(vec![
                Span::styled(
                    format!("  {} ", status_char),
                    Style::default().fg(status_color),
                ),
                Span::raw(format!(
                    "{} (+{} -{})",
                    file.filename, file.additions, file.deletions
                )),
                if i == 0 {
                    Span::styled(" ◀", Style::default().fg(Color::Cyan))
                } else {
                    Span::raw("")
                },
            ]));
        }

        lines.push(Line::raw(""));
        lines
    }

    fn render_file_diff(file: &DiffFile) -> Vec<Line<'static>> {
        let mut lines = Vec::new();

        // File header
        lines.push(Line::styled(
            format!("── {} ──", file.filename),
            Style::default()
                .fg(Color::Cyan)
                .add_modifier(Modifier::BOLD),
        ));

        for hunk in &file.hunks {
            // Hunk header
            lines.push(Line::styled(
                hunk.header.clone(),
                Style::default().fg(Color::Magenta),
            ));

            for diff_line in &hunk.lines {
                let (prefix, style) = match diff_line.kind {
                    DiffLineKind::Add => ("+", Style::default().fg(Color::Green)),
                    DiffLineKind::Remove => ("-", Style::default().fg(Color::Red)),
                    DiffLineKind::Context => (" ", Style::default()),
                    DiffLineKind::Header => ("@", Style::default().fg(Color::Magenta)),
                };

                let old_ln = diff_line
                    .old_line
                    .map(|n| format!("{:>4}", n))
                    .unwrap_or_else(|| "    ".to_string());
                let new_ln = diff_line
                    .new_line
                    .map(|n| format!("{:>4}", n))
                    .unwrap_or_else(|| "    ".to_string());

                lines.push(Line::from(vec![
                    Span::styled(
                        format!("{} {} ", old_ln, new_ln),
                        Style::default().fg(Color::DarkGray),
                    ),
                    Span::styled(format!("{}{}", prefix, diff_line.content), style),
                ]));
            }

            lines.push(Line::raw(""));
        }

        lines
    }
}

impl StatefulWidget for DiffView<'_> {
    type State = DiffViewState;

    fn render(self, area: Rect, buf: &mut Buffer, state: &mut Self::State) {
        let mut all_lines = Vec::new();

        if state.show_all_files || self.files.len() <= 1 {
            // Show file list header then all diffs
            all_lines.extend(self.render_file_list());
            for file in self.files {
                all_lines.extend(Self::render_file_diff(file));
            }
        } else if let Some(file) = self.files.get(state.current_file) {
            all_lines.extend(self.render_file_list());
            all_lines.extend(Self::render_file_diff(file));
        }

        // Clamp scroll
        let max_scroll = all_lines.len().saturating_sub(area.height as usize);
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
