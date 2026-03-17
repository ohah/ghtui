pub mod actions;
pub mod issue;
pub mod notification;
pub mod pr;
pub mod search;
pub mod security;
pub mod settings;

use std::collections::{HashSet, VecDeque};

use crate::config::{AppConfig, GhAccount};
use crate::message::ModalKind;
use crate::router::Route;
use crate::theme::Theme;
use crate::types::common::RepoId;

pub use actions::*;
pub use issue::*;
pub use notification::*;
pub use pr::*;
pub use search::*;
pub use security::*;
pub use settings::*;

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
    pub security: Option<SecurityState>,
    pub settings: Option<SettingsState>,

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
    pub terminal_size: (u16, u16),

    // Account
    pub current_account: Option<GhAccount>,
    pub accounts: Vec<GhAccount>,
    pub account_selected: usize,
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
            security: None,
            settings: None,
            current_repo: repo,
            loading: HashSet::new(),
            toasts: VecDeque::new(),
            modal: None,
            config,
            theme,
            active_tab: 0,
            input_mode: InputMode::Normal,
            input_buffer: String::new(),
            terminal_size: (80, 24),
            current_account,
            accounts,
            account_selected: 0,
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
