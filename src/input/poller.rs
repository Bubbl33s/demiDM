use std::fs::File;
use std::os::fd::AsRawFd;

use crossterm::event::{self, Event};
use tokio::sync::mpsc::Sender;
use tracing::info;

use crate::events::AppEvent;
use crate::input::keybinds::map_key_event;

pub fn run_input_poller(tty: File, tx: Sender<AppEvent>) {
    let fd = tty.as_raw_fd();
    crossterm::terminal::enable_raw_mode().ok();

    loop {
        if event::poll(std::time::Duration::from_millis(50)).unwrap_or(false) {
            match event::read() {
                Ok(Event::Key(key_event)) => {
                    info!(
                        "Input: key={:?}, modifiers={:?}",
                        key_event.code, key_event.modifiers
                    );
                    if let Some(app_event) = map_key_event(key_event) {
                        if tx.blocking_send(app_event).is_err() {
                            break;
                        }
                    }
                }
                Ok(Event::Resize(w, h)) => {
                    if tx.blocking_send(AppEvent::Resize(w, h)).is_err() {
                        break;
                    }
                }
                Ok(_) => {}
                Err(_) => break,
            }
        }
    }

    let _ = fd;
    crossterm::terminal::disable_raw_mode().ok();
}
