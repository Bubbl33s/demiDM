mod auth;
mod errors;
mod events;
mod graphics;
mod input;
mod lua_runtime;
mod renderer;
mod state;
mod tty;
mod widget;
mod widget_runner;

use std::fs::File;
use std::io::stdout;
use std::path::PathBuf;
use std::sync::{Arc, RwLock};
use std::time::Duration;

use crossterm::terminal::{self, EnterAlternateScreen, LeaveAlternateScreen};
use crossterm::ExecutableCommand;
use ratatui::backend::CrosstermBackend;
use ratatui::Terminal;
use tokio::sync::mpsc;
use tracing::info;

use crate::events::AppEvent;
use crate::input::poller::run_input_poller;
use crate::renderer::login_box::draw_login_box;
use crate::renderer::widgets::draw_widgets;
use crate::state::app_state::{apply_event, AppPhase, AppState};
use crate::tty::{install_panic_hook, setup_tty};

async fn run_event_loop(
    tty: File,
    mut rx: mpsc::Receiver<AppEvent>,
    tx: mpsc::Sender<AppEvent>,
) -> errors::AuraResult<()> {
    terminal::enable_raw_mode()?;
    let mut stdout_handle = stdout();
    stdout_handle.execute(EnterAlternateScreen)?;
    stdout_handle.execute(crossterm::cursor::Hide)?;

    let backend = CrosstermBackend::new(stdout_handle);
    let mut terminal = Terminal::new(backend)?;

    let mut state = AppState::new();
    state.phase = AppPhase::Idle;

    let tick_tx = tx.clone();
    tokio::spawn(async move {
        loop {
            tokio::time::sleep(Duration::from_millis(16)).await;
            if tick_tx.send(AppEvent::Tick).await.is_err() {
                break;
            }
        }
    });

    while let Some(event) = rx.recv().await {
        if let AppEvent::AuthRequested {
            ref username,
            ref password,
        } = event
        {
            let worker_tx = tx.clone();
            let uname = username.clone();
            let pwd = password.clone();
            std::thread::spawn(move || {
                let req = crate::auth::PamRequest::new(uname, pwd);
                crate::auth::authenticate(req, worker_tx);
            });
        }

        apply_event(&mut state, event, &tx);

        if matches!(state.phase, AppPhase::Shutdown) {
            break;
        }

        terminal.draw(|f| {
            let size = f.size();
            draw_widgets(f, &state);
            draw_login_box(f, &state, size);
        })?;
    }

    terminal::disable_raw_mode()?;
    let mut stdout_handle = stdout();
    stdout_handle.execute(crossterm::cursor::Show)?;
    stdout_handle.execute(LeaveAlternateScreen)?;

    let _ = tty;
    Ok(())
}

fn parse_tty_path() -> String {
    let args: Vec<String> = std::env::args().collect();
    let mut i = 1;
    while i < args.len() {
        if args[i] == "--tty" && i + 1 < args.len() {
            return args[i + 1].clone();
        }
        i += 1;
    }
    "/dev/tty".to_string()
}

#[tokio::main]
async fn main() -> errors::AuraResult<()> {
    let log_dir = std::env::var("DEMIDM_LOG_DIR")
        .map(PathBuf::from)
        .unwrap_or_else(|_| PathBuf::from("/var/log/demidm"));
    std::fs::create_dir_all(&log_dir).ok();
    let file_appender = tracing_appender::rolling::daily(&log_dir, "demidm.log");
    let (non_blocking, _guard) = tracing_appender::non_blocking(file_appender);
    tracing_subscriber::fmt()
        .with_writer(non_blocking)
        .with_ansi(false)
        .init();

    info!("DemiDM starting");

    let tty_path = parse_tty_path();
    info!("Opening TTY: {}", tty_path);

    let tty = File::options()
        .read(true)
        .write(true)
        .open(&tty_path)
        .map_err(|e| {
            errors::AuraError::SessionLaunch(format!("Failed to open {}: {}", tty_path, e))
        })?;

    install_panic_hook(
        tty.try_clone()
            .map_err(|e| errors::AuraError::SessionLaunch(e.to_string()))?,
    );
    let original = setup_tty(&tty)?;

    info!("TTY setup complete");

    let (tx, rx) = mpsc::channel::<AppEvent>(256);

    let poller_tty = tty
        .try_clone()
        .map_err(|e| errors::AuraError::SessionLaunch(e.to_string()))?;
    let poller_tx = tx.clone();
    std::thread::spawn(move || {
        run_input_poller(poller_tty, poller_tx);
    });

    info!("Event loop starting");

    let config_path = lua_runtime::config::resolve_config_path();
    let lua_tx = tx.clone();
    let widget_defs = Arc::new(RwLock::new(Vec::<widget::WidgetDef>::new()));
    let runner_defs = widget_defs.clone();
    let runner_tx = tx.clone();
    tokio::spawn(async move {
        widget_runner::run_widget_runner(runner_defs, runner_tx).await;
    });
    let _lua_handle = lua_runtime::spawn_lua_runtime(config_path, lua_tx, widget_defs);

    run_event_loop(tty, rx, tx).await?;

    let _ = original;
    info!("DemiDM shutting down");

    Ok(())
}
