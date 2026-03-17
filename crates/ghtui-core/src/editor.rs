/// Simple text editor with cursor position tracking.
/// cursor_col is a **character index** (not byte index).
#[derive(Debug, Clone)]
pub struct TextEditor {
    pub lines: Vec<String>,
    pub cursor_row: usize,
    pub cursor_col: usize, // character index, not byte index
}

impl Default for TextEditor {
    fn default() -> Self {
        Self {
            lines: vec![String::new()],
            cursor_row: 0,
            cursor_col: 0,
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
        }
    }

    pub fn content(&self) -> String {
        self.lines.join("\n")
    }

    pub fn insert_char(&mut self, c: char) {
        if c == '\n' {
            self.insert_newline();
            return;
        }
        if let Some(line) = self.lines.get_mut(self.cursor_row) {
            let col = self.cursor_col.min(char_count(line));
            let byte_idx = char_to_byte(line, col);
            line.insert(byte_idx, c);
            self.cursor_col = col + 1;
        }
    }

    pub fn insert_newline(&mut self) {
        if let Some(line) = self.lines.get_mut(self.cursor_row) {
            let col = self.cursor_col.min(char_count(line));
            let byte_idx = char_to_byte(line, col);
            let rest = line[byte_idx..].to_string();
            line.truncate(byte_idx);
            self.cursor_row += 1;
            self.lines.insert(self.cursor_row, rest);
            self.cursor_col = 0;
        }
    }

    pub fn backspace(&mut self) {
        if self.cursor_col > 0 {
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
            // Merge with previous line
            let current = self.lines.remove(self.cursor_row);
            self.cursor_row -= 1;
            if let Some(prev) = self.lines.get_mut(self.cursor_row) {
                self.cursor_col = char_count(prev);
                prev.push_str(&current);
            }
        }
    }

    pub fn move_left(&mut self) {
        if self.cursor_col > 0 {
            self.cursor_col -= 1;
        } else if self.cursor_row > 0 {
            self.cursor_row -= 1;
            self.cursor_col = char_count(&self.lines[self.cursor_row]);
        }
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
    }

    pub fn move_up(&mut self) {
        if self.cursor_row > 0 {
            self.cursor_row -= 1;
            let line_len = char_count(&self.lines[self.cursor_row]);
            self.cursor_col = self.cursor_col.min(line_len);
        }
    }

    pub fn move_down(&mut self) {
        if self.cursor_row < self.lines.len() - 1 {
            self.cursor_row += 1;
            let line_len = char_count(&self.lines[self.cursor_row]);
            self.cursor_col = self.cursor_col.min(line_len);
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
}
