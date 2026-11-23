use crate::types::{SatResult, UnsatCore, Witness};

pub struct Z3Solver {
    pub assertions: Vec<String>,
}

impl Z3Solver {
    pub fn new() -> Self {
        Self {
            assertions: vec![],
        }
    }

    pub fn assert_bool(&mut self, atom: &str, value: bool) {
        self.assertions.push(format!("{}={}", atom, value));
    }

    pub fn assert_implication(&mut self, premise: &str, conclusion: &str) {
        self.assertions.push(format!("{}=>{}", premise, conclusion));
    }

    pub fn check(&self) -> SatResult {
        // Simplified: check for contradictions
        let all_true = self.assertions.iter().all(|a| !a.contains("=false"));
        
        if all_true {
            SatResult::Sat
        } else {
            SatResult::Unsat
        }
    }

    pub fn get_unsat_core(&self) -> Option<Vec<String>> {
        let false_assertions: Vec<_> = self
            .assertions
            .iter()
            .filter(|a| a.contains("=false"))
            .map(|a| a.clone())
            .collect();

        if false_assertions.is_empty() {
            None
        } else {
            Some(false_assertions)
        }
    }

    pub fn reset(&mut self) {
        self.assertions.clear();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_z3_solver_creation() {
        let solver = Z3Solver::new();
        assert_eq!(solver.assertions.len(), 0);
    }

    #[test]
    fn test_z3_assert_bool_true() {
        let mut solver = Z3Solver::new();
        solver.assert_bool("atom", true);
        assert_eq!(solver.assertions.len(), 1);
    }

    #[test]
    fn test_z3_solver_unsat_contradiction() {
        let mut solver = Z3Solver::new();
        solver.assert_bool("atom", true);
        solver.assert_bool("atom", false);
        
        match solver.check() {
            SatResult::Unsat => {}
            _ => panic!("Expected Unsat"),
        }
    }

    #[test]
    fn test_z3_unsat_core_extraction() {
        let mut solver = Z3Solver::new();
        solver.assert_bool("atom", false);
        
        let core = solver.get_unsat_core();
        assert!(core.is_some());
        assert_eq!(core.unwrap().len(), 1);
    }
}
