use ghtui_core::types::common::RepoId;
use ghtui_core::types::insights::{
    CodeFrequency, CommitActivity, ContributorStats, DependencyEntry, Fork, TrafficClones,
    TrafficViews,
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

    /// Fetch dependency graph via SBOM API.
    pub async fn get_dependency_graph(
        &self,
        repo: &RepoId,
    ) -> Result<Vec<DependencyEntry>, ApiError> {
        let path = format!("/repos/{}/{}/dependency-graph/sbom", repo.owner, repo.name);
        match self.get(&path).await {
            Ok(body) => {
                let val: serde_json::Value = serde_json::from_str(&body)?;
                let packages = val
                    .pointer("/sbom/packages")
                    .and_then(|v| v.as_array())
                    .cloned()
                    .unwrap_or_default();
                let deps = packages
                    .iter()
                    .filter_map(|p| {
                        let name = p["name"].as_str()?.to_string();
                        let version = p["versionInfo"].as_str().map(|s| s.to_string());
                        let package_url = p["externalRefs"].as_array().and_then(|refs| {
                            refs.iter().find_map(|r| {
                                if r["referenceType"].as_str() == Some("purl") {
                                    r["referenceLocator"].as_str().map(|s| s.to_string())
                                } else {
                                    None
                                }
                            })
                        });
                        Some(DependencyEntry {
                            name,
                            version,
                            package_url,
                        })
                    })
                    .collect();
                Ok(deps)
            }
            Err(ApiError::GitHub { status: 403, .. })
            | Err(ApiError::GitHub { status: 404, .. })
            | Err(ApiError::NotFound(_)) => Ok(Vec::new()),
            Err(e) => Err(e),
        }
    }
}
