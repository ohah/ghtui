use crossterm::event::{
    self, Event as CrosstermEvent, KeyCode, KeyEvent, KeyEventKind, MouseEvent,
};
use std::time::Duration;
use tokio::sync::mpsc;

#[derive(Debug)]
pub enum Event {
    Key(KeyEvent),
    Mouse(MouseEvent),
    Resize(u16, u16),
    Tick,
}

pub struct EventHandler {
    rx: mpsc::UnboundedReceiver<Event>,
    _tx: mpsc::UnboundedSender<Event>,
}

impl EventHandler {
    pub fn new(tick_rate_ms: u64) -> Self {
        let (tx, rx) = mpsc::unbounded_channel();
        let _tx = tx.clone();

        tokio::spawn(async move {
            let tick_rate = Duration::from_millis(tick_rate_ms);
            loop {
                if event::poll(tick_rate).unwrap_or(false) {
                    match event::read() {
                        Ok(CrosstermEvent::Key(key)) if key.kind == KeyEventKind::Press => {
                            tracing::debug!("Key event: {:?}", key);

                            // Debounce bare Esc: some terminals (Warp, Terminal.app)
                            // split Shift+Arrow escape sequences into Esc + rest.
                            // Wait briefly to see if more data follows.
                            if key.code == KeyCode::Esc
                                && key.modifiers.is_empty()
                                && event::poll(Duration::from_millis(100)).unwrap_or(false)
                            {
                                // More data arrived — this Esc was part of an
                                // escape sequence. Read the actual key and send it.
                                match event::read() {
                                    Ok(CrosstermEvent::Key(real_key))
                                        if real_key.kind == KeyEventKind::Press =>
                                    {
                                        tracing::debug!(
                                            "Esc debounce: discarded Esc, forwarding {:?}",
                                            real_key
                                        );
                                        let _ = tx.send(Event::Key(real_key));
                                    }
                                    Ok(other) => {
                                        tracing::debug!(
                                            "Esc debounce: discarded Esc, got non-key: {:?}",
                                            other
                                        );
                                    }
                                    _ => {}
                                }
                                continue;
                            }

                            if tx.send(Event::Key(key)).is_err() {
                                break;
                            }
                        }
                        Ok(CrosstermEvent::Mouse(mouse)) => {
                            if tx.send(Event::Mouse(mouse)).is_err() {
                                break;
                            }
                        }
                        Ok(CrosstermEvent::Resize(w, h)) => {
                            if tx.send(Event::Resize(w, h)).is_err() {
                                break;
                            }
                        }
                        _ => {}
                    }
                } else if tx.send(Event::Tick).is_err() {
                    break;
                }
            }
        });

        Self { rx, _tx }
    }

    pub async fn next(&mut self) -> Option<Event> {
        self.rx.recv().await
    }
}
