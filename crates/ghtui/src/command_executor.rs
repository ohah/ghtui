use ghtui_api::GithubClient;
use ghtui_core::types::IssueFilters;
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
                Ok((prs, pagination)) => Message::PrListLoaded(prs, pagination),
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
                Ok((runs, pagination)) => Message::RunsLoaded(runs, pagination),
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

        // Notifications
        Command::FetchNotifications(filters) => match client.list_notifications(&filters).await {
            Ok(notifications) => Message::NotificationsLoaded(notifications),
            Err(e) => Message::Error(e.into()),
        },
        Command::MarkNotificationRead(id) => match client.mark_notification_read(&id).await {
            Ok(()) => Message::NotificationMarkedRead(id),
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
