# CLAUDE.md — Development Conventions for ghtui

## Workspace Structure

```
ghtui/
├── crates/
│   ├── ghtui/          # Binary: views, keybindings, update loop
│   ├── ghtui-core/     # State, messages, types, router, config, editor
│   ├── ghtui-api/      # GitHub REST + GraphQL client, LRU cache, rate limiting
│   └── ghtui-widgets/  # Reusable ratatui widgets (DiffView, EditorView, TabBar, etc.)
├── Cargo.toml          # Workspace root
├── ROADMAP.md          # Feature tracking
└── CLAUDE.md           # This file
```

## Architecture

Elm architecture: **Message -> update() -> Command -> API -> Message**

- All state is in `AppState` (defined in `ghtui-core/src/state/mod.rs`)
- Views are pure functions: `fn render(frame, state, area)` — no side effects
- `update(state, message) -> Vec<Command>` handles all state transitions
- Commands represent side effects (API calls, file I/O, browser open)
- API responses come back as Messages, closing the loop

## Code Style

- **Format**: `cargo fmt` (rustfmt with default settings)
- **Lint**: `cargo clippy -- -D warnings` (treat all warnings as errors)
- **Test**: `cargo test` (run all workspace tests)
- **CI flags**: `RUSTFLAGS=-Dwarnings` in CI to catch all warnings

## PR Workflow

1. Create feature branch from `main`
2. Implement changes
3. Run `cargo fmt && cargo clippy -- -D warnings && cargo test`
4. Create PR with descriptive title and summary
5. Use rebase merge (not merge commit, not squash)

## Key Patterns

### Views
- Views live in `crates/ghtui/src/views/`
- Use `components.rs` for shared UI: `centered_rect()`, loading spinners, sidebar rendering
- Fullscreen editors use `ghtui_widgets::EditorView`
- Inline editors use `ghtui_widgets::InlineEditorView`
- Pickers (label, assignee, milestone) render as overlays with `Clear` widget

### Update Logic
- Main update dispatch is in `crates/ghtui/src/update/mod.rs`
- Shared helpers in `crates/ghtui/src/update/helpers.rs` (action bar mapping, focus helpers)
- Each tab's edit targets are enums (e.g., `PrInlineEditTarget`, `IssueInlineEditTarget`)
- Editor text is retrieved via `detail.editor_text()` on submit

### State
- Per-tab state structs in `crates/ghtui-core/src/state/` (pr.rs, issue.rs, actions.rs, etc.)
- `TextEditor` in `crates/ghtui-core/src/editor.rs` handles all text editing
- Editing state tracked by `edit_target: Option<EditTarget>` pattern

### Messages
- All messages in `crates/ghtui-core/src/message.rs`
- Pattern: `TabAction` for triggering, `TabActionLoaded(data)` for responses
- Edit messages: `TabEditChar`, `TabEditSubmit`, `TabEditCancel`

### Keybindings
- Central keybinding dispatch in `crates/ghtui/src/keybindings.rs`
- Context-aware: different bindings per route and edit mode
- Global keys handled first, then route-specific
