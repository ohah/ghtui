use crate::types::{Issue, IssueDetail, IssueFilters, IssueState, Pagination};

#[derive(Debug)]
pub struct IssueListState {
    pub items: Vec<Issue>,
    pub pagination: Pagination,
    pub selected: usize,
    pub scroll_offset: usize,
    pub filters: IssueFilters,
}

impl IssueListState {
    pub fn new(items: Vec<Issue>, pagination: Pagination) -> Self {
        Self {
            items,
            pagination,
            selected: 0,
            scroll_offset: 0,
            filters: IssueFilters::default(),
        }
    }

    pub fn with_filters(items: Vec<Issue>, pagination: Pagination, filters: IssueFilters) -> Self {
        Self {
            items,
            pagination,
            selected: 0,
            scroll_offset: 0,
            filters,
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
    pub selected_comment: Option<usize>, // None = issue body, Some(i) = comment index
    pub edit_target: Option<InlineEditTarget>,
    pub edit_buffer: String,
}

impl IssueDetailState {
    pub fn new(detail: IssueDetail) -> Self {
        Self {
            detail,
            scroll: 0,
            selected_comment: None,
            edit_target: None,
            edit_buffer: String::new(),
        }
    }

    pub fn is_editing(&self) -> bool {
        self.edit_target.is_some()
    }

    pub fn start_edit_title(&mut self) {
        self.edit_buffer = self.detail.issue.title.clone();
        self.edit_target = Some(InlineEditTarget::IssueTitle);
    }

    pub fn start_edit_body(&mut self) {
        self.edit_buffer = self.detail.issue.body.as_deref().unwrap_or("").to_string();
        self.edit_target = Some(InlineEditTarget::IssueBody);
    }

    pub fn start_edit_comment(&mut self, index: usize) {
        if let Some(comment) = self.detail.comments.get(index) {
            self.edit_buffer = comment.body.clone();
            self.edit_target = Some(InlineEditTarget::Comment(index));
        }
    }

    pub fn start_new_comment(&mut self) {
        self.edit_buffer.clear();
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
            self.edit_buffer = format!("> @{}\n{}\n\n", comment.user.login, quoted);
            self.edit_target = Some(InlineEditTarget::QuoteReply(index));
        }
    }

    pub fn cancel_edit(&mut self) {
        self.edit_target = None;
        self.edit_buffer.clear();
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
