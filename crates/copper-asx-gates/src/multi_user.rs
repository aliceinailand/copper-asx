use crate::{GateDecision, GateError, GateResult};
use copper_asx_profile::Profile;
use std::collections::HashMap;

pub struct MultiUserGateContext {
    profiles: HashMap<String, Profile>,
}

impl MultiUserGateContext {
    pub fn new() -> Self {
        MultiUserGateContext {
            profiles: HashMap::new(),
        }
    }

    pub fn add_profile(&mut self, profile: Profile) {
        self.profiles.insert(profile.id.clone(), profile);
    }

    pub fn select_profile(&self, user_id: &str) -> GateResult<Profile> {
        self.profiles
            .get(user_id)
            .cloned()
            .ok_or_else(|| GateError::LedgerError(format!("profile not found: {}", user_id)))
    }

    pub fn gate_with_profile(
        &self,
        tool_name: &str,
        profile: &Profile,
    ) -> GateResult<GateDecision> {
        if !profile.tool_surface.contains(&tool_name.to_string()) {
            return Ok(GateDecision::Reject);
        }

        if profile.hop_limit == 0 {
            return Ok(GateDecision::Blocked);
        }

        Ok(GateDecision::Allow)
    }

    pub fn record_decision_with_profile(
        &self,
        _decision: GateDecision,
        _profile_id: &str,
        _operator_id: &str,
    ) -> GateResult<()> {
        Ok(())
    }
}

impl Default for MultiUserGateContext {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use copper_asx_profile::PersonaWeights;

    fn create_test_profile(id: &str, tools: Vec<&str>) -> Profile {
        let weights = PersonaWeights::new(0.3, 0.4, 0.2, 0.1).unwrap();
        Profile::new(
            id.to_string(),
            format!("User {}", id),
            weights,
            tools.into_iter().map(|t| t.to_string()).collect(),
            3,
            "moderate".to_string(),
            "high".to_string(),
            false,
        )
        .unwrap()
    }

    #[test]
    fn test_select_profile() {
        let mut ctx = MultiUserGateContext::new();
        let profile = create_test_profile("alice", vec!["fetch_web_page"]);
        ctx.add_profile(profile);

        let selected = ctx.select_profile("alice");
        assert!(selected.is_ok());
        let p = selected.unwrap();
        assert_eq!(p.id, "alice");
    }

    #[test]
    fn test_select_profile_not_found() {
        let ctx = MultiUserGateContext::new();
        let result = ctx.select_profile("alice");
        assert!(result.is_err());
    }

    #[test]
    fn test_gate_with_profile_allow() {
        let ctx = MultiUserGateContext::new();
        let profile = create_test_profile("alice", vec!["fetch_web_page"]);

        let decision = ctx.gate_with_profile("fetch_web_page", &profile);
        assert!(decision.is_ok());
        assert_eq!(decision.unwrap(), GateDecision::Allow);
    }

    #[test]
    fn test_gate_with_profile_reject() {
        let ctx = MultiUserGateContext::new();
        let profile = create_test_profile("alice", vec!["fetch_web_page"]);

        let decision = ctx.gate_with_profile("execute_code", &profile);
        assert!(decision.is_ok());
        assert_eq!(decision.unwrap(), GateDecision::Reject);
    }

    #[test]
    fn test_record_decision_with_profile() {
        let ctx = MultiUserGateContext::new();
        let result = ctx.record_decision_with_profile(
            GateDecision::Allow,
            "alice-123",
            "operator-456",
        );
        assert!(result.is_ok());
    }
}
