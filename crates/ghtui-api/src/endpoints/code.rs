use base64::Engine;
use ghtui_core::types::code::{FileEntry, FileEntryType};
use ghtui_core::types::common::RepoId;

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
}
