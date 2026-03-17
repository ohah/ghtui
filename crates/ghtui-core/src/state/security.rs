use crate::types::security::{CodeScanningAlert, DependabotAlert, SecretScanningAlert};

#[derive(Debug, Default)]
pub struct SecurityState {
    pub dependabot_alerts: Vec<DependabotAlert>,
    pub code_scanning_alerts: Vec<CodeScanningAlert>,
    pub secret_scanning_alerts: Vec<SecretScanningAlert>,
    pub tab: usize, // 0=Dependabot, 1=Code Scanning, 2=Secret Scanning
    pub selected: usize,
    pub scroll: usize,
}

impl SecurityState {
    pub fn new() -> Self {
        Self {
            dependabot_alerts: Vec::new(),
            code_scanning_alerts: Vec::new(),
            secret_scanning_alerts: Vec::new(),
            tab: 0,
            selected: 0,
            scroll: 0,
        }
    }

    pub fn tab_count(&self) -> usize {
        3
    }
}
