use crate::theme::ThemeMode;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
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

/// A GitHub account discovered from gh CLI config or environment.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GhAccount {
    pub host: String,
    pub user: String,
    pub token: String,
}

impl GhAccount {
    pub fn display_name(&self) -> String {
        if self.host == "github.com" {
            self.user.clone()
        } else {
            format!("{}@{}", self.user, self.host)
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

    /// Resolve a token, optionally filtering by host and user.
    pub fn resolve_token_for(
        &self,
        host: Option<&str>,
        user: Option<&str>,
    ) -> Option<String> {
        // 1. Config file token (only if no specific host/user requested)
        if host.is_none() && user.is_none() {
            if let Some(ref token) = self.token {
                return Some(token.clone());
            }
        }

        // 2. Environment variable (only if no specific host/user requested)
        if host.is_none() && user.is_none() {
            if let Ok(token) = std::env::var("GITHUB_TOKEN") {
                return Some(token);
            }
            if let Ok(token) = std::env::var("GH_TOKEN") {
                return Some(token);
            }
        }

        // 3. Find from gh CLI accounts
        let accounts = list_gh_accounts();
        if let Some(account) = find_account(&accounts, host, user) {
            return Some(account.token.clone());
        }

        // 4. Fallback: if no host/user filter, try default gh auth token
        if host.is_none() && user.is_none() {
            if let Some(account) = accounts.first() {
                return Some(account.token.clone());
            }
        }

        None
    }

    /// Original resolve_token for backward compatibility.
    pub fn resolve_token(&self) -> Option<String> {
        self.resolve_token_for(None, None)
    }
}

/// Find a matching account from the list.
fn find_account<'a>(
    accounts: &'a [GhAccount],
    host: Option<&str>,
    user: Option<&str>,
) -> Option<&'a GhAccount> {
    accounts.iter().find(|a| {
        let host_match = host.is_none_or(|h| a.host == h);
        let user_match = user.is_none_or(|u| a.user == u);
        host_match && user_match
    })
}

/// List all GitHub accounts from gh CLI configuration.
pub fn list_gh_accounts() -> Vec<GhAccount> {
    let mut accounts = Vec::new();

    // Try parsing hosts.yml
    if let Some(hosts) = read_gh_hosts_yml() {
        for (host, entries) in hosts {
            for entry in entries {
                accounts.push(GhAccount {
                    host: host.clone(),
                    user: entry.user,
                    token: entry.token,
                });
            }
        }
    }

    // If no accounts found from hosts.yml, try `gh auth token`
    if accounts.is_empty() {
        if let Some(account) = read_gh_auth_token() {
            accounts.push(account);
        }
    }

    accounts
}

/// Parse gh CLI hosts.yml which supports multiple accounts per host.
fn read_gh_hosts_yml() -> Option<HashMap<String, Vec<HostAccount>>> {
    let home = dirs::home_dir()?;
    let hosts_path = home.join(".config").join("gh").join("hosts.yml");
    let content = std::fs::read_to_string(&hosts_path).ok()?;

    // hosts.yml structure:
    // github.com:
    //     user: username
    //     oauth_token: gho_xxx
    //     git_protocol: https
    // -- OR (multi-account, gh 2.40+) --
    // github.com:
    //     users:
    //         user1:
    //             oauth_token: gho_xxx
    //         user2:
    //             oauth_token: gho_yyy
    //     user: user1  (active user)
    //     git_protocol: https

    let parsed: HashMap<String, serde_yaml::Value> =
        serde_yaml::from_str(&content).ok()?;

    let mut result: HashMap<String, Vec<HostAccount>> = HashMap::new();

    for (host, value) in parsed {
        let mut host_accounts = Vec::new();

        let map = value.as_mapping()?;

        // Check for multi-account "users" key (gh 2.40+)
        if let Some(users_val) = map.get(serde_yaml::Value::String("users".to_string())) {
            if let Some(users_map) = users_val.as_mapping() {
                for (username_val, user_data) in users_map {
                    let username = username_val.as_str().unwrap_or_default().to_string();
                    if let Some(user_map) = user_data.as_mapping() {
                        let token = user_map
                            .get(serde_yaml::Value::String("oauth_token".to_string()))
                            .and_then(|v| v.as_str())
                            .unwrap_or_default()
                            .to_string();
                        if !token.is_empty() {
                            host_accounts.push(HostAccount {
                                user: username,
                                token,
                            });
                        }
                    }
                }
            }
        }

        // If no multi-account users found, fall back to single account
        if host_accounts.is_empty() {
            let user = map
                .get(serde_yaml::Value::String("user".to_string()))
                .and_then(|v| v.as_str())
                .unwrap_or_default()
                .to_string();
            let token = map
                .get(serde_yaml::Value::String("oauth_token".to_string()))
                .and_then(|v| v.as_str())
                .unwrap_or_default()
                .to_string();

            if !user.is_empty() && !token.is_empty() {
                host_accounts.push(HostAccount { user, token });
            }
        }

        if !host_accounts.is_empty() {
            result.insert(host, host_accounts);
        }
    }

    Some(result)
}

