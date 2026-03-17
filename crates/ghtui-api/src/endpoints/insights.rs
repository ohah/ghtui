use ghtui_core::types::common::RepoId;
use ghtui_core::types::insights::{
    CodeFrequency, CommitActivity, ContributorStats, Fork, TrafficClones, TrafficViews,
};

use crate::client::GithubClient;
use crate::error::ApiError;

impl GithubClient {
    pub async fn get_contributor_stats(
        &self,
        repo: &RepoId,
    ) -> Result<Vec<ContributorStats>, ApiError> {
        let path = format!("/repos/{}/{}/stats/contributors", repo.owner, repo.name);
        // GitHub returns 202 while computing stats, retry once
        match self.get(&path).await {
            Ok(body) => {
                let stats: Vec<ContributorStats> = serde_json::from_str(&body).unwrap_or_default();
                Ok(stats)
            }
            Err(ApiError::GitHub { status: 202, .. }) => Ok(Vec::new()),
            Err(e) => Err(e),
        }
    }

    pub async fn get_commit_activity(
        &self,
        repo: &RepoId,
    ) -> Result<Vec<CommitActivity>, ApiError> {
        let path = format!("/repos/{}/{}/stats/commit_activity", repo.owner, repo.name);
        match self.get(&path).await {
            Ok(body) => {
                let activity: Vec<CommitActivity> = serde_json::from_str(&body).unwrap_or_default();
                Ok(activity)
            }
            Err(ApiError::GitHub { status: 202, .. }) => Ok(Vec::new()),
            Err(e) => Err(e),
        }
    }

    pub async fn get_traffic_clones(&self, repo: &RepoId) -> Result<TrafficClones, ApiError> {
        let path = format!("/repos/{}/{}/traffic/clones", repo.owner, repo.name);
        match self.get(&path).await {
            Ok(body) => {
                let clones: TrafficClones = serde_json::from_str(&body)?;
                Ok(clones)
            }
            Err(ApiError::GitHub { status: 403, .. }) => Ok(TrafficClones {
                count: 0,
                uniques: 0,
                clones: Vec::new(),
            }),
            Err(e) => Err(e),
        }
    }

    pub async fn get_traffic_views(&self, repo: &RepoId) -> Result<TrafficViews, ApiError> {
        let path = format!("/repos/{}/{}/traffic/views", repo.owner, repo.name);
        match self.get(&path).await {
            Ok(body) => {
                let views: TrafficViews = serde_json::from_str(&body)?;
                Ok(views)
            }
            Err(ApiError::GitHub { status: 403, .. }) => Ok(TrafficViews {
                count: 0,
                uniques: 0,
                views: Vec::new(),
            }),
            Err(e) => Err(e),
        }
    }

    pub async fn get_code_frequency(&self, repo: &RepoId) -> Result<Vec<CodeFrequency>, ApiError> {
        let path = format!("/repos/{}/{}/stats/code_frequency", repo.owner, repo.name);
        match self.get(&path).await {
            Ok(body) => {
                let freq: Vec<CodeFrequency> = serde_json::from_str(&body).unwrap_or_default();
                Ok(freq)
            }
            Err(ApiError::GitHub { status: 202, .. }) => Ok(Vec::new()),
            Err(e) => Err(e),
        }
    }

    pub async fn list_forks(&self, repo: &RepoId) -> Result<Vec<Fork>, ApiError> {
        let path = format!(
            "/repos/{}/{}/forks?sort=stargazers&per_page=30",
            repo.owner, repo.name
        );
        let body = self.get(&path).await?;
        let forks: Vec<Fork> = serde_json::from_str(&body).unwrap_or_default();
        Ok(forks)
    }
}
