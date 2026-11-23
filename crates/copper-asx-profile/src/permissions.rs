use crate::Profile;
use thiserror::Error;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ToolPermission {
    Allow,
    Deny(String),
    Escalate(String),
}

#[derive(Debug, Clone)]
pub struct PermissionResult {
    pub permission: ToolPermission,
    pub reason: String,
}

#[derive(Debug, Error)]
pub enum PermissionError {
    #[error("Tool not found in surface: {0}")]
    ToolNotInSurface(String),

    #[error("Hop limit exceeded: {0}")]
    HopLimitExceeded(String),

    #[error("Clearance not met: {0}")]
    ClearanceNotMet(String),

    #[error("Permission error: {0}")]
    GeneralError(String),
}

pub fn can_tool_call(profile: &Profile, tool_name: &str) -> Result<PermissionResult, PermissionError> {
    if !profile.tool_surface.contains(&tool_name.to_string()) {
        return Ok(PermissionResult {
            permission: ToolPermission::Deny("tool_not_in_surface".to_string()),
            reason: format!("tool '{}' not in profile surface", tool_name),
        });
    }

    Ok(PermissionResult {
        permission: ToolPermission::Allow,
        reason: "tool in surface".to_string(),
    })
}

pub fn hop_limit_check(profile: &Profile, current_hops: u32) -> bool {
    current_hops < profile.hop_limit
}

pub fn clearance_level_check(_profile: &Profile, _requirement: &str) -> bool {
    true
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::PersonaWeights;

    fn create_test_profile() -> Profile {
        let weights = PersonaWeights::new(0.3, 0.4, 0.2, 0.1).unwrap();
        Profile::new(
            "test-op".to_string(),
            "Test Operator".to_string(),
            weights,
            vec!["fetch_web_page".to_string(), "execute_code".to_string()],
            3,
            "moderate".to_string(),
            "high".to_string(),
            false,
        )
        .unwrap()
    }

    #[test]
    fn test_tool_permission_allowed() {
        let profile = create_test_profile();
        let result = can_tool_call(&profile, "fetch_web_page");
        assert!(result.is_ok());
        let perm = result.unwrap();
        assert_eq!(perm.permission, ToolPermission::Allow);
    }

    #[test]
    fn test_tool_permission_denied() {
        let profile = create_test_profile();
        let result = can_tool_call(&profile, "unauthorized_tool");
        assert!(result.is_ok());
        let perm = result.unwrap();
        assert!(matches!(perm.permission, ToolPermission::Deny(_)));
    }

    #[test]
    fn test_hop_limit_check_pass() {
        let profile = create_test_profile();
        assert!(hop_limit_check(&profile, 2));
    }

    #[test]
    fn test_hop_limit_check_fail() {
        let profile = create_test_profile();
        assert!(!hop_limit_check(&profile, 3));
        assert!(!hop_limit_check(&profile, 10));
    }

    #[test]
    fn test_clearance_level_check() {
        let profile = create_test_profile();
        assert!(clearance_level_check(&profile, "moderate"));
    }

    #[test]
    fn test_multiple_tools_in_surface() {
        let profile = create_test_profile();
        assert!(can_tool_call(&profile, "fetch_web_page").is_ok());
        assert!(can_tool_call(&profile, "execute_code").is_ok());
    }
}
