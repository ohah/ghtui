use crate::types::Notification;

#[derive(Debug)]
pub struct NotificationListState {
    pub items: Vec<Notification>,
    pub selected: usize,
}

impl NotificationListState {
    pub fn new(items: Vec<Notification>) -> Self {
        Self { items, selected: 0 }
    }

    pub fn selected_notification(&self) -> Option<&Notification> {
        self.items.get(self.selected)
    }

    pub fn select_next(&mut self) {
        if self.selected < self.items.len().saturating_sub(1) {
            self.selected += 1;
        }
    }

    pub fn select_prev(&mut self) {
        self.selected = self.selected.saturating_sub(1);
    }
}
