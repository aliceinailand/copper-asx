use crate::Profile;
use thiserror::Error;

#[derive(Debug, Clone)]
pub struct GuestOverlay {
    pub host_profile_id: String,
    pub guest_id: String,
    pub read_only_tool_surface: Vec<String>,
}

#[derive(Debug, Error)]
pub enum GuestError {
    #[error("Guest cannot escalate")]
    CannotEscalate,

    #[error("Guest cannot write to host IR")]
    CannotWriteHostIR,

    #[error("Operation requires human review: {0}")]
    RequiresHumanReview(String),

    #[error("Guest error: {0}")]
    GeneralError(String),
}

impl GuestOverlay {
    pub fn new(host_profile_id: String, guest_id: String) -> Self {
        GuestOverlay {
            host_profile_id,
            guest_id,
            read_only_tool_surface: vec![
                "browser_tabs".to_string(),
                "browser_navigate".to_string(),
            ],
        }
    }

    pub fn get_host_profile_id(&self) -> &str {
        &self.host_profile_id
    }

    pub fn get_guest_id(&self) -> &str {
        &self.guest_id
    }
}

pub fn can_escalate_guest(_guest: &GuestOverlay) -> bool {
    false
}

pub fn get_host_profile(_guest: &GuestOverlay) -> Result<Profile, GuestError> {
    Err(GuestError::GeneralError(
        "Host profile retrieval not yet implemented".to_string(),
    ))
}

pub fn validate_guest_mutation(
    _guest: &GuestOverlay,
    operation: &str,
) -> Result<(), GuestError> {
    Err(GuestError::RequiresHumanReview(format!(
        "Guest mutation '{}' requires human review; guest is read-only",
        operation
    )))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_guest_cannot_escalate() {
        let guest = GuestOverlay::new("host-123".to_string(), "guest-456".to_string());
        assert!(!can_escalate_guest(&guest));
    }

    #[test]
    fn test_guest_cannot_write_host_ir() {
        let guest = GuestOverlay::new("host-123".to_string(), "guest-456".to_string());
        let result = validate_guest_mutation(&guest, "write_ir");
        assert!(result.is_err());
        match result {
            Err(GuestError::RequiresHumanReview(_)) => {},
            _ => panic!("expected RequiresHumanReview"),
        }
    }

    #[test]
    fn test_guest_overlay_creation() {
        let guest = GuestOverlay::new("host-123".to_string(), "guest-456".to_string());
        assert_eq!(guest.get_host_profile_id(), "host-123");
        assert_eq!(guest.get_guest_id(), "guest-456");
        assert_eq!(guest.read_only_tool_surface.len(), 2);
        assert!(guest.read_only_tool_surface.contains(&"browser_tabs".to_string()));
    }
}
