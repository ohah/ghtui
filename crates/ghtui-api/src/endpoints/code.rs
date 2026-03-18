use base64::Engine;
use ghtui_core::types::code::{
    CommitDetail, CommitEntry, CommitFile, FileChangeStatus, FileEntry, FileEntryType, TreeNode,
};
use ghtui_core::types::common::RepoId;
use std::collections::BTreeSet;

use crate::client::GithubClient;
use crate::error::ApiError;

impl GithubClient {
    pub async fn list_contents(
        &self,
        repo: &RepoId,
        path: &str,
        git_ref: &str,
    ) -> Result<Vec<FileEntry>, ApiError> {
        let clean_path = path.trim_matches('/');
        let path_segment = if clean_path.is_empty() {
            String::new()
        } else {
            format!("/{}", clean_path)
        };

        let api_path = format!(
            "/repos/{}/{}/contents{}?ref={}",
            repo.owner, repo.name, path_segment, git_ref
        );

        let body = self.get(&api_path).await?;
        let response: serde_json::Value = serde_json::from_str(&body)?;

        let Some(items) = response.as_array() else {
            // Single file response — return empty (caller should use get_file_content)
            return Ok(Vec::new());
        };

        let mut entries: Vec<FileEntry> = items
            .iter()
            .filter_map(|item| {
                let name = item["name"].as_str()?.to_string();
                let path = item["path"].as_str()?.to_string();
                let type_str = item["type"].as_str()?;
                let entry_type = match type_str {
                    "dir" => FileEntryType::Dir,
                    _ => FileEntryType::File,
                };
                let size = item["size"].as_u64();
                let sha = item["sha"].as_str().unwrap_or("").to_string();

                Some(FileEntry {
                    name,
                    path,
                    entry_type,
                    size,
                    sha,
                })
            })
            .collect();

        // Sort: dirs first, then files, alphabetical within each group
        entries.sort_by(|a, b| {
            let type_order = |t: &FileEntryType| match t {
                FileEntryType::Dir => 0,
                FileEntryType::File => 1,
            };
            type_order(&a.entry_type)
                .cmp(&type_order(&b.entry_type))
                .then_with(|| a.name.to_lowercase().cmp(&b.name.to_lowercase()))
        });

        Ok(entries)
    }

