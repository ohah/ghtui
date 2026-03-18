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
    PrMilestoneToggle,      // Open milestone picker
    PrMilestoneSelect(usize),
    PrMilestoneApply,
    PrMilestoneClear,
    PrMilestoneCancel,
    PrDraftToggle,               // Toggle draft status
    PrAutoMergeToggle,           // Toggle auto-merge
    PrDiffMarkViewed,            // Mark file as viewed (local)
    PrDiffToggleSideBySide,      // Toggle side-by-side diff mode
    PrReviewThreadToggleResolve, // Resolve/unresolve review thread at cursor
    PrReviewerToggle,            // Open reviewer picker
    PrReviewerApply,             // Apply reviewer changes
    PrReviewerCancel,            // Cancel reviewer picker
    PrApprove,                   // Approve PR
    PrRequestChanges,            // Request changes on PR
    PrActionBarFocus,            // Toggle focus on action bar
    PrActionBarLeft,             // Move left in action bar
    PrActionBarRight,            // Move right in action bar
    PrActionBarSelect,           // Execute selected action
    PrChangeBase,                // Change merge target branch

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
    RunsLoaded(Vec<WorkflowRun>, Pagination, ActionsFilters),
    RunDetailLoaded(Box<WorkflowRunDetail>),
    JobLogLoaded(u64, Vec<LogLine>),
    RunCancelled(u64),
    RunRerun(u64),
    WorkflowsLoaded(Vec<Workflow>),
    ActionsToggleStatus,
    ActionsCycleEvent,
    ActionsNextPage,
    ActionsPrevPage,
    ActionsSearchStart,
    ActionsSearchInput(String),
    ActionsSearchSubmit,
    ActionsSearchCancel,
    ActionsFilterClear,
    ActionsSelectWorkflow(Option<u64>),
    ActionsOpenInBrowser,
    ActionsCancelRun,              // Cancel selected run from list
    ActionsRerunRun,               // Re-run selected run from list
    ActionsToggleWorkflowSidebar,  // Toggle workflow sidebar
    ActionsWorkflowSidebarUp,      // Navigate up in workflow sidebar
    ActionsWorkflowSidebarDown,    // Navigate down in workflow sidebar
    ActionsWorkflowSidebarSelect,  // Select workflow from sidebar
    ActionsDispatchOpen,           // Open dispatch modal for selected workflow
    ActionsDispatchClose,          // Close dispatch modal
    ActionsDispatchSubmit,         // Submit dispatch
    ActionsDispatchFieldNext,      // Next field in dispatch form
    ActionsDispatchFieldPrev,      // Prev field in dispatch form
    ActionsDispatchEditStart,      // Start editing current field
    ActionsDispatchEditChar(char), // Type char in dispatch field
    ActionsDispatchEditBackspace,  // Backspace in dispatch field
    ActionsDispatchEditDone,       // Finish editing field (Enter/Esc)
    WorkflowInputsLoaded(u64, String, Vec<WorkflowInput>), // workflow_id, name, inputs
    // Action detail
    ActionDetailToggleStep(u32), // toggle step fold by step number
    ActionDetailFocusJobs,
    ActionDetailFocusLog,
    ActionDetailActionBarFocus,
    ActionDetailActionBarLeft,
    ActionDetailActionBarRight,
    ActionDetailActionBarSelect,
    ActionDetailOpenInBrowser,
    RunRerunFailed(u64),
    RunDeleted(u64),
    ArtifactsLoaded(Vec<Artifact>),
    ArtifactDownloaded(String, String), // artifact_name, download_url
    WorkflowDispatched,
    WorkflowFileLoaded(String), // file content
    PendingDeploymentsLoaded(Vec<PendingDeployment>),
    DeploymentApproved,
    DeploymentRejected,

    // Notifications
    NotificationsLoaded(Vec<Notification>),
    NotificationMarkedRead(String),
    NotificationNavigate,
    NotificationMarkRead,
    NotificationMarkAllRead,
    NotificationUnsubscribe,
    NotificationDone,
    NotificationCycleReason,
    NotificationCycleType,
    NotificationToggleGrouped,
    NotificationAllMarkedRead,
    NotificationUnsubscribed(String),
    NotificationDoneResult(String),

    // Search
    SearchResults(SearchResultSet),
    SearchOpen,          // Open global search
    SearchInput(String), // Search query input
    SearchSubmit,        // Submit search query
    SearchCancel,        // Cancel search
    SearchCycleKind,     // Cycle search kind (Repos/Issues/Code)
    SearchNavigate,      // Enter on selected result
    SearchHistoryPrev,   // Previous search history (Up in input mode)
    SearchHistoryNext,   // Next search history (Down in input mode)

    // Insights
    ContributorStatsLoaded(Vec<insights::ContributorStats>),
    CommitActivityLoaded(Vec<insights::CommitActivity>),
    TrafficClonesLoaded(insights::TrafficClones),
    TrafficViewsLoaded(insights::TrafficViews),
    CodeFrequencyLoaded(Vec<insights::CodeFrequency>),
    ForksLoaded(Vec<insights::Fork>),
    DependencyGraphLoaded(Vec<insights::DependencyEntry>),

    // Insights sidebar
    InsightsSidebarFocus,

    // Security
    SecuritySidebarFocus,
    DependabotAlertsLoaded(Vec<security::DependabotAlert>),
    CodeScanningAlertsLoaded(Vec<security::CodeScanningAlert>),
    SecretScanningAlertsLoaded(Vec<security::SecretScanningAlert>),
    SecurityAdvisoriesLoaded(Vec<security::RepoSecurityAdvisory>),
    SecurityToggleDetail,
    SecurityOpenInBrowser,
    SecurityDismissAlert,
    SecurityReopenAlert,
    SecurityAlertUpdated(usize), // tab index: 0=dependabot, 1=code, 2=secret

    // Settings
    SettingsRepoLoaded(Box<settings::Repository>),
    SettingsBranchProtectionsLoaded(Vec<settings::BranchProtection>),
    SettingsCollaboratorsLoaded(Vec<settings::Collaborator>),
    SettingsWebhooksLoaded(Vec<settings::Webhook>),
    SettingsDeployKeysLoaded(Vec<settings::DeployKey>),
    SettingsRepoUpdated(Box<settings::Repository>),
    SettingsStartEdit(String), // field name
    SettingsEditChar(char),
    SettingsEditBackspace,
    SettingsEditSubmit,
    SettingsEditCancel,
    SettingsToggleFeature(String), // feature name (has_issues, has_wiki, etc.)
    SettingsToggleVisibility,      // Toggle public/private (with confirmation)
    SettingsSidebarFocus,          // Toggle sidebar/content focus
    SettingsDeleteCollaborator,    // Remove selected collaborator
    SettingsDeleteWebhook,         // Delete selected webhook
    SettingsToggleWebhook,         // Toggle webhook active/inactive
    SettingsDeleteDeployKey,       // Delete selected deploy key
    SettingsDeleteBranchProtection, // Delete selected branch protection rule
    SettingsToggleBranchEnforceAdmins, // Toggle enforce_admins on selected rule
    SettingsItemUpdated(usize), // tab index: 1=branch, 2=collaborators, 3=webhooks, 4=deploy_keys

    // Code tab
    CodeContentsLoaded(Vec<crate::types::code::FileEntry>),
    CodeFileLoaded(String, String), // filename, content
    CodeReadmeLoaded(String),       // readme content
    CodeNavigateInto,               // Enter on dir -> navigate into; on file -> view
    CodeNavigateBack,               // Backspace/Esc -> go up
    CodeSidebarFocus,               // Toggle sidebar/content focus

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
