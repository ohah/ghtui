mod app;
mod cli;
mod command_executor;
mod event;
mod keybindings;
mod tui;
mod update;
mod view;
mod views;

use anyhow::{Context, Result};
use clap::Parser;
use ghtui_api::GithubClient;
use ghtui_core::AppConfig;
use ghtui_core::types::common::RepoId;

use crate::app::App;
use crate::cli::Cli;

#[tokio::main]
async fn main() -> Result<()> {
    let args = Cli::parse();

    // Load config
    let mut config = AppConfig::load();

    // Override token from CLI
    if let Some(ref token) = args.token {
        config.token = Some(token.clone());
    }

    // Resolve token
    let token = config
        .resolve_token()
        .context("No GitHub token found. Set GITHUB_TOKEN env var or login with `gh auth login`")?;

    // Resolve repo
    let repo_str = args
        .repo
        .or_else(|| config.default_repo.clone())
        .or_else(|| Cli::detect_repo());

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

    // Create API client
    let client = GithubClient::new(token)?;

    // Run app
    let mut app = App::new(config, client, repo);
    app.run().await
}
