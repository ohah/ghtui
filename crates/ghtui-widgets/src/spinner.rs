use ratatui::style::{Color, Style};
use ratatui::text::Span;

const SPINNER_FRAMES: &[&str] = &["⠋", "⠙", "⠹", "⠸", "⠼", "⠴", "⠦", "⠧", "⠇", "⠏"];

pub struct Spinner {
    tick: usize,
}

impl Spinner {
    pub fn new(tick: usize) -> Self {
        Self { tick }
    }

    pub fn span(&self) -> Span<'static> {
        let frame = SPINNER_FRAMES[self.tick % SPINNER_FRAMES.len()];
        Span::styled(
            format!("{} Loading...", frame),
            Style::default().fg(Color::Cyan),
        )
    }

    pub fn span_with_message(&self, message: &str) -> Span<'static> {
        let frame = SPINNER_FRAMES[self.tick % SPINNER_FRAMES.len()];
        Span::styled(
            format!("{} {}", frame, message),
            Style::default().fg(Color::Cyan),
        )
    }
}
