//! Tree-sitter based syntax highlighting with syntect fallback.
//!
//! Uses Tree-sitter AST for 15 major languages, falls back to syntect
//! TextMate grammars for 200+ other languages.

use ratatui::style::{Color, Style};
use ratatui::text::Span;
use std::cell::RefCell;
use std::sync::LazyLock;

/// A highlighted token: (foreground RGB, text)
type HlToken = (u8, u8, u8, String);

/// Cache key: (filename, content_length, line_count)
type CacheKey = (String, usize, usize);

/// Cached highlight result
type CacheEntry = (CacheKey, Vec<Vec<HlToken>>);

thread_local! {
    static HL_CACHE: RefCell<Option<CacheEntry>> = const { RefCell::new(None) };
}

/// Highlight file content. Returns colored spans per line.
/// Uses Tree-sitter for supported languages, syntect for others.
pub fn highlight_file<'a>(content: &str, filename: &str, is_dark: bool) -> Vec<Vec<Span<'a>>> {
    let key = (filename.to_string(), content.len(), content.lines().count());

    // Check cache
    let cached = HL_CACHE.with(|c| {
        let borrow = c.borrow();
        if let Some((ref k, ref data)) = *borrow {
            if k.0 == key.0 && k.1 == key.1 && k.2 == key.2 {
                return Some(tokens_to_spans(data));
            }
        }
        None
    });

    if let Some(result) = cached {
        return result;
    }

    // Try Tree-sitter first
    let ext = filename.rsplit('.').next().unwrap_or("");
    let tokens = if let Some(lang) = get_tree_sitter_language(ext) {
        highlight_with_tree_sitter(content, lang, is_dark)
    } else {
        highlight_with_syntect(content, filename, is_dark)
    };

    // Store in cache
    HL_CACHE.with(|c| {
        *c.borrow_mut() = Some((key, tokens.clone()));
    });

    tokens_to_spans(&tokens)
}

fn tokens_to_spans(tokens: &[Vec<HlToken>]) -> Vec<Vec<Span<'static>>> {
    tokens
        .iter()
        .map(|line| {
            line.iter()
                .map(|(r, g, b, text)| {
                    Span::styled(text.clone(), Style::default().fg(Color::Rgb(*r, *g, *b)))
                })
                .collect()
        })
        .collect()
}

// === Tree-sitter highlighting ===

fn get_tree_sitter_language(ext: &str) -> Option<tree_sitter::Language> {
    match ext {
        "rs" => Some(tree_sitter_rust::LANGUAGE.into()),
        "js" | "mjs" | "cjs" => Some(tree_sitter_javascript::LANGUAGE.into()),
        "ts" => Some(tree_sitter_typescript::LANGUAGE_TYPESCRIPT.into()),
        "tsx" => Some(tree_sitter_typescript::LANGUAGE_TSX.into()),
        "py" | "pyi" => Some(tree_sitter_python::LANGUAGE.into()),
        "go" => Some(tree_sitter_go::LANGUAGE.into()),
        "c" | "h" => Some(tree_sitter_c::LANGUAGE.into()),
        "cpp" | "cc" | "cxx" | "hpp" | "hxx" => Some(tree_sitter_cpp::LANGUAGE.into()),
        "java" => Some(tree_sitter_java::LANGUAGE.into()),
        "rb" | "rake" | "gemspec" => Some(tree_sitter_ruby::LANGUAGE.into()),
        "json" | "jsonc" => Some(tree_sitter_json::LANGUAGE.into()),
        "toml" => Some(tree_sitter_toml_ng::LANGUAGE.into()),
        "html" | "htm" => Some(tree_sitter_html::LANGUAGE.into()),
        "css" | "scss" => Some(tree_sitter_css::LANGUAGE.into()),
        "sh" | "bash" | "zsh" => Some(tree_sitter_bash::LANGUAGE.into()),
        "yml" | "yaml" => Some(tree_sitter_yaml::LANGUAGE.into()),
        _ => None,
    }
}

