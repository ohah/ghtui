use crate::types::{SearchKind, SearchResultSet};

#[derive(Debug)]
pub struct SearchViewState {
    pub query: String,
    pub kind: SearchKind,
    pub results: Option<SearchResultSet>,
    pub selected: usize,
    pub input_mode: bool,
    pub input_query: String,
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
