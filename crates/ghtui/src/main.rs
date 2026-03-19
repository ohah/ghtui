mod app;
mod cli;
mod command_executor;
mod event;
mod highlighter;
mod keybindings;
mod tui;
mod update;
mod view;
mod views;

use anyhow::{Context, Result};
use clap::Parser;
use ghtui_api::GithubClient;
use ghtui_core::types::common::RepoId;
use ghtui_core::{AppConfig, list_gh_accounts};

use crate::app::App;
use crate::cli::Cli;

#[tokio::main]
async fn main() -> Result<()> {
    let args = Cli::parse();

    // Handle --check-update
    if args.check_update {
        let current = env!("CARGO_PKG_VERSION");
        println!("ghtui v{}", current);

        let client = reqwest::Client::builder()
            .user_agent("ghtui")
            .build()
            .expect("Failed to create HTTP client");

        let url = "https://api.github.com/repos/ohah/ghtui/releases/latest";
        match client.get(url).send().await {
            Ok(resp) if resp.status().is_success() => {
                if let Ok(body) = resp.json::<serde_json::Value>().await
                    && let Some(tag) = body["tag_name"].as_str()
                {
                    let latest = tag.strip_prefix('v').unwrap_or(tag);
                    if latest != current {
                        println!("Update available: v{} → v{}", current, latest);
                        println!("Run `brew upgrade ghtui` or `cargo install ghtui` to update");
                    } else {
                        println!("Already up to date!");
                    }
                }
            }
            _ => {
                println!("Could not check for updates");
            }
        }
        return Ok(());
    }

    // Load config
    let mut config = AppConfig::load();

    // Override token from CLI
    if let Some(ref token) = args.token {
        config.token = Some(token.clone());
    }

    // Discover all gh accounts
    let accounts = list_gh_accounts();

    // Resolve token (with optional host/user filter)
    let token = if args.token.is_some() {
        config.resolve_token()
    } else {
        config.resolve_token_for(args.host.as_deref(), args.user.as_deref())
    }
    .context("No GitHub token found. Set GITHUB_TOKEN env var or login with `gh auth login`")?;

    // Find current account
    let current_account = accounts.iter().find(|a| a.token == token).cloned();

    // Resolve repo
    let repo_str = args
        .repo
        .or_else(|| config.default_repo.clone())
        .or_else(Cli::detect_repo);

    let repo = repo_str
        .map(|s| s.parse::<RepoId>())
        .transpose()
        .map_err(|e| anyhow::anyhow!(e))?;

    // Setup logging
    if let Some(ref log_file) = args.log_file {
        let file = std::fs::File::create(log_file)?;
        tracing_subscriber::fmt()
            .with_writer(file)
            .with_env_filter("ghtui=debug")
            .init();
    }

    // Create API client (use enterprise URL if configured)
    let client = if let Some(ref enterprise_url) = config.enterprise_url {
        GithubClient::with_base_url(token, enterprise_url.clone())?
    } else {
        GithubClient::new(token)?
    };

    // Cleanup stale disk cache entries in background (non-blocking startup)
    if config.offline_cache {
        let cleanup_client = client.clone();
        tokio::spawn(async move {
            tokio::task::spawn_blocking(move || cleanup_client.cleanup_disk_cache());
        });
    }

    // Run app
    let mut app = App::new(config, client, repo, current_account, accounts);
    app.run().await
}
