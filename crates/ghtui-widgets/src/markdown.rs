use pulldown_cmark::{Event, Options, Parser, Tag, TagEnd};
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span};
use unicode_width::UnicodeWidthStr;

const TEXT_COLOR: Color = Color::Rgb(230, 237, 243);
const HEADING_H1: Color = Color::Rgb(88, 166, 255);
const HEADING_H2: Color = Color::Rgb(63, 185, 80);
const HEADING_H3: Color = Color::Rgb(210, 153, 34);
const LINK_COLOR: Color = Color::Rgb(88, 166, 255);
const CODE_FG: Color = Color::Rgb(230, 237, 243);
const CODE_BG: Color = Color::Rgb(40, 45, 52);
const INLINE_CODE_FG: Color = Color::Rgb(210, 153, 34);
const INLINE_CODE_BG: Color = Color::Rgb(40, 45, 52);
const BLOCKQUOTE_COLOR: Color = Color::Rgb(125, 133, 144);
const BLOCKQUOTE_BAR: Color = Color::Rgb(48, 54, 61);
const LIST_BULLET: Color = Color::Rgb(125, 133, 144);
const RULE_COLOR: Color = Color::Rgb(48, 54, 61);
const TABLE_BORDER: Color = Color::Rgb(48, 54, 61);
const STRIKETHROUGH_COLOR: Color = Color::Rgb(125, 133, 144);

pub fn render_markdown(text: &str) -> Vec<Line<'static>> {
    let opts = Options::ENABLE_TABLES | Options::ENABLE_STRIKETHROUGH | Options::ENABLE_TASKLISTS;
    let parser = Parser::new_ext(text, opts);
    let mut lines: Vec<Line<'static>> = Vec::new();
    let mut current_spans: Vec<Span<'static>> = Vec::new();
    let mut style_stack: Vec<Style> = vec![Style::default().fg(TEXT_COLOR)];
    let mut in_code_block = false;
    let mut list_depth: usize = 0;
    let mut link_url: Option<String> = None;

    // Table state
    let mut in_table = false;
    let mut table_row: Vec<String> = Vec::new();
    let mut table_rows: Vec<Vec<String>> = Vec::new();
    let mut is_header_row = false;

    for event in parser {
        match event {
            Event::Start(tag) => match tag {
                Tag::Heading { level, .. } => {
                    let style = match level {
                        pulldown_cmark::HeadingLevel::H1 => {
                            Style::default().fg(HEADING_H1).add_modifier(Modifier::BOLD)
                        }
                        pulldown_cmark::HeadingLevel::H2 => {
                            Style::default().fg(HEADING_H2).add_modifier(Modifier::BOLD)
                        }
                        _ => Style::default().fg(HEADING_H3).add_modifier(Modifier::BOLD),
                    };
                    style_stack.push(style);
                }
                Tag::Emphasis => {
                    let style = current_style(&style_stack).add_modifier(Modifier::ITALIC);
                    style_stack.push(style);
                }
                Tag::Strong => {
                    let style = current_style(&style_stack)
                        .fg(Color::Rgb(255, 255, 255))
                        .add_modifier(Modifier::BOLD);
                    style_stack.push(style);
                }
                Tag::Strikethrough => {
                    let style = Style::default()
                        .fg(STRIKETHROUGH_COLOR)
                        .add_modifier(Modifier::CROSSED_OUT);
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
                    current_spans.push(Span::styled(
                        format!("{}• ", indent),
                        Style::default().fg(LIST_BULLET),
                    ));
                }
                Tag::Link { dest_url, .. } => {
                    let style = Style::default()
                        .fg(LINK_COLOR)
                        .add_modifier(Modifier::UNDERLINED);
                    style_stack.push(style);
                    link_url = Some(dest_url.to_string());
                }
                Tag::BlockQuote(_) => {
                    let style = Style::default().fg(BLOCKQUOTE_COLOR);
                    style_stack.push(style);
                    current_spans.push(Span::styled("│ ", Style::default().fg(BLOCKQUOTE_BAR)));
                }
                Tag::Image { dest_url, .. } => {
                    current_spans.push(Span::styled(
                        format!("[image: {}]", dest_url),
                        Style::default()
                            .fg(LINK_COLOR)
                            .add_modifier(Modifier::UNDERLINED),
                    ));
                }
                Tag::Table(_) => {
                    in_table = true;
                    table_rows.clear();
                    flush_line(&mut lines, &mut current_spans);
                }
                Tag::TableHead => {
                    // table_header started
                    is_header_row = true;
                    table_row.clear();
                }
                Tag::TableRow => {
                    table_row.clear();
                }
                Tag::TableCell => {}
                _ => {}
            },
            Event::End(tag_end) => match tag_end {
                TagEnd::Heading(_) => {
                    style_stack.pop();
                    flush_line(&mut lines, &mut current_spans);
                    lines.push(Line::raw(""));
                }
                TagEnd::Emphasis | TagEnd::Strong | TagEnd::Strikethrough => {
                    style_stack.pop();
                }
                TagEnd::Link => {
                    style_stack.pop();
                    if let Some(url) = link_url.take() {
                        // Show URL if different from link text
                        let last_text: String = current_spans
                            .last()
                            .map(|s| s.content.to_string())
                            .unwrap_or_default();
                        if !url.is_empty() && url != last_text {
                            current_spans.push(Span::styled(
                                format!(" ({})", url),
                                Style::default().fg(Color::Rgb(125, 133, 144)),
                            ));
                        }
                    }
                }
                TagEnd::BlockQuote(_) => {
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
                TagEnd::Image => {}
                TagEnd::Table => {
                    in_table = false;
                    render_table(&table_rows, &mut lines);
                    table_rows.clear();
                    lines.push(Line::raw(""));
                }
                TagEnd::TableHead => {
                    // table_header ended
                    table_rows.push(table_row.clone());
                    table_row.clear();
                }
                TagEnd::TableRow => {
                    if !is_header_row {
                        table_rows.push(table_row.clone());
                    }
                    is_header_row = false;
                    table_row.clear();
                }
                TagEnd::TableCell => {}
                _ => {}
            },
            Event::Text(text) => {
                if in_table {
                    table_row.push(text.to_string());
                } else if in_code_block {
                    let style = Style::default().fg(CODE_FG).bg(CODE_BG);
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
                if in_table {
                    table_row.push(format!("`{}`", code));
                } else {
                    let style = Style::default().fg(INLINE_CODE_FG).bg(INLINE_CODE_BG);
                    current_spans.push(Span::styled(format!("`{}`", code), style));
                }
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
                    Style::default().fg(RULE_COLOR),
                ));
                lines.push(Line::raw(""));
            }
            Event::TaskListMarker(checked) => {
                let marker = if checked { "☑ " } else { "☐ " };
                let color = if checked { HEADING_H2 } else { LIST_BULLET };
                // Replace the last bullet
                if let Some(last) = current_spans.last_mut()
                    && last.content.ends_with("• ")
                {
                    *last = Span::styled(
                        last.content.replace("• ", marker),
                        Style::default().fg(color),
                    );
                }
            }
            _ => {}
        }
    }

    if !current_spans.is_empty() {
        flush_line(&mut lines, &mut current_spans);
    }

    lines
}

