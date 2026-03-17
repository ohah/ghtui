use crate::types::{ActionsFilters, LogLine, Pagination, Workflow, WorkflowRun, WorkflowRunDetail};

#[derive(Debug)]
pub struct ActionsListState {
    pub items: Vec<WorkflowRun>,
    pub pagination: Pagination,
    pub selected: usize,
    pub filters: ActionsFilters,
    pub search_mode: bool,
    pub search_query: String,
    pub workflows: Vec<Workflow>,
}

impl ActionsListState {
    pub fn new(items: Vec<WorkflowRun>, pagination: Pagination) -> Self {
        Self {
            items,
            pagination,
            selected: 0,
            filters: ActionsFilters::default(),
            search_mode: false,
            search_query: String::new(),
            workflows: Vec::new(),
        }
    }

    pub fn with_filters(
        items: Vec<WorkflowRun>,
        pagination: Pagination,
        filters: ActionsFilters,
    ) -> Self {
        Self {
            items,
            pagination,
            selected: 0,
            filters,
            search_mode: false,
            search_query: String::new(),
            workflows: Vec::new(),
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

    pub fn cycle_status(&mut self) {
        self.filters.cycle_status();
    }

    pub fn cycle_event(&mut self) {
        self.filters.cycle_event();
    }

    pub fn select_workflow(&mut self, workflow_id: Option<u64>) {
        self.filters.workflow_id = workflow_id;
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
