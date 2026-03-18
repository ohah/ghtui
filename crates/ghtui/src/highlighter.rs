//! Tree-sitter based syntax highlighting for 29 languages.
//! Unsupported file types render as plain text.

use ratatui::style::{Color, Style};
use ratatui::text::Span;
use std::cell::RefCell;

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
/// Uses Tree-sitter for 29 languages, plain text for others.
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

    // Try Tree-sitter first (by extension, then by filename)
    let ext = filename.rsplit('.').next().unwrap_or("");
    let basename = filename.rsplit('/').next().unwrap_or(filename);
    let lang = get_tree_sitter_language(ext).or_else(|| get_tree_sitter_language_by_name(basename));
    let tokens = if let Some(lang) = lang {
        highlight_with_tree_sitter(content, lang, is_dark, ext)
    } else {
        // No Tree-sitter grammar — render as plain text
        plain_tokens(content, is_dark)
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
        "md" | "markdown" | "mdx" => Some(tree_sitter_md::LANGUAGE.into()),
        "swift" => Some(tree_sitter_swift::LANGUAGE.into()),
        "cs" | "csx" => Some(tree_sitter_c_sharp::LANGUAGE.into()),
        "php" | "phtml" => Some(tree_sitter_php::LANGUAGE_PHP.into()),
        "scala" | "sc" => Some(tree_sitter_scala::LANGUAGE.into()),
        "lua" => Some(tree_sitter_lua::LANGUAGE.into()),
        "zig" => Some(tree_sitter_zig::LANGUAGE.into()),
        "ex" | "exs" => Some(tree_sitter_elixir::LANGUAGE.into()),
        "hcl" | "tf" | "tfvars" => Some(tree_sitter_hcl::LANGUAGE.into()),
        "proto" => Some(tree_sitter_proto::LANGUAGE.into()),
        _ => None,
    }
}

/// Match by full filename (for files without extensions like Dockerfile, Makefile).
fn get_tree_sitter_language_by_name(name: &str) -> Option<tree_sitter::Language> {
    let lower = name.to_lowercase();
    match lower.as_str() {
        "makefile" | "gnumakefile" => Some(tree_sitter_bash::LANGUAGE.into()), // close enough
        _ => None,
    }
}

/// Categorize a Tree-sitter node kind.
enum NodeCategory {
    Keyword,
    Type,
    Identifier,
    StringLit,
    Number,
    Comment,
    Operator,
    Macro,
    Heading,
    Link,
    Default,
}

