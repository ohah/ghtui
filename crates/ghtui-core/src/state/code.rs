use crate::editor::TextEditor;
use crate::types::code::{CommitDetail, CommitEntry, FileEntry, TreeNode};
use std::collections::HashSet;

#[derive(Debug)]
pub struct CodeViewState {
    pub entries: Vec<FileEntry>,
    pub selected: usize,
    pub path_stack: Vec<String>,
    pub current_path: String,
    pub git_ref: String,
    pub file_content: Option<String>,
    pub file_name: Option<String>,
    pub file_path: Option<String>, // full path for API calls (e.g. "src/main.rs")
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

    // File editing
    pub editing: bool,
    pub editor: TextEditor,

    // Syntax highlight cache (file content hash → highlighted spans not stored here,
    // but we track which file was highlighted to avoid re-highlighting on every frame)
    pub highlighted_file: Option<String>, // filename of last highlighted file

    // Tree view
    pub tree: Vec<TreeNode>,
    pub expanded_dirs: HashSet<String>,
    pub tree_visible: Vec<usize>,
    pub tree_loaded: bool,

    // Image preview
    pub image_data: Option<Vec<u8>>,
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
            file_path: None,
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

            editing: false,
            editor: TextEditor::new(),
            highlighted_file: None,

            tree: Vec::new(),
            expanded_dirs: HashSet::new(),
            tree_visible: Vec::new(),
            tree_loaded: false,

            image_data: None,
        }
    }

    pub fn select_next(&mut self) {
        if self.show_commits {
            if !self.commits.is_empty() {
                self.commit_selected = (self.commit_selected + 1).min(self.commits.len() - 1);
            }
        } else if self.tree_loaded {
            if !self.tree_visible.is_empty() {
                self.selected = (self.selected + 1).min(self.tree_visible.len() - 1);
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

    /// Rebuild the list of visible tree indices based on expanded_dirs.
    /// A node is visible if all its ancestor directories are expanded.
    pub fn rebuild_visible_tree(&mut self) {
        self.tree_visible.clear();
        for (i, node) in self.tree.iter().enumerate() {
            if node.depth == 0 {
                // Root-level nodes are always visible
                self.tree_visible.push(i);
            } else {
                // Check if all ancestors are expanded
                let parts: Vec<&str> = node.path.split('/').collect();
                let mut all_expanded = true;
                for depth in 1..parts.len() {
                    let ancestor = parts[..depth].join("/");
                    if !self.expanded_dirs.contains(&ancestor) {
                        all_expanded = false;
                        break;
                    }
                }
                if all_expanded {
                    self.tree_visible.push(i);
                }
            }
        }
        // Clamp selection
        if !self.tree_visible.is_empty() {
            self.selected = self.selected.min(self.tree_visible.len() - 1);
        } else {
            self.selected = 0;
        }
    }

    /// Toggle the expand/collapse state of the currently selected directory.
    pub fn toggle_expand(&mut self) {
        if let Some(&idx) = self.tree_visible.get(self.selected) {
            if let Some(node) = self.tree.get(idx) {
                if node.is_dir {
                    let path = node.path.clone();
                    if self.expanded_dirs.contains(&path) {
                        self.expanded_dirs.remove(&path);
                    } else {
                        self.expanded_dirs.insert(path);
                    }
                    self.rebuild_visible_tree();
                }
            }
        }
    }

    /// Get the currently selected visible tree node.
    pub fn tree_selected_node(&self) -> Option<&TreeNode> {
        self.tree_visible
            .get(self.selected)
            .and_then(|&idx| self.tree.get(idx))
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
