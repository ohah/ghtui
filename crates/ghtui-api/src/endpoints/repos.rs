use ghtui_core::types::settings::Repository;

use crate::client::GithubClient;
use crate::error::ApiError;

impl GithubClient {
    pub async fn list_recent_repos(&self) -> Result<Vec<Repository>, ApiError> {
        let body = self
            .get("/user/repos?sort=pushed&per_page=20&affiliation=owner,collaborator,organization_member")
            .await?;
        let repos: Vec<Repository> = serde_json::from_str(&body)?;
        Ok(repos)
    }
}
