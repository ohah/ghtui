pub mod actions;
pub mod code;
pub mod discussions;
pub mod gists;
pub mod insights;
pub mod issue;
pub mod notification;
pub mod org;
pub mod pr;
pub mod search;
pub mod security;
pub mod settings;

use std::collections::{HashSet, VecDeque};

use crate::config::{AppConfig, GhAccount};
use crate::message::{Message, ModalKind};
use crate::router::Route;
use crate::theme::Theme;
use crate::types::common::RepoId;

pub use actions::*;
pub use code::*;
pub use discussions::*;
pub use gists::*;
pub use insights::*;
pub use issue::*;
pub use notification::*;
pub use org::*;
pub use pr::*;
pub use search::*;
pub use security::*;
pub use settings::*;

// --- Keymap Settings ---
#[derive(Debug)]
pub struct KeymapSettingsState {
    pub selected: usize,
    pub capturing: bool,
    /// (category, name, current_key, default_key)
    pub bindings: Vec<(String, String, String, String)>,
}

impl KeymapSettingsState {
    pub fn from_config(config: &crate::config::KeybindingConfig) -> Self {
        let bindings = config
            .all_bindings()
            .into_iter()
            .map(|(cat, name, key, default)| {
                (
                    cat.to_string(),
                    name.to_string(),
                    key.to_string(),
                    default.to_string(),
                )
            })
            .collect();
        Self {
            selected: 0,
            capturing: false,
            bindings,
        }
    }
}

// --- Command Palette ---
#[derive(Debug)]
pub struct CommandPaletteState {
    pub query: String,
    pub items: Vec<PaletteItem>,
    pub filtered: Vec<usize>, // indices into items that match query
    pub selected: usize,      // index into filtered
}

#[derive(Debug)]
pub struct PaletteItem {
    pub label: String,
    pub category: String,
    pub message: Message,
}

impl Default for CommandPaletteState {
    fn default() -> Self {
        Self::new()
    }
}

impl CommandPaletteState {
    pub fn new() -> Self {
        let items = vec![
            PaletteItem {
                label: "Home (Dashboard)".into(),
                category: "Navigate".into(),
                message: Message::GoHome,
            },
            PaletteItem {
                label: "Code".into(),
                category: "Navigate".into(),
                message: Message::GlobalTabSelect(0),
            },
            PaletteItem {
                label: "Issues".into(),
                category: "Navigate".into(),
                message: Message::GlobalTabSelect(1),
            },
            PaletteItem {
                label: "Pull Requests".into(),
                category: "Navigate".into(),
                message: Message::GlobalTabSelect(2),
            },
            PaletteItem {
                label: "Actions".into(),
                category: "Navigate".into(),
                message: Message::GlobalTabSelect(3),
            },
            PaletteItem {
                label: "Security".into(),
                category: "Navigate".into(),
                message: Message::GlobalTabSelect(4),
            },
            PaletteItem {
                label: "Insights".into(),
                category: "Navigate".into(),
                message: Message::GlobalTabSelect(5),
            },
            PaletteItem {
                label: "Settings".into(),
                category: "Navigate".into(),
                message: Message::GlobalTabSelect(6),
            },
            PaletteItem {
                label: "Notifications".into(),
                category: "Navigate".into(),
                message: Message::Navigate(Route::Notifications),
            },
            PaletteItem {
                label: "Search".into(),
                category: "Navigate".into(),
                message: Message::SearchOpen,
            },
            PaletteItem {
                label: "Discussions".into(),
                category: "Navigate".into(),
                message: Message::GlobalTabSelect(8),
            },
            PaletteItem {
                label: "Gists".into(),
                category: "Navigate".into(),
                message: Message::Navigate(Route::Gists),
            },
            PaletteItem {
                label: "Organizations".into(),
                category: "Navigate".into(),
                message: Message::Navigate(Route::Organizations),
            },
            PaletteItem {
                label: "Toggle theme".into(),
                category: "Action".into(),
                message: Message::ToggleTheme,
            },
            PaletteItem {
                label: "Help".into(),
                category: "Action".into(),
                message: Message::ModalOpen(ModalKind::Help),
            },
            PaletteItem {
                label: "Keybindings".into(),
                category: "Action".into(),
                message: Message::KeymapSettingsOpen,
            },
            PaletteItem {
                label: "Quit".into(),
                category: "Action".into(),
                message: Message::Quit,
            },
        ];
        let filtered: Vec<usize> = (0..items.len()).collect();
        Self {
            query: String::new(),
            items,
            filtered,
            selected: 0,
        }
    }

