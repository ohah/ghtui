use crate::editor::TextEditor;
use crate::types::common::{Label, Milestone};
use crate::types::{Issue, IssueDetail, IssueFilters, IssueState, Pagination};

#[derive(Debug)]
pub struct IssueListState {
    pub items: Vec<Issue>,
    pub pagination: Pagination,
    pub selected: usize,
    pub scroll_offset: usize,
    pub filters: IssueFilters,
    pub search_mode: bool,
    pub search_query: String,
}

/// Label picker state for issue detail
#[derive(Debug)]
pub struct LabelPickerState {
    pub available: Vec<Label>,
    pub selected_names: Vec<String>,
    pub cursor: usize,
}

/// Assignee picker state
#[derive(Debug)]
pub struct AssigneePickerState {
    pub available: Vec<String>, // login names
    pub selected_names: Vec<String>,
    pub cursor: usize,
}

/// Milestone picker state
#[derive(Debug)]
pub struct MilestonePickerState {
    pub available: Vec<Milestone>,
    pub selected: Option<u64>, // milestone number
    pub cursor: usize,
}

/// Reaction picker state
#[derive(Debug)]
pub struct ReactionPickerState {
    pub cursor: usize,
}

pub const REACTION_OPTIONS: &[(&str, &str)] = &[
    ("+1", "👍"),
    ("-1", "👎"),
    ("laugh", "😄"),
    ("hooray", "🎉"),
    ("confused", "😕"),
    ("heart", "❤️"),
    ("rocket", "🚀"),
    ("eyes", "👀"),
];

impl IssueListState {
    pub fn new(items: Vec<Issue>, pagination: Pagination) -> Self {
        Self {
            items,
            pagination,
            selected: 0,
            scroll_offset: 0,
            filters: IssueFilters::default(),
            search_mode: false,
            search_query: String::new(),
        }
    }

    pub fn with_filters(items: Vec<Issue>, pagination: Pagination, filters: IssueFilters) -> Self {
        Self {
            items,
            pagination,
            selected: 0,
            scroll_offset: 0,
            filters,
            search_mode: false,
            search_query: String::new(),
        }
    }

    pub fn selected_issue(&self) -> Option<&Issue> {
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

    pub fn toggle_state_filter(&mut self) {
        self.filters.state = match self.filters.state {
            None | Some(IssueState::Open) => Some(IssueState::Closed),
            Some(IssueState::Closed) => Some(IssueState::Open),
        };
    }

    pub fn cycle_sort(&mut self) {
        self.filters.sort = match self.filters.sort.as_deref() {
            None | Some("created") => Some("updated".to_string()),
            Some("updated") => Some("comments".to_string()),
            Some("comments") => Some("created".to_string()),
            _ => Some("created".to_string()),
        };
    }

    pub fn sort_display(&self) -> &str {
        match self.filters.sort.as_deref() {
            None | Some("created") => "Newest",
            Some("updated") => "Recently updated",
            Some("comments") => "Most commented",
            _ => "Newest",
        }
    }
}

/// Focus sections in issue detail view
#[derive(Debug, Clone, PartialEq)]
pub enum IssueSection {
    Title,
    Labels,
    Assignees,
    Milestone,
    Body,
    Comment(usize),
}

impl IssueSection {
    pub fn action_hint(&self) -> &'static str {
        match self {
            IssueSection::Title => "e:Edit  o:Open in browser",
            IssueSection::Labels => "l:Edit labels",
            IssueSection::Assignees => "a:Edit assignees",
            IssueSection::Milestone => "",
            IssueSection::Body => "e:Edit body",
            IssueSection::Comment(_) => "e:Edit  r:Reply  d:Delete",
        }
    }
}

/// What is being edited inline
#[derive(Debug, Clone, PartialEq)]
pub enum InlineEditTarget {
    IssueTitle,
    IssueBody,
    Comment(usize),
    NewComment,
    QuoteReply(usize),
}

#[derive(Debug)]
pub struct IssueDetailState {
    pub detail: IssueDetail,
    pub scroll: usize,
    pub focus: IssueSection,
    pub edit_target: Option<InlineEditTarget>,
    pub editor: TextEditor,
    pub label_picker: Option<LabelPickerState>,
    pub assignee_picker: Option<AssigneePickerState>,
    pub milestone_picker: Option<MilestonePickerState>,
    pub reaction_picker: Option<ReactionPickerState>,
}

impl IssueDetailState {
    pub fn new(detail: IssueDetail) -> Self {
        Self {
            detail,
            scroll: 0,
            focus: IssueSection::Title,
            edit_target: None,
            editor: TextEditor::new(),
            label_picker: None,
            assignee_picker: None,
            milestone_picker: None,
            reaction_picker: None,
        }
    }

    pub fn is_editing(&self) -> bool {
        self.edit_target.is_some()
    }

    pub fn has_picker(&self) -> bool {
        self.label_picker.is_some()
            || self.assignee_picker.is_some()
            || self.milestone_picker.is_some()
            || self.reaction_picker.is_some()
    }

    /// All navigable sections in order
    fn sections(&self) -> Vec<IssueSection> {
        let mut sections = vec![
            IssueSection::Title,
            IssueSection::Labels,
            IssueSection::Assignees,
            IssueSection::Body,
        ];
        for i in 0..self.detail.comments.len() {
            sections.push(IssueSection::Comment(i));
        }
        sections
    }

    pub fn focus_next(&mut self) {
        let sections = self.sections();
        if let Some(idx) = sections.iter().position(|s| s == &self.focus) {
            if idx < sections.len() - 1 {
                self.focus = sections[idx + 1].clone();
            }
        }
    }

    pub fn focus_prev(&mut self) {
        let sections = self.sections();
        if let Some(idx) = sections.iter().position(|s| s == &self.focus) {
            if idx > 0 {
                self.focus = sections[idx - 1].clone();
            }
        }
    }

    pub fn start_edit_title(&mut self) {
        self.editor = TextEditor::from_string(&self.detail.issue.title);
        self.edit_target = Some(InlineEditTarget::IssueTitle);
    }

    pub fn start_edit_body(&mut self) {
        self.editor = TextEditor::from_string(self.detail.issue.body.as_deref().unwrap_or(""));
        self.edit_target = Some(InlineEditTarget::IssueBody);
    }

    pub fn start_edit_comment(&mut self, index: usize) {
        if let Some(comment) = self.detail.comments.get(index) {
            self.editor = TextEditor::from_string(&comment.body);
            self.edit_target = Some(InlineEditTarget::Comment(index));
        }
    }

    pub fn start_new_comment(&mut self) {
        self.editor = TextEditor::new();
        self.edit_target = Some(InlineEditTarget::NewComment);
    }

    pub fn start_quote_reply(&mut self, index: usize) {
        if let Some(comment) = self.detail.comments.get(index) {
            let quoted: String = comment
                .body
                .lines()
                .map(|l| format!("> {}", l))
                .collect::<Vec<_>>()
                .join("\n");
            let text = format!("> @{}\n{}\n\n", comment.user.login, quoted);
            self.editor = TextEditor::from_string(&text);
            self.edit_target = Some(InlineEditTarget::QuoteReply(index));
        }
    }

    pub fn cancel_edit(&mut self) {
        self.edit_target = None;
        self.editor = TextEditor::new();
    }

    pub fn editor_text(&self) -> String {
        self.editor.content()
    }

    // Legacy compat
    pub fn selected_comment(&self) -> Option<usize> {
        match &self.focus {
            IssueSection::Comment(i) => Some(*i),
            _ => None,
        }
    }
}
