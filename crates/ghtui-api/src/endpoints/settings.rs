use ghtui_core::types::common::RepoId;
use ghtui_core::types::settings::{BranchProtection, Collaborator, DeployKey, Repository, Webhook};

use crate::client::GithubClient;
use crate::error::ApiError;

impl GithubClient {
    pub async fn get_repo(&self, repo: &RepoId) -> Result<Repository, ApiError> {
        let path = format!("/repos/{}/{}", repo.owner, repo.name);
        let body = self.get(&path).await?;
        let repository: Repository = serde_json::from_str(&body)?;
        Ok(repository)
    }

    pub async fn list_branch_protections(
        &self,
        repo: &RepoId,
    ) -> Result<Vec<BranchProtection>, ApiError> {
        let path = format!(
            "/repos/{}/{}/branches?protected=true",
            repo.owner, repo.name
        );
        let body = self.get(&path).await?;

        // GitHub returns branches, we extract protection rules
        let branches: Vec<serde_json::Value> = serde_json::from_str(&body)?;
        let mut protections = Vec::new();

        for branch in branches {
            if branch.get("protected").and_then(|v| v.as_bool()) == Some(true) {
                let name = branch
                    .get("name")
                    .and_then(|v| v.as_str())
                    .unwrap_or_default()
                    .to_string();

                // Try to get detailed protection rules
                let detail_path = format!(
                    "/repos/{}/{}/branches/{}/protection",
                    repo.owner, repo.name, name
                );
                let protection = match self.get(&detail_path).await {
                    Ok(detail_body) => {
                        let detail: serde_json::Value =
                            serde_json::from_str(&detail_body).unwrap_or_default();

                        let required_status_checks = detail
                            .get("required_status_checks")
                            .and_then(|v| serde_json::from_value(v.clone()).ok());
                        let enforce_admins = detail
                            .get("enforce_admins")
                            .and_then(|v| serde_json::from_value(v.clone()).ok());
                        let required_pull_request_reviews = detail
                            .get("required_pull_request_reviews")
                            .and_then(|v| serde_json::from_value(v.clone()).ok());

                        BranchProtection {
                            pattern: name,
                            required_status_checks,
                            enforce_admins,
                            required_pull_request_reviews,
                        }
                    }
                    Err(_) => BranchProtection {
                        pattern: name,
                        required_status_checks: None,
                        enforce_admins: None,
                        required_pull_request_reviews: None,
                    },
                };

                protections.push(protection);
            }
        }

        Ok(protections)
    }

    pub async fn list_collaborators(&self, repo: &RepoId) -> Result<Vec<Collaborator>, ApiError> {
        let path = format!(
            "/repos/{}/{}/collaborators?per_page=30",
            repo.owner, repo.name
        );
        let body = self.get(&path).await?;
        let collaborators: Vec<Collaborator> = serde_json::from_str(&body)?;
        Ok(collaborators)
    }

    /// Update repository settings (PATCH /repos/{owner}/{repo})
    pub async fn update_repo(
        &self,
        repo: &RepoId,
        updates: &serde_json::Value,
    ) -> Result<Repository, ApiError> {
        let path = format!("/repos/{}/{}", repo.owner, repo.name);
        let body = self.patch(&path, updates).await?;
        let repository: Repository = serde_json::from_str(&body)?;
        Ok(repository)
    }

    pub async fn list_webhooks(&self, repo: &RepoId) -> Result<Vec<Webhook>, ApiError> {
        let path = format!("/repos/{}/{}/hooks?per_page=30", repo.owner, repo.name);
        match self.get(&path).await {
            Ok(body) => {
                let hooks: Vec<Webhook> = serde_json::from_str(&body)?;
                Ok(hooks)
            }
            Err(ApiError::GitHub { status: 404, .. }) => Ok(Vec::new()),
            Err(e) => Err(e),
        }
    }

    pub async fn list_deploy_keys(&self, repo: &RepoId) -> Result<Vec<DeployKey>, ApiError> {
        let path = format!("/repos/{}/{}/keys?per_page=30", repo.owner, repo.name);
        match self.get(&path).await {
            Ok(body) => {
                let keys: Vec<DeployKey> = serde_json::from_str(&body)?;
                Ok(keys)
            }
            Err(ApiError::GitHub { status: 404, .. }) => Ok(Vec::new()),
            Err(e) => Err(e),
        }
    }

    // Collaborator management

    pub async fn add_collaborator(
        &self,
        repo: &RepoId,
        username: &str,
        permission: &str,
    ) -> Result<(), ApiError> {
        let path = format!(
            "/repos/{}/{}/collaborators/{}",
            repo.owner, repo.name, username
        );
        let body = serde_json::json!({ "permission": permission });
        self.put(&path, &body).await?;
        Ok(())
    }

    pub async fn remove_collaborator(&self, repo: &RepoId, username: &str) -> Result<(), ApiError> {
        let path = format!(
            "/repos/{}/{}/collaborators/{}",
            repo.owner, repo.name, username
        );
        self.delete(&path).await
    }

    // Webhook management

    pub async fn delete_webhook(&self, repo: &RepoId, hook_id: u64) -> Result<(), ApiError> {
        let path = format!("/repos/{}/{}/hooks/{}", repo.owner, repo.name, hook_id);
        self.delete(&path).await
    }

    pub async fn toggle_webhook(
        &self,
        repo: &RepoId,
        hook_id: u64,
        active: bool,
    ) -> Result<(), ApiError> {
        let path = format!("/repos/{}/{}/hooks/{}", repo.owner, repo.name, hook_id);
        let body = serde_json::json!({ "active": active });
        self.patch(&path, &body).await?;
        Ok(())
    }

    // Deploy key management

    pub async fn delete_deploy_key(&self, repo: &RepoId, key_id: u64) -> Result<(), ApiError> {
        let path = format!("/repos/{}/{}/keys/{}", repo.owner, repo.name, key_id);
        self.delete(&path).await
    }
}
