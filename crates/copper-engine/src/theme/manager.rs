use crate::theme::{SemanticTheme, ThemeMode};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ThemeSource {
    System,
    Manual,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ThemeManager {
    pub current_mode: ThemeMode,
    pub source: ThemeSource,
    pub theme: SemanticTheme,
}

impl ThemeManager {
    pub fn new() -> Self {
        let mode = Self::detect_system_theme();
        let theme = SemanticTheme::from_mode(mode);
        Self {
            current_mode: mode,
            source: ThemeSource::System,
            theme,
        }
    }

    pub fn from_mode(mode: ThemeMode) -> Self {
        let theme = SemanticTheme::from_mode(mode);
        Self {
            current_mode: mode,
            source: ThemeSource::Manual,
            theme,
        }
    }

    fn detect_system_theme() -> ThemeMode {
        match std::env::var("GTK_SETTINGS_PREFER_DARK_THEME") {
            Ok(val) => match val.as_str() {
                "1" | "true" => ThemeMode::Dark,
                _ => ThemeMode::Light,
            },
            Err(_) => {
                if std::env::var("GTK_SETTINGS_PREFER_COLOR_SCHEME")
                    .ok()
                    .map(|v| v == "dark")
                    .unwrap_or(false)
                {
                    ThemeMode::Dark
                } else {
                    ThemeMode::Light
                }
            }
        }
    }

    pub fn switch_to(&mut self, mode: ThemeMode) {
        self.current_mode = mode;
        self.theme = SemanticTheme::from_mode(mode);
        self.source = ThemeSource::Manual;
    }

    pub fn switch_to_system(&mut self) {
        let mode = Self::detect_system_theme();
        self.current_mode = mode;
        self.theme = SemanticTheme::from_mode(mode);
        self.source = ThemeSource::System;
    }

    pub fn get_css_variables(&self) -> String {
        self.theme.to_css()
    }

    pub fn get_theme(&self) -> &SemanticTheme {
        &self.theme
    }

    pub fn get_mode(&self) -> ThemeMode {
        self.current_mode
    }

    pub fn is_dark(&self) -> bool {
        matches!(self.current_mode, ThemeMode::Dark)
    }

    pub fn is_light(&self) -> bool {
        matches!(self.current_mode, ThemeMode::Light)
    }
}

impl Default for ThemeManager {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_theme_manager_new() {
        let manager = ThemeManager::new();
        assert!(matches!(manager.source, ThemeSource::System));
        assert!(matches!(
            manager.current_mode,
            ThemeMode::Light | ThemeMode::Dark
        ));
    }

    #[test]
    fn test_theme_manager_from_mode() {
        let manager = ThemeManager::from_mode(ThemeMode::Dark);
        assert_eq!(manager.current_mode, ThemeMode::Dark);
        assert_eq!(manager.source, ThemeSource::Manual);
        assert!(manager.is_dark());
        assert!(!manager.is_light());
    }

    #[test]
    fn test_switch_to() {
        let mut manager = ThemeManager::from_mode(ThemeMode::Light);
        manager.switch_to(ThemeMode::Dark);
        assert_eq!(manager.current_mode, ThemeMode::Dark);
        assert_eq!(manager.source, ThemeSource::Manual);
    }

    #[test]
    fn test_switch_to_system() {
        let mut manager = ThemeManager::from_mode(ThemeMode::Dark);
        manager.switch_to_system();
        assert_eq!(manager.source, ThemeSource::System);
    }

    #[test]
    fn test_get_css_variables() {
        let manager = ThemeManager::from_mode(ThemeMode::Light);
        let css = manager.get_css_variables();
        assert!(css.contains("--copper-canvas:"));
        assert!(css.contains("--copper-brand:"));
        assert!(css.contains("rgb("));
    }

    #[test]
    fn test_theme_manager_serialization() {
        let manager = ThemeManager::from_mode(ThemeMode::Dark);
        let json = serde_json::to_string(&manager).unwrap();
        let restored: ThemeManager = serde_json::from_str(&json).unwrap();
        assert_eq!(restored.current_mode, ThemeMode::Dark);
        assert_eq!(restored.source, ThemeSource::Manual);
    }
}