/// Map Tree-sitter node kind to a color based on theme.
fn node_color(kind: &str, is_dark: bool) -> (u8, u8, u8) {
    if is_dark {
        match kind {
            // Keywords
            "fn" | "let" | "mut" | "pub" | "use" | "mod" | "struct" | "enum" | "impl" | "trait"
            | "type" | "const" | "static" | "async" | "await" | "return" | "if" | "else"
            | "match" | "for" | "while" | "loop" | "break" | "continue" | "where" | "self"
            | "super" | "crate" | "as" | "in" | "ref" | "move" | "unsafe" | "dyn" | "extern"
            | "true" | "false" | "function" | "var" | "class" | "import" | "export" | "from"
            | "default" | "new" | "this" | "def" | "elif" | "except" | "finally" | "try"
            | "with" | "yield" | "lambda" | "pass" | "raise" | "del" | "global" | "nonlocal"
            | "assert" | "is" | "not" | "and" | "or" | "None" | "True" | "False" | "package"
            | "func" | "defer" | "go" | "chan" | "select" | "case" | "switch" | "fallthrough"
            | "range" | "map" | "interface" | "void" | "int" | "float" | "double" | "char"
            | "boolean" | "byte" | "short" | "long" | "abstract" | "final" | "synchronized"
            | "volatile" | "transient" | "native" | "throws" | "instanceof" | "extends"
            | "implements" | "do" | "end" | "then" | "begin" | "rescue" | "ensure" | "module"
            | "require" | "include" | "elsif" | "unless" | "until" | "when" | "defined?" => {
                (255, 123, 114) // red-ish — keywords
            }

            // Types
            "type_identifier"
            | "primitive_type"
            | "builtin_type"
            | "scoped_type_identifier"
            | "generic_type"
            | "lifetime" => {
                (121, 192, 255) // light blue — types
            }

            // Functions
            "identifier" | "field_identifier" | "method_identifier" => {
                (210, 168, 255) // purple — identifiers
            }

            // Strings
            "string_literal" | "string_content" | "raw_string_literal" | "char_literal"
            | "string" | "template_string" | "string_fragment" | "heredoc_body" => {
                (165, 214, 255) // cyan — strings
            }

            // Numbers
            "integer_literal" | "float_literal" | "number" | "integer" => {
                (121, 192, 255) // blue — numbers
            }

            // Comments
            "line_comment" | "block_comment" | "comment" | "comment_content" => {
                (139, 148, 158) // gray — comments
            }

            // Operators / punctuation
            "=" | "+" | "-" | "*" | "/" | "%" | "&" | "|" | "^" | "!" | "<" | ">" | "." | ","
            | ";" | ":" | "::" | "->" | "=>" | ".." | "..." | "?" | "@" | "#" | "~" | "(" | ")"
            | "[" | "]" | "{" | "}" => {
                (230, 237, 243) // white — operators
            }

            // Macros
            "macro_invocation" | "attribute_item" | "attribute" | "inner_attribute_item" => {
                (121, 184, 255) // blue — macros/attributes
            }

            _ => (230, 237, 243), // default white
        }
    } else {
        // Light theme
        match kind {
            "fn" | "let" | "mut" | "pub" | "use" | "mod" | "struct" | "enum" | "impl" | "trait"
            | "type" | "const" | "static" | "async" | "await" | "return" | "if" | "else"
            | "match" | "for" | "while" | "loop" | "break" | "continue" | "function" | "var"
            | "class" | "import" | "export" | "def" | "elif" | "except" | "try" | "with"
            | "true" | "false" | "True" | "False" | "None" | "package" | "func" | "defer"
            | "go" | "do" | "end" | "then" | "begin" | "module" | "require" => {
                (207, 34, 46) // red — keywords
            }
            "type_identifier" | "primitive_type" | "builtin_type" => {
                (5, 80, 174) // blue — types
            }
            "identifier" | "field_identifier" => {
                (102, 57, 186) // purple — identifiers
            }
            "string_literal" | "string_content" | "string" | "template_string" => {
                (10, 104, 71) // green — strings
            }
            "integer_literal" | "float_literal" | "number" => {
                (5, 80, 174) // blue — numbers
            }
            "line_comment" | "block_comment" | "comment" => {
                (110, 119, 129) // gray — comments
            }
            _ => (31, 35, 40), // default dark
        }
    }
}

fn highlight_with_tree_sitter(
    content: &str,
    language: tree_sitter::Language,
    is_dark: bool,
) -> Vec<Vec<HlToken>> {
    let mut parser = tree_sitter::Parser::new();
    if parser.set_language(&language).is_err() {
        return plain_tokens(content, is_dark);
    }

    let Some(tree) = parser.parse(content, None) else {
        return plain_tokens(content, is_dark);
    };

    let source = content.as_bytes();
    let lines: Vec<&str> = content.lines().collect();
    let mut result: Vec<Vec<HlToken>> = lines.iter().map(|_| Vec::new()).collect();

    // Walk the AST and color each leaf node
    let mut cursor = tree.walk();
    walk_tree(&mut cursor, source, &lines, &mut result, is_dark);

    // Fill gaps: any part of a line not covered by a node gets default color
    let default_color = if is_dark {
        (230, 237, 243)
    } else {
        (31, 35, 40)
    };

    for (i, line) in lines.iter().enumerate() {
        if result[i].is_empty() && !line.is_empty() {
            result[i].push((
                default_color.0,
                default_color.1,
                default_color.2,
                line.to_string(),
            ));
        }
    }

    result
}