    /// Fetch the full recursive tree for a repo at a given git ref.
    /// Uses GET /repos/{owner}/{name}/git/trees/{sha}?recursive=1
    /// First resolves the ref to a commit SHA, then fetches the tree SHA.
    pub async fn get_tree_recursive(
        &self,
        repo: &RepoId,
        git_ref: &str,
    ) -> Result<Vec<TreeNode>, ApiError> {
        // 1. Resolve git_ref to a commit → get tree SHA
        let ref_path = format!(
            "/repos/{}/{}/git/ref/heads/{}",
            repo.owner, repo.name, git_ref
        );
        // Try heads first, then tags
        let commit_sha = match self.get(&ref_path).await {
            Ok(body) => {
                let val: serde_json::Value = serde_json::from_str(&body)?;
                val["object"]["sha"].as_str().unwrap_or("").to_string()
            }
            Err(_) => {
                // Try as tag
                let tag_path = format!(
                    "/repos/{}/{}/git/ref/tags/{}",
                    repo.owner, repo.name, git_ref
                );
                match self.get(&tag_path).await {
                    Ok(body) => {
                        let val: serde_json::Value = serde_json::from_str(&body)?;
                        val["object"]["sha"].as_str().unwrap_or("").to_string()
                    }
                    Err(_) => {
                        // Try using the ref directly as a SHA or branch name via commits API
                        let commit_path =
                            format!("/repos/{}/{}/commits/{}", repo.owner, repo.name, git_ref);
                        let body = self.get(&commit_path).await?;
                        let val: serde_json::Value = serde_json::from_str(&body)?;
                        val["sha"].as_str().unwrap_or("").to_string()
                    }
                }
            }
        };

        if commit_sha.is_empty() {
            return Ok(Vec::new());
        }

        // 2. Get the commit to find the tree SHA
        let commit_path = format!(
            "/repos/{}/{}/git/commits/{}",
            repo.owner, repo.name, commit_sha
        );
        let body = self.get(&commit_path).await?;
        let commit_val: serde_json::Value = serde_json::from_str(&body)?;
        let tree_sha = commit_val["tree"]["sha"].as_str().unwrap_or("").to_string();

        if tree_sha.is_empty() {
            return Ok(Vec::new());
        }

        // 3. Fetch recursive tree
        let tree_path = format!(
            "/repos/{}/{}/git/trees/{}?recursive=1",
            repo.owner, repo.name, tree_sha
        );
        let body = self.get(&tree_path).await?;
        let tree_val: serde_json::Value = serde_json::from_str(&body)?;

        let Some(items) = tree_val["tree"].as_array() else {
            return Ok(Vec::new());
        };

        // Collect all directory paths (from "tree" type entries AND implied by file paths)
        let mut dir_paths: BTreeSet<String> = BTreeSet::new();

        // First pass: collect explicit dirs and all entries
        struct RawEntry {
            path: String,
            is_dir: bool,
            size: Option<u64>,
        }

        let mut raw_entries: Vec<RawEntry> = Vec::new();
        for item in items {
            let path = item["path"].as_str().unwrap_or("").to_string();
            let type_str = item["type"].as_str().unwrap_or("");
            let is_dir = type_str == "tree";
            let size = if is_dir { None } else { item["size"].as_u64() };

            if is_dir {
                dir_paths.insert(path.clone());
            }
            // Also register parent dirs implied by this path
            let parts: Vec<&str> = path.split('/').collect();
            for i in 1..parts.len() {
                let parent = parts[..i].join("/");
                dir_paths.insert(parent);
            }

            raw_entries.push(RawEntry { path, is_dir, size });
        }

        // Build tree nodes: dirs from dir_paths, files from raw_entries where !is_dir
        let mut nodes: Vec<TreeNode> = Vec::new();

        // Add directories
        for dir_path in &dir_paths {
            let parts: Vec<&str> = dir_path.split('/').collect();
            let name = parts.last().unwrap_or(&"").to_string();
            let depth = parts.len() - 1;
            nodes.push(TreeNode {
                name,
                path: dir_path.clone(),
                is_dir: true,
                depth,
                expanded: false,
                size: None,
            });
        }

        // Add files
        for entry in &raw_entries {
            if entry.is_dir {
                continue;
            }
            let parts: Vec<&str> = entry.path.split('/').collect();
            let name = parts.last().unwrap_or(&"").to_string();
            let depth = parts.len() - 1;
            nodes.push(TreeNode {
                name,
                path: entry.path.clone(),
                is_dir: false,
                depth,
                expanded: false,
                size: entry.size,
            });
        }

        // Sort: use a sort key that ensures dirs-before-files at each level.
        // For path "src/main.rs" (file): key = "src/\x01main.rs" (dirs get \x00, files get \x01)
        // For path "src" (dir):          key = "\x00src"
        // This naturally sorts dirs before files at each level.
        nodes.sort_by(|a, b| {
            fn sort_key(path: &str, is_dir: bool) -> String {
                let parts: Vec<&str> = path.split('/').collect();
                let mut key_parts: Vec<String> = Vec::new();
                for (i, part) in parts.iter().enumerate() {
                    let is_last = i == parts.len() - 1;
                    let prefix = if is_last && !is_dir { "\x01" } else { "\x00" };
                    key_parts.push(format!("{}{}", prefix, part.to_lowercase()));
                }
                key_parts.join("/")
            }
            sort_key(&a.path, a.is_dir).cmp(&sort_key(&b.path, b.is_dir))
        });

        Ok(nodes)
    }

    pub async fn get_file_content(
        &self,
        repo: &RepoId,
        path: &str,
        git_ref: &str,
    ) -> Result<String, ApiError> {
        let ref_param = if git_ref.is_empty() {
            String::new()
        } else {
            format!("?ref={}", git_ref)
        };
        let api_path = format!(
            "/repos/{}/{}/contents/{}{}",
            repo.owner, repo.name, path, ref_param
        );

        let body = self.get(&api_path).await?;
        let response: serde_json::Value = serde_json::from_str(&body)?;

        let content = response["content"].as_str().unwrap_or("").replace('\n', "");
        let decoded = base64::engine::general_purpose::STANDARD
            .decode(&content)
            .map_err(|e| ApiError::Other(format!("Base64 decode error: {}", e)))?;
        String::from_utf8(decoded).map_err(|e| ApiError::Other(format!("UTF-8 error: {}", e)))
    }

