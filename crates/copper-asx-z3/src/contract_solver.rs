use crate::types::{SatResult, UnsatCore, Witness};
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

    pub fn verify_clearance(
        &mut self,
        pillars_satisfied: bool,
        independence_verified: bool,
        hop_ok: bool,
        no_gaps_open: bool,
    ) -> Result<Witness, UnsatCore> {
        self.solver.reset();
        self.solver.assert_bool("pillars", pillars_satisfied);
        self.solver.assert_bool("independence_ge_2", independence_verified);
        self.solver.assert_bool("hop_ok", hop_ok);
        self.solver.assert_bool("no_open_gaps", no_gaps_open);

        match self.solver.check() {
            SatResult::Sat => Ok(Witness {
                pillars_satisfied,
                independence_verified,
                hop_ok,
                no_gaps_open,
            }),
            SatResult::Unsat => {
                let atoms = self.solver.get_unsat_core().unwrap_or_default();
                Err(UnsatCore { atoms })
            }
            SatResult::Unknown => Err(UnsatCore {
                atoms: vec!["unknown".to_string()],
            }),
        }
    }

    pub fn encode_gate_decision(&mut self, tool_in_surface: bool, hop_ok: bool) -> SatResult {
        self.solver.reset();
        self.solver.assert_bool("tool_in_surface", tool_in_surface);
        self.solver.assert_bool("hop_ok", hop_ok);
        self.solver.check()
    }

    pub fn encode_pillars(&mut self, pillars_satisfied: bool) {
        self.solver.assert_bool("pillars_satisfied", pillars_satisfied);
    }

    pub fn encode_independence(&mut self, independence_verified: bool) {
        self.solver
            .assert_bool("independence_verified", independence_verified);
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
        assert!(solver.solver.assertions.is_empty());
    }

    #[test]
    fn test_encode_gate_decision() {
        let mut solver = ContractSolver::new();
        match solver.encode_gate_decision(true, true) {
            SatResult::Sat => {}
            _ => panic!("Expected Sat"),
        }
    }

    #[test]
    fn test_encode_pillars() {
        let mut solver = ContractSolver::new();
        solver.encode_pillars(true);
        assert_eq!(solver.solver.assertions.len(), 1);
    }

    #[test]
    fn test_encode_independence() {
        let mut solver = ContractSolver::new();
        solver.encode_independence(true);
        assert_eq!(solver.solver.assertions.len(), 1);
    }

    #[test]
    fn test_verify_clearance_all_true() {
        let mut solver = ContractSolver::new();
        match solver.verify_clearance(true, true, true, true) {
            Ok(w) => {
                assert!(w.pillars_satisfied);
                assert!(w.independence_verified);
                assert!(w.hop_ok);
                assert!(w.no_gaps_open);
            }
            _ => panic!("Expected Ok"),
        }
    }
}
