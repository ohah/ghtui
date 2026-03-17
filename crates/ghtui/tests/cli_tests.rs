// CLI tests - test repo URL parsing

#[test]
fn test_parse_ssh_url() {
    let url = "git@github.com:ohah/ghtui.git";
    let result = parse_repo_from_url(url);
    assert_eq!(result, Some("ohah/ghtui".to_string()));
}

#[test]
fn test_parse_https_url() {
    let url = "https://github.com/ohah/ghtui.git";
    let result = parse_repo_from_url(url);
    assert_eq!(result, Some("ohah/ghtui".to_string()));
}

#[test]
fn test_parse_https_url_no_git_suffix() {
    let url = "https://github.com/ohah/ghtui";
    let result = parse_repo_from_url(url);
    assert_eq!(result, Some("ohah/ghtui".to_string()));
}

#[test]
fn test_parse_non_github_url() {
    let url = "https://gitlab.com/user/repo.git";
    let result = parse_repo_from_url(url);
    assert_eq!(result, None);
}

#[test]
fn test_parse_invalid_url() {
    let url = "not-a-url";
    let result = parse_repo_from_url(url);
    assert_eq!(result, None);
}

// Helper function duplicated from cli.rs for testing
fn parse_repo_from_url(url: &str) -> Option<String> {
    if url.starts_with("git@") {
        let path = url.split(':').nth(1)?;
        let repo = path.trim_end_matches(".git");
        return Some(repo.to_string());
    }

    if url.contains("github.com") {
        let parts: Vec<&str> = url.split("github.com/").collect();
        if parts.len() == 2 {
            let repo = parts[1].trim_end_matches(".git").trim_end_matches('/');
            return Some(repo.to_string());
        }
    }

    None
}
