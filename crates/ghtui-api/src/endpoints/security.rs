use ghtui_core::types::common::RepoId;
use ghtui_core::types::security::{CodeScanningAlert, DependabotAlert, SecretScanningAlert};

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
}
