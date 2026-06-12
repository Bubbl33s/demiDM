use mlua::Lua;
use tokio::sync::mpsc::Sender;

use crate::events::AppEvent;
use crate::lua_runtime::config::{
    AxisPosition, BoxPosition, LoginBoxConfig, NamedPosition, Padding,
};
use crate::renderer::theme::BorderStyle;

pub fn create_login_box_api(lua: &Lua, tx: Sender<AppEvent>) -> mlua::Result<mlua::Table<'_>> {
    let table = lua.create_table()?;

    let configure_fn = lua.create_function(move |_lua, opts: mlua::Table| {
        let mut config = LoginBoxConfig::default();

        if let Some(position_table) = opts.get::<_, Option<mlua::Table>>("position")? {
            config.position = parse_position(&position_table)?;
        }

        if let Some(width) = opts.get::<_, Option<u16>>("width")? {
            config.width = width;
        }

        if let Some(title) = opts.get::<_, Option<String>>("title")? {
            config.title = title;
        }

        if let Some(show_hostname) = opts.get::<_, Option<bool>>("show_hostname")? {
            config.show_hostname = show_hostname;
        }

        if let Some(padding_table) = opts.get::<_, Option<mlua::Table>>("padding")? {
            config.padding = parse_padding(&padding_table)?;
        }

        if let Some(border_style) = opts.get::<_, Option<String>>("border_style")? {
            config.border_style = BorderStyle::from_str(&border_style).ok_or_else(|| {
                mlua::Error::external(format!("Invalid border style: {}", border_style))
            })?;
        }

        let _ = tx.try_send(AppEvent::LoginBoxConfigUpdate(config));
        Ok(())
    })?;
    table.set("configure", configure_fn)?;

    Ok(table)
}

fn parse_position(table: &mlua::Table) -> mlua::Result<BoxPosition> {
    let x = parse_axis_position(table, "x")?.unwrap_or(AxisPosition::Named(NamedPosition::Center));
    let y = parse_axis_position(table, "y")?.unwrap_or(AxisPosition::Named(NamedPosition::Center));
    Ok(BoxPosition { x, y })
}

fn parse_axis_position(table: &mlua::Table, key: &str) -> mlua::Result<Option<AxisPosition>> {
    let value: mlua::Value = table.get(key)?;
    match value {
        mlua::Value::Nil => Ok(None),
        mlua::Value::Number(n) => {
            if (0.0..=1.0).contains(&n) {
                Ok(Some(AxisPosition::Fraction(n as f32)))
            } else {
                Ok(Some(AxisPosition::Absolute(n as u16)))
            }
        }
        mlua::Value::Integer(n) => Ok(Some(AxisPosition::Absolute(n as u16))),
        mlua::Value::String(s) => {
            let pos = match s.to_str()?.to_lowercase().as_str() {
                "center" => NamedPosition::Center,
                "start" | "left" | "top" => NamedPosition::Start,
                "end" | "right" | "bottom" => NamedPosition::End,
                other => {
                    return Err(mlua::Error::external(format!(
                        "Unknown position: {}",
                        other
                    )));
                }
            };
            Ok(Some(AxisPosition::Named(pos)))
        }
        _ => Err(mlua::Error::external("Invalid position value")),
    }
}

fn parse_padding(table: &mlua::Table) -> mlua::Result<Padding> {
    Ok(Padding {
        top: table.get::<_, Option<u16>>("top")?.unwrap_or(0),
        bottom: table.get::<_, Option<u16>>("bottom")?.unwrap_or(0),
        left: table.get::<_, Option<u16>>("left")?.unwrap_or(0),
        right: table.get::<_, Option<u16>>("right")?.unwrap_or(0),
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_login_box_config_default() {
        let config = LoginBoxConfig::default();
        assert_eq!(config.width, 50);
        assert_eq!(config.title, " DemiDM ");
        assert!(!config.show_hostname);
    }

    #[test]
    fn test_axis_position_named() {
        let pos = AxisPosition::Named(NamedPosition::Center);
        match pos {
            AxisPosition::Named(NamedPosition::Center) => {}
            _ => panic!("Expected Center"),
        }
    }
}
