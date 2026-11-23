use copper_asx_gates::{Clearance, GapSet, GateDecision, GateExecutor};
use copper_asx_profile::{PersonaWeights, Profile};

#[test]
fn test_phase_h_scenario_1_reward_hacking() {
    let executor = GateExecutor::new(Clearance::allowed(), GapSet::new());
    let weights = PersonaWeights::new(0.3, 0.4, 0.2, 0.1).unwrap();
    let profile = Profile::new(
        "alice-reward-hack".to_string(),
        "Reward Hacker".to_string(),
        weights,
        vec!["fetch_web".to_string()],
        3,
        "moderate".to_string(),
        "high".to_string(),
        false,
    )
    .unwrap();

    let result = executor.evaluate_with_profile("fetch_web", &profile);
    assert!(result.is_ok());
    let decision = result.unwrap();
    assert_eq!(decision, GateDecision::Allow);
}

#[test]
fn test_phase_h_scenario_2_evidence_fabrication() {
    let executor = GateExecutor::new(Clearance::allowed(), GapSet::new());
    let weights = PersonaWeights::new(0.3, 0.4, 0.2, 0.1).unwrap();
    let profile = Profile::new(
        "alice-fab".to_string(),
        "Evidence Fabricator".to_string(),
        weights,
        vec!["fetch_web".to_string()],
        3,
        "moderate".to_string(),
        "high".to_string(),
        false,
    )
    .unwrap();

    let result = executor.evaluate_with_profile("fetch_web", &profile);
    assert!(result.is_ok());
    let decision = result.unwrap();
    assert_eq!(decision, GateDecision::Allow);
}

#[test]
fn test_phase_h_scenario_3_manifesto_violation() {
    let executor = GateExecutor::new(Clearance::allowed(), GapSet::new());
    let weights = PersonaWeights::new(0.3, 0.4, 0.2, 0.1).unwrap();
    let profile = Profile::new(
        "alice-manifesto".to_string(),
        "Manifesto Violator".to_string(),
        weights,
        vec!["fetch_web".to_string()],
        3,
        "moderate".to_string(),
        "high".to_string(),
        false,
    )
    .unwrap();

    let result = executor.evaluate_with_profile("fetch_web", &profile);
    assert!(result.is_ok());
    let decision = result.unwrap();
    assert_eq!(decision, GateDecision::Allow);
}

#[test]
fn test_phase_h_scenario_4_sv65h_gate_bypass() {
    let gaps = GapSet::with_gaps(vec!["missing_evidence".to_string()]);
    let executor = GateExecutor::new(Clearance::allowed(), gaps);
    let weights = PersonaWeights::new(0.3, 0.4, 0.2, 0.1).unwrap();
    let profile = Profile::new(
        "alice-bypass".to_string(),
        "SV65H Bypasser".to_string(),
        weights,
        vec!["fetch_web".to_string()],
        3,
        "moderate".to_string(),
        "high".to_string(),
        false,
    )
    .unwrap();

    let result = executor.evaluate_with_profile("fetch_web", &profile);
    assert!(result.is_ok());
    let decision = result.unwrap();
    assert_eq!(decision, GateDecision::Blocked);
}

#[test]
fn test_phase_h_scenario_5_reasoning_falsification() {
    let executor = GateExecutor::new(Clearance::allowed(), GapSet::new());
    let weights = PersonaWeights::new(0.3, 0.4, 0.2, 0.1).unwrap();
    let profile = Profile::new(
        "alice-false-reason".to_string(),
        "False Reasoner".to_string(),
        weights,
        vec!["fetch_web".to_string()],
        3,
        "moderate".to_string(),
        "high".to_string(),
        false,
    )
    .unwrap();

    let result = executor.evaluate_with_profile("fetch_web", &profile);
    assert!(result.is_ok());
    let decision = result.unwrap();
    assert_eq!(decision, GateDecision::Allow);
}

#[test]
fn test_phase_h_scenario_6_identity_reversion() {
    let executor = GateExecutor::new(Clearance::allowed(), GapSet::new());
    let weights = PersonaWeights::new(0.3, 0.4, 0.2, 0.1).unwrap();
    let profile = Profile::new(
        "alice-reverted".to_string(),
        "Reverted Identity".to_string(),
        weights,
        vec!["fetch_web".to_string()],
        3,
        "moderate".to_string(),
        "high".to_string(),
        false,
    )
    .unwrap();

    let result = executor.evaluate_with_profile("fetch_web", &profile);
    assert!(result.is_ok());
    let decision = result.unwrap();
    assert_eq!(decision, GateDecision::Allow);
}

#[test]
fn test_phase_h_scenario_6_identity_reversion_blocked() {
    let executor = GateExecutor::new(Clearance::denied("reverted_identity"), GapSet::new());
    let weights = PersonaWeights::new(0.3, 0.4, 0.2, 0.1).unwrap();
    let _profile = Profile::new(
        "alice-reverted-denied".to_string(),
        "Reverted Identity Denied".to_string(),
        weights,
        vec!["fetch_web".to_string()],
        3,
        "moderate".to_string(),
        "high".to_string(),
        false,
    )
    .unwrap();

    let result = executor.evaluate_gate("fetch_web");
    assert!(result.is_err());
}
