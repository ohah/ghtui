use crate::types::settings::{BranchProtection, Collaborator, DeployKey, Repository, Webhook};

#[derive(Debug)]
pub struct SettingsState {
    pub repo: Repository,
    pub branch_protections: Vec<BranchProtection>,
    pub collaborators: Vec<Collaborator>,
    pub webhooks: Vec<Webhook>,
    pub deploy_keys: Vec<DeployKey>,
    pub scroll: usize,
    pub tab: usize, // 0=General, 1=Branch Protection, 2=Collaborators, 3=Webhooks, 4=Deploy Keys
    pub selected: usize, // selected item within each tab (collaborators/webhooks/deploy_keys)
    pub sidebar_focused: bool, // true = sidebar focused, false = content focused
    pub editing: Option<SettingsEditField>,
    pub edit_buffer: String,
    pub form: Option<SettingsFormState>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum SettingsEditField {
    Description,
    DefaultBranch,
    Topics,
}

// --- Settings Form (create/edit modal) ---

#[derive(Debug, Clone, PartialEq)]
pub enum SettingsFormKind {
    CreateBranchProtection,
    EditBranchProtection(String), // branch pattern
    AddCollaborator,
    CreateWebhook,
    EditWebhook(u64), // hook_id
    CreateDeployKey,
}

#[derive(Debug, Clone, PartialEq)]
pub enum SettingsFieldType {
    Text,
    Bool,
    Select(Vec<String>),
}

static PERMISSIONS: &[&str] = &["pull", "triage", "push", "maintain", "admin"];
static CONTENT_TYPES: &[&str] = &["json", "form"];

#[derive(Debug, Clone)]
pub struct SettingsFormField {
    pub label: String,
    pub value: String,
    pub field_type: SettingsFieldType,
    pub required: bool,
}

impl SettingsFormField {
    fn text(label: &str, value: &str, required: bool) -> Self {
        Self {
            label: label.into(),
            value: value.into(),
            field_type: SettingsFieldType::Text,
            required,
        }
    }

    fn bool(label: &str, value: bool) -> Self {
        Self {
            label: label.into(),
            value: value.to_string(),
            field_type: SettingsFieldType::Bool,
            required: false,
        }
    }

    fn select(label: &str, value: &str, options: &[&str]) -> Self {
        Self {
            label: label.into(),
            value: value.into(),
            field_type: SettingsFieldType::Select(options.iter().map(|s| (*s).into()).collect()),
            required: true,
        }
    }
}

#[derive(Debug)]
pub struct SettingsFormState {
    pub kind: SettingsFormKind,
    pub fields: Vec<SettingsFormField>,
    pub focused_field: usize,
    pub field_editing: bool,
    pub field_buffer: String,
}

impl SettingsFormState {
    fn new(kind: SettingsFormKind, fields: Vec<SettingsFormField>) -> Self {
        Self {
            kind,
            fields,
            focused_field: 0,
            field_editing: false,
            field_buffer: String::new(),
        }
    }

    /// Look up a field value by label. Returns empty string if not found.
    pub fn get_value(&self, label: &str) -> &str {
        self.fields
            .iter()
            .find(|f| f.label == label)
            .map(|f| f.value.as_str())
            .unwrap_or("")
    }

    /// Look up a boolean field by label.
    pub fn get_bool(&self, label: &str) -> bool {
        self.get_value(label) == "true"
    }

    fn branch_protection_fields(
        branch: &str,
        strict: bool,
        contexts: &str,
        enforce_admins: bool,
        reviews: &str,
        dismiss_stale: bool,
        require_codeowner: bool,
    ) -> Vec<SettingsFormField> {
        vec![
            SettingsFormField::text("Branch", branch, true),
            SettingsFormField::bool("Strict status checks", strict),
            SettingsFormField::text("Status check contexts", contexts, false),
            SettingsFormField::bool("Enforce admins", enforce_admins),
            SettingsFormField::text("Required reviews", reviews, false),
            SettingsFormField::bool("Dismiss stale reviews", dismiss_stale),
            SettingsFormField::bool("Require code owner reviews", require_codeowner),
        ]
    }

