use ghtui_core::types::*;
use ghtui_core::types::common::RepoId;

use crate::client::GithubClient;
use crate::error::ApiError;

impl GithubClient {
    pub async fn list_runs(
        &self,
        repo: &RepoId,
        filters: &ActionsFilters,
        page: u32,
        per_page: u32,
    ) -> Result<(Vec<WorkflowRun>, Pagination), ApiError> {
        let mut params = vec![
            format!("page={}", page),
            format!("per_page={}", per_page),
        ];

        if let Some(ref status) = filters.status {
            params.push(format!("status={}", status));
        }
        if let Some(ref branch) = filters.branch {
            params.push(format!("branch={}", branch));
        }
        if let Some(ref event) = filters.event {
            params.push(format!("event={}", event));
        }
        if let Some(ref actor) = filters.actor {
            params.push(format!("actor={}", actor));
        }
        if let Some(workflow_id) = filters.workflow_id {
            params.push(format!("workflow_id={}", workflow_id));
        }

        let path = format!(
            "/repos/{}/{}/actions/runs?{}",
            repo.owner,
            repo.name,
            params.join("&")
        );

        let body = self.get(&path).await?;
        let response: serde_json::Value = serde_json::from_str(&body)?;

        let runs: Vec<WorkflowRun> = serde_json::from_value(
            response["workflow_runs"].clone(),
        )
        .unwrap_or_default();

        let total = response["total_count"].as_u64().map(|c| c as u32);

        let pagination = Pagination {
            page,
            per_page,
            has_next: runs.len() as u32 >= per_page,
            total,
        };

        Ok((runs, pagination))
    }

    pub async fn get_run_detail(
        &self,
        repo: &RepoId,
        run_id: u64,
    ) -> Result<WorkflowRunDetail, ApiError> {
        let run_path = format!(
            "/repos/{}/{}/actions/runs/{}",
            repo.owner, repo.name, run_id
        );
        let jobs_path = format!(
            "/repos/{}/{}/actions/runs/{}/jobs",
            repo.owner, repo.name, run_id
        );

        let (run_body, jobs_body) =
            tokio::join!(self.get(&run_path), self.get(&jobs_path));

        let run: WorkflowRun = serde_json::from_str(&run_body?)?;
        let jobs_response: serde_json::Value = serde_json::from_str(&jobs_body?)?;
        let jobs: Vec<Job> =
            serde_json::from_value(jobs_response["jobs"].clone()).unwrap_or_default();

        Ok(WorkflowRunDetail { run, jobs })
    }

    pub async fn get_job_log(
        &self,
        repo: &RepoId,
        job_id: u64,
    ) -> Result<Vec<LogLine>, ApiError> {
        let path = format!(
            "/repos/{}/{}/actions/jobs/{}/logs",
            repo.owner, repo.name, job_id
        );
        let url = self.url(&path);

        // GitHub redirects to a download URL for logs
        let response = self.http.get(&url).send().await?;

        let text = if response.status().is_redirection() {
            let redirect_url = response
                .headers()
                .get("location")
                .and_then(|v| v.to_str().ok())
                .ok_or(ApiError::Other("No redirect URL for logs".into()))?;
            self.get_raw(redirect_url).await?
        } else if response.status().is_success() {
            response.text().await?
        } else {
            let status = response.status().as_u16();
            let body = response.text().await?;
            return Err(ApiError::GitHub {
                status,
                message: body,
            });
        };

        let lines: Vec<LogLine> = text
            .lines()
            .map(|line| {
                // GitHub log format: "2024-01-01T00:00:00.000Z content"
                let (timestamp, content) = if line.len() > 24 && line.chars().nth(4) == Some('-') {
                    (
                        Some(line[..23].to_string()),
                        line[24..].to_string(),
                    )
                } else {
                    (None, line.to_string())
                };
                LogLine { content, timestamp }
            })
            .collect();

        Ok(lines)
    }

    pub async fn cancel_run(
        &self,
        repo: &RepoId,
        run_id: u64,
    ) -> Result<(), ApiError> {
        let path = format!(
            "/repos/{}/{}/actions/runs/{}/cancel",
            repo.owner, repo.name, run_id
        );
        self.post(&path, &serde_json::json!({})).await?;
        Ok(())
    }

    pub async fn rerun_run(
        &self,
        repo: &RepoId,
        run_id: u64,
    ) -> Result<(), ApiError> {
        let path = format!(
            "/repos/{}/{}/actions/runs/{}/rerun",
            repo.owner, repo.name, run_id
        );
        self.post(&path, &serde_json::json!({})).await?;
        Ok(())
    }
}
