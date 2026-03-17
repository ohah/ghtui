/// Simple text editor with cursor position tracking.
#[derive(Debug, Clone)]
pub struct TextEditor {
    pub lines: Vec<String>,
    pub cursor_row: usize,
    pub cursor_col: usize,
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
        let cursor_col = lines.last().map(|l| l.len()).unwrap_or(0);
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
            let col = self.cursor_col.min(line.len());
            line.insert(col, c);
            self.cursor_col = col + 1;
        }
    }

    pub fn insert_newline(&mut self) {
        if let Some(line) = self.lines.get_mut(self.cursor_row) {
            let col = self.cursor_col.min(line.len());
            let rest = line[col..].to_string();
            line.truncate(col);
            self.cursor_row += 1;
            self.lines.insert(self.cursor_row, rest);
            self.cursor_col = 0;
        }
    }

    pub fn backspace(&mut self) {
        if self.cursor_col > 0 {
            if let Some(line) = self.lines.get_mut(self.cursor_row) {
                let col = self.cursor_col.min(line.len());
                if col > 0 {
                    line.remove(col - 1);
                    self.cursor_col = col - 1;
                }
            }
        } else if self.cursor_row > 0 {
            // Merge with previous line
            let current = self.lines.remove(self.cursor_row);
            self.cursor_row -= 1;
            if let Some(prev) = self.lines.get_mut(self.cursor_row) {
                self.cursor_col = prev.len();
                prev.push_str(&current);
            }
        }
    }

    pub fn move_left(&mut self) {
        if self.cursor_col > 0 {
            self.cursor_col -= 1;
        } else if self.cursor_row > 0 {
            self.cursor_row -= 1;
            self.cursor_col = self.lines[self.cursor_row].len();
        }
    }

    pub fn move_right(&mut self) {
        let line_len = self
            .lines
            .get(self.cursor_row)
            .map(|l| l.len())
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
            let line_len = self.lines[self.cursor_row].len();
            self.cursor_col = self.cursor_col.min(line_len);
        }
    }

    pub fn move_down(&mut self) {
        if self.cursor_row < self.lines.len() - 1 {
            self.cursor_row += 1;
            let line_len = self.lines[self.cursor_row].len();
            self.cursor_col = self.cursor_col.min(line_len);
        }
    }

    pub fn move_home(&mut self) {
        self.cursor_col = 0;
    }

    pub fn move_end(&mut self) {
        if let Some(line) = self.lines.get(self.cursor_row) {
            self.cursor_col = line.len();
        }
    }

    pub fn line_count(&self) -> usize {
        self.lines.len()
    }

    pub fn is_empty(&self) -> bool {
        self.lines.len() == 1 && self.lines[0].is_empty()
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
}
