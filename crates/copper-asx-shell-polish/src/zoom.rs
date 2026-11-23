use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq)]
pub struct ZoomLevel {
    pub factor: f64,
    pub level: i32,
}

impl Default for ZoomLevel {
    fn default() -> Self {
        Self {
            factor: 1.0,
            level: 0,
        }
    }
}

impl ZoomLevel {
    pub fn new(factor: f64, level: i32) -> Self {
        Self { factor, level }
    }

    pub fn increment() -> Self {
        Self {
            factor: 1.1,
            level: 1,
        }
    }

    pub fn decrement() -> Self {
        Self {
            factor: 0.9,
            level: -1,
        }
    }

    pub fn reset() -> Self {
        Self::default()
    }
}

pub struct ZoomManager {
    current: ZoomLevel,
    per_site_memory: HashMap<String, f64>,
    min_factor: f64,
    max_factor: f64,
}

impl ZoomManager {
    pub fn new() -> Self {
        Self {
            current: ZoomLevel::default(),
            per_site_memory: HashMap::new(),
            min_factor: 0.5,
            max_factor: 2.0,
        }
    }

    pub fn increment(&mut self) {
        let new_factor = (self.current.factor * 1.1).min(self.max_factor);
        self.current.factor = new_factor;
        self.current.level += 1;
    }

    pub fn decrement(&mut self) {
        let new_factor = (self.current.factor / 1.1).max(self.min_factor);
        self.current.factor = new_factor;
        self.current.level -= 1;
    }

    pub fn reset(&mut self) {
        self.current = ZoomLevel::default();
    }

    pub fn set_zoom(&mut self, factor: f64) {
        self.current.factor = factor.max(self.min_factor).min(self.max_factor);
    }

    pub fn current_level(&self) -> ZoomLevel {
        self.current
    }

    pub fn remember_for_site(&mut self, site: &str, factor: f64) {
        self.per_site_memory.insert(site.to_string(), factor);
    }

    pub fn recall_for_site(&self, site: &str) -> Option<f64> {
        self.per_site_memory.get(site).copied()
    }

    pub fn get_zoom_indicator(&self) -> String {
        format!("{}%", (self.current.factor * 100.0) as i32)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_zoom_level_default() {
        let level = ZoomLevel::default();
        assert_eq!(level.factor, 1.0);
        assert_eq!(level.level, 0);
    }

    #[test]
    fn test_zoom_level_increment() {
        let level = ZoomLevel::increment();
        assert!(level.factor > 1.0);
        assert_eq!(level.level, 1);
    }

    #[test]
    fn test_zoom_level_decrement() {
        let level = ZoomLevel::decrement();
        assert!(level.factor < 1.0);
        assert_eq!(level.level, -1);
    }

    #[test]
    fn test_zoom_manager_increment() {
        let mut manager = ZoomManager::new();
        let initial = manager.current_level().factor;
        manager.increment();
        assert!(manager.current_level().factor > initial);
    }

    #[test]
    fn test_zoom_manager_decrement() {
        let mut manager = ZoomManager::new();
        manager.increment();
        let after_inc = manager.current_level().factor;
        manager.decrement();
        assert!(manager.current_level().factor < after_inc);
    }

    #[test]
    fn test_zoom_manager_reset() {
        let mut manager = ZoomManager::new();
        manager.increment();
        manager.increment();
        manager.reset();
        assert_eq!(manager.current_level().factor, 1.0);
    }

    #[test]
    fn test_zoom_manager_clamp() {
        let mut manager = ZoomManager::new();
        manager.set_zoom(3.0);
        assert!(manager.current_level().factor <= 2.0);
        manager.set_zoom(0.1);
        assert!(manager.current_level().factor >= 0.5);
    }

    #[test]
    fn test_per_site_memory() {
        let mut manager = ZoomManager::new();
        manager.remember_for_site("example.com", 1.5);
        assert_eq!(manager.recall_for_site("example.com"), Some(1.5));
        assert_eq!(manager.recall_for_site("unknown.com"), None);
    }

    #[test]
    fn test_zoom_indicator() {
        let mut manager = ZoomManager::new();
        assert_eq!(manager.get_zoom_indicator(), "100%");
        manager.increment();
        let indicator = manager.get_zoom_indicator();
        assert!(indicator.contains("%"));
    }
}
