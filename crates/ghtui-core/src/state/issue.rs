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