#[derive(Debug)]
struct HostAccount {
    user: String,
    token: String,
}

/// Fallback: get current active account from `gh auth token` + `gh auth status`.
fn read_gh_auth_token() -> Option<GhAccount> {
    let token_output = std::process::Command::new("gh")
        .args(["auth", "token"])
        .output()
        .ok()?;

    if !token_output.status.success() {
        return None;
    }

    let token = String::from_utf8(token_output.stdout)
        .ok()?
        .trim()
        .to_string();
    if token.is_empty() {
        return None;
    }

    // Try to get the username
    let status_output = std::process::Command::new("gh")
        .args(["auth", "status", "--active"])
        .output()
        .ok();

    let user = status_output
        .and_then(|o| String::from_utf8(o.stderr).ok())
        .and_then(|s| {
            // Parse "Logged in to github.com account username (...)"
            s.lines()
                .find(|l| l.contains("Logged in to"))
                .and_then(|l| {
                    l.split("account ").nth(1).map(|rest| {
                        rest.split_whitespace()
                            .next()
                            .unwrap_or("unknown")
                            .to_string()
                    })
                })
        })
        .unwrap_or_else(|| "unknown".to_string());

    Some(GhAccount {
        host: "github.com".to_string(),
        user,
        token,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_gh_account_display_name() {
        let account = GhAccount {
            host: "github.com".to_string(),
            user: "testuser".to_string(),
            token: "token".to_string(),
        };
        assert_eq!(account.display_name(), "testuser");

        let enterprise = GhAccount {
            host: "github.example.com".to_string(),
            user: "admin".to_string(),
            token: "token".to_string(),
        };
        assert_eq!(enterprise.display_name(), "admin@github.example.com");
    }

    #[test]
    fn test_find_account() {
        let accounts = vec![
            GhAccount {
                host: "github.com".to_string(),
                user: "user1".to_string(),
                token: "token1".to_string(),
            },
            GhAccount {
                host: "github.com".to_string(),
                user: "user2".to_string(),
                token: "token2".to_string(),
            },
            GhAccount {
                host: "enterprise.com".to_string(),
                user: "admin".to_string(),
                token: "token3".to_string(),
            },
        ];

        // Find by user
        let found = find_account(&accounts, None, Some("user2"));
        assert_eq!(found.unwrap().token, "token2");

        // Find by host + user
        let found = find_account(&accounts, Some("enterprise.com"), Some("admin"));
        assert_eq!(found.unwrap().token, "token3");

        // Find by host only (returns first match)
        let found = find_account(&accounts, Some("github.com"), None);
        assert_eq!(found.unwrap().token, "token1");

        // No match
        let found = find_account(&accounts, Some("other.com"), None);
        assert!(found.is_none());
    }
}