    pub async fn list_branches(&self, repo: &RepoId) -> Result<Vec<String>, ApiError> {
        let api_path = format!("/repos/{}/{}/branches?per_page=100", repo.owner, repo.name);
        let body = self.get(&api_path).await?;
        let items: Vec<serde_json::Value> = serde_json::from_str(&body)?;
        Ok(items
            .iter()
            .filter_map(|item| item["name"].as_str().map(|s| s.to_string()))
            .collect())
    }

    pub async fn list_tags(&self, repo: &RepoId) -> Result<Vec<String>, ApiError> {
        let api_path = format!("/repos/{}/{}/tags?per_page=100", repo.owner, repo.name);
        let body = self.get(&api_path).await?;
        let items: Vec<serde_json::Value> = serde_json::from_str(&body)?;
        Ok(items
            .iter()
            .filter_map(|item| item["name"].as_str().map(|s| s.to_string()))
            .collect())
    }

    pub async fn list_commits(
        &self,
        repo: &RepoId,
        git_ref: &str,
        path: &str,
        per_page: u32,
    ) -> Result<Vec<CommitEntry>, ApiError> {
        let mut api_path = format!(
            "/repos/{}/{}/commits?sha={}&per_page={}",
            repo.owner, repo.name, git_ref, per_page
        );
        if !path.is_empty() {
            api_path.push_str(&format!("&path={}", path));
        }
        let body = self.get(&api_path).await?;
        let items: Vec<serde_json::Value> = serde_json::from_str(&body)?;
        Ok(items
            .iter()
            .filter_map(|item| {
                let sha = item["sha"].as_str()?.to_string();
                let commit = &item["commit"];
                let message = commit["message"]
                    .as_str()
                    .unwrap_or("")
                    .lines()
                    .next()
                    .unwrap_or("")
                    .to_string();
                let author = item["author"]["login"]
                    .as_str()
                    .or_else(|| commit["author"]["name"].as_str())
                    .unwrap_or("unknown")
                    .to_string();
                let date = commit["author"]["date"].as_str().unwrap_or("").to_string();
                Some(CommitEntry {
                    sha,
                    message,
                    author,
                    date,
                })
            })
            .collect())
    }

    /// Update (or create) a file via the GitHub Contents API.
    /// `sha` is the blob SHA of the file being replaced (empty string for new files).
    pub async fn update_file_content(
        &self,
        repo: &RepoId,
        path: &str,
        content: &str,
        message: &str,
        sha: &str,
        branch: &str,
    ) -> Result<(), ApiError> {
        let api_path = format!("/repos/{}/{}/contents/{}", repo.owner, repo.name, path);
        let encoded = base64::engine::general_purpose::STANDARD.encode(content.as_bytes());
        let mut body = serde_json::json!({
            "message": message,
            "content": encoded,
            "branch": branch,
        });
        if !sha.is_empty() {
            body["sha"] = serde_json::Value::String(sha.to_string());
        }
        self.put(&api_path, &body).await?;
        Ok(())
    }

    pub async fn get_commit(&self, repo: &RepoId, sha: &str) -> Result<CommitDetail, ApiError> {
        let api_path = format!("/repos/{}/{}/commits/{}", repo.owner, repo.name, sha);
        let body = self.get(&api_path).await?;
        let item: serde_json::Value = serde_json::from_str(&body)?;

        let commit = &item["commit"];
        let full_sha = item["sha"].as_str().unwrap_or(sha).to_string();
        let message = commit["message"].as_str().unwrap_or("").to_string();
        let author = item["author"]["login"]
            .as_str()
            .or_else(|| commit["author"]["name"].as_str())
            .unwrap_or("unknown")
            .to_string();
        let date = commit["author"]["date"].as_str().unwrap_or("").to_string();
        let stats = &item["stats"];
        let additions = stats["additions"].as_u64().unwrap_or(0);
        let deletions = stats["deletions"].as_u64().unwrap_or(0);

        let files = item["files"]
            .as_array()
            .map(|arr| {
                arr.iter()
                    .filter_map(|f| {
                        Some(CommitFile {
                            filename: f["filename"].as_str()?.to_string(),
                            status: FileChangeStatus::parse(
                                f["status"].as_str().unwrap_or("modified"),
                            ),
                            additions: f["additions"].as_u64().unwrap_or(0),
                            deletions: f["deletions"].as_u64().unwrap_or(0),
                        })
                    })
                    .collect()
            })
            .unwrap_or_default();

        Ok(CommitDetail {
            sha: full_sha,
            message,
            author,
            date,
            additions,
            deletions,
            files,
        })
    }
}