fn categorize_node(kind: &str) -> NodeCategory {
    match kind {
        // Keywords (all languages)
        "fn" | "let" | "mut" | "pub" | "use" | "mod" | "struct" | "enum" | "impl" | "trait"
        | "type" | "const" | "static" | "async" | "await" | "return" | "if" | "else" | "match"
        | "for" | "while" | "loop" | "break" | "continue" | "where" | "self" | "super"
        | "crate" | "as" | "in" | "ref" | "move" | "unsafe" | "dyn" | "extern" | "true"
        | "false" | "function" | "var" | "class" | "import" | "export" | "from" | "default"
        | "new" | "this" | "def" | "elif" | "except" | "finally" | "try" | "with" | "yield"
        | "lambda" | "pass" | "raise" | "del" | "global" | "nonlocal" | "assert" | "is" | "not"
        | "and" | "or" | "None" | "True" | "False" | "package" | "func" | "defer" | "go"
        | "chan" | "select" | "case" | "switch" | "fallthrough" | "range" | "interface"
        | "void" | "int" | "float" | "double" | "char" | "boolean" | "byte" | "short" | "long"
        | "abstract" | "final" | "synchronized" | "volatile" | "transient" | "native"
        | "throws" | "instanceof" | "extends" | "implements" | "do" | "end" | "then" | "begin"
        | "rescue" | "ensure" | "module" | "require" | "include" | "elsif" | "unless" | "until"
        | "when" | "defined?" => NodeCategory::Keyword,
        // Types
        "type_identifier"
        | "primitive_type"
        | "builtin_type"
        | "scoped_type_identifier"
        | "generic_type"
        | "lifetime" => NodeCategory::Type,
        // Identifiers
        "identifier" | "field_identifier" | "method_identifier" => NodeCategory::Identifier,
        // Strings (+ YAML scalars)
        "string_literal"
        | "string_content"
        | "raw_string_literal"
        | "char_literal"
        | "string"
        | "template_string"
        | "string_fragment"
        | "heredoc_body"
        | "double_quote_scalar"
        | "single_quote_scalar"
        | "block_scalar" => NodeCategory::StringLit,
        // YAML plain scalars (values) and keys
        "plain_scalar" | "flow_node" => NodeCategory::Identifier,
        // YAML booleans/nulls
        "boolean_scalar" | "null_scalar" => NodeCategory::Keyword,
        // YAML numbers
        "integer_scalar" | "float_scalar" => NodeCategory::Number,
        // YAML anchors/aliases/tags
        "anchor_name"
        | "alias_name"
        | "tag"
        | "tag_directive"
        | "yaml_directive"
        | "directive_name"
        | "directive_parameter"
        | "anchor"
        | "alias" => NodeCategory::Macro,
        // Numbers
        "integer_literal" | "float_literal" | "number" | "integer" => NodeCategory::Number,
        // Comments (+ YAML)
        "line_comment" | "block_comment" | "comment" | "comment_content" => NodeCategory::Comment,
        // Markdown headings
        "atx_heading" | "atx_h1_marker" | "atx_h2_marker" | "atx_h3_marker" | "atx_h4_marker"
        | "atx_h5_marker" | "atx_h6_marker" | "heading_content" | "setext_heading" => {
            NodeCategory::Heading
        }
        // Markdown links/images
        "link_destination" | "link_text" | "link_label" | "image_description" | "uri_autolink"
        | "inline_link" => NodeCategory::Link,
        // Markdown emphasis/strong
        "emphasis" | "strong_emphasis" => NodeCategory::Type,
        // Markdown code
        "code_span" | "fenced_code_block" | "code_fence_content" | "info_string" => {
            NodeCategory::StringLit
        }
        // Markdown list markers
        "list_marker_dot"
        | "list_marker_minus"
        | "list_marker_plus"
        | "list_marker_star"
        | "list_marker_parenthesis"
        | "task_list_marker_checked"
        | "task_list_marker_unchecked" => NodeCategory::Operator,
        // Operators / punctuation
        "=" | "+" | "-" | "*" | "/" | "%" | "&" | "|" | "^" | "!" | "<" | ">" | "." | "," | ";"
        | ":" | "::" | "->" | "=>" | ".." | "..." | "?" | "@" | "#" | "~" | "(" | ")" | "["
        | "]" | "{" | "}" => NodeCategory::Operator,
        // Macros
        "macro_invocation" | "attribute_item" | "attribute" | "inner_attribute_item" => {
            NodeCategory::Macro
        }
        _ => NodeCategory::Default,
    }
}

/// Default dark theme colors (GitHub-inspired).
fn default_dark_color(cat: &NodeCategory) -> (u8, u8, u8) {
    match cat {
        NodeCategory::Keyword => (255, 123, 114),    // #ff7b72 red
        NodeCategory::Type => (121, 192, 255),       // #79c0ff light blue
        NodeCategory::Identifier => (210, 168, 255), // #d2a8ff purple
        NodeCategory::StringLit => (165, 214, 255),  // #a5d6ff cyan
        NodeCategory::Number => (121, 192, 255),     // #79c0ff blue
        NodeCategory::Heading => (255, 166, 87),     // orange — headings
        NodeCategory::Link => (88, 166, 255),        // blue — links
        NodeCategory::Comment => (139, 148, 158),    // #8b949e gray
        NodeCategory::Operator => (230, 237, 243),   // white
        NodeCategory::Macro => (121, 184, 255),      // blue
        NodeCategory::Default => (230, 237, 243),    // white
    }
}

/// Default light theme colors.
fn default_light_color(cat: &NodeCategory) -> (u8, u8, u8) {
    match cat {
        NodeCategory::Keyword => (207, 34, 46),     // red
        NodeCategory::Type => (5, 80, 174),         // blue
        NodeCategory::Identifier => (102, 57, 186), // purple
        NodeCategory::StringLit => (10, 104, 71),   // green
        NodeCategory::Number => (5, 80, 174),       // blue
        NodeCategory::Heading => (207, 34, 46),     // red — headings
        NodeCategory::Link => (5, 80, 174),         // blue — links
        NodeCategory::Comment => (110, 119, 129),   // gray
        NodeCategory::Operator => (31, 35, 40),     // dark
        NodeCategory::Macro => (5, 80, 174),        // blue
        NodeCategory::Default => (31, 35, 40),      // dark
    }
}

