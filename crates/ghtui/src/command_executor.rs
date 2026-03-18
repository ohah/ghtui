use ghtui_api::GithubClient;
use ghtui_core::types::{IssueFilters, PrFilters};
use ghtui_core::{Command, Message};

pub async fn execute(client: &GithubClient, cmd: Command) -> Message {
    match cmd {
        Command::None => Message::Tick,
        Command::Batch(cmds) => {
            // Execute first command only; in real impl we'd handle all
            if let Some(first) = cmds.into_iter().next() {
                Box::pin(execute(client, first)).await
            } else {
                Message::Tick
            }
        }

        // PR
        Command::FetchPrList(repo, filters, page) => {
            match client.list_pulls(&repo, &filters, page, 30).await {
                Ok((prs, pagination)) => Message::PrListLoaded(prs, pagination, filters),
                Err(e) => Message::Error(e.into()),
            }
        }
        Command::FetchPrDetail(repo, number) => match client.get_pull_detail(&repo, number).await {
            Ok(detail) => Message::PrDetailLoaded(Box::new(detail)),
            Err(e) => Message::Error(e.into()),
        },
        Command::FetchPrDiff(repo, number) => match client.get_pull_diff(&repo, number).await {
            Ok(files) => Message::PrDiffLoaded(files),
            Err(e) => Message::Error(e.into()),
        },
        Command::MergePr(repo, number, method) => {
            match client.merge_pull(&repo, number, method).await {
                Ok(()) => Message::PrMerged(number),
                Err(e) => Message::Error(e.into()),
            }
        }
        Command::ClosePr(repo, number) => match client.close_pull(&repo, number).await {
            Ok(()) => Message::PrClosed(number),
            Err(e) => Message::Error(e.into()),
        },
        Command::ReopenPr(repo, number) => match client.reopen_pull(&repo, number).await {
            Ok(()) => Message::PrReopened(number),
            Err(e) => Message::Error(e.into()),
        },
        Command::CreatePr(repo, input) => match client.create_pull(&repo, &input).await {
            Ok(number) => Message::PrCreated(number),
            Err(e) => Message::Error(e.into()),
        },
        Command::SubmitReview(repo, number, input) => {
            match client.submit_review(&repo, number, &input).await {
                Ok(()) => Message::ReviewSubmitted,
                Err(e) => Message::Error(e.into()),
            }
        }
        Command::SearchPulls(repo, query) => match client.search_pulls(&repo, &query).await {
            Ok((prs, pagination)) => Message::PrListLoaded(prs, pagination, PrFilters::default()),
            Err(e) => Message::Error(e.into()),
        },
        Command::UpdatePr(repo, number, title, body) => {
            match client
                .update_pull(&repo, number, title.as_deref(), body.as_deref())
                .await
            {
                Ok(()) => Message::PrUpdated(number),
                Err(e) => Message::Error(e.into()),
            }
        }
        Command::SetPrLabels(repo, number, labels) => {
            match client.set_issue_labels(&repo, number, &labels).await {
                Ok(()) => Message::PrUpdated(number),
                Err(e) => Message::Error(e.into()),
            }
        }
        Command::SetPrAssignees(repo, number, assignees) => {
            match client.set_issue_assignees(&repo, number, &assignees).await {
                Ok(()) => Message::PrUpdated(number),
                Err(e) => Message::Error(e.into()),
            }
        }
        Command::AddPrComment(repo, number, body) => {
            match client.add_issue_comment(&repo, number, &body).await {
                Ok(()) => Message::CommentAdded,
                Err(e) => Message::Error(e.into()),
            }
        }
        Command::UpdatePrComment(repo, _pr_number, comment_id, body) => {
            match client.update_comment(&repo, comment_id, &body).await {
                Ok(()) => Message::CommentUpdated,
                Err(e) => Message::Error(e.into()),
            }
        }
        Command::DeletePrComment(repo, comment_id) => {
            match client.delete_comment(&repo, comment_id).await {
                Ok(()) => Message::CommentDeleted,
                Err(e) => Message::Error(e.into()),
            }
        }
        Command::ChangePrBase(repo, number, base) => {
            match client.change_pull_base(&repo, number, &base).await {
                Ok(()) => Message::PrUpdated(number),
                Err(e) => Message::Error(e.into()),
            }
        }

        Command::SetPrReviewers(repo, number, reviewers) => {
            match client.request_reviewers(&repo, number, &reviewers).await {
                Ok(()) => Message::PrUpdated(number),
                Err(e) => Message::Error(e.into()),
            }
        }
        Command::SetPrDraft(repo, number, draft) => {
            match client.set_draft(&repo, number, draft).await {
                Ok(()) => Message::PrUpdated(number),
                Err(e) => Message::Error(e.into()),
            }
        }
        Command::SetAutoMerge(repo, number, enable) => {
            let result = if enable {
                client.enable_auto_merge(&repo, number).await
            } else {
                client.disable_auto_merge(&repo, number).await
            };
            match result {
                Ok(()) => Message::PrUpdated(number),
                Err(e) => Message::Error(e.into()),
            }
        }
        Command::ResolveReviewThread(_repo, number, thread_id, resolve) => {
            match client.set_review_thread_resolved(&thread_id, resolve).await {
                Ok(()) => Message::PrUpdated(number),
                Err(e) => Message::Error(e.into()),
            }
        }

        // Issues
        Command::FetchIssueList(repo, filters, page) => {
            match client.list_issues(&repo, &filters, page, 30).await {
                Ok((issues, pagination)) => Message::IssueListLoaded(issues, pagination, filters),
                Err(e) => Message::Error(e.into()),
            }
        }
        Command::FetchIssueDetail(repo, number) => {
            match client.get_issue_detail(&repo, number).await {
                Ok(detail) => Message::IssueDetailLoaded(Box::new(detail)),
                Err(e) => Message::Error(e.into()),
            }
        }
        Command::CloseIssue(repo, number) => match client.close_issue(&repo, number).await {
            Ok(()) => Message::IssueClosed(number),
            Err(e) => Message::Error(e.into()),
        },
        Command::ReopenIssue(repo, number) => match client.reopen_issue(&repo, number).await {
            Ok(()) => Message::IssueReopened(number),
            Err(e) => Message::Error(e.into()),
        },
        Command::LockIssue(repo, number) => match client.lock_issue(&repo, number).await {
            Ok(()) => Message::IssueUpdated(number),
            Err(e) => Message::Error(e.into()),
        },
        Command::UnlockIssue(repo, number) => match client.unlock_issue(&repo, number).await {
            Ok(()) => Message::IssueUpdated(number),
            Err(e) => Message::Error(e.into()),
        },
        Command::PinIssue(repo, number) => match client.pin_issue(&repo, number).await {
            Ok(()) => {
                // Pin succeeded
                Message::IssueUpdated(number)
            }
            Err(_) => {
                // Already pinned, try unpin
                match client.unpin_issue(&repo, number).await {
                    Ok(()) => Message::IssueUpdated(number),
                    Err(e) => Message::Error(e.into()),
                }
            }
        },
        Command::UnpinIssue(repo, number) => match client.unpin_issue(&repo, number).await {
            Ok(()) => Message::IssueUpdated(number),
            Err(e) => Message::Error(e.into()),
        },
        Command::FetchPinnedIssues(repo) => match client.get_pinned_issue_numbers(&repo).await {
            Ok(numbers) => Message::IssuePinnedNumbersLoaded(numbers),
            Err(_) => Message::IssuePinnedNumbersLoaded(Vec::new()),
        },
        Command::TransferIssue(repo, number, dest) => {
            let parts: Vec<&str> = dest.splitn(2, '/').collect();
            if parts.len() == 2 {
                match client
                    .transfer_issue(&repo, number, parts[0], parts[1])
                    .await
                {
                    Ok(()) => Message::IssueUpdated(number),
                    Err(e) => Message::Error(e.into()),
                }
            } else {
                Message::Error(ghtui_core::GhtuiError::Other(
                    "Invalid destination format".into(),
                ))
            }
        }
        Command::FetchIssueTemplates(repo) => match client.list_issue_templates(&repo).await {
            Ok(templates) => Message::IssueTemplatesLoaded(templates),
            Err(_) => Message::IssueTemplatesLoaded(Vec::new()),
        },
        Command::CreateIssue(repo, input) => match client.create_issue(&repo, &input).await {
            Ok(number) => Message::IssueCreated(number),
            Err(e) => Message::Error(e.into()),
        },
        Command::UpdateIssue(repo, number, title, body) => {
            match client
                .update_issue(&repo, number, title.as_deref(), body.as_deref())
                .await
            {
                Ok(()) => Message::IssueUpdated(number),
                Err(e) => Message::Error(e.into()),
            }
        }
        Command::SetIssueLabels(repo, number, labels) => {
            match client.set_issue_labels(&repo, number, &labels).await {
                Ok(()) => Message::IssueUpdated(number),
                Err(e) => Message::Error(e.into()),
            }
        }
        Command::SetIssueAssignees(repo, number, assignees) => {
            match client.set_issue_assignees(&repo, number, &assignees).await {
                Ok(()) => Message::IssueUpdated(number),
                Err(e) => Message::Error(e.into()),
            }
        }
        Command::FetchRepoLabels(repo) => match client.list_repo_labels(&repo).await {
            Ok(labels) => Message::IssueLabelsLoaded(labels),
            Err(e) => Message::Error(e.into()),
        },
        Command::FetchCollaboratorsForPicker(repo) => {
            match client.list_collaborators_logins(&repo).await {
                Ok(logins) => Message::IssueCollaboratorsLoaded(logins),
                Err(e) => Message::Error(e.into()),
            }
        }
        Command::DeleteComment(repo, comment_id) => {
            match client.delete_comment(&repo, comment_id).await {
                Ok(()) => Message::CommentDeleted,
                Err(e) => Message::Error(e.into()),
            }
        }
        Command::AddReaction(repo, id, reaction, is_issue) => {
            let result = if is_issue {
                client.add_issue_reaction(&repo, id, &reaction).await
            } else {
                client.add_reaction(&repo, id, &reaction).await
            };
            match result {
                Ok(()) => Message::ReactionAdded,
                Err(e) => Message::Error(e.into()),
            }
        }
        Command::SetMilestone(repo, number, ms_number) => {
            match client.set_issue_milestone(&repo, number, ms_number).await {
                Ok(()) => Message::IssueUpdated(number),
                Err(e) => Message::Error(e.into()),
            }
        }
        Command::FetchMilestones(repo) => match client.list_milestones(&repo).await {
            Ok(milestones) => Message::IssueMilestonesLoaded(milestones),
            Err(e) => Message::Error(e.into()),
        },
        Command::SearchIssues(repo, query) => match client.search_issues(&repo, &query).await {
            Ok((issues, pagination)) => {
                Message::IssueListLoaded(issues, pagination, IssueFilters::default())
            }
            Err(e) => Message::Error(e.into()),
        },
        Command::AddComment(repo, number, body) => {
            match client.add_issue_comment(&repo, number, &body).await {
                Ok(()) => Message::CommentAdded,
                Err(e) => Message::Error(e.into()),
            }
        }
        Command::UpdateComment(repo, _issue_number, comment_id, body) => {
            match client.update_comment(&repo, comment_id, &body).await {
                Ok(()) => Message::CommentUpdated,
                Err(e) => Message::Error(e.into()),
            }
        }

        // Actions
        Command::FetchRuns(repo, filters, page) => {
            match client.list_runs(&repo, &filters, page, 30).await {
                Ok((runs, pagination)) => Message::RunsLoaded(runs, pagination, filters),
                Err(e) => Message::Error(e.into()),
            }
        }
        Command::FetchRunDetail(repo, run_id) => match client.get_run_detail(&repo, run_id).await {
            Ok(detail) => Message::RunDetailLoaded(Box::new(detail)),
            Err(e) => Message::Error(e.into()),
        },
        Command::FetchJobLog(repo, _run_id, job_id) => {
            match client.get_job_log(&repo, job_id).await {
                Ok(lines) => Message::JobLogLoaded(job_id, lines),
                Err(e) => Message::Error(e.into()),
            }
        }
        Command::CancelRun(repo, run_id) => match client.cancel_run(&repo, run_id).await {
            Ok(()) => Message::RunCancelled(run_id),
            Err(e) => Message::Error(e.into()),
        },
        Command::RerunRun(repo, run_id) => match client.rerun_run(&repo, run_id).await {
            Ok(()) => Message::RunRerun(run_id),
            Err(e) => Message::Error(e.into()),
        },
        Command::RerunFailedJobs(repo, run_id) => {
            match client.rerun_failed_jobs(&repo, run_id).await {
                Ok(()) => Message::RunRerunFailed(run_id),
                Err(e) => Message::Error(e.into()),
            }
        }
        Command::DeleteRun(repo, run_id) => match client.delete_run(&repo, run_id).await {
            Ok(()) => Message::RunDeleted(run_id),
            Err(e) => Message::Error(e.into()),
        },
        Command::FetchWorkflows(repo) => match client.list_workflows(&repo).await {
            Ok(workflows) => Message::WorkflowsLoaded(workflows),
            Err(e) => Message::Error(e.into()),
        },
        Command::FetchRunArtifacts(repo, run_id) => {
            match client.list_run_artifacts(&repo, run_id).await {
                Ok(artifacts) => Message::ArtifactsLoaded(artifacts),
                Err(e) => Message::Error(e.into()),
            }
        }
        Command::DownloadArtifact(repo, artifact_id, name) => {
            match client.download_artifact(&repo, artifact_id).await {
                Ok(url) => Message::ArtifactDownloaded(name, url),
                Err(e) => Message::Error(e.into()),
            }
        }
        Command::DispatchWorkflow(repo, workflow_id, git_ref, inputs) => {
            match client
                .dispatch_workflow(&repo, workflow_id, &git_ref, &inputs)
                .await
            {
                Ok(()) => Message::WorkflowDispatched,
                Err(e) => Message::Error(e.into()),
            }
        }
        Command::FetchWorkflowFile(repo, path) => {
            match client.get_workflow_file(&repo, &path).await {
                Ok(content) => Message::WorkflowFileLoaded(content),
                Err(e) => Message::Error(e.into()),
            }
        }
        Command::FetchWorkflowInputs(repo, workflow_id, workflow_name, path) => {
            match client.get_workflow_file(&repo, &path).await {
                Ok(content) => {
                    let inputs = parse_workflow_inputs(&content);
                    Message::WorkflowInputsLoaded(workflow_id, workflow_name, inputs)
                }
                Err(e) => Message::Error(e.into()),
            }
        }
        Command::FetchPendingDeployments(repo, run_id) => {
            match client.list_pending_deployments(&repo, run_id).await {
                Ok(deployments) => Message::PendingDeploymentsLoaded(deployments),
                Err(e) => Message::Error(e.into()),
            }
        }
        Command::ApproveDeployment(repo, run_id, env_ids) => {
            match client.approve_deployment(&repo, run_id, &env_ids).await {
                Ok(()) => Message::DeploymentApproved,
                Err(e) => Message::Error(e.into()),
            }
        }
        Command::RejectDeployment(repo, run_id, env_ids) => {
            match client.reject_deployment(&repo, run_id, &env_ids).await {
                Ok(()) => Message::DeploymentRejected,
                Err(e) => Message::Error(e.into()),
            }
        }

        // Notifications
        Command::FetchNotifications(filters) => match client.list_notifications(&filters).await {
            Ok(notifications) => Message::NotificationsLoaded(notifications),
            Err(e) => Message::Error(e.into()),
        },
        Command::MarkNotificationRead(id) => match client.mark_notification_read(&id).await {
            Ok(()) => Message::NotificationMarkedRead(id),
            Err(e) => Message::Error(e.into()),
        },
        Command::MarkAllNotificationsRead => match client.mark_all_notifications_read().await {
            Ok(()) => Message::NotificationAllMarkedRead,
            Err(e) => Message::Error(e.into()),
        },
        Command::UnsubscribeThread(id) => match client.unsubscribe_thread(&id).await {
            Ok(()) => Message::NotificationUnsubscribed(id),
            Err(e) => Message::Error(e.into()),
        },
        Command::MarkThreadDone(id) => match client.mark_thread_done(&id).await {
            Ok(()) => Message::NotificationDoneResult(id),
            Err(e) => Message::Error(e.into()),
        },

        // Search
        Command::Search(query, kind, page) => match client.search(&query, kind, page).await {
            Ok(results) => Message::SearchResults(results),
            Err(e) => Message::Error(e.into()),
        },

        // Insights
        Command::FetchContributorStats(repo) => match client.get_contributor_stats(&repo).await {
            Ok(stats) => Message::ContributorStatsLoaded(stats),
            Err(e) => Message::Error(e.into()),
        },
        Command::FetchCommitActivity(repo) => match client.get_commit_activity(&repo).await {
            Ok(activity) => Message::CommitActivityLoaded(activity),
            Err(e) => Message::Error(e.into()),
        },
        Command::FetchTrafficClones(repo) => match client.get_traffic_clones(&repo).await {
            Ok(clones) => Message::TrafficClonesLoaded(clones),
            Err(e) => Message::Error(e.into()),
        },
        Command::FetchTrafficViews(repo) => match client.get_traffic_views(&repo).await {
            Ok(views) => Message::TrafficViewsLoaded(views),
            Err(e) => Message::Error(e.into()),
        },
        Command::FetchCodeFrequency(repo) => match client.get_code_frequency(&repo).await {
            Ok(freq) => Message::CodeFrequencyLoaded(freq),
            Err(e) => Message::Error(e.into()),
        },
        Command::FetchForks(repo) => match client.list_forks(&repo).await {
            Ok(forks) => Message::ForksLoaded(forks),
            Err(e) => Message::Error(e.into()),
        },
        Command::FetchDependencyGraph(repo) => match client.get_dependency_graph(&repo).await {
            Ok(deps) => Message::DependencyGraphLoaded(deps),
            Err(e) => Message::Error(e.into()),
        },

        // Security
        Command::FetchDependabotAlerts(repo) => match client.list_dependabot_alerts(&repo).await {
            Ok(alerts) => Message::DependabotAlertsLoaded(alerts),
            Err(e) => Message::Error(e.into()),
        },
        Command::FetchCodeScanningAlerts(repo) => {
            match client.list_code_scanning_alerts(&repo).await {
                Ok(alerts) => Message::CodeScanningAlertsLoaded(alerts),
                Err(e) => Message::Error(e.into()),
            }
        }
        Command::FetchSecretScanningAlerts(repo) => {
            match client.list_secret_scanning_alerts(&repo).await {
                Ok(alerts) => Message::SecretScanningAlertsLoaded(alerts),
                Err(e) => Message::Error(e.into()),
            }
        }
        Command::FetchSecurityAdvisories(repo) => {
            match client.list_security_advisories(&repo).await {
                Ok(advisories) => Message::SecurityAdvisoriesLoaded(advisories),
                Err(e) => Message::Error(e.into()),
            }
        }
        Command::DismissDependabotAlert(repo, number, reason) => {
            match client
                .dismiss_dependabot_alert(&repo, number, &reason)
                .await
            {
                Ok(()) => Message::SecurityAlertUpdated(0),
                Err(e) => Message::Error(e.into()),
            }
        }
        Command::ReopenDependabotAlert(repo, number) => {
            match client.reopen_dependabot_alert(&repo, number).await {
                Ok(()) => Message::SecurityAlertUpdated(0),
                Err(e) => Message::Error(e.into()),
            }
        }
        Command::DismissCodeScanningAlert(repo, number, reason) => {
            match client
                .dismiss_code_scanning_alert(&repo, number, &reason)
                .await
            {
                Ok(()) => Message::SecurityAlertUpdated(1),
                Err(e) => Message::Error(e.into()),
            }
        }
        Command::ResolveSecretScanningAlert(repo, number, resolution) => {
            match client
                .resolve_secret_scanning_alert(&repo, number, &resolution)
                .await
            {
                Ok(()) => Message::SecurityAlertUpdated(2),
                Err(e) => Message::Error(e.into()),
            }
        }

        // Code
        Command::FetchContents(repo, path, git_ref) => {
            match client.list_contents(&repo, &path, &git_ref).await {
                Ok(entries) => Message::CodeContentsLoaded(entries),
                Err(e) => Message::Error(e.into()),
            }
        }
        Command::FetchFileContent(repo, path, git_ref) => {
            match client.get_file_content(&repo, &path, &git_ref).await {
                Ok(content) => {
                    let filename = path.rsplit('/').next().unwrap_or(&path).to_string();
                    Message::CodeFileLoaded(filename, content)
                }
                Err(e) => Message::Error(e.into()),
            }
        }
        Command::FetchReadme(repo, git_ref) => {
            // Try README.md, then readme.md, then README
            for name in &["README.md", "readme.md", "README"] {
                match client.get_file_content(&repo, name, &git_ref).await {
                    Ok(content) => return Message::CodeReadmeLoaded(content),
                    Err(_) => continue,
                }
            }
            // No README found — not an error, just no content
            Message::Tick
        }

        // Settings
        Command::FetchRepoSettings(repo) => match client.get_repo(&repo).await {
            Ok(repository) => Message::SettingsRepoLoaded(Box::new(repository)),
            Err(e) => Message::Error(e.into()),
        },
        Command::FetchBranchProtections(repo) => {
            match client.list_branch_protections(&repo).await {
                Ok(protections) => Message::SettingsBranchProtectionsLoaded(protections),
                Err(e) => Message::Error(e.into()),
            }
        }
        Command::FetchCollaborators(repo) => match client.list_collaborators(&repo).await {
            Ok(collaborators) => Message::SettingsCollaboratorsLoaded(collaborators),
            Err(e) => Message::Error(e.into()),
        },
        Command::FetchWebhooks(repo) => match client.list_webhooks(&repo).await {
            Ok(hooks) => Message::SettingsWebhooksLoaded(hooks),
            Err(e) => Message::Error(e.into()),
        },
        Command::FetchDeployKeys(repo) => match client.list_deploy_keys(&repo).await {
            Ok(keys) => Message::SettingsDeployKeysLoaded(keys),
            Err(e) => Message::Error(e.into()),
        },
        Command::UpdateRepo(repo, updates) => match client.update_repo(&repo, &updates).await {
            Ok(repository) => Message::SettingsRepoUpdated(Box::new(repository)),
            Err(e) => Message::Error(e.into()),
        },
        Command::RemoveCollaborator(repo, username) => {
            match client.remove_collaborator(&repo, &username).await {
                Ok(()) => Message::SettingsItemUpdated(2),
                Err(e) => Message::Error(e.into()),
            }
        }
        Command::DeleteWebhook(repo, hook_id) => {
            match client.delete_webhook(&repo, hook_id).await {
                Ok(()) => Message::SettingsItemUpdated(3),
                Err(e) => Message::Error(e.into()),
            }
        }
        Command::ToggleWebhook(repo, hook_id, active) => {
            match client.toggle_webhook(&repo, hook_id, active).await {
                Ok(()) => Message::SettingsItemUpdated(3),
                Err(e) => Message::Error(e.into()),
            }
        }
        Command::DeleteDeployKey(repo, key_id) => {
            match client.delete_deploy_key(&repo, key_id).await {
                Ok(()) => Message::SettingsItemUpdated(4),
                Err(e) => Message::Error(e.into()),
            }
        }
        Command::DeleteBranchProtection(repo, branch) => {
            match client.delete_branch_protection(&repo, &branch).await {
                Ok(()) => Message::SettingsItemUpdated(1),
                Err(e) => Message::Error(e.into()),
            }
        }
        Command::ToggleBranchEnforceAdmins(repo, branch, enable) => {
            let result = if enable {
                client.enable_enforce_admins(&repo, &branch).await
            } else {
                client.disable_enforce_admins(&repo, &branch).await
            };
            match result {
                Ok(()) => Message::SettingsItemUpdated(1),
                Err(e) => Message::Error(e.into()),
            }
        }

        // Utility
        Command::OpenInBrowser(url) => {
            let _ = open::that(&url);
            Message::Tick
        }
        Command::SetClipboard(text) => {
            if let Ok(mut clipboard) = arboard::Clipboard::new() {
                let _ = clipboard.set_text(&text);
            }
            Message::Tick
        }
        Command::Quit => Message::Quit,

        // SwitchAccount is handled directly in App::run(), not here
        Command::SwitchAccount(_) => Message::Tick,
    }
}

