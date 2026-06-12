use mlua::Lua;
use tokio::sync::mpsc::Sender;

use crate::events::AppEvent;
use crate::renderer::theme::{parse_hex_color, BorderStyle, ThemeConfig};

pub fn create_theme_api(lua: &Lua, tx: Sender<AppEvent>) -> mlua::Result<mlua::Table<'_>> {
    let table = lua.create_table()?;

    let tx_set = tx.clone();
    let set_fn = lua.create_function(move |_lua, opts: mlua::Table| {
        let mut theme = ThemeConfig::default();

        if let Some(bg) = opts.get::<_, Option<String>>("background")? {
            theme.background = parse_hex_color(&bg)
                .ok_or_else(|| mlua::Error::external(format!("Invalid color: {}", bg)))?;
        }

        if let Some(fg) = opts.get::<_, Option<String>>("foreground")? {
            theme.foreground = parse_hex_color(&fg)
                .ok_or_else(|| mlua::Error::external(format!("Invalid color: {}", fg)))?;
        }

        if let Some(ac) = opts.get::<_, Option<String>>("accent")? {
            theme.accent = parse_hex_color(&ac)
                .ok_or_else(|| mlua::Error::external(format!("Invalid color: {}", ac)))?;
        }

        if let Some(err) = opts.get::<_, Option<String>>("error")? {
            theme.error = parse_hex_color(&err)
                .ok_or_else(|| mlua::Error::external(format!("Invalid color: {}", err)))?;
        }

        if let Some(border) = opts.get::<_, Option<String>>("border_style")? {
            theme.border_style = BorderStyle::from_str(&border).ok_or_else(|| {
                mlua::Error::external(format!("Invalid border style: {}", border))
            })?;
        }

        if let Some(border_color) = opts.get::<_, Option<String>>("border_color")? {
            theme.border_color = parse_hex_color(&border_color)
                .ok_or_else(|| mlua::Error::external(format!("Invalid color: {}", border_color)))?;
        }

        if let Some(bold) = opts.get::<_, Option<bool>>("font_bold")? {
            theme.font_bold = bold;
        }

        let _ = tx_set.try_send(AppEvent::ConfigUpdate(theme));
        Ok(())
    })?;
    table.set("set", set_fn)?;

    let get_fn = lua.create_function(|lua, ()| {
        let theme = ThemeConfig::default();
        let table = lua.create_table()?;
        table.set("background", color_to_hex(theme.background))?;
        table.set("foreground", color_to_hex(theme.foreground))?;
        table.set("accent", color_to_hex(theme.accent))?;
        table.set("error", color_to_hex(theme.error))?;
        table.set("border_style", border_style_to_str(theme.border_style))?;
        table.set("font_bold", theme.font_bold)?;
        Ok(table)
    })?;
    table.set("get", get_fn)?;

    Ok(table)
}

fn color_to_hex(color: ratatui::style::Color) -> String {
    match color {
        ratatui::style::Color::Rgb(r, g, b) => format!("#{:02x}{:02x}{:02x}", r, g, b),
        ratatui::style::Color::Black => "#000000".to_string(),
        ratatui::style::Color::Red => "#ff0000".to_string(),
        ratatui::style::Color::Green => "#00ff00".to_string(),
        ratatui::style::Color::Yellow => "#ffff00".to_string(),
        ratatui::style::Color::Blue => "#0000ff".to_string(),
        ratatui::style::Color::Magenta => "#ff00ff".to_string(),
        ratatui::style::Color::Cyan => "#00ffff".to_string(),
        ratatui::style::Color::White => "#ffffff".to_string(),
        ratatui::style::Color::DarkGray => "#555555".to_string(),
        ratatui::style::Color::LightRed => "#ff5555".to_string(),
        ratatui::style::Color::LightGreen => "#55ff55".to_string(),
        ratatui::style::Color::LightYellow => "#ffff55".to_string(),
        ratatui::style::Color::LightBlue => "#5555ff".to_string(),
        ratatui::style::Color::LightMagenta => "#ff55ff".to_string(),
        ratatui::style::Color::LightCyan => "#55ffff".to_string(),
        ratatui::style::Color::Gray => "#aaaaaa".to_string(),
        _ => "#000000".to_string(),
    }
}

fn border_style_to_str(style: BorderStyle) -> &'static str {
    match style {
        BorderStyle::Rounded => "rounded",
        BorderStyle::Plain => "plain",
        BorderStyle::Double => "double",
        BorderStyle::None => "none",
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_hex_color_valid() {
        assert_eq!(
            parse_hex_color("#ff0000"),
            Some(ratatui::style::Color::Rgb(255, 0, 0))
        );
        assert_eq!(
            parse_hex_color("#00ff00"),
            Some(ratatui::style::Color::Rgb(0, 255, 0))
        );
        assert_eq!(
            parse_hex_color("#0000ff"),
            Some(ratatui::style::Color::Rgb(0, 0, 255))
        );
        assert_eq!(
            parse_hex_color("ffffff"),
            Some(ratatui::style::Color::Rgb(255, 255, 255))
        );
    }

    #[test]
    fn test_parse_hex_color_invalid() {
        assert_eq!(parse_hex_color("not-a-color"), None);
        assert_eq!(parse_hex_color("#gg0000"), None);
        assert_eq!(parse_hex_color("#fff"), None);
        assert_eq!(parse_hex_color(""), None);
    }

    #[test]
    fn test_border_style_from_str() {
        assert_eq!(BorderStyle::from_str("rounded"), Some(BorderStyle::Rounded));
        assert_eq!(BorderStyle::from_str("plain"), Some(BorderStyle::Plain));
        assert_eq!(BorderStyle::from_str("double"), Some(BorderStyle::Double));
        assert_eq!(BorderStyle::from_str("none"), Some(BorderStyle::None));
        assert_eq!(BorderStyle::from_str("invalid"), None);
    }
}
