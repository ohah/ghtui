use crate::config::GhAccount;
use crate::error::GhtuiError;
use crate::router::Route;
use crate::types::*;

#[derive(Debug)]
pub enum Message {
    // Navigation
    Navigate(Route),
    Back,

    // Account
    AccountSwitch(GhAccount),
    AccountSwitched(GhAccount),

    // PR
    PrListLoaded(Vec<PullRequest>, Pagination),
    PrDetailLoaded(Box<PullRequestDetail>),
    PrDiffLoaded(Vec<DiffFile>),
    PrMerged(u64),
    PrClosed(u64),
    PrReopened(u64),
    PrCreated(u64),
    ReviewSubmitted,

    // Issue
    IssueListLoaded(Vec<Issue>, Pagination),
    IssueDetailLoaded(Box<IssueDetail>),
    IssueClosed(u64),
    IssueReopened(u64),
    IssueCreated(u64),
    CommentAdded,

    // Actions
    RunsLoaded(Vec<WorkflowRun>, Pagination),
    RunDetailLoaded(Box<WorkflowRunDetail>),
    JobLogLoaded(u64, Vec<LogLine>),
    RunCancelled(u64),
    RunRerun(u64),

    // Notifications
    NotificationsLoaded(Vec<Notification>),
    NotificationMarkedRead(String),

    // Search
    SearchResults(SearchResultSet),

    // UI
    InputChanged(String),
    ListSelect(usize),
    TabChanged(usize),
    GlobalTabNext,
    GlobalTabPrev,
    GlobalTabSelect(usize),
    ToggleTheme,
    ModalOpen(ModalKind),
    ModalClose,
    Tick,
    Resize(u16, u16),

    // System
    Error(GhtuiError),
    Quit,
}

#[derive(Debug, Clone, PartialEq)]
pub enum ModalKind {
    MergePr,
    CreatePr,
    CreateIssue,
    AddComment,
    SubmitReview,
    Confirm { title: String, message: String },
    AccountSwitcher,
    Help,
}
