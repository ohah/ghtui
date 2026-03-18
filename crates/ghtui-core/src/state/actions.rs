use std::collections::HashSet;

use ratatui::text::Line;

use crate::types::{
    ActionsFilters, Artifact, LogLine, Pagination, PendingDeployment, Workflow, WorkflowRun,
    WorkflowRunDetail,
};

#[derive(Debug)]
pub struct ActionsListState {
    pub items: Vec<WorkflowRun>,
    pub pagination: Pagination,
    pub selected: usize,
    pub filters: ActionsFilters,
    pub search_mode: bool,
    pub search_query: String,
    pub workflows: Vec<Workflow>,
    pub show_workflow_sidebar: bool,
    pub workflow_sidebar_selected: usize,
    pub workflow_sidebar_focused: bool,
    /// Dispatch modal state
    pub dispatch: Option<DispatchState>,
}

/// State for the workflow dispatch modal.
#[derive(Debug)]
pub struct DispatchState {
    pub workflow_id: u64,
    pub workflow_name: String,
    pub git_ref: String,
    pub inputs: Vec<DispatchInputField>,
    pub focused_field: usize, // 0 = ref, 1..N = inputs
    pub editing: bool,
    pub edit_buffer: String,
}

#[derive(Debug)]
pub struct DispatchInputField {
    pub name: String,
    pub value: String,
    pub input_type: String,
    pub required: bool,
    pub options: Vec<String>,
    pub description: Option<String>,
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
            show_workflow_sidebar: false,
            workflow_sidebar_selected: 0,
            workflow_sidebar_focused: false,
            dispatch: None,
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
            show_workflow_sidebar: false,
            workflow_sidebar_selected: 0,
            workflow_sidebar_focused: false,
            dispatch: None,
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
    Log,
    ActionBar,
}

/// Action bar items (typed, not stringly).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ActionBarItem {
    Cancel,
    Rerun,
    RerunFailed,
    Delete,
    OpenInBrowser,
}

impl ActionBarItem {
    pub fn label(&self) -> &'static str {
        match self {
            ActionBarItem::Cancel => "Cancel",
            ActionBarItem::Rerun => "Re-run",
            ActionBarItem::RerunFailed => "Re-run failed",
            ActionBarItem::Delete => "Delete",
            ActionBarItem::OpenInBrowser => "Open in browser",
        }
    }
}

#[derive(Debug)]
pub struct ActionDetailState {
    pub detail: WorkflowRunDetail,
    pub selected_job: usize,
    pub log: Option<Vec<LogLine>>,
    /// Pre-parsed ANSI log lines for rendering (parsed once on load).
    pub parsed_log: Option<Vec<Line<'static>>>,
    pub log_scroll: usize,
    pub auto_scroll: bool,
    /// Which steps are collapsed (by step number). Expanded by default.
    pub collapsed_steps: HashSet<u32>,
    /// Whether all steps for the selected job are collapsed.
    pub steps_collapsed: bool,
    /// Current focus area
    pub focus: ActionDetailFocus,
    /// Action bar: selected action index
    pub action_bar_selected: usize,
    /// Cached action bar items (computed on state change).
    pub action_bar_items: Vec<ActionBarItem>,
    /// Artifacts for this run
    pub artifacts: Vec<Artifact>,
    /// Pending deployments
    pub pending_deployments: Vec<PendingDeployment>,
    /// Workflow file content (YAML)
    pub workflow_file: Option<String>,
    /// Tick counter for log polling (fetch every N ticks)
    pub log_poll_counter: u32,
    /// Whether the selected job is actively running (enables log polling)
    pub log_streaming: bool,
    /// Artifact currently being downloaded (name shown as progress indicator)
    pub downloading_artifact: Option<String>,
}

impl ActionDetailState {
    pub fn new(detail: WorkflowRunDetail) -> Self {
        let items = Self::compute_action_bar_items(&detail);
        Self {
            detail,
            selected_job: 0,
            log: None,
            parsed_log: None,
            log_scroll: 0,
            auto_scroll: true,
            collapsed_steps: HashSet::new(),
            steps_collapsed: false,
            focus: ActionDetailFocus::Jobs,
            action_bar_selected: 0,
            action_bar_items: items,
            artifacts: Vec::new(),
            pending_deployments: Vec::new(),
            workflow_file: None,
            log_poll_counter: 0,
            log_streaming: false,
            downloading_artifact: None,
        }
    }

    /// Set log and pre-parse ANSI sequences.
    pub fn set_log(&mut self, lines: Vec<LogLine>) {
        let parsed: Vec<Line<'static>> = lines
            .iter()
            .map(|l| crate::ansi::parse_ansi_line(&l.content))
            .collect();
        self.parsed_log = Some(parsed);
        self.log = Some(lines);
    }

    /// Toggle collapse state of all steps for the selected job.
    pub fn toggle_steps_collapsed(&mut self) {
        self.steps_collapsed = !self.steps_collapsed;
    }

    /// Compute available action bar items based on run state.
    fn compute_action_bar_items(detail: &WorkflowRunDetail) -> Vec<ActionBarItem> {
        let mut items = Vec::new();
        let run = &detail.run;

        match run.status {
            Some(crate::types::RunStatus::InProgress)
            | Some(crate::types::RunStatus::Queued)
            | Some(crate::types::RunStatus::Waiting) => {
                items.push(ActionBarItem::Cancel);
            }
            _ => {}
        }

        match run.conclusion {
            Some(crate::types::RunConclusion::Failure)
            | Some(crate::types::RunConclusion::Cancelled) => {
                items.push(ActionBarItem::Rerun);
                items.push(ActionBarItem::RerunFailed);
            }
            _ => {
                items.push(ActionBarItem::Rerun);
            }
        }

        items.push(ActionBarItem::Delete);
        items.push(ActionBarItem::OpenInBrowser);
        items
    }
}

/// Format a duration between two optional timestamps.
pub fn format_duration(
    start: Option<chrono::DateTime<chrono::Utc>>,
    end: Option<chrono::DateTime<chrono::Utc>>,
) -> String {
    match (start, end) {
        (Some(s), Some(e)) => {
            let secs = (e - s).num_seconds();
            if secs >= 60 {
                format!("{}m{}s", secs / 60, secs % 60)
            } else if secs > 0 {
                format!("{}s", secs)
            } else {
                String::new()
            }
        }
        (Some(_), None) => "running...".to_string(),
        _ => String::new(),
    }
}
