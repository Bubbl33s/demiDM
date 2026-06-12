use ratatui::style::Color;
use ratatui::widgets::BorderType;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum BorderStyle {
    #[default]
    Rounded,
    Plain,
    Double,
    None,
}

impl BorderStyle {
    pub fn to_border_type(self) -> BorderType {
        match self {
            BorderStyle::Rounded => BorderType::Rounded,
            BorderStyle::Plain => BorderType::Plain,
            BorderStyle::Double => BorderType::Double,
            BorderStyle::None => BorderType::Plain,
        }
    }

    pub fn from_str(s: &str) -> Option<Self> {
        match s.to_lowercase().as_str() {
            "rounded" => Some(BorderStyle::Rounded),
            "plain" => Some(BorderStyle::Plain),
            "double" => Some(BorderStyle::Double),
            "none" => Some(BorderStyle::None),
            _ => None,
        }
    }
}

pub fn parse_hex_color(hex: &str) -> Option<Color> {
    let hex = hex.trim_start_matches('#');
    if hex.len() != 6 {
        return None;
    }
    let r = u8::from_str_radix(&hex[0..2], 16).ok()?;
    let g = u8::from_str_radix(&hex[2..4], 16).ok()?;
    let b = u8::from_str_radix(&hex[4..6], 16).ok()?;
    Some(Color::Rgb(r, g, b))
}

#[derive(Debug, Clone)]
pub struct ThemeConfig {
    pub background: Color,
    pub foreground: Color,
    pub accent: Color,
    pub error: Color,
    pub border_style: BorderStyle,
    pub border_color: Color,
    pub font_bold: bool,
}

impl ThemeConfig {
    pub fn default_theme() -> Self {
        Self {
            background: Color::Black,
            foreground: Color::White,
            accent: Color::Cyan,
            error: Color::Red,
            border_style: BorderStyle::Rounded,
            border_color: Color::DarkGray,
            font_bold: true,
        }
    }
}

impl Default for ThemeConfig {
    fn default() -> Self {
        Self::default_theme()
    }
}
