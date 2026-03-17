use ratatui::buffer::Buffer;
use ratatui::layout::Rect;
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Paragraph, Widget};

pub struct TextInput<'a> {
    text: &'a str,
    placeholder: &'a str,
    focused: bool,
    block: Option<Block<'a>>,
}

impl<'a> TextInput<'a> {
    pub fn new(text: &'a str) -> Self {
        Self {
            text,
            placeholder: "",
            focused: false,
            block: None,
        }
    }

    pub fn placeholder(mut self, placeholder: &'a str) -> Self {
        self.placeholder = placeholder;
        self
    }

    pub fn focused(mut self, focused: bool) -> Self {
        self.focused = focused;
        self
    }

    pub fn block(mut self, block: Block<'a>) -> Self {
        self.block = Some(block);
        self
    }
}

impl Widget for TextInput<'_> {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let display_text = if self.text.is_empty() {
            Line::from(Span::styled(
                self.placeholder,
                Style::default().fg(Color::DarkGray),
            ))
        } else {
            let mut spans = vec![Span::raw(self.text.to_string())];
            if self.focused {
                spans.push(Span::styled(
                    "█",
                    Style::default()
                        .fg(Color::White)
                        .add_modifier(Modifier::SLOW_BLINK),
                ));
            }
            Line::from(spans)
        };

        let style = if self.focused {
            Style::default().fg(Color::White)
        } else {
            Style::default().fg(Color::Gray)
        };

        let paragraph = Paragraph::new(display_text).style(style);
        let paragraph = if let Some(block) = self.block {
            paragraph.block(block)
        } else {
            paragraph
        };

        paragraph.render(area, buf);
    }
}
