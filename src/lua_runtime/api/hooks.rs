use std::sync::{Arc, Mutex};

use mlua::Lua;

use crate::lua_runtime::hook_registry::{HookName, HookRegistry};

pub fn create_hooks_api(
    lua: &Lua,
    hook_registry: Arc<Mutex<HookRegistry>>,
) -> mlua::Result<mlua::Table<'_>> {
    let table = lua.create_table()?;

    let registry_startup = hook_registry.clone();
    let on_startup = lua.create_function(move |lua, func: mlua::Function| {
        let key = lua.create_registry_value(func)?;
        let mut reg = registry_startup
            .lock()
            .map_err(|e| mlua::Error::external(format!("Failed to lock hook registry: {}", e)))?;
        reg.register(HookName::OnStartup, key);
        Ok(())
    })?;
    table.set("on_startup", on_startup)?;

    let registry_success = hook_registry.clone();
    let on_auth_success = lua.create_function(move |lua, func: mlua::Function| {
        let key = lua.create_registry_value(func)?;
        let mut reg = registry_success
            .lock()
            .map_err(|e| mlua::Error::external(format!("Failed to lock hook registry: {}", e)))?;
        reg.register(HookName::OnAuthSuccess, key);
        Ok(())
    })?;
    table.set("on_auth_success", on_auth_success)?;

    let registry_failure = hook_registry.clone();
    let on_auth_failure = lua.create_function(move |lua, func: mlua::Function| {
        let key = lua.create_registry_value(func)?;
        let mut reg = registry_failure
            .lock()
            .map_err(|e| mlua::Error::external(format!("Failed to lock hook registry: {}", e)))?;
        reg.register(HookName::OnAuthFailure, key);
        Ok(())
    })?;
    table.set("on_auth_failure", on_auth_failure)?;

    let registry_shutdown = hook_registry.clone();
    let on_shutdown = lua.create_function(move |lua, func: mlua::Function| {
        let key = lua.create_registry_value(func)?;
        let mut reg = registry_shutdown
            .lock()
            .map_err(|e| mlua::Error::external(format!("Failed to lock hook registry: {}", e)))?;
        reg.register(HookName::OnShutdown, key);
        Ok(())
    })?;
    table.set("on_shutdown", on_shutdown)?;

    Ok(table)
}
