use ghtui_core::types::common::RepoId;
use ghtui_core::types::*;
use serde_json::json;

use crate::client::GithubClient;
use crate::diff;
use crate::error::ApiError;

impl GithubClient {
    pub async fn list_pulls(
        &self,
        repo: &RepoId,
        filters: &PrFilters,
        page: u32,
        per_page: u32,
    ) -> Result<(Vec<PullRequest>, Pagination), ApiError> {
        let mut params = vec![format!("page={}", page), format!("per_page={}", per_page)];

        if let Some(ref state) = filters.state {
            // GitHub API uses "open"/"closed" for PRs, "all" for both
            let state_str = match state {
                PrState::Open => "open",
                PrState::Closed | PrState::Merged => "closed",
            };
            params.push(format!("state={}", state_str));
        } else {
            params.push("state=open".to_string());
        }

        if let Some(ref sort) = filters.sort {
            params.push(format!("sort={}", sort));
        }
        if let Some(ref direction) = filters.direction {
            params.push(format!("direction={}", direction));
        }

        let path = format!(
            "/repos/{}/{}/pulls?{}",
            repo.owner,
            repo.name,
            params.join("&")
        );

        let body = self.get(&path).await?;
        let prs: Vec<PullRequest> = serde_json::from_str(&body)?;

        // For simplicity, detect pagination from response length
        let pagination = Pagination {
            page,
            per_page,
            has_next: prs.len() as u32 >= per_page,
            total: None,
        };

        Ok((prs, pagination))
    }

    pub async fn get_pull(&self, repo: &RepoId, number: u64) -> Result<PullRequest, ApiError> {
        let path = format!("/repos/{}/{}/pulls/{}", repo.owner, repo.name, number);
        let body = self.get(&path).await?;
        let pr: PullRequest = serde_json::from_str(&body)?;
        Ok(pr)
    }

    pub async fn get_pull_detail(
        &self,
        repo: &RepoId,
        number: u64,
    ) -> Result<PullRequestDetail, ApiError> {
        let pr = self.get_pull(repo, number).await?;

        let reviews_path = format!(
            "/repos/{}/{}/pulls/{}/reviews",
            repo.owner, repo.name, number
        );
        let comments_path = format!(
            "/repos/{}/{}/issues/{}/comments",
            repo.owner, repo.name, number
        );
        let review_comments_path = format!(
            "/repos/{}/{}/pulls/{}/comments",
            repo.owner, repo.name, number
        );

        let (reviews_body, comments_body, review_comments_body) = tokio::join!(
            self.get(&reviews_path),
            self.get(&comments_path),
            self.get(&review_comments_path),
        );

        let reviews: Vec<ApiReview> = serde_json::from_str(&reviews_body?).unwrap_or_default();
        let comments: Vec<PrComment> = serde_json::from_str(&comments_body?).unwrap_or_default();
        let review_comments: Vec<ApiReviewComment> =
            serde_json::from_str(&review_comments_body?).unwrap_or_default();

        Ok(PullRequestDetail {
            pr,
            reviews: reviews.into_iter().map(|r| r.into()).collect(),
            comments,
            review_comments: review_comments.into_iter().map(|c| c.into()).collect(),
            checks: vec![],
        })
    }

    pub async fn get_pull_diff(
        &self,
        repo: &RepoId,
        number: u64,
    ) -> Result<Vec<DiffFile>, ApiError> {
        let url = format!(
            "{}/repos/{}/{}/pulls/{}",
            self.base_url, repo.owner, repo.name, number
        );

        let response = self
            .http
            .get(&url)
            .header("Accept", "application/vnd.github.diff")
            .send()
            .await?;

        let status = response.status();
        let text = response.text().await?;

        if !status.is_success() {
            return Err(ApiError::GitHub {
                status: status.as_u16(),
                message: text,
            });
        }

        Ok(diff::parse_diff(&text))
    }

