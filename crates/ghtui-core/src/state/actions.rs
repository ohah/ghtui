use crate::types::{LogLine, Pagination, WorkflowRun, WorkflowRunDetail};

#[derive(Debug)]
pub struct ActionsListState {
    pub items: Vec<WorkflowRun>,
    pub pagination: Pagination,
    pub selected: usize,
}

impl ActionsListState {
    pub fn new(items: Vec<WorkflowRun>, pagination: Pagination) -> Self {
        Self {
            items,
            pagination,
            selected: 0,
        }
    }

    pub fn selected_run(&self) -> Option<&WorkflowRun> {
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
pub struct ActionDetailState {
    pub detail: WorkflowRunDetail,
    pub selected_job: usize,
    pub log: Option<Vec<LogLine>>,
    pub log_scroll: usize,
    pub auto_scroll: bool,
}

impl ActionDetailState {
    pub fn new(detail: WorkflowRunDetail) -> Self {
        Self {
            detail,
            selected_job: 0,
            log: None,
            log_scroll: 0,
            auto_scroll: true,
        }
    }
}