    pub fn filter(&mut self) {
        let q = self.query.to_lowercase();
        self.filtered = self
            .items
            .iter()
            .enumerate()
            .filter(|(_, item)| {
                if q.is_empty() {
                    return true;
                }
                item.label.to_lowercase().contains(&q) || item.category.to_lowercase().contains(&q)
            })
            .map(|(i, _)| i)
            .collect();
        if self.selected >= self.filtered.len() {
            self.selected = 0;
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum InputMode {
    #[default]
    Normal,
    Insert,
}

#[derive(Debug, Clone)]
pub struct Toast {
    pub message: String,
    pub level: ToastLevel,
    pub ttl: u8,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ToastLevel {
    Info,
    Success,
    Warning,
    Error,
}

#[derive(Debug)]
pub struct AppState {
    pub route: Route,
    pub route_history: Vec<Route>,

    // Domain states
    pub pr_list: Option<PrListState>,
    pub pr_detail: Option<PrDetailState>,
    pub issue_list: Option<IssueListState>,
    pub issue_detail: Option<IssueDetailState>,
    pub actions_list: Option<ActionsListState>,
    pub action_detail: Option<ActionDetailState>,
    pub notifications: Option<NotificationListState>,
    pub search: Option<SearchViewState>,
    pub insights: Option<InsightsState>,
    pub security: Option<SecurityState>,
    pub settings: Option<SettingsState>,
    pub code: Option<CodeViewState>,
    pub discussions: Option<DiscussionsState>,
    pub gists: Option<GistsState>,
    pub org: Option<OrgState>,

    // Multi-repo dashboard
    pub recent_repos: Vec<crate::types::settings::Repository>,
    pub dashboard_selected: usize,

    // Repo tab counts (open issues/PRs)
    pub open_issue_count: Option<u32>,
    pub open_pr_count: Option<u32>,

    // Cross-cutting
    pub current_repo: Option<RepoId>,
    pub loading: HashSet<String>,
    pub toasts: VecDeque<Toast>,
    pub modal: Option<ModalKind>,
    pub config: AppConfig,
    pub theme: Theme,
    pub active_tab: usize,
    pub input_mode: InputMode,
    pub input_buffer: String,
    pub modal_editor: crate::editor::TextEditor,
    pub terminal_size: (u16, u16),

    // Account
    pub current_account: Option<GhAccount>,
    pub accounts: Vec<GhAccount>,
    pub account_selected: usize,

    // Command palette
    pub command_palette: Option<CommandPaletteState>,

    // Keymap settings
    pub keymap_settings: Option<KeymapSettingsState>,
}

impl AppState {
    pub fn new(
        config: AppConfig,
        repo: Option<RepoId>,
        current_account: Option<GhAccount>,
        accounts: Vec<GhAccount>,
    ) -> Self {
        let theme = Theme::from_mode(config.theme);
        Self {
            route: Route::Dashboard,
            route_history: Vec::new(),
            pr_list: None,
            pr_detail: None,
            issue_list: None,
            issue_detail: None,
            actions_list: None,
            action_detail: None,
            notifications: None,
            search: None,
            insights: None,
            security: None,
            settings: None,
            code: None,
            discussions: None,
            gists: None,
            org: None,
            recent_repos: Vec::new(),
            dashboard_selected: 0,
            open_issue_count: None,
            open_pr_count: None,
            current_repo: repo,
            loading: HashSet::new(),
            toasts: VecDeque::new(),
            modal: None,
            config,
            theme,
            active_tab: 0,
            input_mode: InputMode::Normal,
            input_buffer: String::new(),
            modal_editor: crate::editor::TextEditor::new(),
            terminal_size: (80, 24),
            current_account,
            accounts,
            account_selected: 0,
            command_palette: None,
            keymap_settings: None,
        }
    }

    pub fn toggle_theme(&mut self) {
        self.theme = match self.theme.mode {
            crate::theme::ThemeMode::Dark => Theme::light(),
            crate::theme::ThemeMode::Light => Theme::dark(),
        };
    }

    pub fn is_loading(&self, key: &str) -> bool {
        self.loading.contains(key)
    }

    pub fn navigate(&mut self, route: Route) {
        let old = std::mem::replace(&mut self.route, route);
        self.route_history.push(old);
    }

    pub fn go_back(&mut self) -> bool {
        if let Some(route) = self.route_history.pop() {
            self.route = route;
            true
        } else {
            false
        }
    }

    /// Clear all repo-specific view state. Does NOT clear current_repo
    /// so tab navigation still works after going home.
    pub fn reset_repo_state(&mut self) {
        self.code = None;
        self.pr_list = None;
        self.pr_detail = None;
        self.issue_list = None;
        self.issue_detail = None;
        self.actions_list = None;
        self.action_detail = None;
        self.security = None;
        self.insights = None;
        self.settings = None;
    }

    pub fn push_toast(&mut self, message: String, level: ToastLevel) {
        self.toasts.push_back(Toast {
            message,
            level,
            ttl: 5,
        });
    }

    pub fn tick_toasts(&mut self) {
        for toast in self.toasts.iter_mut() {
            toast.ttl = toast.ttl.saturating_sub(1);
        }
        self.toasts.retain(|t| t.ttl > 0);
    }
}
