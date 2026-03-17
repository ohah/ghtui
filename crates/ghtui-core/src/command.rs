use crate::config::GhAccount;
use crate::types::common::RepoId;
use crate::types::*;

#[derive(Debug)]
pub enum Command {
    None,
    Batch(Vec<Command>),

    // Account
    SwitchAccount(GhAccount),

    // PR
    FetchPrList(RepoId, PrFilters, u32),
    FetchPrDetail(RepoId, u64),
    FetchPrDiff(RepoId, u64),
    MergePr(RepoId, u64, MergeMethod),
    ClosePr(RepoId, u64),
    ReopenPr(RepoId, u64),
    CreatePr(RepoId, CreatePrInput),
    SubmitReview(RepoId, u64, ReviewInput),

    // Issue
    FetchIssueList(RepoId, IssueFilters, u32),
    FetchIssueDetail(RepoId, u64),
    CloseIssue(RepoId, u64),
    ReopenIssue(RepoId, u64),
    CreateIssue(RepoId, CreateIssueInput),
    AddComment(RepoId, u64, String),

    // Actions
    FetchRuns(RepoId, ActionsFilters, u32),
    FetchRunDetail(RepoId, u64),
    FetchJobLog(RepoId, u64, u64),
    CancelRun(RepoId, u64),
    RerunRun(RepoId, u64),

    // Notifications
    FetchNotifications(NotificationFilters),
    MarkNotificationRead(String),

    // Search
    Search(String, SearchKind, u32),

    // Utility
    OpenInBrowser(String),
    SetClipboard(String),
    Quit,
}
