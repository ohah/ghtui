/// Reusable text editor with cursor, scroll, selection, and undo support.
/// Used for issue editing, comments, PR descriptions, code editing, etc.
#[derive(Debug, Clone)]
pub struct TextEditor {
    pub lines: Vec<String>,
    pub cursor_row: usize,
    pub cursor_col: usize, // character index, not byte index
    pub scroll_offset: usize,
    pub viewport_height: usize,
    undo_stack: Vec<EditorSnapshot>,
    redo_stack: Vec<EditorSnapshot>,
}

#[derive(Debug, Clone)]
struct EditorSnapshot {
    lines: Vec<String>,
    cursor_row: usize,
    cursor_col: usize,
}

impl Default for TextEditor {
    fn default() -> Self {
        Self {
            lines: vec![String::new()],
            cursor_row: 0,
            cursor_col: 0,
            scroll_offset: 0,
            viewport_height: 20,
            undo_stack: Vec::new(),
            redo_stack: Vec::new(),
        }
    }
}

/// Convert character index to byte index in a string.
fn char_to_byte(s: &str, char_idx: usize) -> usize {
    s.char_indices()
        .nth(char_idx)
        .map(|(i, _)| i)
        .unwrap_or(s.len())
}

/// Count characters in a string.
fn char_count(s: &str) -> usize {
    s.chars().count()
}

impl TextEditor {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn from_string(text: &str) -> Self {
        let lines: Vec<String> = if text.is_empty() {
            vec![String::new()]
        } else {
            text.split('\n').map(String::from).collect()
        };
        let cursor_row = lines.len().saturating_sub(1);
        let cursor_col = lines.last().map(|l| char_count(l)).unwrap_or(0);
        Self {
            lines,
            cursor_row,
            cursor_col,
            ..Default::default()
        }
    }

    pub fn content(&self) -> String {
        self.lines.join("\n")
    }

    pub fn set_viewport_height(&mut self, height: usize) {
        self.viewport_height = height.max(1);
    }

    // === Undo / Redo ===

    fn save_undo(&mut self) {
        self.undo_stack.push(EditorSnapshot {
            lines: self.lines.clone(),
            cursor_row: self.cursor_row,
            cursor_col: self.cursor_col,
        });
        self.redo_stack.clear();
        // Limit undo history
        if self.undo_stack.len() > 100 {
            self.undo_stack.remove(0);
        }
    }

    pub fn undo(&mut self) {
        if let Some(snapshot) = self.undo_stack.pop() {
            self.redo_stack.push(EditorSnapshot {
                lines: self.lines.clone(),
                cursor_row: self.cursor_row,
                cursor_col: self.cursor_col,
            });
            self.lines = snapshot.lines;
            self.cursor_row = snapshot.cursor_row;
            self.cursor_col = snapshot.cursor_col;
            self.ensure_scroll();
        }
    }

    pub fn redo(&mut self) {
        if let Some(snapshot) = self.redo_stack.pop() {
            self.undo_stack.push(EditorSnapshot {
                lines: self.lines.clone(),
                cursor_row: self.cursor_row,
                cursor_col: self.cursor_col,
            });
            self.lines = snapshot.lines;
            self.cursor_row = snapshot.cursor_row;
            self.cursor_col = snapshot.cursor_col;
            self.ensure_scroll();
        }
    }

    // === Text Mutation ===

    pub fn insert_char(&mut self, c: char) {
        if c == '\n' {
            self.insert_newline();
            return;
        }
        self.save_undo();
        if let Some(line) = self.lines.get_mut(self.cursor_row) {
            let col = self.cursor_col.min(char_count(line));
            let byte_idx = char_to_byte(line, col);
            line.insert(byte_idx, c);
            self.cursor_col = col + 1;
        }
    }

    pub fn insert_str(&mut self, text: &str) {
        self.save_undo();
        for c in text.chars() {
            if c == '\n' {
                if let Some(line) = self.lines.get_mut(self.cursor_row) {
                    let col = self.cursor_col.min(char_count(line));
                    let byte_idx = char_to_byte(line, col);
                    let rest = line[byte_idx..].to_string();
                    line.truncate(byte_idx);
                    self.cursor_row += 1;
                    self.lines.insert(self.cursor_row, rest);
                    self.cursor_col = 0;
                }
            } else if let Some(line) = self.lines.get_mut(self.cursor_row) {
                let col = self.cursor_col.min(char_count(line));
                let byte_idx = char_to_byte(line, col);
                line.insert(byte_idx, c);
                self.cursor_col = col + 1;
            }
        }
        self.ensure_scroll();
    }

