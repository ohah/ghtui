use crate::types::{SearchKind, SearchResultSet};

const MAX_SEARCH_HISTORY: usize = 20;

#[derive(Debug)]
pub struct SearchViewState {
    pub query: String,
    pub kind: SearchKind,
    pub results: Option<SearchResultSet>,
    pub selected: usize,
    pub input_mode: bool,
    pub input_query: String,
    /// Recent search queries (newest first, max 20).
    pub history: Vec<String>,
    /// When browsing history in input mode, the current history index.
    pub history_cursor: Option<usize>,
}

impl SearchViewState {
    pub fn new(query: String, kind: SearchKind) -> Self {
        let input_mode = query.is_empty();
        Self {
            input_query: query.clone(),
            query,
            kind,
            results: None,
            selected: 0,
            input_mode,
            history: Vec::new(),
            history_cursor: None,
        }
    }

    /// Add a query to search history (deduplicates, newest first).
    pub fn push_history(&mut self, query: &str) {
        if query.is_empty() {
            return;
        }
        // Remove duplicate if exists
        self.history.retain(|h| h != query);
        // Insert at front
        self.history.insert(0, query.to_string());
        // Cap size
        self.history.truncate(MAX_SEARCH_HISTORY);
    }

    /// Navigate to previous history entry (Up arrow in input mode).
    pub fn history_prev(&mut self) {
        if self.history.is_empty() {
            return;
        }
        let next = match self.history_cursor {
            None => 0,
            Some(i) if i + 1 < self.history.len() => i + 1,
            Some(i) => i,
        };
        self.history_cursor = Some(next);
        self.input_query = self.history[next].clone();
    }

    /// Navigate to next (more recent) history entry (Down arrow in input mode).
    pub fn history_next(&mut self) {
        match self.history_cursor {
            Some(0) => {
                self.history_cursor = None;
                self.input_query.clear();
            }
            Some(i) => {
                self.history_cursor = Some(i - 1);
                self.input_query = self.history[i - 1].clone();
            }
            None => {}
        }
    }

    pub fn select_next(&mut self) {
        if let Some(ref results) = self.results
            && self.selected < results.items.len().saturating_sub(1)
        {
            self.selected += 1;
        }
    }

    pub fn select_prev(&mut self) {
        self.selected = self.selected.saturating_sub(1);
    }

    pub fn cycle_kind(&mut self) {
        self.kind = match self.kind {
            SearchKind::Repos => SearchKind::Issues,
            SearchKind::Issues => SearchKind::Code,
            SearchKind::Code => SearchKind::Repos,
        };
    }

    pub fn kind_display(&self) -> &str {
        match self.kind {
            SearchKind::Repos => "Repositories",
            SearchKind::Issues => "Issues & PRs",
            SearchKind::Code => "Code",
        }
    }
}
