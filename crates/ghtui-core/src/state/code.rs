use crate::types::code::{CommitDetail, CommitEntry, FileEntry};

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

    // Branch/tag picker
    pub branches: Vec<String>,
    pub tags: Vec<String>,
    pub ref_picker_open: bool,
    pub ref_picker_selected: usize,
    pub ref_picker_items: Vec<(String, bool)>, // (name, is_branch)

    // Commit history
    pub commits: Vec<CommitEntry>,
    pub commit_detail: Option<CommitDetail>,
    pub show_commits: bool,
    pub commit_selected: usize,
    pub commit_scroll: usize,
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

            branches: Vec::new(),
            tags: Vec::new(),
            ref_picker_open: false,
            ref_picker_selected: 0,
            ref_picker_items: Vec::new(),

            commits: Vec::new(),
            commit_detail: None,
            show_commits: false,
            commit_selected: 0,
            commit_scroll: 0,
        }
    }

    pub fn select_next(&mut self) {
        if self.show_commits {
            if !self.commits.is_empty() {
                self.commit_selected = (self.commit_selected + 1).min(self.commits.len() - 1);
            }
        } else if !self.entries.is_empty() {
            self.selected = (self.selected + 1).min(self.entries.len() - 1);
        }
    }

    pub fn select_prev(&mut self) {
        if self.show_commits {
            self.commit_selected = self.commit_selected.saturating_sub(1);
        } else {
            self.selected = self.selected.saturating_sub(1);
        }
    }

    pub fn build_ref_picker_items(&mut self) {
        self.ref_picker_items.clear();
        for b in &self.branches {
            self.ref_picker_items.push((b.clone(), true));
        }
        for t in &self.tags {
            self.ref_picker_items.push((t.clone(), false));
        }
        self.ref_picker_selected = 0;
    }

    pub fn ref_picker_next(&mut self) {
        if !self.ref_picker_items.is_empty() {
            self.ref_picker_selected =
                (self.ref_picker_selected + 1).min(self.ref_picker_items.len() - 1);
        }
    }

    pub fn ref_picker_prev(&mut self) {
        self.ref_picker_selected = self.ref_picker_selected.saturating_sub(1);
    }
}
