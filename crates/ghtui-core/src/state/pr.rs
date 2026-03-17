use crate::types::{DiffFile, Pagination, PullRequest, PullRequestDetail};

#[derive(Debug)]
pub struct PrListState {
    pub items: Vec<PullRequest>,
    pub pagination: Pagination,
    pub selected: usize,
    pub scroll_offset: usize,
}

impl PrListState {
    pub fn new(items: Vec<PullRequest>, pagination: Pagination) -> Self {
        Self {
            items,
            pagination,
            selected: 0,
            scroll_offset: 0,
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
}

#[derive(Debug)]
pub struct PrDetailState {
    pub detail: PullRequestDetail,
    pub diff: Option<Vec<DiffFile>>,
    pub scroll: usize,
    pub tab: usize,
    pub comment_input: String,
}

impl PrDetailState {
    pub fn new(detail: PullRequestDetail) -> Self {
        Self {
            detail,
            diff: None,
            scroll: 0,
            tab: 0,
            comment_input: String::new(),
        }
    }
}
