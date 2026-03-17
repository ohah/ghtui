use std::collections::HashSet;

use crate::editor::TextEditor;
use crate::types::{DiffFile, Pagination, PrFilters, PrState, PullRequest, PullRequestDetail};

use super::issue::{AssigneePickerState, LabelPickerState, MilestonePickerState};

#[derive(Debug)]
pub struct PrListState {
    pub items: Vec<PullRequest>,
    pub pagination: Pagination,
    pub selected: usize,
    pub scroll_offset: usize,
    pub filters: PrFilters,
    pub search_mode: bool,
    pub search_query: String,
}

impl PrListState {
    pub fn new(items: Vec<PullRequest>, pagination: Pagination) -> Self {
        Self {
            items,
            pagination,
            selected: 0,
            scroll_offset: 0,
            filters: PrFilters::default(),
            search_mode: false,
            search_query: String::new(),
        }
    }

    pub fn with_filters(
        items: Vec<PullRequest>,
        pagination: Pagination,
        filters: PrFilters,
    ) -> Self {
        Self {
            items,
            pagination,
            selected: 0,
            scroll_offset: 0,
            filters,
            search_mode: false,
            search_query: String::new(),
        }
    }

    pub fn selected_pr(&self) -> Option<&PullRequest> {
        self.items.get(self.selected)
    }

    pub fn select_next(&mut self) {
        if self.selected < self.items.len().saturating_sub(1) {
            self.selected += 1;
        }
    }

    pub fn select_prev(&mut self) {
        self.selected = self.selected.saturating_sub(1);
    }

    pub fn toggle_state_filter(&mut self) {
        self.filters.state = match self.filters.state {
            None | Some(PrState::Open) => Some(PrState::Closed),
            Some(PrState::Closed) => Some(PrState::Open),
            Some(PrState::Merged) => Some(PrState::Open),
        };
    }

    pub fn cycle_sort(&mut self) {
        self.filters.sort = match self.filters.sort.as_deref() {
            None | Some("created") => Some("updated".to_string()),
            Some("updated") => Some("popularity".to_string()),
            Some("popularity") => Some("long-running".to_string()),
            Some("long-running") => Some("created".to_string()),
            _ => Some("created".to_string()),
        };
    }

    pub fn sort_display(&self) -> &str {
        match self.filters.sort.as_deref() {
            None | Some("created") => "Newest",
            Some("updated") => "Recently updated",
            Some("popularity") => "Most popular",
            Some("long-running") => "Long-running",
            _ => "Newest",
        }
    }
}

/// Focus sections in PR detail view
#[derive(Debug, Clone, PartialEq)]
pub enum PrSection {
    Title,
    Labels,
    Assignees,
    Body,
    Comment(usize),
}

impl PrSection {
    pub fn action_hint(&self) -> &'static str {
        match self {
            PrSection::Title => "e:Edit  o:Open in browser",
            PrSection::Labels => "l:Edit labels",
            PrSection::Assignees => "a:Edit assignees",
            PrSection::Body => "e:Edit body",
            PrSection::Comment(_) => "e:Edit  r:Reply  d:Delete",
        }
    }
}

/// What is being edited inline in PR detail
#[derive(Debug, Clone, PartialEq)]
pub enum PrInlineEditTarget {
    PrTitle,
    PrBody,
    Comment(usize),
    NewComment,
    QuoteReply(usize),
}

#[derive(Debug)]
pub struct PrDetailState {
    pub detail: PullRequestDetail,
    pub diff: Option<Vec<DiffFile>>,
    pub scroll: usize,                     // conversation scroll
    pub diff_scroll: usize,                // diff tab scroll offset
    pub diff_cursor: usize,                // diff line cursor position
    pub diff_select_anchor: Option<usize>, // shift+move selection anchor
    pub diff_collapsed: HashSet<usize>,    // collapsed file indices
    pub tab: usize,
    pub comment_input: String,
    pub focus: PrSection,
    pub edit_target: Option<PrInlineEditTarget>,
    pub editor: TextEditor,
    pub label_picker: Option<LabelPickerState>,
    pub assignee_picker: Option<AssigneePickerState>,
    pub milestone_picker: Option<MilestonePickerState>,
}

impl PrDetailState {
    pub fn new(detail: PullRequestDetail) -> Self {
        Self {
            detail,
            diff: None,
            scroll: 0,
            diff_scroll: 0,
            diff_cursor: 0,
            diff_select_anchor: None,
            diff_collapsed: HashSet::new(),
            tab: 0,
            comment_input: String::new(),
            focus: PrSection::Title,
            edit_target: None,
            editor: TextEditor::new(),
            label_picker: None,
            assignee_picker: None,
            milestone_picker: None,
        }
    }

    pub fn is_editing(&self) -> bool {
        self.edit_target.is_some()
    }

    pub fn has_picker(&self) -> bool {
        self.label_picker.is_some()
            || self.assignee_picker.is_some()
            || self.milestone_picker.is_some()
    }

    fn sections(&self) -> Vec<PrSection> {
        let mut sections = vec![
            PrSection::Title,
            PrSection::Labels,
            PrSection::Assignees,
            PrSection::Body,
        ];
        for i in 0..self.detail.comments.len() {
            sections.push(PrSection::Comment(i));
        }
        sections
    }

    pub fn focus_next(&mut self) {
        let sections = self.sections();
        if let Some(idx) = sections.iter().position(|s| s == &self.focus) {
            if idx < sections.len() - 1 {
                self.focus = sections[idx + 1].clone();
            }
        }
    }

    pub fn focus_prev(&mut self) {
        let sections = self.sections();
        if let Some(idx) = sections.iter().position(|s| s == &self.focus) {
            if idx > 0 {
                self.focus = sections[idx - 1].clone();
            }
        }
    }

    pub fn start_edit_title(&mut self) {
        self.editor = TextEditor::from_string(&self.detail.pr.title);
        self.edit_target = Some(PrInlineEditTarget::PrTitle);
    }

    pub fn start_edit_body(&mut self) {
        self.editor = TextEditor::from_string(self.detail.pr.body.as_deref().unwrap_or(""));
        self.edit_target = Some(PrInlineEditTarget::PrBody);
    }

    pub fn start_edit_comment(&mut self, index: usize) {
        if let Some(comment) = self.detail.comments.get(index) {
            self.editor = TextEditor::from_string(&comment.body);
            self.edit_target = Some(PrInlineEditTarget::Comment(index));
        }
    }

    pub fn start_new_comment(&mut self) {
        self.editor = TextEditor::new();
        self.edit_target = Some(PrInlineEditTarget::NewComment);
    }

    pub fn start_quote_reply(&mut self, index: usize) {
        if let Some(comment) = self.detail.comments.get(index) {
            let quoted: String = comment
                .body
                .lines()
                .map(|l| format!("> {}", l))
                .collect::<Vec<_>>()
                .join("\n");
            let text = format!("> @{}\n{}\n\n", comment.user.login, quoted);
            self.editor = TextEditor::from_string(&text);
            self.edit_target = Some(PrInlineEditTarget::QuoteReply(index));
        }
    }

    pub fn cancel_edit(&mut self) {
        self.edit_target = None;
        self.editor = TextEditor::new();
    }

    pub fn editor_text(&self) -> String {
        self.editor.content()
    }

    pub fn selected_comment(&self) -> Option<usize> {
        match &self.focus {
            PrSection::Comment(i) => Some(*i),
            _ => None,
        }
    }
}
