use ghtui_core::types::*;
use ghtui_core::types::common::RepoId;
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
        let mut params = vec![
            format!("page={}", page),
            format!("per_page={}", per_page),
        ];

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
            .map(|v| serde_json::from_value(v))
            .collect::<Result<Vec<_>, _>>()?;

        let pagination = Pagination {
            page,
            per_page,
            has_next: issues.len() as u32 >= per_page,
            total: None,
        };

        Ok((issues, pagination))
    }

    pub async fn get_issue(
        &self,
        repo: &RepoId,
        number: u64,
    ) -> Result<Issue, ApiError> {
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
        let comments: Vec<IssueComment> =
            serde_json::from_str(&comments_body).unwrap_or_default();

        Ok(IssueDetail { issue, comments })
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
        let number = issue["number"].as_u64().ok_or(ApiError::Other(
            "Missing issue number in response".into(),
        ))?;
        Ok(number)
    }

    pub async fn close_issue(
        &self,
        repo: &RepoId,
        number: u64,
    ) -> Result<(), ApiError> {
        let path = format!("/repos/{}/{}/issues/{}", repo.owner, repo.name, number);
        let body = json!({ "state": "closed" });
        self.patch(&path, &body).await?;
        Ok(())
    }

    pub async fn reopen_issue(
        &self,
        repo: &RepoId,
        number: u64,
    ) -> Result<(), ApiError> {
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
}
