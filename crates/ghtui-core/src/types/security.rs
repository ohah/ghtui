use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DependabotAlert {
    pub number: u64,
    pub state: String,
    pub dependency: DependabotDependency,
    pub security_advisory: SecurityAdvisory,
    pub security_vulnerability: SecurityVulnerability,
    pub created_at: String,
    pub updated_at: String,
    pub dismissed_at: Option<String>,
    pub fixed_at: Option<String>,
    pub html_url: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DependabotDependency {
    #[serde(rename = "package")]
    pub package: DependabotPackage,
    pub manifest_path: Option<String>,
    pub scope: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DependabotPackage {
    pub ecosystem: String,
    pub name: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SecurityAdvisory {
    pub ghsa_id: String,
    pub summary: String,
    pub description: String,
    pub severity: String,
    pub cve_id: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SecurityVulnerability {
    pub severity: String,
    pub first_patched_version: Option<PatchedVersion>,
    pub vulnerable_version_range: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PatchedVersion {
    pub identifier: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CodeScanningAlert {
    pub number: u64,
    pub state: String,
    pub rule: CodeScanningRule,
    pub tool: CodeScanningTool,
    pub most_recent_instance: Option<CodeScanningInstance>,
    pub created_at: String,
    pub html_url: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CodeScanningRule {
    pub id: Option<String>,
    pub severity: Option<String>,
    pub description: Option<String>,
    pub name: Option<String>,
    pub security_severity_level: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CodeScanningTool {
    pub name: String,
    pub version: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CodeScanningInstance {
    #[serde(rename = "ref")]
    pub git_ref: Option<String>,
    pub state: Option<String>,
    pub location: Option<CodeScanningLocation>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CodeScanningLocation {
    pub path: Option<String>,
    pub start_line: Option<u64>,
    pub end_line: Option<u64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SecretScanningAlert {
    pub number: u64,
    pub state: String,
    pub secret_type: Option<String>,
    pub secret_type_display_name: Option<String>,
    pub resolution: Option<String>,
    pub created_at: String,
    pub html_url: String,
}
