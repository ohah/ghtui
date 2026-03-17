use crate::theme::ThemeMode;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppConfig {
    #[serde(default = "default_per_page")]
    pub per_page: u32,
    #[serde(default = "default_tick_rate")]
    pub tick_rate_ms: u64,
    #[serde(default)]
    pub token: Option<String>,
    #[serde(default)]
    pub default_repo: Option<String>,
    #[serde(default)]
    pub theme: ThemeMode,
}

fn default_per_page() -> u32 {
    30
}

fn default_tick_rate() -> u64 {
    1000
}

impl Default for AppConfig {
    fn default() -> Self {
        Self {
            per_page: default_per_page(),
            tick_rate_ms: default_tick_rate(),
            token: None,
            default_repo: None,
            theme: ThemeMode::Dark,
        }
    }
}

impl AppConfig {
    pub fn config_dir() -> Option<PathBuf> {
        dirs::config_dir().map(|d| d.join("ghtui"))
    }

    pub fn config_path() -> Option<PathBuf> {
        Self::config_dir().map(|d| d.join("config.toml"))
    }

    pub fn load() -> Self {
        let Some(path) = Self::config_path() else {
            return Self::default();
        };

        match std::fs::read_to_string(&path) {
            Ok(content) => toml::from_str(&content).unwrap_or_default(),
            Err(_) => Self::default(),
        }
    }

    pub fn resolve_token(&self) -> Option<String> {
        // 1. Config file token
        if let Some(ref token) = self.token {
            return Some(token.clone());
        }

        // 2. Environment variable
        if let Ok(token) = std::env::var("GITHUB_TOKEN") {
            return Some(token);
        }
        if let Ok(token) = std::env::var("GH_TOKEN") {
            return Some(token);
        }

        // 3. gh CLI token from hosts.yml
        Self::read_gh_token()
    }

    fn read_gh_token() -> Option<String> {
        // 1. Try `gh auth token` command first (most reliable)
        if let Ok(output) = std::process::Command::new("gh")
            .args(["auth", "token"])
            .output()
        {
            if output.status.success() {
                let token = String::from_utf8(output.stdout).ok()?.trim().to_string();
                if !token.is_empty() {
                    return Some(token);
                }
            }
        }

        // 2. Try reading from ~/.config/gh/hosts.yml (XDG path)
        let home = dirs::home_dir()?;
        let candidates = [
            home.join(".config").join("gh").join("hosts.yml"),
        ];

        for hosts_path in &candidates {
            if let Ok(content) = std::fs::read_to_string(hosts_path) {
                for line in content.lines() {
                    let trimmed = line.trim();
                    if trimmed.starts_with("oauth_token:") {
                        return trimmed
                            .strip_prefix("oauth_token:")
                            .map(|s| s.trim().to_string());
                    }
                }
            }
        }

        None
    }
}
