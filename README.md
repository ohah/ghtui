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
- Multi-account support (gh CLI hosts.yml, cross-platform)
- Mouse support (click tabs/lists, scroll)
- Markdown renderer (tables, links, checkboxes, code blocks)
- Open markdown links/images in browser (`o` key with URL picker)
- Diff context expand (`e` key to show surrounding code)
- Update check (startup notification + `--check-update` flag)
- Discussions and Gists views
- Offline mode (disk cache with 24h TTL)

## Installation

### Prerequisites

GitHub authentication is required. Use one of:

```sh
# Option 1: gh CLI (recommended)
gh auth login

# Option 2: Environment variable
export GITHUB_TOKEN=ghp_xxxx
```

### From source (cargo)

```sh
cargo install --git https://github.com/ohah/ghtui
```

### Homebrew

```sh
brew tap ohah/ghtui https://github.com/ohah/ghtui
brew install ghtui
```

### GitHub Releases

Download the latest binary for your platform from [Releases](https://github.com/ohah/ghtui/releases).

| Platform | Binary |
|----------|--------|
| macOS (Apple Silicon) | `ghtui-aarch64-apple-darwin.tar.gz` |
| macOS (Intel) | `ghtui-x86_64-apple-darwin.tar.gz` |
| Linux (x86_64) | `ghtui-x86_64-unknown-linux-gnu.tar.gz` |
| Linux (ARM64) | `ghtui-aarch64-unknown-linux-gnu.tar.gz` |
| Windows | `ghtui-x86_64-pc-windows-msvc.zip` |

## Usage

```sh
# Auto-detect repo from current git directory
ghtui

# Specify a repository
ghtui --repo owner/repo

# Check for updates
ghtui --check-update

# See all options
ghtui --help
```

### Updating

```sh
# Homebrew
brew upgrade ghtui

# Cargo
cargo install ghtui

# Or just re-download from GitHub Releases
```

The app also checks for updates on startup and shows a notification if a newer version is available.

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
| `o` | Open in browser / open markdown link |
| `r` | Refresh current view |

### Editing

| Key | Action |
|-----|--------|
| `e` | Edit focused section |
| `c` | New comment |
| `r` | Reply to comment |
| `d` | Delete comment |
| `l` | Edit labels |
| `a` | Edit assignees |
| `m` | Set milestone |
| `Ctrl+S` | Submit editor content |
| `Ctrl+Z` / `Ctrl+Y` | Undo / Redo |
| Arrow keys | Cursor movement in editor |
| `Ctrl+Left` / `Ctrl+Right` | Word movement |

### PR Detail

| Key | Action |
|-----|--------|
| `Tab` | Switch sub-tabs (Conversation/Commits/Checks/Files changed) |
| `A` | Approve PR |
| `R` | Request changes |
| `M` | Set milestone |
| `D` | Toggle draft |
| `G` | Toggle auto-merge |
| `v` | Add reviewer |
| `b` | Change base branch |
| `x` | Close/reopen PR |

### Files Changed (Diff)

| Key | Action |
|-----|--------|
| `j` / `k` | Move cursor up/down |
| `J` / `K` | Block select (multi-line) |
| `Enter` | Toggle fold / open review comment editor |
| `e` | Expand context (show surrounding code) |
| `s` | Toggle side-by-side diff mode |
| `f` | Toggle file tree panel |
| `V` | Mark file as viewed |
| `z` | Resolve/unresolve review thread |
| `Ctrl+S` | Submit review comment |
| `Ctrl+G` | Insert suggestion template |

### Actions

| Key | Action |
|-----|--------|
| `s` | Cycle status filter |
| `e` | Cycle event filter |
| `w` | Toggle workflow sidebar |
| `d` | Open workflow dispatch |
| `x` | Cancel run |
| `R` | Re-run |
| `F` | Clear filters |

## Configuration

Configuration file location: `~/.config/ghtui/config.toml`

```toml
theme = "dark"              # "dark" or "light"
default_repo = "owner/repo"
offline_cache = true        # Enable disk cache for offline mode

[keybindings]
quit = "q"
help = "?"
theme_toggle = "t"
search = "Ctrl+k"
palette = "Ctrl+p"
home = "H"
account_switch = "S"
nav_down = "j"
nav_up = "k"
# See ? (Help) in-app for full keybinding reference
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
