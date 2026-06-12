use std::sync::{Arc, RwLock};

use mlua::Lua;
use tokio::sync::mpsc::Sender;

use crate::errors::{AbsolutePosition, WidgetId};
use crate::events::AppEvent;
use crate::renderer::theme::BorderStyle;
use crate::widget::{WidgetDef, WidgetSource, WidgetStyle};

struct LuaWidgetSource(WidgetSource);

impl mlua::UserData for LuaWidgetSource {}

struct LuaWidgetHandle(WidgetDef);

impl mlua::UserData for LuaWidgetHandle {}

pub fn create_widgets_api(
    lua: &Lua,
    tx: Sender<AppEvent>,
    widget_defs: Arc<RwLock<Vec<WidgetDef>>>,
) -> mlua::Result<mlua::Table<'_>> {
    let table = lua.create_table()?;

    let cmd_fn = lua.create_function(|lua, cmd: String| {
        let source = WidgetSource::ShellCmd(cmd);
        lua.create_userdata(LuaWidgetSource(source))
    })?;
    table.set("cmd", cmd_fn)?;

    let lua_fn_fn = lua.create_function(|lua, func: mlua::Function| {
        let key = lua.create_registry_value(func)?;
        let source = WidgetSource::LuaFn(key);
        lua.create_userdata(LuaWidgetSource(source))
    })?;
    table.set("lua_fn", lua_fn_fn)?;

    let static_text_fn = lua.create_function(|lua, text: String| {
        let source = WidgetSource::StaticText(text);
        lua.create_userdata(LuaWidgetSource(source))
    })?;
    table.set("static_text", static_text_fn)?;

    let create_fn = lua.create_function(|lua, opts: mlua::Table| {
        let id: String = opts.get("id")?;
        let width: u16 = opts.get("width")?;

        let position = if let Ok(pos) = opts.get::<_, mlua::Table>("position") {
            let x: u16 = pos.get("x")?;
            let y: u16 = pos.get("y")?;
            AbsolutePosition { col: x, row: y }
        } else {
            AbsolutePosition { col: 0, row: 0 }
        };

        let height: Option<u16> = opts.get("height").ok();

        let refresh_ms: u64 = opts.get::<_, Option<u64>>("refresh")?.unwrap_or(5000);
        let refresh = std::time::Duration::from_millis(refresh_ms);

        let source_ud = opts.get::<_, mlua::AnyUserData>("source")?;
        let source_ref = source_ud.borrow::<LuaWidgetSource>()?;
        let source = match &source_ref.0 {
            WidgetSource::ShellCmd(cmd) => WidgetSource::ShellCmd(cmd.clone()),
            WidgetSource::StaticText(text) => WidgetSource::StaticText(text.clone()),
            WidgetSource::LuaFn(key) => {
                let func: mlua::Function = lua.registry_value(key)?;
                let new_key = lua.create_registry_value(func)?;
                WidgetSource::LuaFn(new_key)
            }
        };

        let style = if let Ok(style_table) = opts.get::<_, mlua::Table>("style") {
            parse_widget_style(lua, &style_table)?
        } else {
            WidgetStyle::default()
        };

        let def = WidgetDef {
            id,
            position,
            width,
            height,
            refresh,
            source,
            style,
        };

        lua.create_userdata(LuaWidgetHandle(def))
    })?;
    table.set("create", create_fn)?;

    let register_tx = tx.clone();
    let register_defs = widget_defs.clone();
    let register_fn = lua.create_function(move |_, handle: mlua::AnyUserData| {
        let handle_ref = handle.borrow::<LuaWidgetHandle>()?;
        let def = handle_ref.0.clone();

        if let Ok(mut defs) = register_defs.write() {
            if !defs.iter().any(|d| d.id == def.id) {
                defs.push(def.clone());
            }
        }

        let _ = register_tx.try_send(AppEvent::WidgetRegister(def));
        Ok(())
    })?;
    table.set("register", register_fn)?;

    let unregister_tx = tx.clone();
    let unregister_defs = widget_defs.clone();
    let unregister_fn = lua.create_function(move |_, id: WidgetId| {
        if let Ok(mut defs) = unregister_defs.write() {
            defs.retain(|d| d.id != id);
        }

        let _ = unregister_tx.try_send(AppEvent::WidgetRemove(id));
        Ok(())
    })?;
    table.set("unregister", unregister_fn)?;

    let update_tx = tx;
    let update_fn = lua.create_function(move |_, (id, content): (WidgetId, String)| {
        let _ = update_tx.try_send(AppEvent::WidgetUpdate { id, content });
        Ok(())
    })?;
    table.set("update", update_fn)?;

    Ok(table)
}