    pub fn insert_newline(&mut self) {
        self.save_undo();
        if let Some(line) = self.lines.get_mut(self.cursor_row) {
            let col = self.cursor_col.min(char_count(line));
            let byte_idx = char_to_byte(line, col);
            let rest = line[byte_idx..].to_string();
            line.truncate(byte_idx);
            self.cursor_row += 1;
            self.lines.insert(self.cursor_row, rest);
            self.cursor_col = 0;
        }
        self.ensure_scroll();
    }

    pub fn insert_tab(&mut self) {
        self.save_undo();
        if let Some(line) = self.lines.get_mut(self.cursor_row) {
            let col = self.cursor_col.min(char_count(line));
            let byte_idx = char_to_byte(line, col);
            line.insert_str(byte_idx, "    ");
            self.cursor_col = col + 4;
        }
    }

    pub fn backspace(&mut self) {
        if self.cursor_col > 0 {
            self.save_undo();
            if let Some(line) = self.lines.get_mut(self.cursor_row) {
                let col = self.cursor_col.min(char_count(line));
                if col > 0 {
                    let byte_start = char_to_byte(line, col - 1);
                    let byte_end = char_to_byte(line, col);
                    line.drain(byte_start..byte_end);
                    self.cursor_col = col - 1;
                }
            }
        } else if self.cursor_row > 0 {
            self.save_undo();
            let current = self.lines.remove(self.cursor_row);
            self.cursor_row -= 1;
            if let Some(prev) = self.lines.get_mut(self.cursor_row) {
                self.cursor_col = char_count(prev);
                prev.push_str(&current);
            }
            self.ensure_scroll();
        }
    }

    pub fn delete(&mut self) {
        let line_len = self
            .lines
            .get(self.cursor_row)
            .map(|l| char_count(l))
            .unwrap_or(0);

        if self.cursor_col < line_len {
            self.save_undo();
            if let Some(line) = self.lines.get_mut(self.cursor_row) {
                let col = self.cursor_col;
                let byte_start = char_to_byte(line, col);
                let byte_end = char_to_byte(line, col + 1);
                line.drain(byte_start..byte_end);
            }
        } else if self.cursor_row < self.lines.len() - 1 {
            self.save_undo();
            let next = self.lines.remove(self.cursor_row + 1);
            if let Some(line) = self.lines.get_mut(self.cursor_row) {
                line.push_str(&next);
            }
        }
    }

    // === Cursor Movement ===

    pub fn move_left(&mut self) {
        if self.cursor_col > 0 {
            self.cursor_col -= 1;
        } else if self.cursor_row > 0 {
            self.cursor_row -= 1;
            self.cursor_col = char_count(&self.lines[self.cursor_row]);
        }
        self.ensure_scroll();
    }

    pub fn move_right(&mut self) {
        let line_len = self
            .lines
            .get(self.cursor_row)
            .map(|l| char_count(l))
            .unwrap_or(0);
        if self.cursor_col < line_len {
            self.cursor_col += 1;
        } else if self.cursor_row < self.lines.len() - 1 {
            self.cursor_row += 1;
            self.cursor_col = 0;
        }
        self.ensure_scroll();
    }

    pub fn move_up(&mut self) {
        if self.cursor_row > 0 {
            self.cursor_row -= 1;
            let line_len = char_count(&self.lines[self.cursor_row]);
            self.cursor_col = self.cursor_col.min(line_len);
        }
        self.ensure_scroll();
    }

    pub fn move_down(&mut self) {
        if self.cursor_row < self.lines.len() - 1 {
            self.cursor_row += 1;
            let line_len = char_count(&self.lines[self.cursor_row]);
            self.cursor_col = self.cursor_col.min(line_len);
        }
        self.ensure_scroll();
    }

    pub fn move_word_left(&mut self) {
        if self.cursor_col == 0 {
            self.move_left();
            return;
        }
        if let Some(line) = self.lines.get(self.cursor_row) {
            let chars: Vec<char> = line.chars().collect();
            let mut col = self.cursor_col.min(chars.len());
            // Skip whitespace
            while col > 0 && chars[col - 1].is_whitespace() {
                col -= 1;
            }
            // Skip word chars
            while col > 0 && !chars[col - 1].is_whitespace() {
                col -= 1;
            }
            self.cursor_col = col;
        }
    }

