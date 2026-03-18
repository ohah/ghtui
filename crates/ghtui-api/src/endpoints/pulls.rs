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
        let checks_path = format!(
            "/repos/{}/{}/commits/{}/check-runs",
            repo.owner, repo.name, pr.head_sha
        );
        let status_path = format!(
            "/repos/{}/{}/commits/{}/status",
            repo.owner, repo.name, pr.head_sha
        );
        let commits_path = format!(
            "/repos/{}/{}/pulls/{}/commits?per_page=100",
            repo.owner, repo.name, number
        );

        let timeline_path = format!(
            "/repos/{}/{}/issues/{}/timeline?per_page=50",
            repo.owner, repo.name, number
        );

        let (
            reviews_body,
            comments_body,
            review_comments_body,
            checks_body,
            status_body,
            commits_body,
            timeline_body,
            review_threads_result,
        ) = tokio::join!(
            self.get(&reviews_path),
            self.get(&comments_path),
            self.get(&review_comments_path),
            self.get(&checks_path),
            self.get(&status_path),
            self.get(&commits_path),
            self.get(&timeline_path),
            self.get_review_threads(repo, number),
        );
        let review_threads = review_threads_result.unwrap_or_default();

        let reviews: Vec<ApiReview> = serde_json::from_str(&reviews_body?).unwrap_or_default();
        let comments: Vec<PrComment> = serde_json::from_str(&comments_body?).unwrap_or_default();
        let review_comments: Vec<ApiReviewComment> =
            serde_json::from_str(&review_comments_body?).unwrap_or_default();

        // Parse check runs
        let mut checks: Vec<CheckStatus> = Vec::new();
        if let Ok(body) = checks_body {
            if let Ok(val) = serde_json::from_str::<serde_json::Value>(&body) {
                if let Some(runs) = val["check_runs"].as_array() {
                    for run in runs {
                        checks.push(CheckStatus {
                            name: run["name"].as_str().unwrap_or("").to_string(),
                            status: run["status"].as_str().unwrap_or("").to_string(),
                            conclusion: run["conclusion"].as_str().map(|s| s.to_string()),
                            url: run["html_url"].as_str().map(|s| s.to_string()),
                        });
                    }
                }
            }
        }

        // Parse commit statuses (older CI systems use this)
        if let Ok(body) = status_body {
            if let Ok(val) = serde_json::from_str::<serde_json::Value>(&body) {
                if let Some(statuses) = val["statuses"].as_array() {
                    for status in statuses {
                        checks.push(CheckStatus {
                            name: status["context"].as_str().unwrap_or("").to_string(),
                            status: status["state"].as_str().unwrap_or("").to_string(),
                            conclusion: status["state"].as_str().map(|s| s.to_string()),
                            url: status["target_url"].as_str().map(|s| s.to_string()),
                        });
                    }
                }
            }
        }

        // Parse commits
        let mut commits: Vec<PrCommit> = Vec::new();
        if let Ok(body) = commits_body {
            if let Ok(arr) = serde_json::from_str::<Vec<serde_json::Value>>(&body) {
                for c in arr {
                    commits.push(PrCommit {
                        sha: c["sha"].as_str().unwrap_or("").to_string(),
                        message: c["commit"]["message"]
                            .as_str()
                            .unwrap_or("")
                            .lines()
                            .next()
                            .unwrap_or("")
                            .to_string(),
                        author: c["commit"]["author"]["name"]
                            .as_str()
                            .or_else(|| c["author"]["login"].as_str())
                            .unwrap_or("unknown")
                            .to_string(),
                        date: c["commit"]["author"]["date"]
                            .as_str()
                            .and_then(|s| s.parse().ok()),
                    });
                }
            }
        }

        // Parse timeline (individual items to handle varied event schemas)
        let mut timeline = Vec::new();
        if let Ok(body) = timeline_body {
            if let Ok(arr) = serde_json::from_str::<Vec<serde_json::Value>>(&body) {
                for item in arr {
                    if let Ok(event) =
                        serde_json::from_value::<ghtui_core::types::issue::TimelineEvent>(item)
                    {
                        if !event.event.is_empty() {
                            timeline.push(event);
                        }
                    }
                }
            }
        }

        Ok(PullRequestDetail {
            pr,
            reviews: reviews.into_iter().map(|r| r.into()).collect(),
            comments,
            review_comments: review_comments.into_iter().map(|c| c.into()).collect(),
            review_threads,
            checks,
            commits,
            timeline,
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

        let mut payload = serde_json::Map::new();
        payload.insert("event".into(), json!(input.event.as_str()));
        if let Some(ref b) = input.body {
            payload.insert("body".into(), json!(b));
        }
        if !comments.is_empty() {
            payload.insert("comments".into(), json!(comments));
        }

        self.post(&path, &serde_json::Value::Object(payload))
            .await?;
        Ok(())
    }

    pub async fn search_pulls(
        &self,
        repo: &RepoId,
        query: &str,
    ) -> Result<(Vec<PullRequest>, Pagination), ApiError> {
        let search_query = format!("repo:{}/{} is:pr {}", repo.owner, repo.name, query);
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
        let items = result["items"]
            .as_array()
            .map(|arr| {
                arr.iter()
                    .filter_map(|v| serde_json::from_value::<PullRequest>(v.clone()).ok())
                    .collect::<Vec<_>>()
            })
            .unwrap_or_default();
        let pagination = Pagination {
            page: 1,
            per_page: 30,
            has_next: false,
            total: result["total_count"].as_u64().map(|n| n as u32),
        };
        Ok((items, pagination))
    }

    pub async fn change_pull_base(
        &self,
        repo: &RepoId,
        number: u64,
        base: &str,
    ) -> Result<(), ApiError> {
        let path = format!("/repos/{}/{}/pulls/{}", repo.owner, repo.name, number);
        let body = json!({ "base": base });
        self.patch(&path, &body).await?;
        Ok(())
    }

    pub async fn update_pull(
        &self,
        repo: &RepoId,
        number: u64,
        title: Option<&str>,
        body: Option<&str>,
    ) -> Result<(), ApiError> {
        let path = format!("/repos/{}/{}/pulls/{}", repo.owner, repo.name, number);
        let mut update = serde_json::Map::new();
        if let Some(t) = title {
            update.insert("title".into(), json!(t));
        }
        if let Some(b) = body {
            update.insert("body".into(), json!(b));
        }
        self.patch(&path, &serde_json::Value::Object(update))
            .await?;
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

    pub async fn request_reviewers(
        &self,
        repo: &RepoId,
        number: u64,
        reviewers: &[String],
    ) -> Result<(), ApiError> {
        let path = format!(
            "/repos/{}/{}/pulls/{}/requested_reviewers",
            repo.owner, repo.name, number
        );
        let body = json!({ "reviewers": reviewers });
        self.post(&path, &body).await?;
        Ok(())
    }
    pub async fn enable_auto_merge(&self, repo: &RepoId, number: u64) -> Result<(), ApiError> {
        // Get PR node_id
        let pr_path = format!("/repos/{}/{}/pulls/{}", repo.owner, repo.name, number);
        let pr_body = self.get(&pr_path).await?;
        let pr_json: serde_json::Value = serde_json::from_str(&pr_body)?;
        let node_id = pr_json["node_id"]
            .as_str()
            .ok_or_else(|| ApiError::Other("Missing node_id".into()))?
            .to_string();

        let query = "mutation($id: ID!) { enablePullRequestAutoMerge(input: {pullRequestId: $id, mergeMethod: REBASE}) { pullRequest { autoMergeRequest { enabledAt } } } }";
        self.graphql(query, json!({"id": node_id})).await?;
        Ok(())
    }

    pub async fn disable_auto_merge(&self, repo: &RepoId, number: u64) -> Result<(), ApiError> {
        // Get PR node_id
        let pr_path = format!("/repos/{}/{}/pulls/{}", repo.owner, repo.name, number);
        let pr_body = self.get(&pr_path).await?;
        let pr_json: serde_json::Value = serde_json::from_str(&pr_body)?;
        let node_id = pr_json["node_id"]
            .as_str()
            .ok_or_else(|| ApiError::Other("Missing node_id".into()))?
            .to_string();

        let query = "mutation($id: ID!) { disablePullRequestAutoMerge(input: {pullRequestId: $id}) { pullRequest { autoMergeRequest { enabledAt } } } }";
        self.graphql(query, json!({"id": node_id})).await?;
        Ok(())
    }

    pub async fn get_review_threads(
        &self,
        repo: &RepoId,
        number: u64,
    ) -> Result<Vec<ReviewThread>, ApiError> {
        let query = r#"query($owner: String!, $name: String!, $number: Int!) {
            repository(owner: $owner, name: $name) {
                pullRequest(number: $number) {
                    reviewThreads(first: 100) {
                        nodes {
                            id
                            isResolved
                            comments(first: 1) {
                                nodes {
                                    databaseId
                                }
                            }
                        }
                    }
                }
            }
        }"#;
        let vars = json!({
            "owner": repo.owner,
            "name": repo.name,
            "number": number as i64,
        });
        let result = self.graphql(query, vars).await?;
        let mut threads = Vec::new();
        if let Some(nodes) = result
            .pointer("/data/repository/pullRequest/reviewThreads/nodes")
            .and_then(|v| v.as_array())
        {
            for node in nodes {
                let node_id = node["id"].as_str().unwrap_or_default().to_string();
                let is_resolved = node["isResolved"].as_bool().unwrap_or(false);
                let root_comment_id = node
                    .pointer("/comments/nodes/0/databaseId")
                    .and_then(|v| v.as_u64())
                    .unwrap_or(0);
                if !node_id.is_empty() && root_comment_id > 0 {
                    threads.push(ReviewThread {
                        node_id,
                        is_resolved,
                        root_comment_id,
                    });
                }
            }
        }
        Ok(threads)
    }

    pub async fn set_review_thread_resolved(
        &self,
        thread_node_id: &str,
        resolve: bool,
    ) -> Result<(), ApiError> {
        let query = if resolve {
            "mutation($id: ID!) { resolveReviewThread(input: {threadId: $id}) { thread { isResolved } } }"
        } else {
            "mutation($id: ID!) { unresolveReviewThread(input: {threadId: $id}) { thread { isResolved } } }"
        };
        self.graphql(query, json!({"id": thread_node_id})).await?;
        Ok(())
    }

    pub async fn set_draft(&self, repo: &RepoId, number: u64, draft: bool) -> Result<(), ApiError> {
        // Get PR node_id
        let pr_path = format!("/repos/{}/{}/pulls/{}", repo.owner, repo.name, number);
        let pr_body = self.get(&pr_path).await?;
        let pr_json: serde_json::Value = serde_json::from_str(&pr_body)?;
        let node_id = pr_json["node_id"]
            .as_str()
            .ok_or_else(|| ApiError::Other("Missing node_id".into()))?
            .to_string();

        if draft {
            let query = "mutation($id: ID!) { convertPullRequestToDraft(input: {pullRequestId: $id}) { pullRequest { isDraft } } }";
            self.graphql(query, json!({"id": node_id})).await?;
        } else {
            let query = "mutation($id: ID!) { markPullRequestReadyForReview(input: {pullRequestId: $id}) { pullRequest { isDraft } } }";
            self.graphql(query, json!({"id": node_id})).await?;
        }
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
