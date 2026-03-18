#[derive(Debug, Clone)]
pub struct FileEntry {
    pub name: String,
    pub path: String,
    pub entry_type: FileEntryType,
    pub size: Option<u64>,
    pub sha: String,
}

#[derive(Debug, Clone, PartialEq)]
pub enum FileEntryType {
    File,
    Dir,
}

#[derive(Debug, Clone)]
pub struct CommitEntry {
    pub sha: String,
    pub message: String, // first line only
    pub author: String,
    pub date: String,
}

#[derive(Debug, Clone)]
pub struct CommitDetail {
    pub sha: String,
    pub message: String, // full message
    pub author: String,
    pub date: String,
    pub additions: u64,
    pub deletions: u64,
    pub files: Vec<CommitFile>,
}

#[derive(Debug, Clone)]
pub struct CommitFile {
    pub filename: String,
    pub status: String, // "added", "modified", "removed"
    pub additions: u64,
    pub deletions: u64,
}