/// Map node kind to color based on theme and language (file extension).
fn node_color(kind: &str, is_dark: bool, ext: &str) -> (u8, u8, u8) {
    let cat = categorize_node(kind);
    if is_dark {
        match ext {
            // ── Rust ──
            "rs" => match cat {
                NodeCategory::Keyword => (255, 123, 114),    // #ff7b72 red
                NodeCategory::Type => (255, 166, 87),        // #ffa657 orange (NOT blue)
                NodeCategory::Identifier => (210, 168, 255), // #d2a8ff purple
                NodeCategory::Macro => (121, 192, 255),      // #79c0ff blue
                NodeCategory::StringLit => (165, 214, 255),  // #a5d6ff light blue
                NodeCategory::Comment => (139, 148, 158),    // #8b949e gray
                _ => default_dark_color(&cat),
            },
            // ── JavaScript / TypeScript ──
            "js" | "mjs" | "cjs" | "ts" | "tsx" | "jsx" => match kind {
                // JSX tags get green
                "tag_name"
                | "jsx_opening_element"
                | "jsx_closing_element"
                | "jsx_self_closing_element" => (126, 231, 135), // #7ee787 green
                _ => match cat {
                    NodeCategory::Keyword => (255, 123, 114),    // #ff7b72 red
                    NodeCategory::Type => (121, 192, 255),       // #79c0ff blue
                    NodeCategory::Identifier => (210, 168, 255), // #d2a8ff purple
                    NodeCategory::StringLit => (165, 214, 255),  // #a5d6ff light blue
                    NodeCategory::Number => (121, 192, 255),     // #79c0ff blue
                    _ => default_dark_color(&cat),
                },
            },
            // ── Python ──
            "py" | "pyi" => match kind {
                // self/cls as keyword-red
                "self" | "cls" => (255, 123, 114), // #ff7b72 red
                // Builtins
                "print" | "len" | "range" | "type" | "int" | "str" | "float" | "list" | "dict"
                | "set" | "tuple" | "bool" | "enumerate" | "zip" | "map" | "filter" | "sorted"
                | "reversed" | "sum" | "min" | "max" | "abs" | "round" | "isinstance"
                | "issubclass" | "hasattr" | "getattr" | "setattr" | "delattr" | "super"
                | "open" | "input" | "repr" | "iter" | "next" | "any" | "all" | "hex" | "oct"
                | "bin" | "ord" | "chr" => (121, 192, 255), // #79c0ff blue
                // Decorators
                "decorator" | "decorator_identifier" => (210, 168, 255), // #d2a8ff purple
                _ => match cat {
                    NodeCategory::Keyword => (255, 123, 114),   // #ff7b72 red
                    NodeCategory::StringLit => (165, 214, 255), // #a5d6ff light blue
                    _ => default_dark_color(&cat),
                },
            },
            // ── Go ──
            "go" => match kind {
                // Package names
                "package_identifier" => (255, 166, 87), // #ffa657 orange
                _ => match cat {
                    NodeCategory::Keyword => (255, 123, 114),    // #ff7b72 red
                    NodeCategory::Type => (121, 192, 255),       // #79c0ff blue
                    NodeCategory::Identifier => (210, 168, 255), // #d2a8ff purple
                    NodeCategory::StringLit => (165, 214, 255),  // #a5d6ff light blue
                    _ => default_dark_color(&cat),
                },
            },
            // ── Java / Kotlin / C# ──
            "java" | "kt" | "kts" | "cs" | "csx" => match cat {
                NodeCategory::Keyword => (255, 123, 114),   // #ff7b72 red
                NodeCategory::Type => (255, 166, 87),       // #ffa657 orange
                NodeCategory::Macro => (210, 168, 255),     // #d2a8ff purple (annotations)
                NodeCategory::StringLit => (165, 214, 255), // #a5d6ff light blue
                _ => default_dark_color(&cat),
            },
            // ── HTML / CSS ──
            "html" | "htm" => match kind {
                "tag_name" | "start_tag" | "end_tag" => (126, 231, 135), // #7ee787 green
                "attribute_name" => (121, 192, 255),                     // #79c0ff blue
                _ => match cat {
                    NodeCategory::StringLit => (165, 214, 255), // #a5d6ff light blue
                    _ => default_dark_color(&cat),
                },
            },
            "css" | "scss" => match kind {
                "property_name" => (121, 192, 255), // #79c0ff blue
                "tag_name" => (126, 231, 135),      // #7ee787 green
                "class_name" => (210, 168, 255),    // #d2a8ff purple
                "id_name" => (255, 166, 87),        // #ffa657 orange
                _ => match cat {
                    NodeCategory::StringLit => (165, 214, 255), // #a5d6ff light blue
                    NodeCategory::Number => (255, 166, 87),     // #ffa657 orange
                    _ => default_dark_color(&cat),
                },
            },
            // ── YAML / JSON / TOML ──
            "yml" | "yaml" | "json" | "jsonc" | "toml" => match kind {
                // Keys
                "block_mapping_pair" | "flow_pair" | "pair" | "bare_key" | "dotted_key"
                | "property_name" => (121, 192, 255), // #79c0ff blue
                // Booleans / nulls
                "true" | "false" | "null" | "boolean_scalar" | "null_scalar" => {
                    (255, 123, 114) // #ff7b72 red
                }
                _ => match cat {
                    NodeCategory::StringLit => (165, 214, 255), // #a5d6ff light blue
                    NodeCategory::Number => (255, 166, 87),     // #ffa657 orange
                    NodeCategory::Keyword => (255, 123, 114),   // #ff7b72 red
                    _ => default_dark_color(&cat),
                },
            },
            // ── All other languages: default dark palette ──
            _ => default_dark_color(&cat),
        }
    } else {
        // Light theme — no language overrides for now
        default_light_color(&cat)
    }
}

