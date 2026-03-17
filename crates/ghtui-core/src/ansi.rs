use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span};

/// Parse a string containing ANSI escape sequences into ratatui Spans.
pub fn parse_ansi_line(input: &str) -> Line<'static> {
    let mut spans = Vec::new();
    let mut current_style = Style::default();
    let mut chars = input.chars().peekable();
    let mut buf = String::new();

    while let Some(ch) = chars.next() {
        if ch == '\x1b' {
            // Flush buffered text
            if !buf.is_empty() {
                spans.push(Span::styled(std::mem::take(&mut buf), current_style));
            }

            // Parse ESC sequence
            if chars.peek() == Some(&'[') {
                chars.next(); // consume '['
                let mut params = String::new();
                loop {
                    match chars.peek() {
                        Some(&c) if c.is_ascii_digit() || c == ';' => {
                            params.push(c);
                            chars.next();
                        }
                        Some(&c) if c.is_ascii_alphabetic() => {
                            chars.next();
                            if c == 'm' {
                                current_style = apply_sgr(&params, current_style);
                            }
                            // Other escape codes (H, J, K, etc.) are ignored
                            break;
                        }
                        _ => break,
                    }
                }
            }
        } else {
            buf.push(ch);
        }
    }

    // Flush remaining text
    if !buf.is_empty() {
        spans.push(Span::styled(buf, current_style));
    }

    if spans.is_empty() {
        Line::default()
    } else {
        Line::from(spans)
    }
}

