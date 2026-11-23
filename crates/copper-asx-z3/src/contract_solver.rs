use crate::types::{SatResult, Witness, UnsatCore};
use crate::z3_binding::Z3Solver;

pub struct ContractSolver {
    solver: Z3Solver,
}

impl ContractSolver {
    pub fn new() -> Self {
        ContractSolver {
            solver: Z3Solver::new(),
        }
    }

    pub fn encode_gate_decision(&self, tool_name: &str, _in_surface: bool) -> Vec<String> {
        vec![format!("tool_{}_in_surface", tool_name)]
    }

    pub fn encode_pillars(&self, _ir_data: &str) -> Vec<String> {
        vec!["pillars".to_string()]
    }

    pub fn encode_independence(&self, _graph_data: &str) -> Vec<String> {
        vec!["independence_ge_2".to_string()]
    }

    pub fn encode_hop_limit(&self, _hop_limit: u32) -> Vec<String> {
        vec!["hop_ok".to_string()]
    }

    pub fn encode_no_gaps(&self, _gaps: &[String]) -> Vec<String> {
        vec!["no_open_gaps".to_string()]
    }

    pub fn encode_clearance_implication(
        &self,
        pillars: bool,
        independence: bool,
        hop_ok: bool,
        no_gaps: bool,
    ) -> bool {
        self.solver.assert_bool("pillars", pillars);
        self.solver.assert_bool("independence_ge_2", independence);
        self.solver.assert_bool("hop_ok", hop_ok);
        self.solver.assert_bool("no_open_gaps", no_gaps);

        self.solver.assert_implication(
            "clearance_permitted",
            vec!["pillars", "independence_ge_2", "hop_ok", "no_open_gaps"],
        );

        true
    }

    pub fn verify_clearance(
        &self,
        pillars_satisfied: bool,
        independence_verified: bool,
        hop_ok: bool,
        no_gaps_open: bool,
    ) -> Result<Witness, UnsatCore> {
        self.encode_clearance_implication(
            pillars_satisfied,
            independence_verified,
            hop_ok,
            no_gaps_open,
        );

        match self.solver.check() {
            SatResult::Sat => Ok(Witness {
                pillars_satisfied,
                independence_verified,
                hop_ok,
                no_gaps_open,
            }),
            SatResult::Unsat => Err(self.solver.get_unsat_core()),
            SatResult::Unknown => Err(UnsatCore {
                atoms: vec!["unknown".to_string()],
            }),
        }
    }
}

impl Default for ContractSolver {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_contract_solver_creation() {
        let solver = ContractSolver::new();
        assert!(solver.verify_clearance(true, true, true, true).is_ok());
    }

    #[test]
    fn test_encode_gate_decision() {
        let solver = ContractSolver::new();
        let atoms = solver.encode_gate_decision("test_tool", true);
        assert_eq!(atoms.len(), 1);
        assert!(atoms[0].contains("test_tool"));
    }

    #[test]
    fn test_encode_pillars() {
        let solver = ContractSolver::new();
        let atoms = solver.encode_pillars("some_ir");
        assert_eq!(atoms[0], "pillars");
    }

    #[test]
    fn test_encode_independence() {
        let solver = ContractSolver::new();
        let atoms = solver.encode_independence("graph_data");
        assert_eq!(atoms[0], "independence_ge_2");
    }

    #[test]
    fn test_verify_clearance_all_true() {
        let solver = ContractSolver::new();
        let result = solver.verify_clearance(true, true, true, true);
        assert!(result.is_ok());
        let witness = result.unwrap();
        assert!(witness.pillars_satisfied);
        assert!(witness.independence_verified);
        assert!(witness.hop_ok);
        assert!(witness.no_gaps_open);
    }
}
