use crate::types::{Issue, IssueDetail, Pagination};

#[derive(Debug)]
pub struct IssueListState {
    pub items: Vec<Issue>,
    pub pagination: Pagination,
    pub selected: usize,
    pub scroll_offset: usize,
}

impl IssueListState {
    pub fn new(items: Vec<Issue>, pagination: Pagination) -> Self {
        Self {
            items,
            pagination,
            selected: 0,
            scroll_offset: 0,
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
}

#[derive(Debug)]
pub struct IssueDetailState {
    pub detail: IssueDetail,
    pub scroll: usize,
    pub comment_input: String,
}

impl IssueDetailState {
    pub fn new(detail: IssueDetail) -> Self {
        Self {
            detail,
            scroll: 0,
            comment_input: String::new(),
        }
    }
}
