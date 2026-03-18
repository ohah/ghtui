use crate::types::GistEntry;

#[derive(Debug)]
pub struct GistsState {
    pub items: Vec<GistEntry>,
    pub selected: usize,
    pub scroll: usize,
}

impl GistsState {
    pub fn new(items: Vec<GistEntry>) -> Self {
        Self {
            items,
            selected: 0,
            scroll: 0,
        }
    }
}
