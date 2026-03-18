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
    pub editing: Option<SettingsEditField>,
    pub edit_buffer: String,
}

#[derive(Debug, Clone, PartialEq)]
pub enum SettingsEditField {
    Description,
    DefaultBranch,
    Topics,
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
            editing: None,
            edit_buffer: String::new(),
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
