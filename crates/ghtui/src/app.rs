use anyhow::Result;
use ghtui_api::GithubClient;
use ghtui_core::config::GhAccount;
use ghtui_core::types::common::RepoId;
use ghtui_core::{AppConfig, AppState, Command, Message};
use tokio::sync::mpsc;

use crate::command_executor;
use crate::event::{Event, EventHandler};
use crate::keybindings;
use crate::tui;
use crate::update;
use crate::view;

pub struct App {
    state: AppState,
    client: GithubClient,
    tick: usize,
    /// Last click position and time for double-click detection
    last_click: Option<(u16, u16, std::time::Instant)>,
}

impl App {
    pub fn new(
        config: AppConfig,
        client: GithubClient,
        repo: Option<RepoId>,
        current_account: Option<GhAccount>,
        accounts: Vec<GhAccount>,
    ) -> Self {
        Self {
            state: AppState::new(config.clone(), repo, current_account, accounts),
            client,
            tick: 0,
            last_click: None,
        }
    }

    pub async fn run(&mut self) -> Result<()> {
        let mut terminal = tui::init()?;
        let mut events = EventHandler::new(self.state.config.tick_rate_ms);
        let (msg_tx, mut msg_rx) = mpsc::unbounded_channel::<Message>();

        // Fetch recent repos on startup for dashboard (and repo counts if repo is set)
        {
            let mut initial_cmds = vec![Command::FetchRecentRepos, Command::CheckUpdate];
            if let Some(ref repo) = self.state.current_repo {
                initial_cmds.push(Command::FetchRepoCounts(repo.clone()));
            }
            self.state.loading.insert("recent_repos".to_string());
            for cmd in initial_cmds {
                let client = self.client.clone();
                let tx = msg_tx.clone();
                tokio::spawn(async move {
                    let result = command_executor::execute(&client, cmd).await;
                    let _ = tx.send(result);
                });
            }
        }

        loop {
            // Render
            let tick = self.tick;
            terminal.draw(|frame| {
                view::render(frame, &self.state, tick);
            })?;

            // Wait for event
            let msg = tokio::select! {
                Some(event) = events.next() => {
                    match event {
                        Event::Key(key) => keybindings::handle_key(key, &self.state),
                        Event::Mouse(mouse) => {
                            use crossterm::event::MouseEventKind;
                            match mouse.kind {
                                MouseEventKind::ScrollUp => Some(Message::ScrollUp),
                                MouseEventKind::ScrollDown => Some(Message::ScrollDown),
                                MouseEventKind::Down(crossterm::event::MouseButton::Left) => {
                                    let col = mouse.column;
                                    let row = mouse.row;
                                    let now = std::time::Instant::now();

                                    // Double-click detection (300ms, same position)
                                    if let Some((lc, lr, lt)) = self.last_click {
                                        if lc == col && lr == row && now.duration_since(lt).as_millis() < 300 {
                                            self.last_click = None;
                                            Some(Message::MouseDoubleClick(col, row))
                                        } else {
                                            self.last_click = Some((col, row, now));
                                            Some(Message::MouseClick(col, row))
                                        }
                                    } else {
                                        self.last_click = Some((col, row, now));
                                        Some(Message::MouseClick(col, row))
                                    }
                                }
                                _ => None,
                            }
                        }
                        Event::Resize(w, h) => Some(Message::Resize(w, h)),
                        Event::Tick => {
                            self.tick = self.tick.wrapping_add(1);
                            Some(Message::Tick)
                        }
                    }
                }
                Some(msg) = msg_rx.recv() => Some(msg),
            };

            if let Some(msg) = msg {
                let is_quit = matches!(msg, Message::Quit);
                let commands = update::update(&mut self.state, msg);

                for cmd in commands {
                    if matches!(cmd, Command::Quit) {
                        tui::restore()?;
                        return Ok(());
                    }

                    // Handle account switch inline (needs mutable access to self.client)
                    if let Command::SwitchAccount(ref account) = cmd {
                        match GithubClient::new(account.token.clone()) {
                            Ok(new_client) => {
                                self.client = new_client;
                                let _ = msg_tx.send(Message::AccountSwitched(account.clone()));
                            }
                            Err(e) => {
                                let _ = msg_tx.send(Message::Error(e.into()));
                            }
                        }
                        continue;
                    }

                    let tx = msg_tx.clone();
                    let client = self.client.clone();
                    tokio::spawn(async move {
                        let result = command_executor::execute(&client, cmd).await;
                        let _ = tx.send(result);
                    });
                }

                if is_quit {
                    tui::restore()?;
                    return Ok(());
                }
            }
        }
    }
}
