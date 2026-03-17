use ghtui_core::types::common::RepoId;
use ghtui_core::types::*;
use serde_json::json;

use crate::client::GithubClient;
use crate::error::ApiError;

impl GithubClient {
    pub async fn list_issues(
        &self,
        repo: &RepoId,
        filters: &IssueFilters,
        page: u32,
        per_page: u32,
    ) -> Result<(Vec<Issue>, Pagination), ApiError> {
        let mut params = vec![format!("page={}", page), format!("per_page={}", per_page)];

        if let Some(ref state) = filters.state {
            params.push(format!("state={}", state));
        } else {
            params.push("state=open".to_string());
        }

        if let Some(ref sort) = filters.sort {
            params.push(format!("sort={}", sort));
        }
        if let Some(ref direction) = filters.direction {
            params.push(format!("direction={}", direction));
        }
        if let Some(ref label) = filters.label {
            params.push(format!("labels={}", label));
        }
        if let Some(ref assignee) = filters.assignee {
            params.push(format!("assignee={}", assignee));
        }

        let path = format!(
            "/repos/{}/{}/issues?{}",
            repo.owner,
            repo.name,
            params.join("&")
        );

        let body = self.get(&path).await?;
        let all_issues: Vec<serde_json::Value> = serde_json::from_str(&body)?;

        // Filter out pull requests (GitHub API returns PRs in issues endpoint)
        let issues: Vec<Issue> = all_issues
            .into_iter()
            .filter(|i| i.get("pull_request").is_none())
            .map(serde_json::from_value)
            .collect::<Result<Vec<_>, _>>()?;

        let pagination = Pagination {
            page,
            per_page,
            has_next: issues.len() as u32 >= per_page,
            total: None,
        };

        Ok((issues, pagination))
    }

    pub async fn search_issues(
        &self,
        repo: &RepoId,
        query: &str,
    ) -> Result<(Vec<Issue>, Pagination), ApiError> {
        // Build search query with proper encoding
        let search_query = format!("repo:{}/{} is:issue {}", repo.owner, repo.name, query);
        // Simple percent encoding for search query
        let encoded: String = search_query
            .chars()
            .map(|c| match c {
                ' ' => "+".to_string(),
                '/' => "%2F".to_string(),
                ':' => "%3A".to_string(),
                '#' => "%23".to_string(),
                _ => c.to_string(),
            })
            .collect();
        let path = format!("/search/issues?q={}&per_page=30", encoded);
        let body = self.get(&path).await?;
        let result: serde_json::Value = serde_json::from_str(&body)?;

        let items = result
            .get("items")
            .and_then(|v| v.as_array())
            .cloned()
            .unwrap_or_default();

        // Filter out PRs (search API may return them)
        let issues: Vec<Issue> = items
            .into_iter()
            .filter(|i| i.get("pull_request").is_none())
            .map(serde_json::from_value)
            .collect::<Result<Vec<_>, _>>()
            .unwrap_or_default();

        let total = result
            .get("total_count")
            .and_then(|v| v.as_u64())
            .map(|v| v as u32);

        let pagination = Pagination {
            page: 1,
            per_page: 30,
            has_next: issues.len() >= 30,
            total,
        };

        Ok((issues, pagination))
    }

    pub async fn get_issue(&self, repo: &RepoId, number: u64) -> Result<Issue, ApiError> {
        let path = format!("/repos/{}/{}/issues/{}", repo.owner, repo.name, number);
        let body = self.get(&path).await?;
        let issue: Issue = serde_json::from_str(&body)?;
        Ok(issue)
    }

    pub async fn get_issue_detail(
        &self,
        repo: &RepoId,
        number: u64,
    ) -> Result<IssueDetail, ApiError> {
        let issue = self.get_issue(repo, number).await?;

        let comments_path = format!(
            "/repos/{}/{}/issues/{}/comments",
            repo.owner, repo.name, number
        );
        let comments_body = self.get(&comments_path).await?;
        let comments: Vec<IssueComment> = serde_json::from_str(&comments_body).unwrap_or_default();

        // Fetch timeline events
        let timeline_path = format!(
            "/repos/{}/{}/issues/{}/timeline?per_page=50",
            repo.owner, repo.name, number
        );
        let timeline = match self.get(&timeline_path).await {
            Ok(body) => serde_json::from_str(&body).unwrap_or_default(),
            Err(_) => Vec::new(),
        };

        Ok(IssueDetail {
            issue,
            comments,
            timeline,
        })
    }

    pub async fn add_reaction(
        &self,
        repo: &RepoId,
        comment_id: u64,
        reaction: &str,
    ) -> Result<(), ApiError> {
        let path = format!(
            "/repos/{}/{}/issues/comments/{}/reactions",
            repo.owner, repo.name, comment_id
        );
        let body = json!({ "content": reaction });
        self.post(&path, &body).await?;
        Ok(())
    }

    pub async fn add_issue_reaction(
        &self,
        repo: &RepoId,
        number: u64,
        reaction: &str,
    ) -> Result<(), ApiError> {
        let path = format!(
            "/repos/{}/{}/issues/{}/reactions",
            repo.owner, repo.name, number
        );
        let body = json!({ "content": reaction });
        self.post(&path, &body).await?;
        Ok(())
    }

    pub async fn set_issue_milestone(
        &self,
        repo: &RepoId,
        number: u64,
        milestone_number: Option<u64>,
    ) -> Result<(), ApiError> {
        let path = format!("/repos/{}/{}/issues/{}", repo.owner, repo.name, number);
        let body = json!({ "milestone": milestone_number });
        self.patch(&path, &body).await?;
        Ok(())
    }

