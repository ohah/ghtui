use ghtui_core::types::common::RepoId;
use ghtui_core::types::settings::Repository;

use crate::client::GithubClient;
use crate::error::ApiError;

/// Counts of open issues and PRs for a repository.
#[derive(Debug, Clone, Default)]
pub struct RepoCounts {
    pub open_issues: u32,
    pub open_prs: u32,
}

impl GithubClient {
    pub async fn list_recent_repos(&self) -> Result<Vec<Repository>, ApiError> {
        let body = self
            .get("/user/repos?sort=pushed&per_page=20&affiliation=owner,collaborator,organization_member")
            .await?;
        let repos: Vec<Repository> = serde_json::from_str(&body)?;
        Ok(repos)
    }

    /// Fetch open issue and PR counts via GraphQL.
    pub async fn fetch_repo_counts(&self, repo: &RepoId) -> Result<RepoCounts, ApiError> {
        let query = r#"
            query($owner: String!, $name: String!) {
                repository(owner: $owner, name: $name) {
                    issues(states: OPEN) { totalCount }
                    pullRequests(states: OPEN) { totalCount }
                }
            }
        "#;
        let variables = serde_json::json!({
            "owner": repo.owner,
            "name": repo.name,
        });
        let result = self.graphql(query, variables).await?;
        let repo_data = &result["data"]["repository"];
        Ok(RepoCounts {
            open_issues: repo_data["issues"]["totalCount"].as_u64().unwrap_or(0) as u32,
            open_prs: repo_data["pullRequests"]["totalCount"]
                .as_u64()
                .unwrap_or(0) as u32,
        })
    }
}