fn walk_tree(
    cursor: &mut tree_sitter::TreeCursor,
    _source: &[u8],
    lines: &[&str],
    result: &mut Vec<Vec<HlToken>>,
    is_dark: bool,
) {
    let node = cursor.node();

    // Only color leaf nodes (no children) to avoid overlapping
    if node.child_count() == 0 {
        let start = node.start_position();
        let end = node.end_position();
        let kind = node.kind();
        let (r, g, b) = node_color(kind, is_dark);

        // Handle single-line nodes
        if start.row == end.row {
            if let Some(line) = lines.get(start.row) {
                let byte_start = start.column.min(line.len());
                let byte_end = end.column.min(line.len());
                if byte_start < byte_end {
                    let text = &line[byte_start..byte_end];
                    if let Some(tokens) = result.get_mut(start.row) {
                        tokens.push((r, g, b, text.to_string()));
                    }
                }
            }
        } else {
            // Multi-line node (e.g., block comments, strings)
            for row in start.row..=end.row {
                if let Some(line) = lines.get(row) {
                    let col_start = if row == start.row {
                        start.column.min(line.len())
                    } else {
                        0
                    };
                    let col_end = if row == end.row {
                        end.column.min(line.len())
                    } else {
                        line.len()
                    };
                    if col_start < col_end {
                        let text = &line[col_start..col_end];
                        if let Some(tokens) = result.get_mut(row) {
                            tokens.push((r, g, b, text.to_string()));
                        }
                    }
                }
            }
        }
    }

    // Recurse into children
    if cursor.goto_first_child() {
        loop {
            walk_tree(cursor, _source, lines, result, is_dark);
            if !cursor.goto_next_sibling() {
                break;
            }
        }
        cursor.goto_parent();
    }
}

fn plain_tokens(content: &str, is_dark: bool) -> Vec<Vec<HlToken>> {
    let (r, g, b) = if is_dark {
        (230, 237, 243)
    } else {
        (31, 35, 40)
    };
    content
        .lines()
        .map(|line| vec![(r, g, b, line.to_string())])
        .collect()
}

// === syntect fallback ===

fn highlight_with_syntect(content: &str, filename: &str, is_dark: bool) -> Vec<Vec<HlToken>> {
    use syntect::easy::HighlightLines;
    use syntect::highlighting::ThemeSet;
    use syntect::parsing::SyntaxSet;

    static SYNTAX_SET: LazyLock<SyntaxSet> = LazyLock::new(SyntaxSet::load_defaults_newlines);
    static THEME_SET: LazyLock<ThemeSet> = LazyLock::new(ThemeSet::load_defaults);

    let ss = &*SYNTAX_SET;
    let ts = &*THEME_SET;

    let theme_name = if is_dark {
        "base16-ocean.dark"
    } else {
        "base16-ocean.light"
    };
    let Some(syn_theme) = ts.themes.get(theme_name) else {
        return plain_tokens(content, is_dark);
    };

    let ext = filename.rsplit('.').next().unwrap_or("");
    let syntax = ss
        .find_syntax_by_extension(ext)
        .unwrap_or_else(|| ss.find_syntax_plain_text());

    let mut highlighter = HighlightLines::new(syntax, syn_theme);
    let mut result: Vec<Vec<HlToken>> = Vec::new();

    for line in content.lines() {
        let line_nl = format!("{}\n", line);
        match highlighter.highlight_line(&line_nl, ss) {
            Ok(ranges) => {
                let tokens: Vec<HlToken> = ranges
                    .iter()
                    .map(|(style, text)| {
                        let clean = text.trim_end_matches('\n').to_string();
                        (
                            style.foreground.r,
                            style.foreground.g,
                            style.foreground.b,
                            clean,
                        )
                    })
                    .filter(|(_, _, _, text)| !text.is_empty())
                    .collect();
                result.push(tokens);
            }
            Err(_) => {
                let (r, g, b) = if is_dark {
                    (230, 237, 243)
                } else {
                    (31, 35, 40)
                };
                result.push(vec![(r, g, b, line.to_string())]);
            }
        }
    }

    result
}