    pub async fn list_milestones(
        &self,
        repo: &RepoId,
    ) -> Result<Vec<ghtui_core::types::common::Milestone>, ApiError> {
        let path = format!(
            "/repos/{}/{}/milestones?state=open&per_page=30",
            repo.owner, repo.name
        );
        let body = self.get(&path).await?;
        let milestones: Vec<ghtui_core::types::common::Milestone> = serde_json::from_str(&body)?;
        Ok(milestones)
    }

    pub async fn create_issue(
        &self,
        repo: &RepoId,
        input: &CreateIssueInput,
    ) -> Result<u64, ApiError> {
        let path = format!("/repos/{}/{}/issues", repo.owner, repo.name);
        let body = json!({
            "title": input.title,
            "body": input.body,
            "labels": input.labels,
            "assignees": input.assignees,
        });
        let response = self.post(&path, &body).await?;
        let issue: serde_json::Value = serde_json::from_str(&response)?;
        let number = issue["number"]
            .as_u64()
            .ok_or(ApiError::Other("Missing issue number in response".into()))?;
        Ok(number)
    }

    pub async fn update_issue(
        &self,
        repo: &RepoId,
        number: u64,
        title: Option<&str>,
        body: Option<&str>,
    ) -> Result<(), ApiError> {
        let path = format!("/repos/{}/{}/issues/{}", repo.owner, repo.name, number);
        let mut payload = serde_json::Map::new();
        if let Some(t) = title {
            payload.insert("title".to_string(), json!(t));
        }
        if let Some(b) = body {
            payload.insert("body".to_string(), json!(b));
        }
        self.patch(&path, &serde_json::Value::Object(payload))
            .await?;
        Ok(())
    }

    pub async fn update_comment(
        &self,
        repo: &RepoId,
        comment_id: u64,
        body: &str,
    ) -> Result<(), ApiError> {
        let path = format!(
            "/repos/{}/{}/issues/comments/{}",
            repo.owner, repo.name, comment_id
        );
        let payload = json!({ "body": body });
        self.patch(&path, &payload).await?;
        Ok(())
    }

    pub async fn close_issue(&self, repo: &RepoId, number: u64) -> Result<(), ApiError> {
        let path = format!("/repos/{}/{}/issues/{}", repo.owner, repo.name, number);
        let body = json!({ "state": "closed" });
        self.patch(&path, &body).await?;
        Ok(())
    }

    pub async fn reopen_issue(&self, repo: &RepoId, number: u64) -> Result<(), ApiError> {
        let path = format!("/repos/{}/{}/issues/{}", repo.owner, repo.name, number);
        let body = json!({ "state": "open" });
        self.patch(&path, &body).await?;
        Ok(())
    }

    pub async fn add_issue_comment(
        &self,
        repo: &RepoId,
        number: u64,
        body_text: &str,
    ) -> Result<(), ApiError> {
        let path = format!(
            "/repos/{}/{}/issues/{}/comments",
            repo.owner, repo.name, number
        );
        let body = json!({ "body": body_text });
        self.post(&path, &body).await?;
        Ok(())
    }

    pub async fn set_issue_labels(
        &self,
        repo: &RepoId,
        number: u64,
        labels: &[String],
    ) -> Result<(), ApiError> {
        let path = format!(
            "/repos/{}/{}/issues/{}/labels",
            repo.owner, repo.name, number
        );
        let body = json!({ "labels": labels });
        // PUT replaces all labels
        self.put(&path, &body).await?;
        Ok(())
    }

    pub async fn list_repo_labels(
        &self,
        repo: &RepoId,
    ) -> Result<Vec<ghtui_core::types::common::Label>, ApiError> {
        let path = format!("/repos/{}/{}/labels?per_page=100", repo.owner, repo.name);
        let body = self.get(&path).await?;
        let labels: Vec<ghtui_core::types::common::Label> = serde_json::from_str(&body)?;
        Ok(labels)
    }

    pub async fn set_issue_assignees(
        &self,
        repo: &RepoId,
        number: u64,
        assignees: &[String],
    ) -> Result<(), ApiError> {
        let path = format!("/repos/{}/{}/issues/{}", repo.owner, repo.name, number);
        let body = json!({ "assignees": assignees });
        self.patch(&path, &body).await?;
        Ok(())
    }

    pub async fn delete_comment(&self, repo: &RepoId, comment_id: u64) -> Result<(), ApiError> {
        let path = format!(
            "/repos/{}/{}/issues/comments/{}",
            repo.owner, repo.name, comment_id
        );
        self.delete(&path).await?;
        Ok(())
    }

    pub async fn list_collaborators_logins(&self, repo: &RepoId) -> Result<Vec<String>, ApiError> {
        let path = format!(
            "/repos/{}/{}/collaborators?per_page=50",
            repo.owner, repo.name
        );
        match self.get(&path).await {
            Ok(body) => {
                let collabs: Vec<serde_json::Value> = serde_json::from_str(&body)?;
                Ok(collabs
                    .iter()
                    .filter_map(|c| c.get("login").and_then(|l| l.as_str()).map(String::from))
                    .collect())
            }
            Err(ApiError::GitHub { status: 403, .. }) => Ok(Vec::new()),
            Err(e) => Err(e),
        }
    }

    pub async fn lock_issue(&self, repo: &RepoId, number: u64) -> Result<(), ApiError> {
        let path = format!("/repos/{}/{}/issues/{}/lock", repo.owner, repo.name, number);
        let body = json!({ "lock_reason": "resolved" });
        self.put(&path, &body).await?;
        Ok(())
    }

    pub async fn unlock_issue(&self, repo: &RepoId, number: u64) -> Result<(), ApiError> {
        let path = format!("/repos/{}/{}/issues/{}/lock", repo.owner, repo.name, number);
        self.delete(&path).await?;
        Ok(())
    }
}
