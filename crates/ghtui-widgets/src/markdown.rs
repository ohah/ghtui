use pulldown_cmark::{Event, Parser, Tag, TagEnd};
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span};

pub fn render_markdown(text: &str) -> Vec<Line<'static>> {
    let parser = Parser::new(text);
    let mut lines: Vec<Line<'static>> = Vec::new();
    let mut current_spans: Vec<Span<'static>> = Vec::new();
    let mut style_stack: Vec<Style> = vec![Style::default()];
    let mut in_code_block = false;
    let mut list_depth: usize = 0;

    for event in parser {
        match event {
            Event::Start(tag) => match tag {
                Tag::Heading { level, .. } => {
                    let style = match level {
                        pulldown_cmark::HeadingLevel::H1 => Style::default()
                            .fg(Color::Cyan)
                            .add_modifier(Modifier::BOLD),
                        pulldown_cmark::HeadingLevel::H2 => Style::default()
                            .fg(Color::Green)
                            .add_modifier(Modifier::BOLD),
                        _ => Style::default()
                            .fg(Color::Yellow)
                            .add_modifier(Modifier::BOLD),
                    };
                    style_stack.push(style);
                }
                Tag::Emphasis => {
                    let style = current_style(&style_stack).add_modifier(Modifier::ITALIC);
                    style_stack.push(style);
                }
                Tag::Strong => {
                    let style = current_style(&style_stack).add_modifier(Modifier::BOLD);
                    style_stack.push(style);
                }
                Tag::CodeBlock(_) => {
                    in_code_block = true;
                    flush_line(&mut lines, &mut current_spans);
                }
                Tag::List(_) => {
                    list_depth += 1;
                }
                Tag::Item => {
                    let indent = "  ".repeat(list_depth.saturating_sub(1));
                    current_spans.push(Span::raw(format!("{}• ", indent)));
                }
                Tag::Link { dest_url, .. } => {
                    let style = Style::default()
                        .fg(Color::Blue)
                        .add_modifier(Modifier::UNDERLINED);
                    style_stack.push(style);
                    // Store URL for later
                    let _ = dest_url;
                }
                Tag::BlockQuote(_) => {
                    let style = Style::default().fg(Color::DarkGray);
                    style_stack.push(style);
                    current_spans.push(Span::styled("│ ", Style::default().fg(Color::DarkGray)));
                }
                _ => {}
            },
            Event::End(tag_end) => match tag_end {
                TagEnd::Heading(_) => {
                    style_stack.pop();
                    flush_line(&mut lines, &mut current_spans);
                    lines.push(Line::raw(""));
                }
                TagEnd::Emphasis | TagEnd::Strong | TagEnd::Link | TagEnd::BlockQuote(_) => {
                    style_stack.pop();
                }
                TagEnd::Paragraph => {
                    flush_line(&mut lines, &mut current_spans);
                    lines.push(Line::raw(""));
                }
                TagEnd::CodeBlock => {
                    in_code_block = false;
                    flush_line(&mut lines, &mut current_spans);
                }
                TagEnd::List(_) => {
                    list_depth = list_depth.saturating_sub(1);
                    if list_depth == 0 {
                        lines.push(Line::raw(""));
                    }
                }
                TagEnd::Item => {
                    flush_line(&mut lines, &mut current_spans);
                }
                _ => {}
            },
            Event::Text(text) => {
                if in_code_block {
                    let style = Style::default().fg(Color::White).bg(Color::DarkGray);
                    for code_line in text.lines() {
                        current_spans.push(Span::styled(format!(" {} ", code_line), style));
                        flush_line(&mut lines, &mut current_spans);
                    }
                } else {
                    let style = current_style(&style_stack);
                    current_spans.push(Span::styled(text.to_string(), style));
                }
            }
            Event::Code(code) => {
                let style = Style::default().fg(Color::Yellow).bg(Color::DarkGray);
                current_spans.push(Span::styled(format!("`{}`", code), style));
            }
            Event::SoftBreak => {
                current_spans.push(Span::raw(" "));
            }
            Event::HardBreak => {
                flush_line(&mut lines, &mut current_spans);
            }
            Event::Rule => {
                flush_line(&mut lines, &mut current_spans);
                lines.push(Line::styled(
                    "─".repeat(40),
                    Style::default().fg(Color::DarkGray),
                ));
                lines.push(Line::raw(""));
            }
            _ => {}
        }
    }

    // Flush remaining
    if !current_spans.is_empty() {
        flush_line(&mut lines, &mut current_spans);
    }

    lines
}

fn current_style(stack: &[Style]) -> Style {
    stack.last().copied().unwrap_or_default()
}

fn flush_line(lines: &mut Vec<Line<'static>>, spans: &mut Vec<Span<'static>>) {
    if !spans.is_empty() {
        lines.push(Line::from(std::mem::take(spans)));
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_render_heading() {
        let lines = render_markdown("# Hello World");
        assert!(!lines.is_empty());
    }

    #[test]
    fn test_render_bold() {
        let lines = render_markdown("This is **bold** text");
        assert!(!lines.is_empty());
    }

    #[test]
    fn test_render_code_block() {
        let lines = render_markdown("```\nlet x = 1;\n```");
        assert!(!lines.is_empty());
    }

    #[test]
    fn test_render_list() {
        let lines = render_markdown("- item 1\n- item 2\n- item 3");
        assert!(lines.len() >= 3);
    }

    #[test]
    fn test_render_empty() {
        let lines = render_markdown("");
        assert!(lines.is_empty());
    }
}
