use std::collections::HashSet;

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

/// Focus area within the action detail view.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ActionDetailFocus {
    Jobs,
    Steps,
    Log,
}

#[derive(Debug)]
pub struct ActionDetailState {
    pub detail: WorkflowRunDetail,
    pub selected_job: usize,
    pub log: Option<Vec<LogLine>>,
    pub log_scroll: usize,
    pub auto_scroll: bool,
    /// Which steps are collapsed (by step number). Expanded by default.
    pub collapsed_steps: HashSet<u32>,
    /// Current focus area
    pub focus: ActionDetailFocus,
    /// Action bar: focused?
    pub action_bar_focused: bool,
    /// Action bar: selected action index
    pub action_bar_selected: usize,
}

impl ActionDetailState {
    pub fn new(detail: WorkflowRunDetail) -> Self {
        Self {
            detail,
            selected_job: 0,
            log: None,
            log_scroll: 0,
            auto_scroll: true,
            collapsed_steps: HashSet::new(),
            focus: ActionDetailFocus::Jobs,
            action_bar_focused: false,
            action_bar_selected: 0,
        }
    }

    /// Toggle collapse state of a step
    pub fn toggle_step(&mut self, step_number: u32) {
        if !self.collapsed_steps.remove(&step_number) {
            self.collapsed_steps.insert(step_number);
        }
    }

    /// Check if a step is collapsed
    pub fn is_step_collapsed(&self, step_number: u32) -> bool {
        self.collapsed_steps.contains(&step_number)
    }

    /// Get available action bar items based on run state
    pub fn action_bar_items(&self) -> Vec<&'static str> {
        let mut items = Vec::new();
        let run = &self.detail.run;

        match run.status {
            Some(crate::types::RunStatus::InProgress)
            | Some(crate::types::RunStatus::Queued)
            | Some(crate::types::RunStatus::Waiting) => {
                items.push("Cancel");
            }
            _ => {}
        }

        match run.conclusion {
            Some(crate::types::RunConclusion::Failure)
            | Some(crate::types::RunConclusion::Cancelled) => {
                items.push("Re-run");
                items.push("Re-run failed");
            }
            Some(crate::types::RunConclusion::Success) => {
                items.push("Re-run");
            }
            _ => {
                items.push("Re-run");
            }
        }

        items.push("Delete");
        items.push("Open in browser");
        items
    }
}