    pub fn move_word_right(&mut self) {
        if let Some(line) = self.lines.get(self.cursor_row) {
            let chars: Vec<char> = line.chars().collect();
            let len = chars.len();
            let mut col = self.cursor_col.min(len);
            // Skip word chars
            while col < len && !chars[col].is_whitespace() {
                col += 1;
            }
            // Skip whitespace
            while col < len && chars[col].is_whitespace() {
                col += 1;
            }
            self.cursor_col = col;
        } else {
            self.move_right();
        }
    }

    pub fn move_home(&mut self) {
        self.cursor_col = 0;
    }

    pub fn move_end(&mut self) {
        if let Some(line) = self.lines.get(self.cursor_row) {
            self.cursor_col = char_count(line);
        }
    }

    pub fn move_to_top(&mut self) {
        self.cursor_row = 0;
        self.cursor_col = 0;
        self.ensure_scroll();
    }

    pub fn move_to_bottom(&mut self) {
        self.cursor_row = self.lines.len().saturating_sub(1);
        self.cursor_col = 0;
        self.ensure_scroll();
    }

    pub fn page_up(&mut self) {
        let jump = self.viewport_height.saturating_sub(2);
        self.cursor_row = self.cursor_row.saturating_sub(jump);
        let line_len = char_count(&self.lines[self.cursor_row]);
        self.cursor_col = self.cursor_col.min(line_len);
        self.ensure_scroll();
    }

    pub fn page_down(&mut self) {
        let jump = self.viewport_height.saturating_sub(2);
        self.cursor_row = (self.cursor_row + jump).min(self.lines.len().saturating_sub(1));
        let line_len = char_count(&self.lines[self.cursor_row]);
        self.cursor_col = self.cursor_col.min(line_len);
        self.ensure_scroll();
    }

    // === Scroll ===

    fn ensure_scroll(&mut self) {
        if self.cursor_row < self.scroll_offset {
            self.scroll_offset = self.cursor_row;
        }
        if self.cursor_row >= self.scroll_offset + self.viewport_height {
            self.scroll_offset = self.cursor_row - self.viewport_height + 1;
        }
    }

    // === Utility ===

    pub fn line_count(&self) -> usize {
        self.lines.len()
    }

    pub fn is_empty(&self) -> bool {
        self.lines.len() == 1 && self.lines[0].is_empty()
    }

    /// Get byte index of cursor in current line (for rendering split).
    pub fn cursor_byte_col(&self) -> usize {
        self.lines
            .get(self.cursor_row)
            .map(|l| char_to_byte(l, self.cursor_col.min(char_count(l))))
            .unwrap_or(0)
    }

    /// Visible lines based on scroll_offset and viewport_height.
    pub fn visible_lines(&self) -> impl Iterator<Item = (usize, &String)> {
        let start = self.scroll_offset;
        let end = (start + self.viewport_height).min(self.lines.len());
        (start..end).map(move |i| (i, &self.lines[i]))
    }

