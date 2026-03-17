use ghtui_core::state::{Toast, ToastLevel};
use ratatui::buffer::Buffer;
use ratatui::layout::Rect;
use ratatui::style::{Color, Style};
use ratatui::text::Line;
use ratatui::widgets::Widget;

pub struct ToastWidget<'a> {
    toast: &'a Toast,
}

impl<'a> ToastWidget<'a> {
    pub fn new(toast: &'a Toast) -> Self {
        Self { toast }
    }
}

impl Widget for ToastWidget<'_> {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let (prefix, color) = match self.toast.level {
            ToastLevel::Info => ("ℹ", Color::Blue),
            ToastLevel::Success => ("✓", Color::Green),
            ToastLevel::Warning => ("⚠", Color::Yellow),
            ToastLevel::Error => ("✗", Color::Red),
        };

        let style = Style::default().fg(color);
        let line = Line::styled(format!(" {} {} ", prefix, self.toast.message), style);

        buf.set_line(area.x, area.y, &line, area.width);
    }
}
