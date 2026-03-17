use crate::types::{SearchKind, SearchResultSet};

#[derive(Debug)]
pub struct SearchViewState {
    pub query: String,
    pub kind: SearchKind,
    pub results: Option<SearchResultSet>,
    pub selected: usize,
}

impl SearchViewState {
    pub fn new(query: String, kind: SearchKind) -> Self {
        Self {
            query,
            kind,
            results: None,
            selected: 0,
        }
    }

    pub fn select_next(&mut self) {
        if let Some(ref results) = self.results {
            if self.selected < results.items.len().saturating_sub(1) {
                self.selected += 1;
            }
        }
    }

    pub fn select_prev(&mut self) {
        self.selected = self.selected.saturating_sub(1);
    }
}
