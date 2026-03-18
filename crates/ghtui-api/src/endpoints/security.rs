use ghtui_core::types::common::RepoId;
use ghtui_core::types::security::{
    CodeScanningAlert, DependabotAlert, RepoSecurityAdvisory, SecretScanningAlert,
};

use crate::client::GithubClient;
use crate::error::ApiError;

impl GithubClient {
    pub async fn list_dependabot_alerts(
        &self,
        repo: &RepoId,
    ) -> Result<Vec<DependabotAlert>, ApiError> {
        let path = format!(
            "/repos/{}/{}/dependabot/alerts?state=open&per_page=30",
            repo.owner, repo.name
        );
        match self.get(&path).await {
            Ok(body) => {
                let alerts: Vec<DependabotAlert> = serde_json::from_str(&body)?;
                Ok(alerts)
            }
            Err(ApiError::NotFound(_)) => Ok(Vec::new()),
            Err(ApiError::GitHub { status: 403, .. }) => Ok(Vec::new()),
            Err(e) => Err(e),
        }
    }

    pub async fn list_code_scanning_alerts(
        &self,
        repo: &RepoId,
    ) -> Result<Vec<CodeScanningAlert>, ApiError> {
        let path = format!(
            "/repos/{}/{}/code-scanning/alerts?state=open&per_page=30",
            repo.owner, repo.name
        );
        match self.get(&path).await {
            Ok(body) => {
                let alerts: Vec<CodeScanningAlert> = serde_json::from_str(&body)?;
                Ok(alerts)
            }
            Err(ApiError::NotFound(_)) => Ok(Vec::new()),
            Err(ApiError::GitHub { status: 403, .. }) => Ok(Vec::new()),
            // 404 for repos without code scanning
            Err(ApiError::GitHub { status: 404, .. }) => Ok(Vec::new()),
            Err(e) => Err(e),
        }
    }

    pub async fn list_secret_scanning_alerts(
        &self,
        repo: &RepoId,
    ) -> Result<Vec<SecretScanningAlert>, ApiError> {
        let path = format!(
            "/repos/{}/{}/secret-scanning/alerts?state=open&per_page=30",
            repo.owner, repo.name
        );
        match self.get(&path).await {
            Ok(body) => {
                let alerts: Vec<SecretScanningAlert> = serde_json::from_str(&body)?;
                Ok(alerts)
            }
            Err(ApiError::NotFound(_)) => Ok(Vec::new()),
            Err(ApiError::GitHub { status: 403, .. }) => Ok(Vec::new()),
            Err(ApiError::GitHub { status: 404, .. }) => Ok(Vec::new()),
            Err(e) => Err(e),
        }
    }

    pub async fn list_security_advisories(
        &self,
        repo: &RepoId,
    ) -> Result<Vec<RepoSecurityAdvisory>, ApiError> {
        let path = format!(
            "/repos/{}/{}/security-advisories?per_page=30",
            repo.owner, repo.name
        );
        match self.get(&path).await {
            Ok(body) => {
                let advisories: Vec<RepoSecurityAdvisory> = serde_json::from_str(&body)?;
                Ok(advisories)
            }
            Err(ApiError::NotFound(_)) => Ok(Vec::new()),
            Err(ApiError::GitHub { status: 403, .. }) => Ok(Vec::new()),
            Err(ApiError::GitHub { status: 404, .. }) => Ok(Vec::new()),
            Err(e) => Err(e),
        }
    }

    /// Dismiss a Dependabot alert.
    /// reason: "fix_started", "inaccurate", "no_bandwidth", "not_used", "tolerable_risk"
    pub async fn dismiss_dependabot_alert(
        &self,
        repo: &RepoId,
        number: u64,
        reason: &str,
    ) -> Result<(), ApiError> {
        let path = format!(
            "/repos/{}/{}/dependabot/alerts/{}",
            repo.owner, repo.name, number
        );
        let body = serde_json::json!({
            "state": "dismissed",
            "dismissed_reason": reason
        });
        self.patch(&path, &body).await?;
        Ok(())
    }

    /// Reopen a dismissed Dependabot alert.
    pub async fn reopen_dependabot_alert(
        &self,
        repo: &RepoId,
        number: u64,
    ) -> Result<(), ApiError> {
        let path = format!(
            "/repos/{}/{}/dependabot/alerts/{}",
            repo.owner, repo.name, number
        );
        let body = serde_json::json!({
            "state": "open"
        });
        self.patch(&path, &body).await?;
        Ok(())
    }

    /// Dismiss a code scanning alert.
    /// reason: "false positive", "won't fix", "used in tests"
    pub async fn dismiss_code_scanning_alert(
        &self,
        repo: &RepoId,
        number: u64,
        reason: &str,
    ) -> Result<(), ApiError> {
        let path = format!(
            "/repos/{}/{}/code-scanning/alerts/{}",
            repo.owner, repo.name, number
        );
        let body = serde_json::json!({
            "state": "dismissed",
            "dismissed_reason": reason
        });
        self.patch(&path, &body).await?;
        Ok(())
    }

    /// Resolve a secret scanning alert.
    /// resolution: "false_positive", "wont_fix", "revoked", "used_in_tests"
    pub async fn resolve_secret_scanning_alert(
        &self,
        repo: &RepoId,
        number: u64,
        resolution: &str,
    ) -> Result<(), ApiError> {
        let path = format!(
            "/repos/{}/{}/secret-scanning/alerts/{}",
            repo.owner, repo.name, number
        );
        let body = serde_json::json!({
            "state": "resolved",
            "resolution": resolution
        });
        self.patch(&path, &body).await?;
        Ok(())
    }
}
