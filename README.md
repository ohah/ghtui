# ghtui

A comprehensive GitHub TUI (Terminal User Interface) built in Rust. Browse repositories, manage issues and pull requests, monitor CI/CD workflows, review security alerts, and more — all from your terminal.

## Features

### Code
- File tree browser with directory expansion
- Syntax-highlighted file viewer
- In-terminal file editing with commit support
- Branch/tag switching via ref picker
- Commit history and commit detail view

### Issues
- List with card UI, filters (state/label/author/assignee), search, sort, pagination
- Detail view with section focus navigation (Title, Labels, Assignees, Body, Comments)
- Full CRUD: create, edit title/body, close/reopen
- Label, assignee, and milestone management via picker UI
- Reactions, timeline events, pin/lock/transfer
- Issue templates support

### Pull Requests
- List with filters, search, sort, pagination
- 4-tab detail view: Conversation, Commits, Checks, Files Changed
- Inline editing (title, body, comments) with fullscreen markdown editor
- Approve and Request Changes with review comment editor
- Diff viewer with file tree, side-by-side mode, line selection
- Review comments, suggestions, and thread resolve/unresolve
- Reviewer management, draft toggle, auto-merge, base branch change
- PR creation with template support

### Actions (CI/CD)
- Workflow runs with filters (status, event, workflow), search, pagination
- ANSI color log viewer with step folding
- Cancel, re-run, delete runs
- Artifact download
- Workflow dispatch with input fields
- Environment approval/rejection
- Real-time log streaming

### Security
- Dependabot alerts with dismiss/resolve
- Code scanning alerts
- Secret scanning alerts
- Security advisories

### Insights
- Contributors stats
- Commit activity
- Traffic (clones and views)
- Code frequency
- Forks
- Dependency graph

### Settings
- General settings (description, topics, features toggle)
- Branch protection rules management
- Collaborator management
- Webhook management
- Deploy key management
- Visibility toggle (public/private)

### Extras
- Notifications with mark read, unsubscribe, done, filtering, grouping
- Global search (repos, issues, code)
- Command palette (Ctrl+P)
- GitHub Primer theme (Dark/Light, toggle with `t`)
- Multi-account support (gh CLI hosts.yml)
- Mouse support (click tabs/lists, scroll)
- Markdown renderer (tables, links, checkboxes, code blocks)
- Discussions and Gists views

## Installation

### From source (cargo)

```sh
cargo install --git https://github.com/ohah/ghtui
```

### Homebrew (planned)

```sh
brew install ohah/tap/ghtui
```

### GitHub Releases

Download the latest binary from [Releases](https://github.com/ohah/ghtui/releases).

## Usage

```sh
# Browse a repository
ghtui owner/repo

# Uses gh CLI authentication automatically
# Make sure you are logged in:
gh auth login
```

## Keybindings

### Global

| Key | Action |
|-----|--------|
| `1`-`7` | Switch tabs (Code, Issues, PRs, Actions, Security, Insights, Settings) |
| `Tab` / `Shift+Tab` | Next/previous tab or toggle sidebar/content focus |
| `H` | Go to dashboard (home) |
| `t` | Toggle theme (Dark/Light) |
| `S` | Switch account |
| `Ctrl+P` | Open command palette |
| `/` | Search |
| `?` | Help |
| `q` | Quit |

### Navigation

| Key | Action |
|-----|--------|
| `j` / `k` | Move down/up |
| `Enter` | Open / select |
| `Esc` / `Backspace` | Go back |
| `n` / `p` | Next/previous page |
| `s` | Toggle state filter |
| `o` | Cycle sort / open in browser |

### Editing

| Key | Action |
|-----|--------|
| `e` | Edit focused section |
| `c` | New comment |
| `r` | Reply to comment |
| `d` | Delete comment |
| `l` | Edit labels |
| `a` | Edit assignees |
| `m` | Merge PR |
| `A` | Approve PR |
| `R` | Request changes on PR |
| `Ctrl+S` | Submit editor content |
| `Ctrl+Z` / `Ctrl+Y` | Undo / Redo |
| Arrow keys | Cursor movement in editor |
| `Ctrl+Left` / `Ctrl+Right` | Word movement |

## Configuration

Configuration file location: `~/.config/ghtui/config.toml`

```toml
[general]
theme = "dark"          # "dark" or "light"
default_repo = "owner/repo"

[keybindings]
# Custom keybindings (planned)
```

## Architecture

ghtui is organized as a Cargo workspace with 4 crates:

| Crate | Purpose |
|-------|---------|
| `ghtui` | Main binary — rendering (views), keybindings, update loop |
| `ghtui-core` | State management, messages, types, router, config |
| `ghtui-api` | GitHub REST and GraphQL API client with LRU cache |
| `ghtui-widgets` | Reusable ratatui widgets (DiffView, EditorView, TabBar, Spinner, etc.) |

The application follows the **Elm architecture**:

```
Event → Message → update() → Command → API call → Message → update() → re-render
```

All state lives in a single `AppState` struct. Views are pure functions of state. Side effects are expressed as `Command` values returned from `update()`.

## License

MIT
