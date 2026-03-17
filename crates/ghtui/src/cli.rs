use clap::Parser;

#[derive(Parser, Debug)]
#[command(name = "ghtui", about = "A comprehensive GitHub TUI")]
pub struct Cli {
    /// Repository in owner/name format (auto-detected from git remote if omitted)
    #[arg(short, long)]
    pub repo: Option<String>,

    /// GitHub token (reads from GH_TOKEN/GITHUB_TOKEN env or gh CLI config if omitted)
    #[arg(short, long)]
    pub token: Option<String>,

    /// Log file path for debug output
    #[arg(long)]
    pub log_file: Option<String>,
}

impl Cli {
    pub fn detect_repo() -> Option<String> {
        let output = std::process::Command::new("git")
            .args(["remote", "get-url", "origin"])
            .output()
            .ok()?;

        if !output.status.success() {
            return None;
        }

        let url = String::from_utf8(output.stdout).ok()?.trim().to_string();
        parse_repo_from_url(&url)
    }
}

fn parse_repo_from_url(url: &str) -> Option<String> {
    // SSH: git@github.com:owner/repo.git
    if url.starts_with("git@") {
        let path = url.split(':').nth(1)?;
        let repo = path.trim_end_matches(".git");
        return Some(repo.to_string());
    }

    // HTTPS: https://github.com/owner/repo.git
    if url.contains("github.com") {
        let parts: Vec<&str> = url.split("github.com/").collect();
        if parts.len() == 2 {
            let repo = parts[1].trim_end_matches(".git").trim_end_matches('/');
            return Some(repo.to_string());
        }
    }

    None
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_ssh_url() {
        assert_eq!(
            parse_repo_from_url("git@github.com:ohah/ghtui.git"),
            Some("ohah/ghtui".to_string())
        );
    }

    #[test]
    fn test_parse_https_url() {
        assert_eq!(
            parse_repo_from_url("https://github.com/ohah/ghtui.git"),
            Some("ohah/ghtui".to_string())
        );
    }

    #[test]
    fn test_parse_https_url_no_git() {
        assert_eq!(
            parse_repo_from_url("https://github.com/ohah/ghtui"),
            Some("ohah/ghtui".to_string())
        );
    }
}
