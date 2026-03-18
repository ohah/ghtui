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
    #[serde(default)]
    pub keybindings: KeybindingConfig,
    #[serde(default)]
    pub enterprise_url: Option<String>,
    #[serde(default = "default_offline_cache")]
    pub offline_cache: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KeybindingConfig {
    // Global
    #[serde(default = "default_quit")]
    pub quit: String,
    #[serde(default = "default_help")]
    pub help: String,
    #[serde(default = "default_theme_toggle")]
    pub theme_toggle: String,
    #[serde(default = "default_search")]
    pub search: String,
    #[serde(default = "default_palette")]
    pub palette: String,
    #[serde(default = "default_home")]
    pub home: String,
    #[serde(default = "default_account_switch")]
    pub account_switch: String,

    // Navigation
    #[serde(default = "default_nav_down")]
    pub nav_down: String,
    #[serde(default = "default_nav_up")]
    pub nav_up: String,
    #[serde(default = "default_nav_next_page")]
    pub nav_next_page: String,
    #[serde(default = "default_nav_prev_page")]
    pub nav_prev_page: String,
    #[serde(default = "default_nav_open")]
    pub nav_open: String,
    #[serde(default = "default_nav_back")]
    pub nav_back: String,
    #[serde(default = "default_nav_refresh")]
    pub nav_refresh: String,

    // Editor
    #[serde(default = "default_edit_submit")]
    pub edit_submit: String,
    #[serde(default = "default_edit_cancel")]
    pub edit_cancel: String,
    #[serde(default = "default_edit_undo")]
    pub edit_undo: String,
    #[serde(default = "default_edit_redo")]
    pub edit_redo: String,

    // List
    #[serde(default = "default_filter_toggle")]
    pub filter_toggle: String,
    #[serde(default = "default_sort_cycle")]
    pub sort_cycle: String,
    #[serde(default = "default_search_start")]
    pub search_start: String,
    #[serde(default = "default_filter_clear")]
    pub filter_clear: String,
    #[serde(default = "default_create")]
    pub create: String,
    #[serde(default = "default_delete")]
    pub delete: String,
    #[serde(default = "default_open_browser")]
    pub open_browser: String,

    // Code
    #[serde(default = "default_code_branch")]
    pub code_branch: String,
    #[serde(default = "default_code_commits")]
    pub code_commits: String,
    #[serde(default = "default_code_edit")]
    pub code_edit: String,
}

impl Default for KeybindingConfig {
    fn default() -> Self {
        Self {
            quit: default_quit(),
            help: default_help(),
            theme_toggle: default_theme_toggle(),
            search: default_search(),
            palette: default_palette(),
            home: default_home(),
            account_switch: default_account_switch(),
            nav_down: default_nav_down(),
            nav_up: default_nav_up(),
            nav_next_page: default_nav_next_page(),
            nav_prev_page: default_nav_prev_page(),
            nav_open: default_nav_open(),
            nav_back: default_nav_back(),
            nav_refresh: default_nav_refresh(),
            edit_submit: default_edit_submit(),
            edit_cancel: default_edit_cancel(),
            edit_undo: default_edit_undo(),
            edit_redo: default_edit_redo(),
            filter_toggle: default_filter_toggle(),
            sort_cycle: default_sort_cycle(),
            search_start: default_search_start(),
            filter_clear: default_filter_clear(),
            create: default_create(),
            delete: default_delete(),
            open_browser: default_open_browser(),
            code_branch: default_code_branch(),
            code_commits: default_code_commits(),
            code_edit: default_code_edit(),
        }
    }
}

