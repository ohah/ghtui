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
    SetAutoMerge(RepoId, u64, bool),   // repo, number, enable
    ResolveReviewThread(RepoId, u64, String, bool), // repo, pr_number, thread_node_id, resolve

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
    RerunFailedJobs(RepoId, u64),
    DeleteRun(RepoId, u64),
    FetchWorkflows(RepoId),
    FetchRunArtifacts(RepoId, u64),
    DownloadArtifact(RepoId, u64, String), // repo, artifact_id, artifact_name
    DispatchWorkflow(RepoId, u64, String, serde_json::Value), // repo, workflow_id, ref, inputs
    FetchWorkflowFile(RepoId, String),     // repo, workflow_path
    FetchWorkflowInputs(RepoId, u64, String, String), // repo, workflow_id, workflow_name, path
    FetchPendingDeployments(RepoId, u64),
    ApproveDeployment(RepoId, u64, Vec<u64>), // repo, run_id, environment_ids
    RejectDeployment(RepoId, u64, Vec<u64>),

    // Notifications
    FetchNotifications(NotificationFilters),
    MarkNotificationRead(String),
    MarkAllNotificationsRead,
    UnsubscribeThread(String),
    MarkThreadDone(String),

    // Search
    Search(String, SearchKind, u32),

    // Insights
    FetchContributorStats(RepoId),
    FetchCommitActivity(RepoId),
    FetchTrafficClones(RepoId),
    FetchTrafficViews(RepoId),
    FetchCodeFrequency(RepoId),
    FetchForks(RepoId),
    FetchDependencyGraph(RepoId),

    // Security
    FetchDependabotAlerts(RepoId),
    FetchCodeScanningAlerts(RepoId),
    FetchSecretScanningAlerts(RepoId),
    FetchSecurityAdvisories(RepoId),
    DismissDependabotAlert(RepoId, u64, String),
    ReopenDependabotAlert(RepoId, u64),
    DismissCodeScanningAlert(RepoId, u64, String),
    ResolveSecretScanningAlert(RepoId, u64, String),

    // Code
    FetchTree(RepoId, String),                   // repo, git_ref
    FetchContents(RepoId, String, String),       // repo, path, git_ref
    FetchFileContent(RepoId, String, String),    // repo, path, git_ref
    FetchFileBytes(RepoId, String, String),      // repo, path, git_ref (for binary/image files)
    FetchReadme(RepoId, String, Option<String>), // repo, git_ref, optional known path
    FetchBranches(RepoId),
    FetchTags(RepoId),
    FetchCommits(RepoId, String, String, u32), // repo, git_ref, path, per_page
    FetchCommitDetail(RepoId, String),         // repo, sha
    UpdateFileContent(RepoId, String, String, String, String, String), // repo, path, content, message, sha, branch

    // Settings
    FetchRepoSettings(RepoId),
    FetchBranchProtections(RepoId),
    FetchCollaborators(RepoId),
    FetchWebhooks(RepoId),
    FetchDeployKeys(RepoId),
    UpdateRepo(RepoId, serde_json::Value),
    RemoveCollaborator(RepoId, String),     // repo, username
    DeleteWebhook(RepoId, u64),             // repo, hook_id
    ToggleWebhook(RepoId, u64, bool),       // repo, hook_id, active
    DeleteDeployKey(RepoId, u64),           // repo, key_id
    DeleteBranchProtection(RepoId, String), // repo, branch_name
    ToggleBranchEnforceAdmins(RepoId, String, bool), // repo, branch, enable

    // Discussions
    FetchDiscussions(RepoId),

    // Gists
    FetchGists,

    // Organizations
    FetchOrgs,
    FetchOrgMembers(String), // org login

    // Multi-repo dashboard
    FetchRecentRepos,

    // Utility
    OpenInBrowser(String),
    SetClipboard(String),
    Quit,
}