    pub async fn merge_pull(
        &self,
        repo: &RepoId,
        number: u64,
        method: MergeMethod,
    ) -> Result<(), ApiError> {
        let path = format!("/repos/{}/{}/pulls/{}/merge", repo.owner, repo.name, number);
        let body = json!({
            "merge_method": method.as_str(),
        });
        self.put(&path, &body).await?;
        Ok(())
    }

    pub async fn close_pull(&self, repo: &RepoId, number: u64) -> Result<(), ApiError> {
        let path = format!("/repos/{}/{}/pulls/{}", repo.owner, repo.name, number);
        let body = json!({ "state": "closed" });
        self.patch(&path, &body).await?;
        Ok(())
    }

    pub async fn reopen_pull(&self, repo: &RepoId, number: u64) -> Result<(), ApiError> {
        let path = format!("/repos/{}/{}/pulls/{}", repo.owner, repo.name, number);
        let body = json!({ "state": "open" });
        self.patch(&path, &body).await?;
        Ok(())
    }

    pub async fn create_pull(&self, repo: &RepoId, input: &CreatePrInput) -> Result<u64, ApiError> {
        let path = format!("/repos/{}/{}/pulls", repo.owner, repo.name);
        let body = json!({
            "title": input.title,
            "body": input.body,
            "head": input.head,
            "base": input.base,
            "draft": input.draft,
        });
        let response = self.post(&path, &body).await?;
        let pr: serde_json::Value = serde_json::from_str(&response)?;
        let number = pr["number"]
            .as_u64()
            .ok_or(ApiError::Other("Missing PR number in response".into()))?;
        Ok(number)
    }

    pub async fn submit_review(
        &self,
        repo: &RepoId,
        number: u64,
        input: &ReviewInput,
    ) -> Result<(), ApiError> {
        let path = format!(
            "/repos/{}/{}/pulls/{}/reviews",
            repo.owner, repo.name, number
        );

        let comments: Vec<serde_json::Value> = input
            .comments
            .iter()
            .map(|c| {
                json!({
                    "path": c.path,
                    "line": c.line,
                    "body": c.body,
                })
            })
            .collect();

        let body = json!({
            "event": input.event.as_str(),
            "body": input.body,
            "comments": comments,
        });

        self.post(&path, &body).await?;
        Ok(())
    }

    pub async fn add_pr_comment(
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

// Internal API response types for deserialization
#[derive(serde::Deserialize)]
struct ApiReview {
    id: u64,
    user: User,
    state: String,
    body: Option<String>,
    submitted_at: Option<chrono::DateTime<chrono::Utc>>,
}

impl From<ApiReview> for Review {
    fn from(r: ApiReview) -> Self {
        let state = match r.state.as_str() {
            "APPROVED" => ReviewState::Approved,
            "CHANGES_REQUESTED" => ReviewState::ChangesRequested,
            "COMMENTED" => ReviewState::Commented,
            "DISMISSED" => ReviewState::Dismissed,
            _ => ReviewState::Pending,
        };
        Review {
            id: r.id,
            user: r.user,
            state,
            body: r.body,
            submitted_at: r.submitted_at,
        }
    }
}

#[derive(serde::Deserialize)]
struct ApiReviewComment {
    id: u64,
    user: User,
    body: String,
    path: String,
    line: Option<u32>,
    original_line: Option<u32>,
    diff_hunk: String,
    created_at: chrono::DateTime<chrono::Utc>,
    in_reply_to_id: Option<u64>,
}

impl From<ApiReviewComment> for ReviewComment {
    fn from(c: ApiReviewComment) -> Self {
        ReviewComment {
            id: c.id,
            user: c.user,
            body: c.body,
            path: c.path,
            line: c.line,
            original_line: c.original_line,
            diff_hunk: c.diff_hunk,
            created_at: c.created_at,
            in_reply_to_id: c.in_reply_to_id,
        }
    }
}
