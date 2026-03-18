#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SearchKind {
    Repos,
    Issues,
    Code,
}

#[derive(Debug, Clone)]
pub struct SearchResultSet {
    pub kind: SearchKind,
    pub total_count: u32,
    pub items: Vec<SearchResultItem>,
}

#[derive(Debug, Clone)]
pub enum SearchResultItem {
    Repo {
        full_name: String,
        description: Option<String>,
        stars: u32,
        language: Option<String>,
    },
    Issue {
        repo: String,
        number: u64,
        title: String,
        state: String,
        is_pr: bool,
        labels: Vec<super::common::Label>,
        created_at: Option<chrono::DateTime<chrono::Utc>>,
        user: String,
    },
    Code {
        repo: String,
        path: String,
        fragment: String,
    },
}
