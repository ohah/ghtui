use crate::types::security::{
    CodeScanningAlert, DependabotAlert, RepoSecurityAdvisory, SecretScanningAlert,
};

#[derive(Debug, Default)]
pub struct SecurityState {
    pub dependabot_alerts: Vec<DependabotAlert>,
    pub code_scanning_alerts: Vec<CodeScanningAlert>,
    pub secret_scanning_alerts: Vec<SecretScanningAlert>,
    pub advisories: Vec<RepoSecurityAdvisory>,
    pub tab: usize, // 0=Dependabot, 1=Code Scanning, 2=Secret Scanning, 3=Advisories
    pub selected: usize,
    pub scroll: usize,
    pub detail_open: bool,
    pub detail_scroll: usize,
    pub sidebar_focused: bool,
}

impl SecurityState {
    pub fn new() -> Self {
        Self {
            dependabot_alerts: Vec::new(),
            code_scanning_alerts: Vec::new(),
            secret_scanning_alerts: Vec::new(),
            advisories: Vec::new(),
            tab: 0,
            selected: 0,
            scroll: 0,
            detail_open: false,
            detail_scroll: 0,
            sidebar_focused: true,
        }
    }

    pub fn tab_count(&self) -> usize {
        4
    }
}
