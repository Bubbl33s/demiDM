use std::path::PathBuf;

use crate::renderer::theme::{BorderStyle, ThemeConfig};

#[derive(Debug, Clone)]
pub struct LoginBoxConfig {
    pub position: BoxPosition,
    pub width: u16,
    pub padding: Padding,
    pub title: String,
    pub show_hostname: bool,
    pub border_style: BorderStyle,
}

impl Default for LoginBoxConfig {
    fn default() -> Self {
        Self {
            position: BoxPosition::default(),
            width: 50,
            padding: Padding::default(),
            title: " DemiDM ".to_string(),
            show_hostname: false,
            border_style: BorderStyle::Rounded,
        }
    }
}

impl LoginBoxConfig {
    pub fn border_style(&self) -> BorderStyle {
        self.border_style
    }
}

#[derive(Debug, Clone)]
pub struct BoxPosition {
    pub x: AxisPosition,
    pub y: AxisPosition,
}

impl Default for BoxPosition {
    fn default() -> Self {
        Self {
            x: AxisPosition::Named(NamedPosition::Center),
            y: AxisPosition::Named(NamedPosition::Center),
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub enum AxisPosition {
    Absolute(u16),
    Fraction(f32),
    Named(NamedPosition),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NamedPosition {
    Center,
    Start,
    End,
}

#[derive(Debug, Clone, Copy, Default)]
#[allow(dead_code)]
pub struct Padding {
    pub top: u16,
    pub bottom: u16,
    pub left: u16,
    pub right: u16,
}

#[derive(Debug, Clone, Default)]
#[allow(dead_code)]
pub struct AuraConfig {
    pub theme: ThemeConfig,
    pub login_box: LoginBoxConfig,
    pub background: Option<BackgroundConfig>,
}

#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct BackgroundConfig {
    pub path: PathBuf,
}

pub fn resolve_config_path() -> Option<PathBuf> {
    if let Ok(dir) = std::env::var("DEMIDM_CONFIG_DIR") {
        let path = PathBuf::from(&dir).join("init.lua");
        if path.exists() {
            return Some(path);
        }
    }

    if let Ok(xdg) = std::env::var("XDG_CONFIG_HOME") {
        let path = PathBuf::from(&xdg).join("demidm").join("init.lua");
        if path.exists() {
            return Some(path);
        }
    }

    let system_path = PathBuf::from("/etc/demidm/init.lua");
    if system_path.exists() {
        return Some(system_path);
    }

    None
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    #[test]
    fn test_resolve_config_path_demidm_config_dir() {
        let tmp = std::env::temp_dir().join("demidm_test_config");
        let _ = fs::create_dir_all(&tmp);
        let config_file = tmp.join("init.lua");
        fs::write(&config_file, "-- test config").unwrap();

        std::env::set_var("DEMIDM_CONFIG_DIR", &tmp);
        let result = resolve_config_path();
        std::env::remove_var("DEMIDM_CONFIG_DIR");

        assert_eq!(result, Some(config_file.clone()));
        let _ = fs::remove_dir_all(&tmp);
    }

    #[test]
    fn test_resolve_config_path_returns_none_when_no_config() {
        std::env::remove_var("DEMIDM_CONFIG_DIR");
        std::env::remove_var("XDG_CONFIG_HOME");

        let result = resolve_config_path();
        if result.is_some() {
            let path = result.unwrap();
            assert!(path.to_string_lossy().contains("/etc/demidm"));
        }
    }
}
