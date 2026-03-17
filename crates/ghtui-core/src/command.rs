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
    SearchPulls(RepoId, String),
    UpdatePr(RepoId, u64, Option<String>, Option<String>), // repo, number, title, body
    SetPrLabels(RepoId, u64, Vec<String>),
    SetPrAssignees(RepoId, u64, Vec<String>),
    AddPrComment(RepoId, u64, String),
    UpdatePrComment(RepoId, u64, u64, String),
    DeletePrComment(RepoId, u64),
    ChangePrBase(RepoId, u64, String), // repo, number, new_base_branch
    SetPrReviewers(RepoId, u64, Vec<String>), // repo, number, reviewer logins
    SetPrDraft(RepoId, u64, bool),     // repo, number, draft

    // Issue
    FetchIssueList(RepoId, IssueFilters, u32),
    FetchIssueDetail(RepoId, u64),
    CloseIssue(RepoId, u64),
    ReopenIssue(RepoId, u64),
    CreateIssue(RepoId, CreateIssueInput),
    LockIssue(RepoId, u64),
    UnlockIssue(RepoId, u64),
    PinIssue(RepoId, u64),
    UnpinIssue(RepoId, u64),
    FetchPinnedIssues(RepoId),
    TransferIssue(RepoId, u64, String), // repo, number, dest_repo (owner/name)
    FetchIssueTemplates(RepoId),
    UpdateIssue(RepoId, u64, Option<String>, Option<String>), // repo, number, title, body
    SetIssueLabels(RepoId, u64, Vec<String>),
    SetIssueAssignees(RepoId, u64, Vec<String>),
    FetchRepoLabels(RepoId),
    FetchCollaboratorsForPicker(RepoId),
    DeleteComment(RepoId, u64),
    AddReaction(RepoId, u64, String, bool), // repo, id, reaction, is_issue (vs comment)
    SetMilestone(RepoId, u64, Option<u64>), // repo, issue_number, milestone_number
    FetchMilestones(RepoId),
    SearchIssues(RepoId, String), // repo, query - search issues and return as IssueListLoaded
    AddComment(RepoId, u64, String),
    UpdateComment(RepoId, u64, u64, String),

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

    // Insights
    FetchContributorStats(RepoId),
    FetchCommitActivity(RepoId),
    FetchTrafficClones(RepoId),
    FetchTrafficViews(RepoId),

    // Security
    FetchDependabotAlerts(RepoId),
    FetchCodeScanningAlerts(RepoId),
    FetchSecretScanningAlerts(RepoId),

    // Settings
    FetchRepoSettings(RepoId),
    FetchBranchProtections(RepoId),
    FetchCollaborators(RepoId),

    // Utility
    OpenInBrowser(String),
    SetClipboard(String),
    Quit,
}
