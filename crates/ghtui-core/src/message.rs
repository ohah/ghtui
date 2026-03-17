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
    IssueListLoaded(Vec<Issue>, Pagination, IssueFilters),
    IssueDetailLoaded(Box<IssueDetail>),
    IssueClosed(u64),
    IssueReopened(u64),
    IssueCreated(u64),
    IssueUpdated(u64),
    CommentAdded,
    CommentUpdated,
    IssueToggleStateFilter,
    IssueSortCycle, // Cycle sort: created/updated/comments
    IssueNextPage,
    IssuePrevPage,
    IssueSearchStart,
    IssueLockToggle, // Lock/unlock issue
    IssueSearchInput(String),
    IssueSearchSubmit,
    IssueSearchCancel,
    IssueLabelToggle,
    IssueLabelSelect(usize),
    IssueLabelApply,
    IssueLabelCancel,
    IssueLabelsLoaded(Vec<common::Label>),
    IssueAssigneeToggle,
    IssueAssigneeSelect(usize),
    IssueAssigneeApply,
    IssueAssigneeCancel,
    IssueCollaboratorsLoaded(Vec<String>),
    IssueDeleteComment,
    CommentDeleted,
    IssueToggleState,
    IssueOpenInBrowser,
    IssueAddReaction(String), // reaction content ("+1", "heart", etc.)
    ReactionAdded,
    IssueMilestoneToggle,
    IssueMilestoneSelect(usize),
    IssueMilestoneApply,
    IssueMilestoneClear,
    IssueMilestoneCancel,
    IssueMilestonesLoaded(Vec<common::Milestone>),
    IssueStartEditTitle,
    IssueStartEditBody,
    IssueStartComment,
    IssueStartReply, // Start quote reply to selected comment
    IssueEditChar(char),
    IssueEditNewline,
    IssueEditBackspace,
    IssueEditDelete,
    IssueEditTab,
    IssueEditCursorLeft,
    IssueEditCursorRight,
    IssueEditCursorUp,
    IssueEditCursorDown,
    IssueEditWordLeft,
    IssueEditWordRight,
    IssueEditHome,
    IssueEditEnd,
    IssueEditPageUp,
    IssueEditPageDown,
    IssueEditUndo,
    IssueEditRedo,
    IssueEditSubmit,
    IssueEditCancel,

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

    // Insights
    ContributorStatsLoaded(Vec<insights::ContributorStats>),
    CommitActivityLoaded(Vec<insights::CommitActivity>),
    TrafficClonesLoaded(insights::TrafficClones),
    TrafficViewsLoaded(insights::TrafficViews),

    // Security
    DependabotAlertsLoaded(Vec<security::DependabotAlert>),
    CodeScanningAlertsLoaded(Vec<security::CodeScanningAlert>),
    SecretScanningAlertsLoaded(Vec<security::SecretScanningAlert>),

    // Settings
    SettingsRepoLoaded(Box<settings::Repository>),
    SettingsBranchProtectionsLoaded(Vec<settings::BranchProtection>),
    SettingsCollaboratorsLoaded(Vec<settings::Collaborator>),

    // Mouse
    ScrollUp,
    ScrollDown,
    MouseClick(u16, u16), // (column, row)

    // UI
    InputChanged(String),
    ListSelect(usize),
    TabChanged(usize),
    GlobalTabNext,
    GlobalTabPrev,
    GlobalTabSelect(usize),
    ToggleTheme,
    ModalOpen(ModalKind),
    ModalSubmit,
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
    EditIssue,
    AddComment,
    EditComment(u64), // comment_id
    SubmitReview,
    Confirm { title: String, message: String },
    AccountSwitcher,
    Help,
}
