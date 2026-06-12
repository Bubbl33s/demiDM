use std::collections::HashMap;

use mlua::{Lua, RegistryKey};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[allow(clippy::enum_variant_names)]
pub enum HookName {
    OnStartup,
    OnAuthSuccess,
    OnAuthFailure,
    OnShutdown,
}

impl HookName {
    #[allow(dead_code)]
    pub fn lua_key(&self) -> &'static str {
        match self {
            HookName::OnStartup => "on_startup",
            HookName::OnAuthSuccess => "on_auth_success",
            HookName::OnAuthFailure => "on_auth_failure",
            HookName::OnShutdown => "on_shutdown",
        }
    }
}

pub struct HookRegistry {
    hooks: HashMap<HookName, RegistryKey>,
}

impl HookRegistry {
    pub fn new() -> Self {
        Self {
            hooks: HashMap::new(),
        }
    }

    pub fn register(&mut self, hook: HookName, key: RegistryKey) {
        self.hooks.insert(hook, key);
    }

    pub fn invoke(&self, lua: &Lua, hook: HookName, context: &mlua::Table) -> mlua::Result<()> {
        if let Some(key) = self.hooks.get(&hook) {
            let func: mlua::Function = lua.registry_value(key)?;
            func.call::<_, ()>(context)?;
        }
        Ok(())
    }

    #[allow(dead_code)]
    pub fn has_hook(&self, hook: HookName) -> bool {
        self.hooks.contains_key(&hook)
    }
}

impl Default for HookRegistry {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_hook_name_lua_key() {
        assert_eq!(HookName::OnStartup.lua_key(), "on_startup");
        assert_eq!(HookName::OnAuthSuccess.lua_key(), "on_auth_success");
        assert_eq!(HookName::OnAuthFailure.lua_key(), "on_auth_failure");
        assert_eq!(HookName::OnShutdown.lua_key(), "on_shutdown");
    }

    #[test]
    fn test_hook_registry_new() {
        let registry = HookRegistry::new();
        assert!(!registry.has_hook(HookName::OnStartup));
        assert!(!registry.has_hook(HookName::OnAuthSuccess));
    }
}
