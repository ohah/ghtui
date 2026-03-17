use ratatui::buffer::Buffer;
use ratatui::layout::Rect;
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::Widget;

pub struct TabBar<'a> {
    tabs: &'a [&'a str],
    selected: usize,
}

impl<'a> TabBar<'a> {
    pub fn new(tabs: &'a [&'a str], selected: usize) -> Self {
        Self { tabs, selected }
    }
}

impl Widget for TabBar<'_> {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let spans: Vec<Span> = self
            .tabs
            .iter()
            .enumerate()
            .flat_map(|(i, tab)| {
                let style = if i == self.selected {
                    Style::default()
                        .fg(Color::White)
                        .add_modifier(Modifier::BOLD | Modifier::UNDERLINED)
                } else {
                    Style::default().fg(Color::DarkGray)
                };

                let mut result = vec![Span::styled(format!(" {} ", tab), style)];
                if i < self.tabs.len() - 1 {
                    result.push(Span::styled(" │ ", Style::default().fg(Color::DarkGray)));
                }
                result
            })
            .collect();

        let line = Line::from(spans);
        buf.set_line(area.x, area.y, &line, area.width);
    }
}
