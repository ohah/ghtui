use crate::editor::TextEditor;
use crate::types::{Issue, IssueDetail, IssueFilters, IssueState, Pagination};

use crate::types::common::Label;

#[derive(Debug)]
pub struct IssueListState {
    pub items: Vec<Issue>,
    pub pagination: Pagination,
    pub selected: usize,
    pub scroll_offset: usize,
    pub filters: IssueFilters,
    pub search_mode: bool,
    pub search_query: String,
}

/// Label picker state for issue detail
#[derive(Debug)]
pub struct LabelPickerState {
    pub available: Vec<Label>,
    pub selected_names: Vec<String>, // currently toggled labels
    pub cursor: usize,
}

impl IssueListState {
    pub fn new(items: Vec<Issue>, pagination: Pagination) -> Self {
        Self {
            items,
            pagination,
            selected: 0,
            scroll_offset: 0,
            filters: IssueFilters::default(),
            search_mode: false,
            search_query: String::new(),
        }
    }

    pub fn with_filters(items: Vec<Issue>, pagination: Pagination, filters: IssueFilters) -> Self {
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

    pub fn selected_issue(&self) -> Option<&Issue> {
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
            None | Some(IssueState::Open) => Some(IssueState::Closed),
            Some(IssueState::Closed) => Some(IssueState::Open),
        };
    }
}

/// What is being edited inline
#[derive(Debug, Clone, PartialEq)]
pub enum InlineEditTarget {
    /// Editing issue title (inline in header)
    IssueTitle,
    /// Editing issue body (fullscreen editor)
    IssueBody,
    /// Editing a specific comment by index (bottom panel)
    Comment(usize),
    /// Writing a new comment (bottom panel)
    NewComment,
    /// Quote-replying to a comment (bottom panel)
    QuoteReply(usize),
}

#[derive(Debug)]
pub struct IssueDetailState {
    pub detail: IssueDetail,
    pub scroll: usize,
    pub selected_comment: Option<usize>,
    pub edit_target: Option<InlineEditTarget>,
    pub editor: TextEditor,
    pub label_picker: Option<LabelPickerState>,
}

impl IssueDetailState {
    pub fn new(detail: IssueDetail) -> Self {
        Self {
            detail,
            scroll: 0,
            selected_comment: None,
            edit_target: None,
            editor: TextEditor::new(),
            label_picker: None,
        }
    }

    pub fn is_editing(&self) -> bool {
        self.edit_target.is_some()
    }

    pub fn start_edit_title(&mut self) {
        self.editor = TextEditor::from_string(&self.detail.issue.title);
        self.edit_target = Some(InlineEditTarget::IssueTitle);
    }

    pub fn start_edit_body(&mut self) {
        self.editor = TextEditor::from_string(self.detail.issue.body.as_deref().unwrap_or(""));
        self.edit_target = Some(InlineEditTarget::IssueBody);
    }

    pub fn start_edit_comment(&mut self, index: usize) {
        if let Some(comment) = self.detail.comments.get(index) {
            self.editor = TextEditor::from_string(&comment.body);
            self.edit_target = Some(InlineEditTarget::Comment(index));
        }
    }

    pub fn start_new_comment(&mut self) {
        self.editor = TextEditor::new();
        self.edit_target = Some(InlineEditTarget::NewComment);
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
            self.edit_target = Some(InlineEditTarget::QuoteReply(index));
        }
    }

    pub fn cancel_edit(&mut self) {
        self.edit_target = None;
        self.editor = TextEditor::new();
    }

    pub fn editor_text(&self) -> String {
        self.editor.content()
    }

    pub fn select_next_comment(&mut self) {
        let max = self.detail.comments.len();
        match self.selected_comment {
            None => {
                if max > 0 {
                    self.selected_comment = Some(0);
                }
            }
            Some(i) => {
                if i < max.saturating_sub(1) {
                    self.selected_comment = Some(i + 1);
                }
            }
        }
    }

    pub fn select_prev_comment(&mut self) {
        match self.selected_comment {
            None => {}
            Some(0) => self.selected_comment = None,
            Some(i) => self.selected_comment = Some(i - 1),
        }
    }
}
