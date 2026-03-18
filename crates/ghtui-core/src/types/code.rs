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
