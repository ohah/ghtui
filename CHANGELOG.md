# Changelog

[한국어](CHANGELOG_KO.md)

All notable changes to this project will be documented in this file.

## [0.1.0] - 2026-03-19

### Initial Release

A comprehensive GitHub TUI built with Rust and ratatui, covering all major GitHub features.

### Features

#### Tabs
- **Code** — File tree browser, syntax-highlighted file viewer, inline edit + commit, branch/tag switcher, commit history & detail
- **Issues** — List (card UI), detail (section focus), full CRUD, filter/search/sort, labels/assignees/milestones, reactions, timeline, pin/lock/transfer
- **Pull Requests** — 4-tab detail (Conversation/Commits/Checks/Files changed), inline editing, approve/request changes, file tree, diff review comments, suggestions, CI status, timeline, PR creation, reviewer management, draft toggle, auto-merge, side-by-side diff
- **Actions** — Filter/search/pagination, ANSI color logs, step folding, action bar, artifacts, workflow dispatch, environment approvals, live log streaming
- **Security** — Dependabot, Code Scanning, Secret Scanning, Advisories with dismiss/resolve
- **Insights** — Contributors, Commit Activity, Traffic, Code Frequency, Forks, Dependency Graph
- **Settings** — General settings edit, branch protection, collaborators, webhooks, deploy keys, visibility toggle

#### Extra Views
- **Notifications** — Filter by reason/type, repo grouping, unsubscribe, done, mark all read
- **Search** — Code/Issues/Repos search via GitHub Search API, Ctrl+K shortcut
- **Discussions** — GraphQL-based, category/answer display
- **Gists** — Public/secret badge, file listing
- **Organizations** — Member listing

#### Core
- Elm architecture (Message → update → Command → API → Message)
- GitHub REST + GraphQL API client with LRU cache and rate limiting
- GitHub Primer theme (Dark/Light, `t` key toggle)
- Multi-account support (gh CLI hosts.yml, `S` key switcher)
- Mouse support (click tabs/lists, scroll)
- TextEditor with cursor tracking, word movement, undo/redo
- EditorView / InlineEditorView reusable widgets
- Command palette (Ctrl+P)
- Custom keybindings (config.toml)
- Markdown renderer (tables, links, strikethrough, checkboxes, code blocks, images)
- Offline mode (disk cache, 24h TTL, auto fallback)
- Image preview (halfblock/sixel/kitty auto-detection)
- Cross-platform gh CLI config detection (macOS/Linux/Windows)

#### CI/CD
- GitHub Actions CI (check, test, clippy, fmt, MSRV, security audit)
- Multi-platform release builds (macOS/Linux/Windows)
- Dependabot for Cargo and GitHub Actions dependencies
- Homebrew formula

[0.1.0]: https://github.com/ohah/ghtui/releases/tag/v0.1.0
