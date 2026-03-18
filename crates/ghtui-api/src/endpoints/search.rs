use ghtui_core::types::*;

use crate::client::GithubClient;
use crate::error::ApiError;

impl GithubClient {
    pub async fn search(
        &self,
        query: &str,
        kind: SearchKind,
        page: u32,
    ) -> Result<SearchResultSet, ApiError> {
        let endpoint = match kind {
            SearchKind::Repos => "repositories",
            SearchKind::Issues => "issues",
            SearchKind::Code => "code",
        };

        let path = format!(
            "/search/{}?q={}&page={}&per_page=30",
            endpoint,
            urlencoding(query),
            page
        );

        let body = self.get_with_ttl(&path, 60).await?;
        let response: serde_json::Value = serde_json::from_str(&body)?;

        let total_count = response["total_count"].as_u64().unwrap_or(0) as u32;
        let items_json = response["items"].as_array();

        let items = match (kind, items_json) {
            (SearchKind::Repos, Some(arr)) => arr
                .iter()
                .map(|v| SearchResultItem::Repo {
                    full_name: v["full_name"].as_str().unwrap_or("").to_string(),
                    description: v["description"].as_str().map(String::from),
                    stars: v["stargazers_count"].as_u64().unwrap_or(0) as u32,
                    language: v["language"].as_str().map(String::from),
                })
                .collect(),
            (SearchKind::Issues, Some(arr)) => arr
                .iter()
                .map(|v| SearchResultItem::Issue {
                    repo: v["repository_url"]
                        .as_str()
                        .unwrap_or("")
                        .rsplit("/repos/")
                        .next()
                        .unwrap_or("")
                        .to_string(),
                    number: v["number"].as_u64().unwrap_or(0),
                    title: v["title"].as_str().unwrap_or("").to_string(),
                    state: v["state"].as_str().unwrap_or("").to_string(),
                    is_pr: v.get("pull_request").is_some(),
                    labels: v["labels"]
                        .as_array()
                        .map(|arr| {
                            arr.iter()
                                .filter_map(|l| serde_json::from_value(l.clone()).ok())
                                .collect()
                        })
                        .unwrap_or_default(),
                    created_at: v["created_at"]
                        .as_str()
                        .and_then(|s| s.parse().ok()),
                    user: v["user"]["login"]
                        .as_str()
                        .unwrap_or("")
                        .to_string(),
                })
                .collect(),
            (SearchKind::Code, Some(arr)) => arr
                .iter()
                .map(|v| SearchResultItem::Code {
                    repo: v["repository"]["full_name"]
                        .as_str()
                        .unwrap_or("")
                        .to_string(),
                    path: v["path"].as_str().unwrap_or("").to_string(),
                    fragment: v["text_matches"]
                        .as_array()
                        .and_then(|m| m.first())
                        .and_then(|m| m["fragment"].as_str())
                        .unwrap_or("")
                        .to_string(),
                })
                .collect(),
            _ => Vec::new(),
        };

        Ok(SearchResultSet {
            kind,
            total_count,
            items,
        })
    }
}

fn urlencoding(s: &str) -> String {
    s.chars()
        .map(|c| match c {
            ' ' => "+".to_string(),
            c if c.is_ascii_alphanumeric() || "-_.~".contains(c) => c.to_string(),
            c => format!("%{:02X}", c as u8),
        })
        .collect()
}
