use crate::zoom::ZoomManager;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum KeyCode {
    Equal,
    Minus,
    Zero,
    Unknown,
}

#[derive(Debug, Clone, Copy)]
pub struct KeyEvent {
    pub code: KeyCode,
    pub ctrl: bool,
    pub shift: bool,
    pub alt: bool,
    pub meta: bool,
}

impl KeyEvent {
    pub fn new_key_press(keyval: u32, state: u32) -> Self {
        let code = Self::parse_keyval(keyval);
        let ctrl = (state & 0x04) != 0;
        let shift = (state & 0x01) != 0;
        let alt = (state & 0x08) != 0;
        let meta = (state & 0x10) != 0;

        Self {
            code,
            ctrl,
            shift,
            alt,
            meta,
        }
    }

    fn parse_keyval(keyval: u32) -> KeyCode {
        match keyval {
            61 | 43 => KeyCode::Equal,
            45 => KeyCode::Minus,
            48 => KeyCode::Zero,
            _ => KeyCode::Unknown,
        }
    }

    pub fn handle_zoom(&self, manager: &mut ZoomManager) -> bool {
        if !self.ctrl {
            return false;
        }

        match self.code {
            KeyCode::Equal => {
                manager.zoom_in();
                true
            }
            KeyCode::Minus => {
                manager.zoom_out();
                true
            }
            KeyCode::Zero => {
                manager.reset();
                true
            }
            KeyCode::Unknown => false,
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub struct ScrollEvent {
    pub delta_y: f64,
    pub ctrl: bool,
    pub shift: bool,
    pub alt: bool,
    pub meta: bool,
}

impl ScrollEvent {
    pub fn new_scroll(delta_y: f64, state: u32) -> Self {
        let ctrl = (state & 0x04) != 0;
        let shift = (state & 0x01) != 0;
        let alt = (state & 0x08) != 0;
        let meta = (state & 0x10) != 0;

        Self {
            delta_y,
            ctrl,
            shift,
            alt,
            meta,
        }
    }

    pub fn handle_zoom(&self, manager: &mut ZoomManager) -> bool {
        if !self.ctrl {
            return false;
        }

        if self.delta_y < 0.0 {
            manager.zoom_in();
        } else if self.delta_y > 0.0 {
            manager.zoom_out();
        } else {
            return false;
        }

        true
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_key_event_ctrl_equal() {
        let event = KeyEvent {
            code: KeyCode::Equal,
            ctrl: true,
            shift: false,
            alt: false,
            meta: false,
        };

        let mut manager = ZoomManager::new();
        let handled = event.handle_zoom(&manager);
        assert!(handled);
    }

    #[test]
    fn test_scroll_event_ctrl_up() {
        let event = ScrollEvent::new_scroll(-1.0, 0x04);
        let mut manager = ZoomManager::new();
        let handled = event.handle_zoom(&manager);

        assert!(handled);
        assert!(manager.current_zoom > 1.0);
    }
}