    /// Is the cursor on this absolute line index?
    pub fn is_cursor_line(&self, line_idx: usize) -> bool {
        line_idx == self.cursor_row
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_editor() {
        let editor = TextEditor::new();
        assert_eq!(editor.lines, vec![""]);
        assert_eq!(editor.cursor_row, 0);
        assert_eq!(editor.cursor_col, 0);
    }

    #[test]
    fn test_from_string() {
        let editor = TextEditor::from_string("hello\nworld");
        assert_eq!(editor.lines, vec!["hello", "world"]);
        assert_eq!(editor.cursor_row, 1);
        assert_eq!(editor.cursor_col, 5);
    }

    #[test]
    fn test_insert_char() {
        let mut editor = TextEditor::new();
        editor.insert_char('h');
        editor.insert_char('i');
        assert_eq!(editor.content(), "hi");
        assert_eq!(editor.cursor_col, 2);
    }

    #[test]
    fn test_insert_in_middle() {
        let mut editor = TextEditor::from_string("hllo");
        editor.cursor_col = 1;
        editor.insert_char('e');
        assert_eq!(editor.content(), "hello");
        assert_eq!(editor.cursor_col, 2);
    }

    #[test]
    fn test_newline() {
        let mut editor = TextEditor::from_string("hello world");
        editor.cursor_col = 5;
        editor.insert_newline();
        assert_eq!(editor.lines, vec!["hello", " world"]);
        assert_eq!(editor.cursor_row, 1);
        assert_eq!(editor.cursor_col, 0);
    }

    #[test]
    fn test_backspace() {
        let mut editor = TextEditor::from_string("hello");
        editor.backspace();
        assert_eq!(editor.content(), "hell");
    }

    #[test]
    fn test_backspace_merge_lines() {
        let mut editor = TextEditor::from_string("hello\nworld");
        editor.cursor_row = 1;
        editor.cursor_col = 0;
        editor.backspace();
        assert_eq!(editor.content(), "helloworld");
        assert_eq!(editor.cursor_row, 0);
        assert_eq!(editor.cursor_col, 5);
    }

    #[test]
    fn test_delete_char() {
        let mut editor = TextEditor::from_string("hello");
        editor.cursor_col = 0;
        editor.delete();
        assert_eq!(editor.content(), "ello");
    }

    #[test]
    fn test_delete_merge_lines() {
        let mut editor = TextEditor::from_string("hello\nworld");
        editor.cursor_row = 0;
        editor.cursor_col = 5;
        editor.delete();
        assert_eq!(editor.content(), "helloworld");
    }

    #[test]
    fn test_move_left_right() {
        let mut editor = TextEditor::from_string("ab");
        editor.cursor_col = 1;
        editor.move_left();
        assert_eq!(editor.cursor_col, 0);
        editor.move_right();
        assert_eq!(editor.cursor_col, 1);
    }

    #[test]
    fn test_move_up_down() {
        let mut editor = TextEditor::from_string("line1\nline2\nline3");
        editor.cursor_row = 1;
        editor.cursor_col = 2;
        editor.move_up();
        assert_eq!(editor.cursor_row, 0);
        assert_eq!(editor.cursor_col, 2);
        editor.move_down();
        assert_eq!(editor.cursor_row, 1);
    }

    #[test]
    fn test_unicode_insert() {
        let mut editor = TextEditor::new();
        editor.insert_char('한');
        editor.insert_char('글');
        assert_eq!(editor.content(), "한글");
        assert_eq!(editor.cursor_col, 2);
    }

    #[test]
    fn test_unicode_backspace() {
        let mut editor = TextEditor::from_string("한글테스트");
        assert_eq!(editor.cursor_col, 5);
        editor.backspace();
        assert_eq!(editor.content(), "한글테스");
        assert_eq!(editor.cursor_col, 4);
        editor.cursor_col = 2;
        editor.backspace();
        assert_eq!(editor.content(), "한테스");
        assert_eq!(editor.cursor_col, 1);
    }

    #[test]
    fn test_unicode_insert_middle() {
        let mut editor = TextEditor::from_string("한글");
        editor.cursor_col = 1;
        editor.insert_char('국');
        assert_eq!(editor.content(), "한국글");
        assert_eq!(editor.cursor_col, 2);
    }

    #[test]
    fn test_backspace_at_start_does_nothing() {
        let mut editor = TextEditor::from_string("hello");
        editor.cursor_col = 0;
        editor.cursor_row = 0;
        editor.backspace();
        assert_eq!(editor.content(), "hello");
    }

    #[test]
    fn test_undo_redo() {
        let mut editor = TextEditor::from_string("hello");
        editor.cursor_col = 5;
        editor.insert_char('!');
        assert_eq!(editor.content(), "hello!");
        editor.undo();
        assert_eq!(editor.content(), "hello");
        editor.redo();
        assert_eq!(editor.content(), "hello!");
    }

    #[test]
    fn test_word_movement() {
        let mut editor = TextEditor::from_string("hello world foo");
        editor.cursor_col = 0;
        editor.move_word_right();
        assert_eq!(editor.cursor_col, 6); // after "hello "
        editor.move_word_right();
        assert_eq!(editor.cursor_col, 12); // after "world "
        editor.move_word_left();
        assert_eq!(editor.cursor_col, 6);
    }

    #[test]
    fn test_tab_insert() {
        let mut editor = TextEditor::new();
        editor.insert_tab();
        assert_eq!(editor.content(), "    ");
        assert_eq!(editor.cursor_col, 4);
    }

    #[test]
    fn test_insert_str() {
        let mut editor = TextEditor::new();
        editor.insert_str("hello\nworld");
        assert_eq!(editor.lines, vec!["hello", "world"]);
        assert_eq!(editor.cursor_row, 1);
        assert_eq!(editor.cursor_col, 5);
    }

    #[test]
    fn test_visible_lines() {
        let mut editor = TextEditor::from_string("a\nb\nc\nd\ne");
        editor.set_viewport_height(3);
        editor.scroll_offset = 1;
        let visible: Vec<(usize, &String)> = editor.visible_lines().collect();
        assert_eq!(visible.len(), 3);
        assert_eq!(visible[0].0, 1);
        assert_eq!(visible[0].1, "b");
    }
}
