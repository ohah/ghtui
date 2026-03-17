use crate::types::settings::{BranchProtection, Collaborator, Repository};

#[derive(Debug)]
pub struct SettingsState {
    pub repo: Repository,
    pub branch_protections: Vec<BranchProtection>,
    pub collaborators: Vec<Collaborator>,
    pub scroll: usize,
    pub tab: usize, // 0=General, 1=Branch Protection, 2=Collaborators
}

impl SettingsState {
    pub fn new(repo: Repository) -> Self {
        Self {
            repo,
            branch_protections: Vec::new(),
            collaborators: Vec::new(),
            scroll: 0,
            tab: 0,
        }
    }

    pub fn tab_count(&self) -> usize {
        3
    }
}
