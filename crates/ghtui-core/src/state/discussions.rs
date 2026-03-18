use crate::types::Discussion;

#[derive(Debug)]
pub struct DiscussionsState {
    pub items: Vec<Discussion>,
    pub selected: usize,
    pub scroll: usize,
}

impl DiscussionsState {
    pub fn new(items: Vec<Discussion>) -> Self {
        Self {
            items,
            selected: 0,
            scroll: 0,
        }
    }
}
