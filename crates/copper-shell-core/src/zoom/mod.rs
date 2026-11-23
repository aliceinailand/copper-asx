mod event_handler;

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

pub use event_handler::{KeyCode, KeyEvent, ScrollEvent};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ZoomManager {
    pub current_zoom: f64,
    pub per_site_memory: HashMap<String, f64>,
}

impl ZoomManager {
    pub fn new() -> Self {
        Self {
            current_zoom: 1.0,
            per_site_memory: HashMap::new(),
        }
    }

    pub fn zoom_in(&mut self) {
        self.current_zoom = (self.current_zoom * 1.1).min(2.0);
    }

    pub fn zoom_out(&mut self) {
        self.current_zoom = (self.current_zoom / 1.1).max(0.5);
    }

    pub fn reset(&mut self) {
        self.current_zoom = 1.0;
    }

    pub fn set_zoom(&mut self, level: f64) {
        self.current_zoom = level.max(0.5).min(2.0);
    }

    pub fn get_zoom_percentage(&self) -> i32 {
        (self.current_zoom * 100.0) as i32
    }

    pub fn remember_for_site(&mut self, url: &str, zoom: f64) {
        self.per_site_memory.insert(url.to_string(), zoom.max(0.5).min(2.0));
    }

    pub fn recall_for_site(&self, url: &str) -> Option<f64> {
        self.per_site_memory.get(url).copied()
    }

    pub fn forget_site(&mut self, url: &str) {
        self.per_site_memory.remove(url);
    }

    pub fn clear_all_memory(&mut self) {
        self.per_site_memory.clear();
    }
}

impl Default for ZoomManager {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_zoom_manager_new() {
        let manager = ZoomManager::new();
        assert_eq!(manager.current_zoom, 1.0);
        assert!(manager.per_site_memory.is_empty());
    }

    #[test]
    fn test_zoom_in() {
        let mut manager = ZoomManager::new();
        manager.zoom_in();
        assert!(manager.current_zoom > 1.0);
        assert!(manager.current_zoom <= 2.0);
    }

    #[test]
    fn test_zoom_out() {
        let mut manager = ZoomManager::new();
        manager.zoom_out();
        assert!(manager.current_zoom < 1.0);
        assert!(manager.current_zoom >= 0.5);
    }

    #[test]
    fn test_reset() {
        let mut manager = ZoomManager::new();
        manager.set_zoom(1.5);
        manager.reset();
        assert_eq!(manager.current_zoom, 1.0);
    }
}