fn highlight_with_tree_sitter(
    content: &str,
    language: tree_sitter::Language,
    is_dark: bool,
    ext: &str,
) -> Vec<Vec<HlToken>> {
    let mut parser = tree_sitter::Parser::new();
    if parser.set_language(&language).is_err() {
        return plain_tokens(content, is_dark);
    }

    let Some(tree) = parser.parse(content, None) else {
        return plain_tokens(content, is_dark);
    };

    let lines: Vec<&str> = content.lines().collect();
    let mut result: Vec<Vec<HlToken>> = lines.iter().map(|_| Vec::new()).collect();

    // Walk the AST and color each leaf node
    let mut cursor = tree.walk();
    walk_tree(&mut cursor, &lines, &mut result, is_dark, ext);

    // Fill gaps: tokens only cover AST nodes, not whitespace/indentation between them.
    // For each line, sort tokens by position and fill gaps with default-colored text.
    let default_color = if is_dark {
        (230, 237, 243)
    } else {
        (31, 35, 40)
    };

    for (i, line) in lines.iter().enumerate() {
        if line.is_empty() {
            continue;
        }
        let tokens = &result[i];
        if tokens.is_empty() {
            result[i] = vec![(
                default_color.0,
                default_color.1,
                default_color.2,
                line.to_string(),
            )];
            continue;
        }

        // Rebuild the line by filling gaps between tokens
        let mut filled: Vec<HlToken> = Vec::new();
        let mut pos = 0usize; // current byte position in line

        for (r, g, b, text) in tokens {
            // Find where this token starts in the line
            if let Some(token_start) = line[pos..].find(text.as_str()) {
                let abs_start = pos + token_start;
                // Fill gap before this token (whitespace/indentation)
                if abs_start > pos {
                    let gap = &line[pos..abs_start];
                    // Convert tabs to spaces for display
                    let gap_display = gap.replace('\t', "    ");
                    filled.push((
                        default_color.0,
                        default_color.1,
                        default_color.2,
                        gap_display,
                    ));
                }
                // Convert tabs in token text too
                let token_display = text.replace('\t', "    ");
                filled.push((*r, *g, *b, token_display));
                pos = abs_start + text.len();
            } else {
                // Token text not found at expected position — just append
                let token_display = text.replace('\t', "    ");
                filled.push((*r, *g, *b, token_display));
            }
        }

        // Fill trailing gap
        if pos < line.len() {
            let trailing = &line[pos..];
            let trailing_display = trailing.replace('\t', "    ");
            filled.push((
                default_color.0,
                default_color.1,
                default_color.2,
                trailing_display,
            ));
        }

        result[i] = filled;
    }

    result
}

fn walk_tree(
    cursor: &mut tree_sitter::TreeCursor,
    lines: &[&str],
    result: &mut Vec<Vec<HlToken>>,
    is_dark: bool,
    ext: &str,
) {
    let node = cursor.node();

    // Only color leaf nodes (no children) to avoid overlapping
    if node.child_count() == 0 {
        let start = node.start_position();
        let end = node.end_position();
        let kind = node.kind();
        let (r, g, b) = node_color(kind, is_dark, ext);

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
            walk_tree(cursor, lines, result, is_dark, ext);
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