    pub fn branch_protection_create() -> Self {
        Self::new(
            SettingsFormKind::CreateBranchProtection,
            Self::branch_protection_fields("", false, "", false, "0", false, false),
        )
    }

    pub fn branch_protection_edit(bp: &BranchProtection) -> Self {
        let sc = bp.required_status_checks.as_ref();
        let pr = bp.required_pull_request_reviews.as_ref();
        let ea = bp.enforce_admins.as_ref().is_some_and(|e| e.enabled);
        Self::new(
            SettingsFormKind::EditBranchProtection(bp.pattern.clone()),
            Self::branch_protection_fields(
                &bp.pattern,
                sc.map(|s| s.strict).unwrap_or(false),
                &sc.map(|s| s.contexts.join(", ")).unwrap_or_default(),
                ea,
                &pr.and_then(|p| p.required_approving_review_count)
                    .unwrap_or(0)
                    .to_string(),
                pr.map(|p| p.dismiss_stale_reviews).unwrap_or(false),
                pr.map(|p| p.require_code_owner_reviews).unwrap_or(false),
            ),
        )
    }

    pub fn add_collaborator() -> Self {
        Self::new(
            SettingsFormKind::AddCollaborator,
            vec![
                SettingsFormField::text("Username", "", true),
                SettingsFormField::select("Permission", "push", PERMISSIONS),
            ],
        )
    }

    fn webhook_fields(
        url: &str,
        content_type: &str,
        events: &str,
        active: bool,
    ) -> Vec<SettingsFormField> {
        vec![
            SettingsFormField::text("URL", url, true),
            SettingsFormField::select("Content type", content_type, CONTENT_TYPES),
            SettingsFormField::text("Events", events, true),
            SettingsFormField::bool("Active", active),
        ]
    }

    pub fn create_webhook() -> Self {
        Self::new(
            SettingsFormKind::CreateWebhook,
            Self::webhook_fields("", "json", "push", true),
        )
    }

    pub fn edit_webhook(hook: &Webhook) -> Self {
        Self::new(
            SettingsFormKind::EditWebhook(hook.id),
            Self::webhook_fields(
                hook.config.url.as_deref().unwrap_or_default(),
                hook.config.content_type.as_deref().unwrap_or("json"),
                &hook.events.join(", "),
                hook.active,
            ),
        )
    }

    pub fn create_deploy_key() -> Self {
        Self::new(
            SettingsFormKind::CreateDeployKey,
            vec![
                SettingsFormField::text("Title", "", true),
                SettingsFormField::text("Key (public key)", "", true),
                SettingsFormField::bool("Read only", true),
            ],
        )
    }

    pub fn title(&self) -> &str {
        match &self.kind {
            SettingsFormKind::CreateBranchProtection => "Create Branch Protection",
            SettingsFormKind::EditBranchProtection(_) => "Edit Branch Protection",
            SettingsFormKind::AddCollaborator => "Add Collaborator",
            SettingsFormKind::CreateWebhook => "Create Webhook",
            SettingsFormKind::EditWebhook(_) => "Edit Webhook",
            SettingsFormKind::CreateDeployKey => "Create Deploy Key",
        }
    }
}

impl SettingsState {
    pub fn new(repo: Repository) -> Self {
        Self {
            repo,
            branch_protections: Vec::new(),
            collaborators: Vec::new(),
            webhooks: Vec::new(),
            deploy_keys: Vec::new(),
            scroll: 0,
            tab: 0,
            selected: 0,
            sidebar_focused: true,
            editing: None,
            edit_buffer: String::new(),
            form: None,
        }
    }

    pub fn tab_count(&self) -> usize {
        5
    }

    pub fn start_edit(&mut self, field: SettingsEditField) {
        self.edit_buffer = match &field {
            SettingsEditField::Description => self.repo.description.clone().unwrap_or_default(),
            SettingsEditField::DefaultBranch => self.repo.default_branch.clone(),
            SettingsEditField::Topics => self
                .repo
                .topics
                .as_ref()
                .map(|t| t.join(", "))
                .unwrap_or_default(),
        };
        self.editing = Some(field);
    }

    pub fn cancel_edit(&mut self) {
        self.editing = None;
        self.edit_buffer.clear();
    }

    pub fn is_editing(&self) -> bool {
        self.editing.is_some()
    }
}
