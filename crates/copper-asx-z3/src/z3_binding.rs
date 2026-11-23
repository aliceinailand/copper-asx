use crate::types::{SatResult, UnsatCore};
use parking_lot::Mutex;
use std::collections::{HashMap, HashSet};
use std::sync::Arc;

pub struct Z3Solver {
    assertions: Arc<Mutex<Vec<Assertion>>>,
    implications: Arc<Mutex<Vec<Implication>>>,
}

#[derive(Debug, Clone)]
struct Assertion {
    var: String,
    value: bool,
}

#[derive(Debug, Clone)]
struct Implication {
    consequent: String,
    antecedents: Vec<String>,
}

// Safe: Z3Solver owns Arc<Mutex<>> which is Send+Sync. Data is safely synchronized via parking_lot.
unsafe impl Send for Z3Solver {}
unsafe impl Sync for Z3Solver {}

impl Z3Solver {
    pub fn new() -> Self {
        Z3Solver {
            assertions: Arc::new(Mutex::new(Vec::new())),
            implications: Arc::new(Mutex::new(Vec::new())),
        }
    }

    pub fn assert_bool(&self, name: &str, value: bool) {
        let mut assertions = self.assertions.lock();
        assertions.push(Assertion {
            var: name.to_string(),
            value,
        });
    }

    pub fn assert_implication(&self, consequent: &str, antecedents: Vec<&str>) {
        let mut implications = self.implications.lock();
        implications.push(Implication {
            consequent: consequent.to_string(),
            antecedents: antecedents.iter().map(|s| s.to_string()).collect(),
        });
    }

    pub fn check(&self) -> SatResult {
        let assertions = self.assertions.lock();
        let implications = self.implications.lock();

        let var_map: HashMap<String, bool> = assertions
            .iter()
            .fold(HashMap::new(), |mut map, a| {
                map.insert(a.var.clone(), a.value);
                map
            });

        for Implication {
            consequent,
            antecedents,
        } in implications.iter()
        {
            let all_antecedents_true = antecedents.iter().all(|ant| {
                var_map.get(ant).copied().unwrap_or(false)
            });

            if all_antecedents_true {
                if let Some(false) = var_map.get(consequent) {
                    return SatResult::Unsat;
                }
            }
        }

        for Assertion { var, value } in assertions.iter() {
            if assertions
                .iter()
                .any(|a| a.var == *var && a.value != *value)
            {
                return SatResult::Unsat;
            }
        }

        SatResult::Sat
    }

    pub fn get_unsat_core(&self) -> UnsatCore {
        let assertions = self.assertions.lock();
        let implications = self.implications.lock();

        let var_map: HashMap<String, bool> = assertions
            .iter()
            .fold(HashMap::new(), |mut map, a| {
                map.insert(a.var.clone(), a.value);
                map
            });

        let mut failing_atoms = HashSet::new();

        for Implication {
            consequent,
            antecedents,
        } in implications.iter()
        {
            let all_antecedents_true = antecedents.iter().all(|ant| {
                var_map.get(ant).copied().unwrap_or(false)
            });

            if all_antecedents_true {
                if let Some(false) = var_map.get(consequent) {
                    failing_atoms.insert(consequent.clone());
                }
            } else {
                for ant in antecedents.iter() {
                    if !var_map.get(ant).copied().unwrap_or(false) {
                        failing_atoms.insert(ant.clone());
                    }
                }
            }
        }

        for Assertion { var, value } in assertions.iter() {
            if assertions
                .iter()
                .any(|a| a.var == *var && a.value != *value)
            {
                failing_atoms.insert(var.clone());
            }
        }

        let atoms: Vec<String> = failing_atoms.into_iter().collect();

        if atoms.is_empty() {
            UnsatCore {
                atoms: vec!["unknown".to_string()],
            }
        } else {
            UnsatCore { atoms }
        }
    }
}

impl Default for Z3Solver {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_z3_solver_creation() {
        let solver = Z3Solver::new();
        let result = solver.check();
        assert_eq!(result, SatResult::Sat);
    }

    #[test]
    fn test_z3_assert_bool_true() {
        let solver = Z3Solver::new();
        solver.assert_bool("test", true);
        let result = solver.check();
        assert_eq!(result, SatResult::Sat);
    }

    #[test]
    fn test_z3_solver_unsat_contradiction() {
        let solver = Z3Solver::new();
        solver.assert_bool("contradiction", true);
        solver.assert_bool("contradiction", false);
        let result = solver.check();
        assert_eq!(result, SatResult::Unsat);
    }

    #[test]
    fn test_z3_unsat_core_extraction() {
        let solver = Z3Solver::new();
        solver.assert_bool("atom1", true);
        solver.assert_bool("atom1", false);
        let result = solver.check();
        assert_eq!(result, SatResult::Unsat);

        let core = solver.get_unsat_core();
        assert!(!core.atoms.is_empty());
        assert!(core.atoms.contains(&"atom1".to_string()));
    }
}