/// Apply SGR (Select Graphic Rendition) parameters to a style.
fn apply_sgr(params: &str, base: Style) -> Style {
    let mut style = base;

    if params.is_empty() {
        return Style::default();
    }

    let codes: Vec<u8> = params
        .split(';')
        .filter_map(|s| s.parse::<u8>().ok())
        .collect();

    let mut i = 0;
    while i < codes.len() {
        match codes[i] {
            0 => style = Style::default(),
            1 => style = style.add_modifier(Modifier::BOLD),
            2 => style = style.add_modifier(Modifier::DIM),
            3 => style = style.add_modifier(Modifier::ITALIC),
            4 => style = style.add_modifier(Modifier::UNDERLINED),
            7 => style = style.add_modifier(Modifier::REVERSED),
            22 => {
                style = style
                    .remove_modifier(Modifier::BOLD)
                    .remove_modifier(Modifier::DIM);
            }
            23 => style = style.remove_modifier(Modifier::ITALIC),
            24 => style = style.remove_modifier(Modifier::UNDERLINED),
            27 => style = style.remove_modifier(Modifier::REVERSED),

            // Standard foreground colors (30-37)
            30 => style = style.fg(Color::Black),
            31 => style = style.fg(Color::Red),
            32 => style = style.fg(Color::Green),
            33 => style = style.fg(Color::Yellow),
            34 => style = style.fg(Color::Blue),
            35 => style = style.fg(Color::Magenta),
            36 => style = style.fg(Color::Cyan),
            37 => style = style.fg(Color::White),
            39 => style = style.fg(Color::Reset),

            // Bright foreground colors (90-97)
            90 => style = style.fg(Color::DarkGray),
            91 => style = style.fg(Color::LightRed),
            92 => style = style.fg(Color::LightGreen),
            93 => style = style.fg(Color::LightYellow),
            94 => style = style.fg(Color::LightBlue),
            95 => style = style.fg(Color::LightMagenta),
            96 => style = style.fg(Color::LightCyan),
            97 => style = style.fg(Color::White),

            // Standard background colors (40-47)
            40 => style = style.bg(Color::Black),
            41 => style = style.bg(Color::Red),
            42 => style = style.bg(Color::Green),
            43 => style = style.bg(Color::Yellow),
            44 => style = style.bg(Color::Blue),
            45 => style = style.bg(Color::Magenta),
            46 => style = style.bg(Color::Cyan),
            47 => style = style.bg(Color::White),
            49 => style = style.bg(Color::Reset),

            // 256-color mode: ESC[38;5;Nm or ESC[48;5;Nm
            38 if i + 2 < codes.len() && codes[i + 1] == 5 => {
                style = style.fg(Color::Indexed(codes[i + 2]));
                i += 2;
            }
            48 if i + 2 < codes.len() && codes[i + 1] == 5 => {
                style = style.bg(Color::Indexed(codes[i + 2]));
                i += 2;
            }

            _ => {} // Ignore unknown codes
        }
        i += 1;
    }

    style
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_plain_text() {
        let line = parse_ansi_line("Hello World");
        assert_eq!(line.spans.len(), 1);
        assert_eq!(line.spans[0].content, "Hello World");
    }

    #[test]
    fn test_empty_input() {
        let line = parse_ansi_line("");
        assert!(line.spans.is_empty());
    }

    #[test]
    fn test_bold() {
        let line = parse_ansi_line("\x1b[1mBold\x1b[0m Normal");
        assert_eq!(line.spans.len(), 2);
        assert!(line.spans[0].style.add_modifier.contains(Modifier::BOLD));
        assert_eq!(line.spans[0].content, "Bold");
        assert_eq!(line.spans[1].content, " Normal");
    }

    #[test]
    fn test_colors() {
        let line = parse_ansi_line("\x1b[31mRed\x1b[32mGreen\x1b[0mReset");
        assert_eq!(line.spans.len(), 3);
        assert_eq!(line.spans[0].style.fg, Some(Color::Red));
        assert_eq!(line.spans[1].style.fg, Some(Color::Green));
        assert_eq!(line.spans[2].style.fg, None);
    }

    #[test]
    fn test_256_color() {
        let line = parse_ansi_line("\x1b[38;5;208mOrange\x1b[0m");
        assert_eq!(line.spans.len(), 1);
        assert_eq!(line.spans[0].style.fg, Some(Color::Indexed(208)));
    }

    #[test]
    fn test_combined_attributes() {
        let line = parse_ansi_line("\x1b[1;31mBold Red\x1b[0m");
        assert_eq!(line.spans.len(), 1);
        assert!(line.spans[0].style.add_modifier.contains(Modifier::BOLD));
        assert_eq!(line.spans[0].style.fg, Some(Color::Red));
    }

    #[test]
    fn test_reset_only() {
        let line = parse_ansi_line("\x1b[0m");
        // After reset with no text, should be empty
        assert!(line.spans.is_empty());
    }

    #[test]
    fn test_mixed_text_and_escapes() {
        let line = parse_ansi_line("Start \x1b[33mYellow\x1b[0m End");
        assert_eq!(line.spans.len(), 3);
        assert_eq!(line.spans[0].content, "Start ");
        assert_eq!(line.spans[1].content, "Yellow");
        assert_eq!(line.spans[1].style.fg, Some(Color::Yellow));
        assert_eq!(line.spans[2].content, " End");
    }

    #[test]
    fn test_bright_colors() {
        let line = parse_ansi_line("\x1b[91mBright Red\x1b[0m");
        assert_eq!(line.spans[0].style.fg, Some(Color::LightRed));
    }

    #[test]
    fn test_background_color() {
        let line = parse_ansi_line("\x1b[42mGreen BG\x1b[0m");
        assert_eq!(line.spans[0].style.bg, Some(Color::Green));
    }

    #[test]
    fn test_github_actions_group_marker() {
        // GitHub Actions uses ##[group] markers that may contain ANSI
        let line = parse_ansi_line("\x1b[36;1mRun actions/checkout@v4\x1b[0m");
        assert_eq!(line.spans.len(), 1);
        assert_eq!(line.spans[0].style.fg, Some(Color::Cyan));
        assert!(line.spans[0].style.add_modifier.contains(Modifier::BOLD));
    }
}
