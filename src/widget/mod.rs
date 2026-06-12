use std::fmt;
use std::time::Duration;

use ratatui::style::Color;

use crate::errors::{AbsolutePosition, WidgetId};
use crate::renderer::theme::BorderStyle;

pub struct WidgetDef {
    pub id: WidgetId,
    pub position: AbsolutePosition,
    pub width: u16,
    pub height: Option<u16>,
    pub refresh: Duration,
    pub source: WidgetSource,
    pub style: WidgetStyle,
}

impl Clone for WidgetDef {
    fn clone(&self) -> Self {
        Self {
            id: self.id.clone(),
            position: self.position,
            width: self.width,
            height: self.height,
            refresh: self.refresh,
            source: self.source.clone(),
            style: self.style,
        }
    }
}

impl fmt::Debug for WidgetDef {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("WidgetDef")
            .field("id", &self.id)
            .field("position", &self.position)
            .field("width", &self.width)
            .field("height", &self.height)
            .field("refresh", &self.refresh)
            .field("source", &self.source)
            .finish()
    }
}

pub enum WidgetSource {
    ShellCmd(String),
    LuaFn(mlua::RegistryKey),
    StaticText(String),
}

impl Clone for WidgetSource {
    fn clone(&self) -> Self {
        match self {
            WidgetSource::ShellCmd(cmd) => WidgetSource::ShellCmd(cmd.clone()),
            WidgetSource::LuaFn(_) => WidgetSource::StaticText("[lua_fn]".to_string()),
            WidgetSource::StaticText(text) => WidgetSource::StaticText(text.clone()),
        }
    }
}

impl fmt::Debug for WidgetSource {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            WidgetSource::ShellCmd(cmd) => write!(f, "ShellCmd({:?})", cmd),
            WidgetSource::LuaFn(_) => write!(f, "LuaFn(<registry_key>)"),
            WidgetSource::StaticText(text) => write!(f, "StaticText({:?})", text),
        }
    }
}

#[derive(Clone)]
pub struct WidgetInstance {
    pub def: WidgetDef,
    pub content: String,
    pub dirty: bool,
}

impl WidgetInstance {
    #[allow(dead_code)]
    pub fn new(def: WidgetDef) -> Self {
        Self {
            def,
            content: String::new(),
            dirty: false,
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub struct WidgetStyle {
    pub border: BorderStyle,
    pub fg: Color,
    pub bg: Color,
}

impl Default for WidgetStyle {
    fn default() -> Self {
        Self {
            border: BorderStyle::Rounded,
            fg: Color::White,
            bg: Color::Black,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn test_widget_def() -> WidgetDef {
        WidgetDef {
            id: "test".to_string(),
            position: AbsolutePosition { col: 2, row: 2 },
            width: 30,
            height: None,
            refresh: Duration::from_secs(5),
            source: WidgetSource::StaticText("hello".to_string()),
            style: WidgetStyle::default(),
        }
    }

    #[test]
    fn test_widget_instance_creation() {
        let def = test_widget_def();
        let instance = WidgetInstance::new(def);
        assert_eq!(instance.content, "");
        assert!(!instance.dirty);
        assert_eq!(instance.def.id, "test");
    }

    #[test]
    fn test_widget_style_default() {
        let style = WidgetStyle::default();
        assert_eq!(style.border, BorderStyle::Rounded);
        assert_eq!(style.fg, Color::White);
        assert_eq!(style.bg, Color::Black);
    }

    #[test]
    fn test_widget_source_clone_static_text() {
        let source = WidgetSource::StaticText("hello".to_string());
        let cloned = source.clone();
        match cloned {
            WidgetSource::StaticText(text) => assert_eq!(text, "hello"),
            _ => panic!("Expected StaticText"),
        }
    }

    #[test]
    fn test_widget_source_clone_shell_cmd() {
        let source = WidgetSource::ShellCmd("echo hi".to_string());
        let cloned = source.clone();
        match cloned {
            WidgetSource::ShellCmd(cmd) => assert_eq!(cmd, "echo hi"),
            _ => panic!("Expected ShellCmd"),
        }
    }

    #[test]
    fn test_widget_def_clone() {
        let def = test_widget_def();
        let cloned = def.clone();
        assert_eq!(cloned.id, def.id);
        assert_eq!(cloned.width, def.width);
    }
}
