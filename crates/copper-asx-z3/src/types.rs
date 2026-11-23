use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum SatResult {
    Sat,
    Unsat,
    Unknown,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Witness {
    pub pillars_satisfied: bool,
    pub independence_verified: bool,
    pub hop_ok: bool,
    pub no_gaps_open: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UnsatCore {
    pub atoms: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Z3Atom {
    Bool(String),
    Int(String),
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_witness_creation() {
        let witness = Witness {
            pillars_satisfied: true,
            independence_verified: true,
            hop_ok: true,
            no_gaps_open: true,
        };
        assert!(witness.pillars_satisfied);
    }

    #[test]
    fn test_unsat_core_creation() {
        let core = UnsatCore {
            atoms: vec!["pillars".to_string(), "independence".to_string()],
        };
        assert_eq!(core.atoms.len(), 2);
        assert_eq!(core.atoms[0], "pillars");
    }
}