fn parse_widget_style(_lua: &Lua, style_table: &mlua::Table) -> mlua::Result<WidgetStyle> {
    let mut style = WidgetStyle::default();

    if let Ok(border_str) = style_table.get::<_, String>("border") {
        if let Some(bs) = BorderStyle::from_str(&border_str) {
            style.border = bs;
        }
    }

    if let Ok(fg_str) = style_table.get::<_, String>("fg") {
        if let Some(color) = crate::renderer::theme::parse_hex_color(&fg_str) {
            style.fg = color;
        }
    }

    if let Ok(bg_str) = style_table.get::<_, String>("bg") {
        if let Some(color) = crate::renderer::theme::parse_hex_color(&bg_str) {
            style.bg = color;
        }
    }

    Ok(style)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_create_widget_from_lua() {
        let lua = Lua::new();
        let (tx, _rx) = tokio::sync::mpsc::channel(16);
        let widget_defs = Arc::new(RwLock::new(Vec::<WidgetDef>::new()));

        let api = create_widgets_api(&lua, tx, widget_defs.clone()).unwrap();
        lua.globals().set("demidm_widgets", api).unwrap();

        lua.load(
            r#"
            local source = demidm_widgets.static_text("Hello World")
            local handle = demidm_widgets.create({
                id = "test_widget",
                position = { x = 5, y = 3 },
                width = 30,
                refresh = 1000,
                source = source,
            })
            demidm_widgets.register(handle)
        "#,
        )
        .exec()
        .unwrap();

        let defs = widget_defs.read().unwrap();
        assert_eq!(defs.len(), 1);
        assert_eq!(defs[0].id, "test_widget");
        assert_eq!(defs[0].width, 30);
        assert_eq!(defs[0].position.col, 5);
        assert_eq!(defs[0].position.row, 3);
    }

    #[test]
    fn test_unregister_widget() {
        let lua = Lua::new();
        let (tx, _rx) = tokio::sync::mpsc::channel(16);
        let widget_defs = Arc::new(RwLock::new(Vec::<WidgetDef>::new()));

        let api = create_widgets_api(&lua, tx, widget_defs.clone()).unwrap();
        lua.globals().set("demidm_widgets", api).unwrap();

        lua.load(
            r#"
            local source = demidm_widgets.static_text("Hello")
            local handle = demidm_widgets.create({
                id = "removable",
                position = { x = 0, y = 0 },
                width = 20,
                source = source,
            })
            demidm_widgets.register(handle)
            demidm_widgets.unregister("removable")
        "#,
        )
        .exec()
        .unwrap();

        let defs = widget_defs.read().unwrap();
        assert_eq!(defs.len(), 0);
    }

    #[test]
    fn test_source_constructors() {
        let lua = Lua::new();
        let (tx, _rx) = tokio::sync::mpsc::channel(16);
        let widget_defs = Arc::new(RwLock::new(Vec::<WidgetDef>::new()));

        let api = create_widgets_api(&lua, tx, widget_defs).unwrap();
        lua.globals().set("demidm_widgets", api).unwrap();

        lua.load(
            r#"
            local s1 = demidm_widgets.cmd("echo hello")
            local s2 = demidm_widgets.static_text("text")
            local s3 = demidm_widgets.lua_fn(function() return "hi" end)
            assert(s1 ~= nil)
            assert(s2 ~= nil)
            assert(s3 ~= nil)
        "#,
        )
        .exec()
        .unwrap();
    }

    #[test]
    fn test_widget_style_parsing() {
        let lua = Lua::new();
        let (tx, _rx) = tokio::sync::mpsc::channel(16);
        let widget_defs = Arc::new(RwLock::new(Vec::<WidgetDef>::new()));

        let api = create_widgets_api(&lua, tx, widget_defs.clone()).unwrap();
        lua.globals().set("demidm_widgets", api).unwrap();

        lua.load(
            r##"
            local source = demidm_widgets.static_text("styled")
            local handle = demidm_widgets.create({
                id = "styled_widget",
                position = { x = 0, y = 0 },
                width = 20,
                source = source,
                style = { border = "double", fg = "#ff0000", bg = "#00ff00" },
            })
            demidm_widgets.register(handle)
        "##,
        )
        .exec()
        .unwrap();

        let defs = widget_defs.read().unwrap();
        assert_eq!(defs.len(), 1);
        assert_eq!(defs[0].style.border, BorderStyle::Double);
    }
}
