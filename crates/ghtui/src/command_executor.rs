use ghtui_api::GithubClient;
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
                Ok((issues, pagination)) => Message::IssueListLoaded(issues, pagination),
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
        Command::CreateIssue(repo, input) => match client.create_issue(&repo, &input).await {
            Ok(number) => Message::IssueCreated(number),
            Err(e) => Message::Error(e.into()),
        },
        Command::AddComment(repo, number, body) => {
            match client.add_issue_comment(&repo, number, &body).await {
                Ok(()) => Message::CommentAdded,
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
    }
}
