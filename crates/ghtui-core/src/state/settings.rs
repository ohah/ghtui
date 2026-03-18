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

#[derive(Debug, Clone)]
pub struct SettingsFormField {
    pub label: String,
    pub value: String,
    pub field_type: SettingsFieldType,
    pub required: bool,
}

#[derive(Debug)]
pub struct SettingsFormState {
    pub kind: SettingsFormKind,
    pub fields: Vec<SettingsFormField>,
    pub focused_field: usize,
    pub editing: bool,
    pub edit_buffer: String,
}

impl SettingsFormState {
    pub fn branch_protection_create() -> Self {
        Self {
            kind: SettingsFormKind::CreateBranchProtection,
            fields: vec![
                SettingsFormField {
                    label: "Branch".into(),
                    value: String::new(),
                    field_type: SettingsFieldType::Text,
                    required: true,
                },
                SettingsFormField {
                    label: "Strict status checks".into(),
                    value: "false".into(),
                    field_type: SettingsFieldType::Bool,
                    required: false,
                },
                SettingsFormField {
                    label: "Status check contexts".into(),
                    value: String::new(),
                    field_type: SettingsFieldType::Text,
                    required: false,
                },
                SettingsFormField {
                    label: "Enforce admins".into(),
                    value: "false".into(),
                    field_type: SettingsFieldType::Bool,
                    required: false,
                },
                SettingsFormField {
                    label: "Required reviews".into(),
                    value: "0".into(),
                    field_type: SettingsFieldType::Text,
                    required: false,
                },
                SettingsFormField {
                    label: "Dismiss stale reviews".into(),
                    value: "false".into(),
                    field_type: SettingsFieldType::Bool,
                    required: false,
                },
                SettingsFormField {
                    label: "Require code owner reviews".into(),
                    value: "false".into(),
                    field_type: SettingsFieldType::Bool,
                    required: false,
                },
            ],
            focused_field: 0,
            editing: false,
            edit_buffer: String::new(),
        }
    }

    pub fn branch_protection_edit(bp: &BranchProtection) -> Self {
        let sc = bp.required_status_checks.as_ref();
        let pr = bp.required_pull_request_reviews.as_ref();
        let ea = bp
            .enforce_admins
            .as_ref()
            .map(|e| e.enabled)
            .unwrap_or(false);
        Self {
            kind: SettingsFormKind::EditBranchProtection(bp.pattern.clone()),
            fields: vec![
                SettingsFormField {
                    label: "Branch".into(),
                    value: bp.pattern.clone(),
                    field_type: SettingsFieldType::Text,
                    required: true,
                },
                SettingsFormField {
                    label: "Strict status checks".into(),
                    value: sc.map(|s| s.strict).unwrap_or(false).to_string(),
                    field_type: SettingsFieldType::Bool,
                    required: false,
                },
                SettingsFormField {
                    label: "Status check contexts".into(),
                    value: sc.map(|s| s.contexts.join(", ")).unwrap_or_default(),
                    field_type: SettingsFieldType::Text,
                    required: false,
                },
                SettingsFormField {
                    label: "Enforce admins".into(),
                    value: ea.to_string(),
                    field_type: SettingsFieldType::Bool,
                    required: false,
                },
                SettingsFormField {
                    label: "Required reviews".into(),
                    value: pr
                        .and_then(|p| p.required_approving_review_count)
                        .unwrap_or(0)
                        .to_string(),
                    field_type: SettingsFieldType::Text,
                    required: false,
                },
                SettingsFormField {
                    label: "Dismiss stale reviews".into(),
                    value: pr
                        .map(|p| p.dismiss_stale_reviews)
                        .unwrap_or(false)
                        .to_string(),
                    field_type: SettingsFieldType::Bool,
                    required: false,
                },
                SettingsFormField {
                    label: "Require code owner reviews".into(),
                    value: pr
                        .map(|p| p.require_code_owner_reviews)
                        .unwrap_or(false)
                        .to_string(),
                    field_type: SettingsFieldType::Bool,
                    required: false,
                },
            ],
            focused_field: 0,
            editing: false,
            edit_buffer: String::new(),
        }
    }

    pub fn add_collaborator() -> Self {
        Self {
            kind: SettingsFormKind::AddCollaborator,
            fields: vec![
                SettingsFormField {
                    label: "Username".into(),
                    value: String::new(),
                    field_type: SettingsFieldType::Text,
                    required: true,
                },
                SettingsFormField {
                    label: "Permission".into(),
                    value: "push".into(),
                    field_type: SettingsFieldType::Select(vec![
                        "pull".into(),
                        "triage".into(),
                        "push".into(),
                        "maintain".into(),
                        "admin".into(),
                    ]),
                    required: true,
                },
            ],
            focused_field: 0,
            editing: false,
            edit_buffer: String::new(),
        }
    }

    pub fn create_webhook() -> Self {
        Self {
            kind: SettingsFormKind::CreateWebhook,
            fields: vec![
                SettingsFormField {
                    label: "URL".into(),
                    value: String::new(),
                    field_type: SettingsFieldType::Text,
                    required: true,
                },
                SettingsFormField {
                    label: "Content type".into(),
                    value: "json".into(),
                    field_type: SettingsFieldType::Select(vec!["json".into(), "form".into()]),
                    required: true,
                },
                SettingsFormField {
                    label: "Events".into(),
                    value: "push".into(),
                    field_type: SettingsFieldType::Text,
                    required: true,
                },
                SettingsFormField {
                    label: "Active".into(),
                    value: "true".into(),
                    field_type: SettingsFieldType::Bool,
                    required: false,
                },
            ],
            focused_field: 0,
            editing: false,
            edit_buffer: String::new(),
        }
    }

    pub fn edit_webhook(hook: &Webhook) -> Self {
        Self {
            kind: SettingsFormKind::EditWebhook(hook.id),
            fields: vec![
                SettingsFormField {
                    label: "URL".into(),
                    value: hook.config.url.clone().unwrap_or_default(),
                    field_type: SettingsFieldType::Text,
                    required: true,
                },
                SettingsFormField {
                    label: "Content type".into(),
                    value: hook.config.content_type.clone().unwrap_or("json".into()),
                    field_type: SettingsFieldType::Select(vec!["json".into(), "form".into()]),
                    required: true,
                },
                SettingsFormField {
                    label: "Events".into(),
                    value: hook.events.join(", "),
                    field_type: SettingsFieldType::Text,
                    required: true,
                },
                SettingsFormField {
                    label: "Active".into(),
                    value: hook.active.to_string(),
                    field_type: SettingsFieldType::Bool,
                    required: false,
                },
            ],
            focused_field: 0,
            editing: false,
            edit_buffer: String::new(),
        }
    }

    pub fn create_deploy_key() -> Self {
        Self {
            kind: SettingsFormKind::CreateDeployKey,
            fields: vec![
                SettingsFormField {
                    label: "Title".into(),
                    value: String::new(),
                    field_type: SettingsFieldType::Text,
                    required: true,
                },
                SettingsFormField {
                    label: "Key (public key)".into(),
                    value: String::new(),
                    field_type: SettingsFieldType::Text,
                    required: true,
                },
                SettingsFormField {
                    label: "Read only".into(),
                    value: "true".into(),
                    field_type: SettingsFieldType::Bool,
                    required: false,
                },
            ],
            focused_field: 0,
            editing: false,
            edit_buffer: String::new(),
        }
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
