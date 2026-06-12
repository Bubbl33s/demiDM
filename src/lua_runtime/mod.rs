pub mod api;
pub mod config;
pub mod hook_registry;

use std::path::PathBuf;
use std::sync::{Arc, Mutex, RwLock};

use mlua::Lua;
use tokio::sync::mpsc;
use tokio::sync::mpsc::Sender;

use crate::errors::WidgetId;
use crate::events::AppEvent;
use crate::lua_runtime::hook_registry::{HookName, HookRegistry};
use crate::widget::WidgetDef;

#[derive(Debug)]
#[allow(dead_code)]
pub enum LuaCommand {
    LoadConfig { path: PathBuf },
    RunHook { hook: HookName },
    UpdateWidget { widget_id: WidgetId },
}

pub struct LuaRuntimeHandle {
    cmd_tx: mpsc::Sender<LuaCommand>,
}

impl LuaRuntimeHandle {
    #[allow(dead_code)]
    pub fn send_command(&self, cmd: LuaCommand) -> Result<(), mpsc::error::SendError<LuaCommand>> {
        self.cmd_tx.try_send(cmd).map_err(|e| {
            tracing::warn!("Failed to send Lua command: {}", e);
            mpsc::error::SendError(LuaCommand::LoadConfig {
                path: PathBuf::new(),
            })
        })
    }

    #[allow(dead_code)]
    pub async fn send_command_async(
        &self,
        cmd: LuaCommand,
    ) -> Result<(), mpsc::error::SendError<LuaCommand>> {
        self.cmd_tx.send(cmd).await
    }
}

pub fn spawn_lua_runtime(
    config_path: Option<PathBuf>,
    tx: Sender<AppEvent>,
    widget_defs: Arc<RwLock<Vec<WidgetDef>>>,
) -> LuaRuntimeHandle {
    let (cmd_tx, cmd_rx) = mpsc::channel::<LuaCommand>(64);

    std::thread::spawn(move || {
        run(config_path, tx, cmd_rx, widget_defs);
    });

    LuaRuntimeHandle { cmd_tx }
}

fn run(
    config_path: Option<PathBuf>,
    tx: Sender<AppEvent>,
    mut cmd_rx: mpsc::Receiver<LuaCommand>,
    widget_defs: Arc<RwLock<Vec<WidgetDef>>>,
) {
    let lua = Lua::new();

    let hook_registry = Arc::new(Mutex::new(HookRegistry::new()));

    if let Err(e) = api::register_demidm_api(&lua, tx.clone(), hook_registry.clone(), widget_defs) {
        let _ = tx.try_send(AppEvent::ConfigError(format!(
            "Failed to register demidm API: {}",
            e
        )));
    }

    if let Some(path) = config_path {
        load_config(&lua, &path, &tx);
    }

    let _ = tx.try_send(AppEvent::ConfigLoaded);

    while let Some(cmd) = cmd_rx.blocking_recv() {
        match cmd {
            LuaCommand::LoadConfig { path } => {
                load_config(&lua, &path, &tx);
            }
            LuaCommand::RunHook { hook } => {
                invoke_hook(&lua, &hook_registry, hook, &tx);
            }
            LuaCommand::UpdateWidget { widget_id: _ } => {}
        }
    }
}

fn load_config(lua: &Lua, path: &PathBuf, tx: &Sender<AppEvent>) {
    let source = match std::fs::read_to_string(path) {
        Ok(s) => s,
        Err(e) => {
            let _ = tx.try_send(AppEvent::ConfigError(format!(
                "Failed to read config {}: {}",
                path.display(),
                e
            )));
            return;
        }
    };

    match lua.load(&source).exec() {
        Ok(()) => {
            tracing::info!("Config loaded successfully from {}", path.display());
        }
        Err(e) => {
            let _ = tx.try_send(AppEvent::ConfigError(format!(
                "Lua error in {}: {}",
                path.display(),
                e
            )));
        }
    }
}

fn invoke_hook(
    lua: &Lua,
    hook_registry: &Arc<Mutex<HookRegistry>>,
    hook: HookName,
    tx: &Sender<AppEvent>,
) {
    let reg = match hook_registry.lock() {
        Ok(r) => r,
        Err(e) => {
            tracing::error!("Failed to lock hook registry: {}", e);
            return;
        }
    };

    let context = match lua.create_table() {
        Ok(t) => t,
        Err(e) => {
            tracing::error!("Failed to create hook context table: {}", e);
            return;
        }
    };

    match reg.invoke(lua, hook, &context) {
        Ok(()) => {
            tracing::debug!("Hook {:?} invoked successfully", hook);
        }
        Err(e) => {
            tracing::error!("Error invoking hook {:?}: {}", hook, e);
            let _ = tx.try_send(AppEvent::ConfigError(format!(
                "Hook error ({:?}): {}",
                hook, e
            )));
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_lua_runtime_starts_and_loads_empty_script() {
        let (tx, mut rx) = tokio::sync::mpsc::channel::<AppEvent>(16);
        let widget_defs = Arc::new(RwLock::new(Vec::<WidgetDef>::new()));

        let tmp_dir = std::env::temp_dir();
        let config_path = tmp_dir.join("test_empty_init.lua");
        std::fs::write(&config_path, "-- empty config").unwrap();

        let handle = spawn_lua_runtime(Some(config_path.clone()), tx, widget_defs);

        let event = rx.blocking_recv();
        assert!(event.is_some());
        assert!(matches!(event.unwrap(), AppEvent::ConfigLoaded));

        drop(handle);
        let _ = std::fs::remove_file(config_path);
    }
}
