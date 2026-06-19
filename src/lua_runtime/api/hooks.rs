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

#[cfg(test)]
mod tests {
    use super::*;

    fn noop(lua: &Lua) -> mlua::Function<'_> {
        lua.create_function(|_, ()| Ok(())).unwrap()
    }

    #[test]
    fn test_on_startup_registers_in_hook_registry() {
        let lua = Lua::new();
        let registry = Arc::new(Mutex::new(HookRegistry::new()));
        let table = create_hooks_api(&lua, registry.clone()).unwrap();

        let on_startup: mlua::Function = table.get("on_startup").unwrap();
        on_startup.call::<_, ()>(noop(&lua)).unwrap();

        let reg = registry.lock().unwrap();
        assert!(reg.has_hook(HookName::OnStartup));
        assert!(!reg.has_hook(HookName::OnAuthSuccess));
        assert!(!reg.has_hook(HookName::OnAuthFailure));
        assert!(!reg.has_hook(HookName::OnShutdown));
    }

    #[test]
    fn test_on_auth_success_registers_in_hook_registry() {
        let lua = Lua::new();
        let registry = Arc::new(Mutex::new(HookRegistry::new()));
        let table = create_hooks_api(&lua, registry.clone()).unwrap();

        let on_auth_success: mlua::Function = table.get("on_auth_success").unwrap();
        on_auth_success.call::<_, ()>(noop(&lua)).unwrap();

        let reg = registry.lock().unwrap();
        assert!(reg.has_hook(HookName::OnAuthSuccess));
        assert!(!reg.has_hook(HookName::OnStartup));
    }

    #[test]
    fn test_on_auth_failure_registers_in_hook_registry() {
        let lua = Lua::new();
        let registry = Arc::new(Mutex::new(HookRegistry::new()));
        let table = create_hooks_api(&lua, registry.clone()).unwrap();

        let on_auth_failure: mlua::Function = table.get("on_auth_failure").unwrap();
        on_auth_failure.call::<_, ()>(noop(&lua)).unwrap();

        let reg = registry.lock().unwrap();
        assert!(reg.has_hook(HookName::OnAuthFailure));
        assert!(!reg.has_hook(HookName::OnStartup));
    }

    #[test]
    fn test_on_shutdown_registers_in_hook_registry() {
        let lua = Lua::new();
        let registry = Arc::new(Mutex::new(HookRegistry::new()));
        let table = create_hooks_api(&lua, registry.clone()).unwrap();

        let on_shutdown: mlua::Function = table.get("on_shutdown").unwrap();
        on_shutdown.call::<_, ()>(noop(&lua)).unwrap();

        let reg = registry.lock().unwrap();
        assert!(reg.has_hook(HookName::OnShutdown));
        assert!(!reg.has_hook(HookName::OnStartup));
    }

    #[test]
    fn test_all_four_hooks_register_independently() {
        let lua = Lua::new();
        let registry = Arc::new(Mutex::new(HookRegistry::new()));
        let table = create_hooks_api(&lua, registry.clone()).unwrap();

        for key in [
            "on_startup",
            "on_auth_success",
            "on_auth_failure",
            "on_shutdown",
        ] {
            let func: mlua::Function = table.get(key).unwrap();
            func.call::<_, ()>(noop(&lua)).unwrap();
        }

        let reg = registry.lock().unwrap();
        assert!(reg.has_hook(HookName::OnStartup));
        assert!(reg.has_hook(HookName::OnAuthSuccess));
        assert!(reg.has_hook(HookName::OnAuthFailure));
        assert!(reg.has_hook(HookName::OnShutdown));
    }

    // NOTE(wire-post-auth-lifecycle): end-to-end hook firing (a registered hook
    // actually invoking through LuaCommand::RunHook) is covered by
    // `lua_runtime::tests::test_run_hook_fires_registered_on_startup_hook` in
    // `src/lua_runtime/mod.rs`, since that's where `invoke_hook` lives.
}
