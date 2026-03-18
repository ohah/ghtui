use ghtui_core::types::Discussion;
use ghtui_core::types::common::RepoId;

use crate::client::GithubClient;
use crate::error::ApiError;

impl GithubClient {
    pub async fn list_discussions(&self, repo: &RepoId) -> Result<Vec<Discussion>, ApiError> {
        let query = r#"
            query($owner: String!, $name: String!, $first: Int!) {
                repository(owner: $owner, name: $name) {
                    discussions(first: $first, orderBy: {field: UPDATED_AT, direction: DESC}) {
                        nodes {
                            number
                            title
                            author { login }
                            createdAt
                            category { name }
                            comments { totalCount }
                            isAnswered
                            url
                        }
                    }
                }
            }
        "#;

        let variables = serde_json::json!({
            "owner": repo.owner,
            "name": repo.name,
            "first": 30,
        });

        let result = self.graphql(query, variables).await?;

        let nodes = result
            .pointer("/data/repository/discussions/nodes")
            .and_then(|v| v.as_array())
            .cloned()
            .unwrap_or_default();

        let discussions = nodes
            .into_iter()
            .filter_map(|node| {
                Some(Discussion {
                    number: node.get("number")?.as_u64()?,
                    title: node.get("title")?.as_str()?.to_string(),
                    author: node
                        .pointer("/author/login")
                        .and_then(|v| v.as_str())
                        .unwrap_or("ghost")
                        .to_string(),
                    created_at: node.get("createdAt")?.as_str()?.to_string(),
                    category: node
                        .pointer("/category/name")
                        .and_then(|v| v.as_str())
                        .unwrap_or("")
                        .to_string(),
                    comments_count: node
                        .pointer("/comments/totalCount")
                        .and_then(|v| v.as_u64())
                        .unwrap_or(0),
                    is_answered: node
                        .get("isAnswered")
                        .and_then(|v| v.as_bool())
                        .unwrap_or(false),
                    url: node.get("url")?.as_str()?.to_string(),
                })
            })
            .collect();

        Ok(discussions)
    }
}
