use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq)]
pub enum ColorScheme {
    Light,
    Dark,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ThemeColors {
    pub canvas: &'static str,
    pub brand: &'static str,
    pub data: &'static str,
    pub success: &'static str,
    pub failure: &'static str,
    pub scheme: ColorScheme,
}

impl ThemeColors {
    pub fn light() -> Self {
        Self {
            canvas: "#F7F4FC",
            brand: "#6D28D9",
            data: "#0EA5E9",
            success: "#10B981",
            failure: "#EF4444",
            scheme: ColorScheme::Light,
        }
    }

    pub fn dark() -> Self {
        Self {
            canvas: "#0A0809",
            brand: "#A78BFA",
            data: "#38BDF8",
            success: "#34D399",
            failure: "#F87171",
            scheme: ColorScheme::Dark,
        }
    }

    pub fn get_canvas(&self) -> &'static str {
        self.canvas
    }

    pub fn get_brand(&self) -> &'static str {
        self.brand
    }

    pub fn is_light(&self) -> bool {
        self.scheme == ColorScheme::Light
    }

    pub fn is_dark(&self) -> bool {
        self.scheme == ColorScheme::Dark
    }

    pub fn as_css_variables(&self) -> String {
        format!(
            "--canvas: {}; --brand: {}; --data: {}; --success: {}; --failure: {};",
            self.canvas, self.brand, self.data, self.success, self.failure
        )
    }

    pub fn get_verdict_color(&self, verdict: &str) -> &'static str {
        match verdict {
            "success" | "verified" => self.success,
            "failure" | "contradicted" => self.failure,
            "data" | "unknown" => self.data,
            _ => self.brand,
        }
    }
}

impl Default for ThemeColors {
    fn default() -> Self {
        Self::light()
    }
}

pub struct ThemeManager {
    current: ThemeColors,
}

impl ThemeManager {
    pub fn new() -> Self {
        Self {
            current: ThemeColors::default(),
        }
    }

    pub fn new_dark() -> Self {
        Self {
            current: ThemeColors::dark(),
        }
    }

    pub fn switch_to_light(&mut self) {
        self.current = ThemeColors::light();
    }

    pub fn switch_to_dark(&mut self) {
        self.current = ThemeColors::dark();
    }

    pub fn current_theme(&self) -> &ThemeColors {
        &self.current
    }

    pub fn apply_to_root(&self) -> String {
        format!(":root {{ {} }}", self.current.as_css_variables())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_light_theme() {
        let colors = ThemeColors::light();
        assert_eq!(colors.canvas, "#F7F4FC");
        assert_eq!(colors.brand, "#6D28D9");
        assert!(colors.is_light());
        assert!(!colors.is_dark());
    }

    #[test]
    fn test_dark_theme() {
        let colors = ThemeColors::dark();
        assert_eq!(colors.canvas, "#0A0809");
        assert_eq!(colors.brand, "#A78BFA");
        assert!(colors.is_dark());
        assert!(!colors.is_light());
    }

    #[test]
    fn test_color_scheme_equality() {
        assert_eq!(ColorScheme::Light, ColorScheme::Light);
        assert_ne!(ColorScheme::Light, ColorScheme::Dark);
    }

    #[test]
    fn test_theme_colors_default() {
        let colors = ThemeColors::default();
        assert!(colors.is_light());
    }

    #[test]
    fn test_canvas_and_brand_getters() {
        let colors = ThemeColors::light();
        assert_eq!(colors.get_canvas(), "#F7F4FC");
        assert_eq!(colors.get_brand(), "#6D28D9");
    }

    #[test]
    fn test_css_variables() {
        let colors = ThemeColors::light();
        let css = colors.as_css_variables();
        assert!(css.contains("--canvas"));
        assert!(css.contains("--brand"));
        assert!(css.contains("#F7F4FC"));
        assert!(css.contains("#6D28D9"));
    }

    #[test]
    fn test_verdict_color_success() {
        let colors = ThemeColors::light();
        assert_eq!(colors.get_verdict_color("success"), colors.success);
        assert_eq!(colors.get_verdict_color("verified"), colors.success);
    }

    #[test]
    fn test_verdict_color_failure() {
        let colors = ThemeColors::light();
        assert_eq!(colors.get_verdict_color("failure"), colors.failure);
        assert_eq!(colors.get_verdict_color("contradicted"), colors.failure);
    }

    #[test]
    fn test_verdict_color_data() {
        let colors = ThemeColors::light();
        assert_eq!(colors.get_verdict_color("data"), colors.data);
        assert_eq!(colors.get_verdict_color("unknown"), colors.data);
    }

    #[test]
    fn test_theme_manager_light() {
        let manager = ThemeManager::new();
        assert!(manager.current_theme().is_light());
    }

    #[test]
    fn test_theme_manager_dark() {
        let manager = ThemeManager::new_dark();
        assert!(manager.current_theme().is_dark());
    }

    #[test]
    fn test_theme_manager_switch() {
        let mut manager = ThemeManager::new();
        assert!(manager.current_theme().is_light());
        manager.switch_to_dark();
        assert!(manager.current_theme().is_dark());
        manager.switch_to_light();
        assert!(manager.current_theme().is_light());
    }

    #[test]
    fn test_theme_manager_apply_to_root() {
        let manager = ThemeManager::new();
        let css = manager.apply_to_root();
        assert!(css.contains(":root"));
        assert!(css.contains("--canvas"));
    }

    #[test]
    fn test_no_color_alone_verdict() {
        let colors = ThemeColors::light();
        // Verify that every verdict returns a semantic color, not color-alone
        assert!(!colors.get_verdict_color("success").is_empty());
        assert!(!colors.get_verdict_color("failure").is_empty());
        assert!(!colors.get_verdict_color("unknown").is_empty());
    }
}