/// Parse workflow YAML to extract `on.workflow_dispatch.inputs`.
fn parse_workflow_inputs(yaml_content: &str) -> Vec<ghtui_core::types::WorkflowInput> {
    let Ok(doc) = serde_yaml::from_str::<serde_json::Value>(yaml_content) else {
        return Vec::new();
    };

    // Look for on.workflow_dispatch.inputs
    let Some(inputs_obj) = doc
        .pointer("/on/workflow_dispatch/inputs")
        .and_then(|v| v.as_object())
    else {
        return Vec::new();
    };

    inputs_obj
        .iter()
        .map(|(name, val)| {
            let input_type = val["type"].as_str().unwrap_or("string").to_string();
            let required = val["required"].as_bool().unwrap_or(false);
            let description = val["description"].as_str().map(|s| s.to_string());
            let default = val["default"]
                .as_str()
                .map(|s| s.to_string())
                .or_else(|| val["default"].as_bool().map(|b| b.to_string()));
            let options = val["options"]
                .as_array()
                .map(|arr| {
                    arr.iter()
                        .filter_map(|v| v.as_str().map(|s| s.to_string()))
                        .collect()
                })
                .unwrap_or_default();

            ghtui_core::types::WorkflowInput {
                name: name.clone(),
                required,
                input_type,
                description,
                default,
                options,
            }
        })
        .collect()
}
