use crate::types::{ActionsFilters, IssueFilters, PrFilters, SearchKind, common::RepoId};

#[derive(Debug, Clone, PartialEq, Default)]
pub enum PrTab {
    #[default]
    Conversation,
    Diff,
    Checks,
}

#[derive(Debug, Clone, PartialEq, Default)]
pub enum Route {
    #[default]
    Dashboard,

    // Code tab
    Code {
        repo: RepoId,
        path: String,
        git_ref: String,
    },

    // Issues tab
    IssueList {
        repo: RepoId,
        filters: IssueFilters,
    },
    IssueDetail {
        repo: RepoId,
        number: u64,
    },

    // Pull Requests tab
    PrList {
        repo: RepoId,
        filters: PrFilters,
    },
    PrDetail {
        repo: RepoId,
        number: u64,
        tab: PrTab,
    },

    // Actions tab
    ActionsList {
        repo: RepoId,
        filters: ActionsFilters,
    },
    ActionDetail {
        repo: RepoId,
        run_id: u64,
    },
    JobLog {
        repo: RepoId,
        run_id: u64,
        job_id: u64,
    },

    // Security tab
    Security {
        repo: RepoId,
    },

    // Insights tab
    Insights {
        repo: RepoId,
    },

    // Settings tab
    Settings {
        repo: RepoId,
    },

    // Non-tab views (accessible via shortcuts)
    Notifications,
    Search {
        query: String,
        kind: SearchKind,
    },
}

/// Global tab indices
pub const TAB_CODE: usize = 0;
pub const TAB_ISSUES: usize = 1;
pub const TAB_PRS: usize = 2;
pub const TAB_ACTIONS: usize = 3;
pub const TAB_SECURITY: usize = 4;
pub const TAB_INSIGHTS: usize = 5;
pub const TAB_SETTINGS: usize = 6;

pub const TAB_LABELS: &[&str] = &[
    "Code",
    "Issues",
    "Pull requests",
    "Actions",
    "Security",
    "Insights",
    "Settings",
];

impl Route {
    pub fn title(&self) -> String {
        match self {
            Route::Dashboard => "Dashboard".to_string(),
            Route::Code { repo, path, .. } => format!("{} - {}", repo, path),
            Route::IssueList { repo, .. } => format!("{} - Issues", repo),
            Route::IssueDetail { repo, number } => format!("{} - Issue #{}", repo, number),
            Route::PrList { repo, .. } => format!("{} - Pull Requests", repo),
            Route::PrDetail { repo, number, .. } => format!("{} - PR #{}", repo, number),
            Route::ActionsList { repo, .. } => format!("{} - Actions", repo),
            Route::ActionDetail { repo, run_id } => format!("{} - Run #{}", repo, run_id),
            Route::JobLog { repo, job_id, .. } => format!("{} - Job #{}", repo, job_id),
            Route::Security { repo } => format!("{} - Security", repo),
            Route::Insights { repo } => format!("{} - Insights", repo),
            Route::Settings { repo } => format!("{} - Settings", repo),
            Route::Notifications => "Notifications".to_string(),
            Route::Search { query, kind } => format!("Search {:?}: {}", kind, query),
        }
    }

    /// Returns which global tab index this route belongs to, if any
    pub fn tab_index(&self) -> Option<usize> {
        match self {
            Route::Dashboard | Route::Code { .. } => Some(TAB_CODE),
            Route::IssueList { .. } | Route::IssueDetail { .. } => Some(TAB_ISSUES),
            Route::PrList { .. } | Route::PrDetail { .. } => Some(TAB_PRS),
            Route::ActionsList { .. } | Route::ActionDetail { .. } | Route::JobLog { .. } => {
                Some(TAB_ACTIONS)
            }
            Route::Security { .. } => Some(TAB_SECURITY),
            Route::Insights { .. } => Some(TAB_INSIGHTS),
            Route::Settings { .. } => Some(TAB_SETTINGS),
            Route::Notifications | Route::Search { .. } => None,
        }
    }
}
