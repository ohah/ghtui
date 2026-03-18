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

#[derive(Debug, Clone, PartialEq)]
pub enum FileChangeStatus {
    Added,
    Modified,
    Removed,
    Renamed,
    Other(String),
}

impl FileChangeStatus {
    pub fn parse(s: &str) -> Self {
        match s {
            "added" => Self::Added,
            "modified" => Self::Modified,
            "removed" => Self::Removed,
            "renamed" => Self::Renamed,
            other => Self::Other(other.to_string()),
        }
    }

    pub fn label(&self) -> &str {
        match self {
            Self::Added => "A",
            Self::Modified => "M",
            Self::Removed => "D",
            Self::Renamed => "R",
            Self::Other(_) => "?",
        }
    }
}

#[derive(Debug, Clone)]
pub struct CommitFile {
    pub filename: String,
    pub status: FileChangeStatus,
    pub additions: u64,
    pub deletions: u64,
}
