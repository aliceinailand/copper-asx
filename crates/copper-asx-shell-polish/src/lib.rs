pub mod zoom;
pub mod colors;

pub use zoom::{ZoomLevel, ZoomManager};
pub use colors::{ColorScheme, ThemeColors};

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_zoom_level_creation() {
        let zoom = ZoomLevel::default();
        assert_eq!(zoom.factor, 1.0);
    }

    #[test]
    fn test_color_scheme_light() {
        let scheme = ColorScheme::Light;
        assert_eq!(scheme, ColorScheme::Light);
    }

    #[test]
    fn test_theme_colors_creation() {
        let colors = ThemeColors::light();
        assert_eq!(colors.canvas, "#F7F4FC");
        assert_eq!(colors.brand, "#6D28D9");
    }

    #[test]
    fn test_zoom_increment() {
        let mut manager = ZoomManager::new();
        manager.increment();
        assert!(manager.current_level().factor > 1.0);
    }

    #[test]
    fn test_zoom_reset() {
        let mut manager = ZoomManager::new();
        manager.increment();
        manager.reset();
        assert_eq!(manager.current_level().factor, 1.0);
    }
}