fn render_table(rows: &[Vec<String>], lines: &mut Vec<Line<'static>>) {
    if rows.is_empty() {
        return;
    }

    // Calculate column widths
    let col_count = rows.iter().map(|r| r.len()).max().unwrap_or(0);
    let mut widths = vec![0usize; col_count];
    for row in rows {
        for (i, cell) in row.iter().enumerate() {
            if i < col_count {
                widths[i] = widths[i].max(cell.width());
            }
        }
    }

    let border_style = Style::default().fg(TABLE_BORDER);
    let header_style = Style::default()
        .fg(Color::Rgb(255, 255, 255))
        .add_modifier(Modifier::BOLD);
    let cell_style = Style::default().fg(TEXT_COLOR);

    // Top border
    let top: String = widths
        .iter()
        .map(|w| "─".repeat(w + 2))
        .collect::<Vec<_>>()
        .join("┬");
    lines.push(Line::styled(format!("  ┌{}┐", top), border_style));

    for (ri, row) in rows.iter().enumerate() {
        let mut spans = vec![Span::styled("  │", border_style)];
        for (ci, width) in widths.iter().enumerate() {
            let cell = row.get(ci).map(|s| s.as_str()).unwrap_or("");
            let cell_w = cell.width();
            let pad = width.saturating_sub(cell_w);
            let padded = format!(" {}{} ", cell, " ".repeat(pad));
            let style = if ri == 0 { header_style } else { cell_style };
            spans.push(Span::styled(padded, style));
            spans.push(Span::styled("│", border_style));
        }
        lines.push(Line::from(spans));

        // Separator after header
        if ri == 0 {
            let sep: String = widths
                .iter()
                .map(|w| "─".repeat(w + 2))
                .collect::<Vec<_>>()
                .join("┼");
            lines.push(Line::styled(format!("  ├{}┤", sep), border_style));
        }
    }

    // Bottom border
    let btm: String = widths
        .iter()
        .map(|w| "─".repeat(w + 2))
        .collect::<Vec<_>>()
        .join("┴");
    lines.push(Line::styled(format!("  └{}┘", btm), border_style));
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

    #[test]
    fn test_render_image_as_link() {
        let lines = render_markdown("![alt](https://example.com/img.png)");
        assert!(!lines.is_empty());
    }

    #[test]
    fn test_render_table() {
        let lines = render_markdown("| A | B |\n|---|---|\n| 1 | 2 |");
        assert!(lines.len() >= 3);
    }

    #[test]
    fn test_render_strikethrough() {
        let lines = render_markdown("~~deleted~~");
        assert!(!lines.is_empty());
    }

    #[test]
    fn test_render_task_list() {
        let lines = render_markdown("- [x] done\n- [ ] todo");
        assert!(lines.len() >= 2);
    }

    #[test]
    fn test_render_link_with_url() {
        let lines = render_markdown("[GitHub](https://github.com)");
        assert!(!lines.is_empty());
    }
}
