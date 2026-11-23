use crate::{Clearance, GapSet, GateDecision, GateError, GateResult};
use copper_asx_profile::Profile;

pub struct GateExecutor {
    clearance: Clearance,
    open_gaps: GapSet,
}

impl GateExecutor {
    pub fn new(clearance: Clearance, open_gaps: GapSet) -> Self {
        GateExecutor { clearance, open_gaps }
    }

    pub fn new_with_phase_one_config() -> Self {
        GateExecutor {
            clearance: Clearance::denied("phase_i_clearance_disabled"),
            open_gaps: GapSet::new(),
        }
    }

    pub fn evaluate_gate(&self, tool_name: &str) -> GateResult<GateDecision> {
        if !self.clearance.permitted {
            let reason = self
                .clearance
                .reason
                .as_ref()
                .map(|r| r.as_str())
                .unwrap_or("clearance not granted");
            return Err(GateError::ClearanceDenied(reason.to_string()));
        }

        if self.open_gaps.has_gaps() {
            let gaps_str = self
                .open_gaps
                .gaps
                .iter()
                .cloned()
                .collect::<Vec<_>>()
                .join(", ");
            return Err(GateError::OpenGaps(format!(
                "tool '{}' blocked by open gaps: {}",
                tool_name, gaps_str
            )));
        }

        Ok(GateDecision::Allow)
    }

    pub fn get_decision_for_tool(&self, tool_name: &str) -> (GateDecision, String) {
        match self.evaluate_gate(tool_name) {
            Ok(decision) => (decision, "gate_permit".to_string()),
            Err(GateError::ClearanceDenied(reason)) => (GateDecision::Blocked, format!("clearance={}", reason)),
            Err(GateError::OpenGaps(reason)) => (GateDecision::Reject, reason),
            Err(e) => (GateDecision::Blocked, format!("gate_error: {}", e)),
        }
    }

    pub fn set_clearance(&mut self, clearance: Clearance) {
        self.clearance = clearance;
    }

    pub fn set_open_gaps(&mut self, gaps: GapSet) {
        self.open_gaps = gaps;
    }

    pub fn get_clearance(&self) -> &Clearance {
        &self.clearance
    }

    pub fn get_open_gaps(&self) -> &GapSet {
        &self.open_gaps
    }

    pub fn evaluate_with_profile(
        &self,
        tool_name: &str,
        profile: &copper_asx_profile::Profile,
    ) -> Result<GateDecision, String> {
        if !profile.tool_surface.contains(&tool_name.to_string()) {
            return Ok(GateDecision::Reject);
        }

        if profile.hop_limit == 0 {
            return Ok(GateDecision::Blocked);
        }

        if self.open_gaps.has_gaps() {
            let _gaps_str = self
                .open_gaps
                .gaps
                .iter()
                .cloned()
                .collect::<Vec<_>>()
                .join(", ");
            return Ok(GateDecision::Blocked);
        }

        Ok(GateDecision::Allow)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_phase_one_always_blocked() {
        let executor = GateExecutor::new_with_phase_one_config();
        let (decision, _reason) = executor.get_decision_for_tool("fetch_web");
        assert_eq!(decision, GateDecision::Blocked);
    }

    #[test]
    fn test_clearance_denied_rejection() {
        let clearance = Clearance::denied("not_authorized");
        let gaps = GapSet::new();
        let executor = GateExecutor::new(clearance, gaps);

        let result = executor.evaluate_gate("test_tool");
        assert!(result.is_err());
        match result {
            Err(GateError::ClearanceDenied(reason)) => {
                assert_eq!(reason, "not_authorized");
            }
            _ => panic!("expected ClearanceDenied error"),
        }
    }

    #[test]
    fn test_open_gaps_rejection() {
        let clearance = Clearance::allowed();
        let gaps = GapSet::with_gaps(vec!["gap_a".to_string(), "gap_b".to_string()]);
        let executor = GateExecutor::new(clearance, gaps);

        let result = executor.evaluate_gate("test_tool");
        assert!(result.is_err());
        match result {
            Err(GateError::OpenGaps(reason)) => {
                assert!(reason.contains("gap_a"));
                assert!(reason.contains("gap_b"));
            }
            _ => panic!("expected OpenGaps error"),
        }
    }

    #[test]
    fn test_clearance_allowed_no_gaps_permit() {
        let clearance = Clearance::allowed();
        let gaps = GapSet::new();
        let executor = GateExecutor::new(clearance, gaps);

        let result = executor.evaluate_gate("test_tool");
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), GateDecision::Allow);
    }

    #[test]
    fn test_decision_for_tool() {
        let clearance = Clearance::denied("test_reason");
        let gaps = GapSet::new();
        let executor = GateExecutor::new(clearance, gaps);

        let (decision, reason) = executor.get_decision_for_tool("fetch_web");
        assert_eq!(decision, GateDecision::Blocked);
        assert!(reason.contains("clearance"));
    }

    #[test]
    fn test_mutable_state_changes() {
        let mut executor = GateExecutor::new_with_phase_one_config();
        let (decision, _) = executor.get_decision_for_tool("test");
        assert_eq!(decision, GateDecision::Blocked);

        executor.set_clearance(Clearance::allowed());
        executor.set_open_gaps(GapSet::new());

        let (decision, _) = executor.get_decision_for_tool("test");
        assert_eq!(decision, GateDecision::Allow);
    }

    #[test]
    fn test_evaluate_with_profile_allow() {
        use copper_asx_profile::PersonaWeights;

        let executor = GateExecutor::new(Clearance::allowed(), GapSet::new());
        let weights = PersonaWeights::new(0.3, 0.4, 0.2, 0.1).unwrap();
        let profile = Profile::new(
            "alice-123".to_string(),
            "Alice Operator".to_string(),
            weights,
            vec!["fetch_web_page".to_string()],
            3,
            "moderate".to_string(),
            "high".to_string(),
            false,
        )
        .unwrap();

        let result = executor.evaluate_with_profile("fetch_web_page", &profile);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), GateDecision::Allow);
    }

    #[test]
    fn test_evaluate_with_profile_reject() {
        use copper_asx_profile::PersonaWeights;

        let executor = GateExecutor::new(Clearance::allowed(), GapSet::new());
        let weights = PersonaWeights::new(0.3, 0.4, 0.2, 0.1).unwrap();
        let profile = Profile::new(
            "alice-123".to_string(),
            "Alice Operator".to_string(),
            weights,
            vec!["fetch_web_page".to_string()],
            3,
            "moderate".to_string(),
            "high".to_string(),
            false,
        )
        .unwrap();

        let result = executor.evaluate_with_profile("execute_code", &profile);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), GateDecision::Reject);
    }
}
