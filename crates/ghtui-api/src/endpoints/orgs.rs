use ghtui_core::types::{OrgInfo, OrgMember};

use crate::client::GithubClient;
use crate::error::ApiError;

impl GithubClient {
    pub async fn list_user_orgs(&self) -> Result<Vec<OrgInfo>, ApiError> {
        let body = self.get("/user/orgs?per_page=30").await?;
        let raw: Vec<serde_json::Value> = serde_json::from_str(&body)?;

        let orgs = raw
            .into_iter()
            .filter_map(|v| {
                Some(OrgInfo {
                    login: v.get("login")?.as_str()?.to_string(),
                    name: v
                        .get("name")
                        .and_then(|n| n.as_str())
                        .map(|s| s.to_string()),
                    description: v
                        .get("description")
                        .and_then(|d| d.as_str())
                        .map(|s| s.to_string()),
                    members_count: v.get("members_count").and_then(|m| m.as_u64()),
                })
            })
            .collect();

        Ok(orgs)
    }

    pub async fn list_org_members(&self, org: &str) -> Result<Vec<OrgMember>, ApiError> {
        let body = self
            .get(&format!("/orgs/{}/members?per_page=50", org))
            .await?;
        let raw: Vec<serde_json::Value> = serde_json::from_str(&body)?;

        let members = raw
            .into_iter()
            .filter_map(|v| {
                Some(OrgMember {
                    login: v.get("login")?.as_str()?.to_string(),
                    role: v
                        .get("role")
                        .and_then(|r| r.as_str())
                        .map(|s| s.to_string()),
                })
            })
            .collect();

        Ok(members)
    }
}