impl KeybindingConfig {
    /// Get all bindings as (category, name, key, default_key) tuples for display
    pub fn all_bindings(&self) -> Vec<(&str, &str, &str, &str)> {
        vec![
            ("Global", "Quit", &self.quit, "q"),
            ("Global", "Help", &self.help, "?"),
            ("Global", "Theme toggle", &self.theme_toggle, "t"),
            ("Global", "Search", &self.search, "Ctrl+k"),
            ("Global", "Command palette", &self.palette, "Ctrl+p"),
            ("Global", "Home", &self.home, "H"),
            ("Global", "Account switch", &self.account_switch, "S"),
            ("Navigation", "Down", &self.nav_down, "j"),
            ("Navigation", "Up", &self.nav_up, "k"),
            ("Navigation", "Next page", &self.nav_next_page, "n"),
            ("Navigation", "Prev page", &self.nav_prev_page, "p"),
            ("Navigation", "Open", &self.nav_open, "Enter"),
            ("Navigation", "Back", &self.nav_back, "Esc"),
            ("Navigation", "Refresh", &self.nav_refresh, "r"),
            ("Editor", "Submit", &self.edit_submit, "Ctrl+s"),
            ("Editor", "Cancel", &self.edit_cancel, "Esc"),
            ("Editor", "Undo", &self.edit_undo, "Ctrl+z"),
            ("Editor", "Redo", &self.edit_redo, "Ctrl+y"),
            ("List", "Filter toggle", &self.filter_toggle, "s"),
            ("List", "Sort cycle", &self.sort_cycle, "o"),
            ("List", "Search start", &self.search_start, "/"),
            ("List", "Filter clear", &self.filter_clear, "F"),
            ("List", "Create", &self.create, "c"),
            ("List", "Delete", &self.delete, "d"),
            ("List", "Open in browser", &self.open_browser, "o"),
            ("Code", "Branch picker", &self.code_branch, "b"),
            ("Code", "Commits", &self.code_commits, "c"),
            ("Code", "Edit file", &self.code_edit, "e"),
        ]
    }

    /// Set a binding by index (matching all_bindings order)
    pub fn set_binding(&mut self, index: usize, value: String) {
        match index {
            0 => self.quit = value,
            1 => self.help = value,
            2 => self.theme_toggle = value,
            3 => self.search = value,
            4 => self.palette = value,
            5 => self.home = value,
            6 => self.account_switch = value,
            7 => self.nav_down = value,
            8 => self.nav_up = value,
            9 => self.nav_next_page = value,
            10 => self.nav_prev_page = value,
            11 => self.nav_open = value,
            12 => self.nav_back = value,
            13 => self.nav_refresh = value,
            14 => self.edit_submit = value,
            15 => self.edit_cancel = value,
            16 => self.edit_undo = value,
            17 => self.edit_redo = value,
            18 => self.filter_toggle = value,
            19 => self.sort_cycle = value,
            20 => self.search_start = value,
            21 => self.filter_clear = value,
            22 => self.create = value,
            23 => self.delete = value,
            24 => self.open_browser = value,
            25 => self.code_branch = value,
            26 => self.code_commits = value,
            27 => self.code_edit = value,
            _ => {}
        }
    }

    /// Reset all bindings to defaults
    pub fn reset_to_defaults(&mut self) {
        *self = Self::default();
    }

    /// Save config to disk
    pub fn save(&self) -> Result<(), std::io::Error> {
        if let Some(path) = AppConfig::config_path() {
            let mut config = AppConfig::load();
            config.keybindings = self.clone();
            let content = toml::to_string_pretty(&config)
                .map_err(std::io::Error::other)?;
            if let Some(parent) = path.parent() {
                std::fs::create_dir_all(parent)?;
            }
            std::fs::write(&path, content)?;
        }
        Ok(())
    }
}

fn default_quit() -> String {
    "q".to_string()
}
fn default_help() -> String {
    "?".to_string()
}
fn default_theme_toggle() -> String {
    "t".to_string()
}
fn default_search() -> String {
    "Ctrl+k".to_string()
}
fn default_palette() -> String {
    "Ctrl+p".to_string()
}
fn default_home() -> String {
    "H".to_string()
}
fn default_account_switch() -> String {
    "S".to_string()
}
fn default_nav_down() -> String {
    "j".to_string()
}
fn default_nav_up() -> String {
    "k".to_string()
}
fn default_nav_next_page() -> String {
    "n".to_string()
}
fn default_nav_prev_page() -> String {
    "p".to_string()
}
fn default_nav_open() -> String {
    "Enter".to_string()
}
fn default_nav_back() -> String {
    "Esc".to_string()
}
fn default_nav_refresh() -> String {
    "r".to_string()
}
fn default_edit_submit() -> String {
    "Ctrl+s".to_string()
}
fn default_edit_cancel() -> String {
    "Esc".to_string()
}
fn default_edit_undo() -> String {
    "Ctrl+z".to_string()
}
fn default_edit_redo() -> String {
    "Ctrl+y".to_string()
}
fn default_filter_toggle() -> String {
    "s".to_string()
}
fn default_sort_cycle() -> String {
    "o".to_string()
}
fn default_search_start() -> String {
    "/".to_string()
}
fn default_filter_clear() -> String {
    "F".to_string()
}
fn default_create() -> String {
    "c".to_string()
}
fn default_delete() -> String {
    "d".to_string()
}
fn default_open_browser() -> String {
    "o".to_string()
}
fn default_code_branch() -> String {
    "b".to_string()
}
fn default_code_commits() -> String {
    "c".to_string()
}
fn default_code_edit() -> String {
    "e".to_string()
}

