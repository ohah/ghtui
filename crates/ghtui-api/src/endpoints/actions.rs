use ghtui_core::types::common::RepoId;
use ghtui_core::types::*;

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
        let mut params = vec![format!("page={}", page), format!("per_page={}", per_page)];

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

        let runs: Vec<WorkflowRun> =
            serde_json::from_value(response["workflow_runs"].clone()).unwrap_or_default();

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

        let (run_body, jobs_body) = tokio::join!(self.get(&run_path), self.get(&jobs_path));

        let run: WorkflowRun = serde_json::from_str(&run_body?)?;
        let jobs_response: serde_json::Value = serde_json::from_str(&jobs_body?)?;
        let jobs: Vec<Job> =
            serde_json::from_value(jobs_response["jobs"].clone()).unwrap_or_default();

        Ok(WorkflowRunDetail { run, jobs })
    }

    pub async fn get_job_log(&self, repo: &RepoId, job_id: u64) -> Result<Vec<LogLine>, ApiError> {
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
                    (Some(line[..23].to_string()), line[24..].to_string())
                } else {
                    (None, line.to_string())
                };
                LogLine { content, timestamp }
            })
            .collect();

        Ok(lines)
    }

    pub async fn list_workflows(&self, repo: &RepoId) -> Result<Vec<Workflow>, ApiError> {
        let path = format!(
            "/repos/{}/{}/actions/workflows?per_page=100",
            repo.owner, repo.name
        );
        let body = self.get(&path).await?;
        let response: serde_json::Value = serde_json::from_str(&body)?;
        let workflows: Vec<Workflow> =
            serde_json::from_value(response["workflows"].clone()).unwrap_or_default();
        Ok(workflows)
    }

    pub async fn cancel_run(&self, repo: &RepoId, run_id: u64) -> Result<(), ApiError> {
        let path = format!(
            "/repos/{}/{}/actions/runs/{}/cancel",
            repo.owner, repo.name, run_id
        );
        self.post(&path, &serde_json::json!({})).await?;
        Ok(())
    }

    pub async fn rerun_run(&self, repo: &RepoId, run_id: u64) -> Result<(), ApiError> {
        let path = format!(
            "/repos/{}/{}/actions/runs/{}/rerun",
            repo.owner, repo.name, run_id
        );
        self.post(&path, &serde_json::json!({})).await?;
        Ok(())
    }

    pub async fn rerun_failed_jobs(&self, repo: &RepoId, run_id: u64) -> Result<(), ApiError> {
        let path = format!(
            "/repos/{}/{}/actions/runs/{}/rerun-failed-jobs",
            repo.owner, repo.name, run_id
        );
        self.post(&path, &serde_json::json!({})).await?;
        Ok(())
    }

    pub async fn delete_run(&self, repo: &RepoId, run_id: u64) -> Result<(), ApiError> {
        let path = format!(
            "/repos/{}/{}/actions/runs/{}",
            repo.owner, repo.name, run_id
        );
        self.delete(&path).await?;
        Ok(())
    }

    pub async fn list_run_artifacts(
        &self,
        repo: &RepoId,
        run_id: u64,
    ) -> Result<Vec<Artifact>, ApiError> {
        let path = format!(
            "/repos/{}/{}/actions/runs/{}/artifacts",
            repo.owner, repo.name, run_id
        );
        let body = self.get(&path).await?;
        let response: serde_json::Value = serde_json::from_str(&body)?;
        let artifacts: Vec<Artifact> =
            serde_json::from_value(response["artifacts"].clone()).unwrap_or_default();
        Ok(artifacts)
    }

    /// Get artifact download URL. Uses a no-redirect request to capture the S3 URL.
    pub async fn download_artifact(
        &self,
        repo: &RepoId,
        artifact_id: u64,
    ) -> Result<String, ApiError> {
        let path = format!(
            "/repos/{}/{}/actions/artifacts/{}/zip",
            repo.owner, repo.name, artifact_id
        );
        let url = self.url(&path);

        // Build a one-off client that does NOT follow redirects
        let no_redirect = reqwest::Client::builder()
            .redirect(reqwest::redirect::Policy::none())
            .build()
            .map_err(ApiError::Http)?;

        let response = no_redirect
            .get(&url)
            .header("Authorization", format!("Bearer {}", self.token))
            .header("Accept", "application/vnd.github+json")
            .send()
            .await?;

        if response.status().is_redirection() {
            let redirect_url = response
                .headers()
                .get("location")
                .and_then(|v| v.to_str().ok())
                .ok_or(ApiError::Other("No redirect URL for artifact".into()))?
                .to_string();
            Ok(redirect_url)
        } else {
            let status = response.status().as_u16();
            let body = response.text().await?;
            Err(ApiError::GitHub {
                status,
                message: body,
            })
        }
    }

    pub async fn dispatch_workflow(
        &self,
        repo: &RepoId,
        workflow_id: u64,
        git_ref: &str,
        inputs: &serde_json::Value,
    ) -> Result<(), ApiError> {
        let path = format!(
            "/repos/{}/{}/actions/workflows/{}/dispatches",
            repo.owner, repo.name, workflow_id
        );
        let body = serde_json::json!({
            "ref": git_ref,
            "inputs": inputs,
        });
        self.post(&path, &body).await?;
        Ok(())
    }

    pub async fn get_workflow_file(
        &self,
        repo: &RepoId,
        workflow_path: &str,
    ) -> Result<String, ApiError> {
        // Delegate to shared get_file_content (default branch via empty ref)
        self.get_file_content(repo, workflow_path, "").await
    }

    pub async fn list_pending_deployments(
        &self,
        repo: &RepoId,
        run_id: u64,
    ) -> Result<Vec<PendingDeployment>, ApiError> {
        let path = format!(
            "/repos/{}/{}/actions/runs/{}/pending_deployments",
            repo.owner, repo.name, run_id
        );
        let body = self.get(&path).await?;
        let deployments: Vec<PendingDeployment> = serde_json::from_str(&body)?;
        Ok(deployments)
    }

    pub async fn approve_deployment(
        &self,
        repo: &RepoId,
        run_id: u64,
        environment_ids: &[u64],
    ) -> Result<(), ApiError> {
        self.respond_to_deployment(repo, run_id, environment_ids, "approved")
            .await
    }

    pub async fn reject_deployment(
        &self,
        repo: &RepoId,
        run_id: u64,
        environment_ids: &[u64],
    ) -> Result<(), ApiError> {
        self.respond_to_deployment(repo, run_id, environment_ids, "rejected")
            .await
    }

    async fn respond_to_deployment(
        &self,
        repo: &RepoId,
        run_id: u64,
        environment_ids: &[u64],
        state: &str,
    ) -> Result<(), ApiError> {
        let path = format!(
            "/repos/{}/{}/actions/runs/{}/pending_deployments",
            repo.owner, repo.name, run_id
        );
        let body = serde_json::json!({
            "environment_ids": environment_ids,
            "state": state,
            "comment": "",
        });
        self.post(&path, &body).await?;
        Ok(())
    }
}
