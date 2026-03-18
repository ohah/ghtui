use crate::types::{OrgInfo, OrgMember};

#[derive(Debug)]
pub struct OrgState {
    pub orgs: Vec<OrgInfo>,
    pub selected_org: usize,
    pub members: Vec<OrgMember>,
    pub scroll: usize,
}

impl OrgState {
    pub fn new(orgs: Vec<OrgInfo>) -> Self {
        Self {
            orgs,
            selected_org: 0,
            members: Vec::new(),
            scroll: 0,
        }
    }
}
