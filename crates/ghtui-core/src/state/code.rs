use crate::types::code::FileEntry;

#[derive(Debug)]
pub struct CodeViewState {
    pub entries: Vec<FileEntry>,
    pub selected: usize,
    pub path_stack: Vec<String>,
    pub current_path: String,
    pub git_ref: String,
    pub file_content: Option<String>,
    pub file_name: Option<String>,
    pub scroll: usize,
    pub readme_content: Option<String>,
    pub sidebar_focused: bool,
}

impl CodeViewState {
    pub fn new(git_ref: String) -> Self {
        Self {
            entries: Vec::new(),
            selected: 0,
            path_stack: Vec::new(),
            current_path: String::new(),
            git_ref,
            file_content: None,
            file_name: None,
            scroll: 0,
            readme_content: None,
            sidebar_focused: true,
        }
    }

    pub fn select_next(&mut self) {
        if !self.entries.is_empty() {
            self.selected = (self.selected + 1).min(self.entries.len() - 1);
        }
    }

    pub fn select_prev(&mut self) {
        self.selected = self.selected.saturating_sub(1);
    }
}
