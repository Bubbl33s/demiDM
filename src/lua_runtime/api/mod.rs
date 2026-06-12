pub mod background;
pub mod hooks;
pub mod login_box;
pub mod theme;
pub mod widgets;

use std::sync::{Arc, RwLock};

use mlua::Lua;
use tokio::sync::mpsc::Sender;

use crate::events::AppEvent;
use crate::lua_runtime::hook_registry::HookRegistry;
use crate::renderer::theme::ThemeConfig;
use crate::widget::WidgetDef;

#[allow(dead_code)]
pub struct ApiContext {
    pub tx: Sender<AppEvent>,
    pub theme: ThemeConfig,
}

pub fn register_demidm_api(
    lua: &Lua,
    tx: Sender<AppEvent>,
    hook_registry: std::sync::Arc<std::sync::Mutex<HookRegistry>>,
    widget_defs: Arc<RwLock<Vec<WidgetDef>>>,
) -> mlua::Result<()> {
    let globals = lua.globals();
    let demidm = lua.create_table()?;

    let theme_table = theme::create_theme_api(lua, tx.clone())?;
    demidm.set("theme", theme_table)?;

    let login_box_table = login_box::create_login_box_api(lua, tx.clone())?;
    demidm.set("login_box", login_box_table)?;

    let hooks_table = hooks::create_hooks_api(lua, hook_registry)?;
    demidm.set("hooks", hooks_table)?;

    let widgets_table = widgets::create_widgets_api(lua, tx.clone(), widget_defs)?;
    demidm.set("widgets", widgets_table)?;

    let background_table = background::create_background_api(lua, tx.clone())?;
    demidm.set("background", background_table)?;

    let log_table = create_log_api(lua)?;
    demidm.set("log", log_table)?;

    let tx_notify = tx.clone();
    let notify_fn = lua.create_function(move |_, msg: String| {
        let _ = tx_notify.try_send(AppEvent::Notification(msg));
        Ok(())
    })?;
    demidm.set("notify", notify_fn)?;

    globals.set("demidm", demidm)?;

    let package: mlua::Table = globals.get("package")?;
    let preload: mlua::Table = package.get("preload")?;
    let demidm_module = lua.create_function(|lua, ()| {
        let globals = lua.globals();
        let demidm: mlua::Value = globals.get("demidm")?;
        Ok(demidm)
    })?;
    preload.set("demidm", demidm_module)?;

    Ok(())
}

fn create_log_api(lua: &Lua) -> mlua::Result<mlua::Table<'_>> {
    let table = lua.create_table()?;

    let info_fn = lua.create_function(|_, msg: String| {
        tracing::info!(target: "demidm::lua", "{}", msg);
        Ok(())
    })?;
    table.set("info", info_fn)?;

    let warn_fn = lua.create_function(|_, msg: String| {
        tracing::warn!(target: "demidm::lua", "{}", msg);
        Ok(())
    })?;
    table.set("warn", warn_fn)?;

    let error_fn = lua.create_function(|_, msg: String| {
        tracing::error!(target: "demidm::lua", "{}", msg);
        Ok(())
    })?;
    table.set("error", error_fn)?;

    Ok(table)
}
