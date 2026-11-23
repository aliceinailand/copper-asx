use serde::{Deserialize, Serialize};
use std::str::FromStr;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct Color {
    pub r: u8,
    pub g: u8,
    pub b: u8,
}

impl Color {
    pub fn new(r: u8, g: u8, b: u8) -> Self {
        Self { r, g, b }
    }

    pub fn from_hex(hex: &str) -> Result<Self, String> {
        let hex = hex.trim_start_matches('#');
        if hex.len() != 6 {
            return Err(format!("Invalid hex color: {}", hex));
        }

        let r = u8::from_str_radix(&hex[0..2], 16)
            .map_err(|_| format!("Invalid hex color: {}", hex))?;
        let g = u8::from_str_radix(&hex[2..4], 16)
            .map_err(|_| format!("Invalid hex color: {}", hex))?;
        let b = u8::from_str_radix(&hex[4..6], 16)
            .map_err(|_| format!("Invalid hex color: {}", hex))?;

        Ok(Self { r, g, b })
    }

    pub fn to_hex(&self) -> String {
        format!("#{:02x}{:02x}{:02x}", self.r, self.g, self.b)
    }

    pub fn to_rgb_string(&self) -> String {
        format!("rgb({}, {}, {})", self.r, self.g, self.b)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ThemeMode {
    Light,
    Dark,
    White,
}

impl FromStr for ThemeMode {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "light" => Ok(ThemeMode::Light),
            "dark" => Ok(ThemeMode::Dark),
            "white" => Ok(ThemeMode::White),
            _ => Err(format!("Unknown theme mode: {}", s)),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SemanticTheme {
    pub mode: ThemeMode,
    pub canvas: Color,
    pub panel: Color,
    pub border: Color,
    pub text_primary: Color,
    pub brand: Color,
    pub action: Color,
    pub action_text: Color,
    pub weight: Color,
    pub data: Color,
    pub pass: Color,
    pub fail: Color,
}

impl SemanticTheme {
    pub fn from_mode(mode: ThemeMode) -> Self {
        match mode {
            ThemeMode::Light => Self::light(),
            ThemeMode::Dark => Self::dark(),
            ThemeMode::White => Self::white(),
        }
    }

    pub fn light() -> Self {
        Self {
            mode: ThemeMode::Light,
            canvas: Color::from_hex("#F7F4FC").unwrap(),
            panel: Color::from_hex("#FFFFFF").unwrap(),
            border: Color::from_hex("#D4C4EF").unwrap(),
            text_primary: Color::from_hex("#1A1228").unwrap(),
            brand: Color::from_hex("#6D28D9").unwrap(),
            action: Color::from_hex("#6D28D9").unwrap(),
            action_text: Color::from_hex("#F5EDD8").unwrap(),
            weight: Color::from_hex("#A8863F").unwrap(),
            data: Color::from_hex("#0E7490").unwrap(),
            pass: Color::from_hex("#059669").unwrap(),
            fail: Color::from_hex("#DC2626").unwrap(),
        }
    }

    pub fn dark() -> Self {
        Self {
            mode: ThemeMode::Dark,
            canvas: Color::from_hex("#0A0809").unwrap(),
            panel: Color::from_hex("#13111A").unwrap(),
            border: Color::from_hex("#2D1B4E").unwrap(),
            text_primary: Color::from_hex("#E8E6F0").unwrap(),
            brand: Color::from_hex("#A78BFA").unwrap(),
            action: Color::from_hex("#7C3AED").unwrap(),
            action_text: Color::from_hex("#F5EDD8").unwrap(),
            weight: Color::from_hex("#C8A35A").unwrap(),
            data: Color::from_hex("#22D3EE").unwrap(),
            pass: Color::from_hex("#34C399").unwrap(),
            fail: Color::from_hex("#F87171").unwrap(),
        }
    }

    pub fn white() -> Self {
        Self {
            mode: ThemeMode::White,
            canvas: Color::from_hex("#FFFFFF").unwrap(),
            panel: Color::from_hex("#FFFFFF").unwrap(),
            border: Color::from_hex("#E8E4F0").unwrap(),
            text_primary: Color::from_hex("#0F0A1A").unwrap(),
            brand: Color::from_hex("#6D28D9").unwrap(),
            action: Color::from_hex("#6D28D9").unwrap(),
            action_text: Color::from_hex("#F5EDD8").unwrap(),
            weight: Color::from_hex("#6D28D9").unwrap(),
            data: Color::from_hex("#0E7490").unwrap(),
            pass: Color::from_hex("#059669").unwrap(),
            fail: Color::from_hex("#DC2626").unwrap(),
        }
    }

    pub fn to_css(&self) -> String {
        format!(
            "--copper-canvas: {};
--copper-panel: {};
--copper-border: {};
--copper-text-primary: {};
--copper-brand: {};
--copper-action: {};
--copper-action-text: {};
--copper-weight: {};
--copper-data: {};
--copper-pass: {};
--copper-fail: {};",
            self.canvas.to_rgb_string(),
            self.panel.to_rgb_string(),
            self.border.to_rgb_string(),
            self.text_primary.to_rgb_string(),
            self.brand.to_rgb_string(),
            self.action.to_rgb_string(),
            self.action_text.to_rgb_string(),
            self.weight.to_rgb_string(),
            self.data.to_rgb_string(),
            self.pass.to_rgb_string(),
            self.fail.to_rgb_string(),
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_color_from_hex() {
        let color = Color::from_hex("#F7F4FC").unwrap();
        assert_eq!(color.r, 0xF7);
        assert_eq!(color.g, 0xF4);
        assert_eq!(color.b, 0xFC);
    }

    #[test]
    fn test_color_to_hex() {
        let color = Color::new(247, 244, 252);
        assert_eq!(color.to_hex(), "#f7f4fc");
    }

    #[test]
    fn test_theme_mode_light() {
        let theme = SemanticTheme::light();
        assert_eq!(theme.mode, ThemeMode::Light);
        assert_eq!(theme.canvas, Color::from_hex("#F7F4FC").unwrap());
        assert_eq!(theme.brand, Color::from_hex("#6D28D9").unwrap());
    }

    #[test]
    fn test_theme_mode_dark() {
        let theme = SemanticTheme::dark();
        assert_eq!(theme.mode, ThemeMode::Dark);
        assert_eq!(theme.canvas, Color::from_hex("#0A0809").unwrap());
        assert_eq!(theme.brand, Color::from_hex("#A78BFA").unwrap());
    }

    #[test]
    fn test_theme_mode_white() {
        let theme = SemanticTheme::white();
        assert_eq!(theme.mode, ThemeMode::White);
        assert_eq!(theme.canvas, Color::from_hex("#FFFFFF").unwrap());
    }

    #[test]
    fn test_semantic_theme_to_css() {
        let theme = SemanticTheme::light();
        let css = theme.to_css();
        assert!(css.contains("--copper-canvas:"));
        assert!(css.contains("--copper-brand:"));
        assert!(css.contains("rgb(109, 40, 217)"));
    }

    #[test]
    fn test_theme_mode_parsing() {
        assert_eq!("light".parse::<ThemeMode>().unwrap(), ThemeMode::Light);
        assert_eq!("dark".parse::<ThemeMode>().unwrap(), ThemeMode::Dark);
        assert_eq!("white".parse::<ThemeMode>().unwrap(), ThemeMode::White);
        assert!("invalid".parse::<ThemeMode>().is_err());
    }
}
