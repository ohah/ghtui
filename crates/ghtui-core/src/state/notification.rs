use crate::types::{Notification, NotificationFilters};

#[derive(Debug)]
pub struct NotificationListState {
    pub items: Vec<Notification>,
    pub selected: usize,
    pub filters: NotificationFilters,
    pub grouped: bool,
}

impl NotificationListState {
    pub fn new(items: Vec<Notification>) -> Self {
        Self {
            items,
            selected: 0,
            filters: NotificationFilters::default(),
            grouped: false,
        }
    }

    pub fn selected_notification(&self) -> Option<&Notification> {
        let filtered = self.filtered_items();
        filtered.get(self.selected).copied()
    }

    pub fn select_next(&mut self) {
        let max = self.filtered_items().len().saturating_sub(1);
        if self.selected < max {
            self.selected += 1;
        }
    }

    pub fn select_prev(&mut self) {
        self.selected = self.selected.saturating_sub(1);
    }

    /// Return filtered items based on active filters.
    pub fn filtered_items(&self) -> Vec<&Notification> {
        self.items
            .iter()
            .filter(|n| {
                if let Some(ref reason) = self.filters.reason
                    && n.reason != *reason
                {
                    return false;
                }
                if let Some(ref stype) = self.filters.subject_type
                    && n.subject.subject_type != *stype
                {
                    return false;
                }
                true
            })
            .collect()
    }

    /// Get unique repo names for grouping.
    pub fn repo_groups(&self) -> Vec<String> {
        let mut repos: Vec<String> = self
            .filtered_items()
            .iter()
            .map(|n| n.repository.full_name.clone())
            .collect();
        repos.sort();
        repos.dedup();
        repos
    }

    pub fn cycle_reason(&mut self) {
        self.filters.cycle_reason();
        self.selected = 0;
    }

    pub fn cycle_type(&mut self) {
        self.filters.cycle_type();
        self.selected = 0;
    }

    pub fn toggle_grouped(&mut self) {
        self.grouped = !self.grouped;
    }
}
