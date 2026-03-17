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
    PrListLoaded(Vec<PullRequest>, Pagination, PrFilters),
    PrDetailLoaded(Box<PullRequestDetail>),
    PrDiffLoaded(Vec<DiffFile>),
    PrMerged(u64),
    PrClosed(u64),
    PrReopened(u64),
    PrCreated(u64),
    PrUpdated(u64),
    ReviewSubmitted,
    PrToggleStateFilter,
    PrSortCycle,
    PrNextPage,
    PrPrevPage,
    PrSearchStart,
    PrSearchInput(String),
    PrSearchSubmit,
    PrSearchCancel,
    PrFilterByLabel(String),
    PrFilterByAuthor(String),
    PrFilterByAssignee(String),
    PrFilterClear,
    // PR detail
    PrToggleState,
    PrOpenInBrowser,
    PrDeleteComment,
    PrStartEditTitle,
    PrStartEditBody,
    PrStartComment,
    PrStartReply,
    PrEditChar(char),
    PrEditNewline,
    PrEditBackspace,
    PrEditDelete,
    PrEditTab,
    PrEditCursorLeft,
    PrEditCursorRight,
    PrEditCursorUp,
    PrEditCursorDown,
    PrEditWordLeft,
    PrEditWordRight,
    PrEditHome,
    PrEditEnd,
    PrEditPageUp,
    PrEditPageDown,
    PrEditUndo,
    PrEditRedo,
    PrEditSubmit,
    PrEditCancel,
    PrLabelToggle,
    PrLabelSelect(usize),
    PrLabelApply,
    PrLabelCancel,
    PrLabelsLoaded(Vec<common::Label>),
    PrAssigneeToggle,
    PrAssigneeSelect(usize),
    PrAssigneeApply,
    PrAssigneeCancel,
    PrCollaboratorsLoaded(Vec<String>),
    PrAddReaction(String),
    // PR diff navigation
    PrDiffCursorDown,
    PrDiffCursorUp,
    PrDiffSelectDown,       // Shift+j/Down — extend selection
    PrDiffSelectUp,         // Shift+k/Up — extend selection
    PrDiffToggleCollapse,   // Enter — fold/unfold file
    PrDiffExpand,           // l/Right — unfold file
    PrDiffCollapse,         // h/Left — fold file
    PrDiffClearSelection,   // Esc in diff
    PrDiffCommentSubmit,    // Ctrl+Enter in diff comment editor
    PrDiffCommentCancel,    // Esc in diff comment editor
    PrDiffInsertSuggestion, // Insert suggestion template
    PrDiffToggleTree,       // Toggle file tree panel
    PrDiffTreeFocus,        // Toggle focus between tree and diff
    PrDiffTreeUp,           // Move up in file tree
    PrDiffTreeDown,         // Move down in file tree
    PrDiffTreeSelect,       // Select file in tree → jump to diff
    PrChangeBase,           // Change merge target branch

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
    IssueLockToggle,
    IssuePinToggle,
    IssuePinnedNumbersLoaded(Vec<u64>),
    IssueFilterByLabel(String),        // Set label filter
    IssueFilterByAuthor(String),       // Set author filter
    IssueFilterByAssignee(String),     // Set assignee filter
    IssueFilterClear,                  // Clear all filters
    IssueTransfer,                     // Transfer issue to another repo
    IssueTemplatesLoaded(Vec<String>), // Template names loaded
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
