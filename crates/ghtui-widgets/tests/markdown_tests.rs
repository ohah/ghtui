use ghtui_widgets::render_markdown;

#[test]
fn test_render_empty_markdown() {
    let lines = render_markdown("");
    assert!(lines.is_empty());
}

#[test]
fn test_render_plain_text() {
    let lines = render_markdown("Hello, world!");
    assert!(!lines.is_empty());
}

#[test]
fn test_render_heading() {
    let lines = render_markdown("# Title\n\nSome text");
    assert!(lines.len() >= 2);
}

#[test]
fn test_render_bold_text() {
    let lines = render_markdown("This is **bold** text");
    assert!(!lines.is_empty());
}

#[test]
fn test_render_italic_text() {
    let lines = render_markdown("This is *italic* text");
    assert!(!lines.is_empty());
}

#[test]
fn test_render_code_block() {
    let lines = render_markdown("```rust\nfn main() {}\n```");
    assert!(!lines.is_empty());
}

#[test]
fn test_render_inline_code() {
    let lines = render_markdown("Use `cargo build` to build");
    assert!(!lines.is_empty());
}

#[test]
fn test_render_bullet_list() {
    let md = "- item 1\n- item 2\n- item 3";
    let lines = render_markdown(md);
    assert!(lines.len() >= 3);
}

#[test]
fn test_render_blockquote() {
    let lines = render_markdown("> This is a quote");
    assert!(!lines.is_empty());
}

#[test]
fn test_render_horizontal_rule() {
    let lines = render_markdown("---");
    assert!(!lines.is_empty());
}

#[test]
fn test_render_link() {
    let lines = render_markdown("[click here](https://example.com)");
    assert!(!lines.is_empty());
}

#[test]
fn test_render_complex_markdown() {
    let md = r#"# Pull Request

## Changes

This PR does the following:

- Adds **new feature**
- Fixes `bug #123`
- Updates *documentation*

### Code

```rust
fn hello() {
    println!("world");
}
```

> Note: this is important

---

See [docs](https://docs.rs) for more info.
"#;

    let lines = render_markdown(md);
    assert!(lines.len() > 10);
}
