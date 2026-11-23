use crate::GateResult;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AtomEvaluation {
    pub atom_id: String,
    pub satisfiable: bool,
    pub reason: Option<String>,
}

pub struct Z3Stub {
    enabled: bool,
}

impl Z3Stub {
    pub fn new() -> Self {
        Z3Stub { enabled: false }
    }

    pub fn new_disabled() -> Self {
        Z3Stub { enabled: false }
    }

    pub fn evaluate_atom(&self, atom_id: &str) -> GateResult<AtomEvaluation> {
        if !self.enabled {
            return Ok(AtomEvaluation {
                atom_id: atom_id.to_string(),
                satisfiable: false,
                reason: Some("Z3 stub disabled in Phase 1".to_string()),
            });
        }

        Ok(AtomEvaluation {
            atom_id: atom_id.to_string(),
            satisfiable: false,
            reason: Some("Z3 integration not yet implemented".to_string()),
        })
    }

    pub fn evaluate_constraint_set(&self, _constraint_ids: Vec<String>) -> GateResult<bool> {
        if !self.enabled {
            return Ok(false);
        }

        Ok(false)
    }

    pub fn enable(&mut self) {
        self.enabled = true;
    }

    pub fn disable(&mut self) {
        self.enabled = false;
    }

    pub fn is_enabled(&self) -> bool {
        self.enabled
    }
}

impl Default for Z3Stub {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_z3_stub_creation() {
        let stub = Z3Stub::new();
        assert!(!stub.is_enabled());
    }

    #[test]
    fn test_z3_stub_disabled_evaluation() {
        let stub = Z3Stub::new_disabled();
        let result = stub.evaluate_atom("test_atom").unwrap();
        assert_eq!(result.atom_id, "test_atom");
        assert!(!result.satisfiable);
        assert!(result.reason.is_some());
    }

    #[test]
    fn test_z3_stub_enable_disable() {
        let mut stub = Z3Stub::new();
        assert!(!stub.is_enabled());

        stub.enable();
        assert!(stub.is_enabled());

        stub.disable();
        assert!(!stub.is_enabled());
    }

    #[test]
    fn test_z3_stub_constraint_evaluation_disabled() {
        let stub = Z3Stub::new_disabled();
        let result = stub.evaluate_constraint_set(vec!["c1".to_string()]).unwrap();
        assert!(!result);
    }

    #[test]
    fn test_z3_stub_enabled_evaluation_returns_false() {
        let mut stub = Z3Stub::new();
        stub.enable();

        let result = stub.evaluate_atom("atom1").unwrap();
        assert!(!result.satisfiable);
    }
}