fn default_offline_cache() -> bool {
    true
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
            keybindings: KeybindingConfig::default(),
            enterprise_url: None,
            offline_cache: default_offline_cache(),
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
    pub fn resolve_token_for(&self, host: Option<&str>, user: Option<&str>) -> Option<String> {
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
    parse_hosts_yml(&content)
}

/// Parse hosts.yml content into a map of host -> accounts.
/// Extracted for testability.
fn parse_hosts_yml(content: &str) -> Option<HashMap<String, Vec<HostAccount>>> {
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

    let parsed: HashMap<String, serde_yaml::Value> = serde_yaml::from_str(content).ok()?;

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

    // -- GhAccount --

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
    fn test_gh_account_equality() {
        let a = GhAccount {
            host: "github.com".to_string(),
            user: "user1".to_string(),
            token: "token1".to_string(),
        };
        let b = a.clone();
        assert_eq!(a, b);

        let c = GhAccount {
            host: "github.com".to_string(),
            user: "user2".to_string(),
            token: "token1".to_string(),
        };
        assert_ne!(a, c);
    }

    // -- find_account --

    fn make_accounts() -> Vec<GhAccount> {
        vec![
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
        ]
    }

    #[test]
    fn test_find_account_by_user() {
        let accounts = make_accounts();
        let found = find_account(&accounts, None, Some("user2"));
        assert_eq!(found.unwrap().token, "token2");
    }

    #[test]
    fn test_find_account_by_host_and_user() {
        let accounts = make_accounts();
        let found = find_account(&accounts, Some("enterprise.com"), Some("admin"));
        assert_eq!(found.unwrap().token, "token3");
    }

    #[test]
    fn test_find_account_by_host_only() {
        let accounts = make_accounts();
        let found = find_account(&accounts, Some("github.com"), None);
        assert_eq!(found.unwrap().token, "token1");
    }

    #[test]
    fn test_find_account_no_match() {
        let accounts = make_accounts();
        assert!(find_account(&accounts, Some("other.com"), None).is_none());
    }

    #[test]
    fn test_find_account_no_filters() {
        let accounts = make_accounts();
        // With no filters, returns first account
        let found = find_account(&accounts, None, None);
        assert!(found.is_some());
    }

    #[test]
    fn test_find_account_empty_list() {
        let accounts: Vec<GhAccount> = vec![];
        assert!(find_account(&accounts, None, None).is_none());
        assert!(find_account(&accounts, Some("github.com"), Some("user1")).is_none());
    }

    #[test]
    fn test_find_account_wrong_user_on_host() {
        let accounts = make_accounts();
        // enterprise.com has "admin", not "user1"
        assert!(find_account(&accounts, Some("enterprise.com"), Some("user1")).is_none());
    }

    // -- parse_hosts_yml --

    #[test]
    fn test_parse_hosts_yml_single_account() {
        let yaml = r#"
github.com:
    user: myuser
    oauth_token: gho_abc123
    git_protocol: https
"#;
        let result = parse_hosts_yml(yaml).unwrap();
        assert_eq!(result.len(), 1);

        let github = &result["github.com"];
        assert_eq!(github.len(), 1);
        assert_eq!(github[0].user, "myuser");
        assert_eq!(github[0].token, "gho_abc123");
    }

    #[test]
    fn test_parse_hosts_yml_multi_account() {
        let yaml = r#"
github.com:
    users:
        user1:
            oauth_token: gho_token1
        user2:
            oauth_token: gho_token2
    user: user1
    git_protocol: https
"#;
        let result = parse_hosts_yml(yaml).unwrap();
        assert_eq!(result.len(), 1);

        let github = &result["github.com"];
        assert_eq!(github.len(), 2);

        let tokens: Vec<&str> = github.iter().map(|a| a.token.as_str()).collect();
        assert!(tokens.contains(&"gho_token1"));
        assert!(tokens.contains(&"gho_token2"));
    }

    #[test]
    fn test_parse_hosts_yml_multiple_hosts() {
        let yaml = r#"
github.com:
    user: personal
    oauth_token: gho_personal
    git_protocol: https
github.enterprise.com:
    user: work
    oauth_token: gho_work
    git_protocol: https
"#;
        let result = parse_hosts_yml(yaml).unwrap();
        assert_eq!(result.len(), 2);

        assert_eq!(result["github.com"][0].user, "personal");
        assert_eq!(result["github.enterprise.com"][0].user, "work");
    }

    #[test]
    fn test_parse_hosts_yml_mixed_single_and_multi() {
        let yaml = r#"
github.com:
    users:
        dev:
            oauth_token: gho_dev
        bot:
            oauth_token: gho_bot
    user: dev
    git_protocol: https
gitlab.example.com:
    user: admin
    oauth_token: glpat_123
    git_protocol: ssh
"#;
        let result = parse_hosts_yml(yaml).unwrap();
        assert_eq!(result.len(), 2);
        assert_eq!(result["github.com"].len(), 2);
        assert_eq!(result["gitlab.example.com"].len(), 1);
        assert_eq!(result["gitlab.example.com"][0].user, "admin");
    }

    #[test]
    fn test_parse_hosts_yml_empty_content() {
        // Empty YAML parses as null/empty map, no accounts returned
        let result = parse_hosts_yml("");
        assert!(result.is_none() || result.unwrap().is_empty());
    }

    #[test]
    fn test_parse_hosts_yml_invalid_yaml() {
        assert!(parse_hosts_yml("{{not valid yaml").is_none());
    }

    #[test]
    fn test_parse_hosts_yml_missing_token() {
        let yaml = r#"
github.com:
    user: myuser
    git_protocol: https
"#;
        let result = parse_hosts_yml(yaml).unwrap();
        // No token means no account should be returned
        assert!(!result.contains_key("github.com"));
    }

    #[test]
    fn test_parse_hosts_yml_missing_user() {
        let yaml = r#"
github.com:
    oauth_token: gho_abc123
    git_protocol: https
"#;
        let result = parse_hosts_yml(yaml).unwrap();
        // No user means no account should be returned
        assert!(!result.contains_key("github.com"));
    }

    #[test]
    fn test_parse_hosts_yml_multi_account_empty_token_skipped() {
        let yaml = r#"
github.com:
    users:
        user1:
            oauth_token: gho_valid
        user2:
            oauth_token: ""
    user: user1
    git_protocol: https
"#;
        let result = parse_hosts_yml(yaml).unwrap();
        let github = &result["github.com"];
        // Only user1 should be included (user2 has empty token)
        assert_eq!(github.len(), 1);
        assert_eq!(github[0].user, "user1");
    }

    // -- resolve_token_for --

    #[test]
    fn test_resolve_token_for_config_token_no_filter() {
        let config = AppConfig {
            token: Some("config_token".to_string()),
            ..Default::default()
        };
        // No host/user filter → should return config token
        assert_eq!(
            config.resolve_token_for(None, None),
            Some("config_token".to_string())
        );
    }

    #[test]
    fn test_resolve_token_for_config_token_skipped_with_filter() {
        let config = AppConfig {
            token: Some("config_token".to_string()),
            ..Default::default()
        };
        // With a host filter, config token is skipped → falls through to gh accounts
        // This will return None in test env (no real gh config)
        let result = config.resolve_token_for(Some("github.com"), Some("nonexistent"));
        assert!(result.is_none());
    }

    #[test]
    fn test_resolve_token_backward_compat() {
        let config = AppConfig {
            token: Some("my_token".to_string()),
            ..Default::default()
        };
        // resolve_token() should behave same as resolve_token_for(None, None)
        assert_eq!(config.resolve_token(), config.resolve_token_for(None, None));
    }
}
