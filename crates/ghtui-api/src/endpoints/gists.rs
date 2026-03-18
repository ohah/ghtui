use ghtui_core::types::GistEntry;

use crate::client::GithubClient;
use crate::error::ApiError;

impl GithubClient {
    pub async fn list_gists(&self) -> Result<Vec<GistEntry>, ApiError> {
        let body = self.get("/gists?per_page=30").await?;
        let raw: Vec<serde_json::Value> = serde_json::from_str(&body)?;

        let gists = raw
            .into_iter()
            .filter_map(|v| {
                Some(GistEntry {
                    id: v.get("id")?.as_str()?.to_string(),
                    description: v
                        .get("description")
                        .and_then(|d| d.as_str())
                        .map(|s| s.to_string()),
                    public: v.get("public")?.as_bool().unwrap_or(false),
                    files_count: v.get("files")?.as_object()?.len(),
                    created_at: v.get("created_at")?.as_str()?.to_string(),
                    html_url: v.get("html_url")?.as_str()?.to_string(),
                })
            })
            .collect();

        Ok(gists)
    }
}
