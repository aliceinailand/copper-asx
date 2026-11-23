mod evidence;
mod executor;
mod recorder;
mod z3_stub;
pub mod gates_integration;

pub use evidence::{EvidenceLedger, GateDecision, GateDecisionEvent};
pub use executor::GateExecutor;
pub use gates_integration::MutableGateContext;
pub use recorder::EvidenceRecorder;
pub use z3_stub::Z3Stub;

use serde::{Deserialize, Serialize};
use std::collections::HashSet;
use thiserror::Error;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct Clearance {
    pub permitted: bool,
    pub reason: Option<String>,
}

impl Clearance {
    pub fn allowed() -> Self {
        Clearance {
            permitted: true,
            reason: None,
        }
    }

    pub fn denied(reason: impl Into<String>) -> Self {
        Clearance {
            permitted: false,
            reason: Some(reason.into()),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct GapSet {
    pub gaps: HashSet<String>,
}

impl GapSet {
    pub fn new() -> Self {
        GapSet {
            gaps: HashSet::new(),
        }
    }

    pub fn with_gaps(gaps: Vec<String>) -> Self {
        GapSet {
            gaps: gaps.into_iter().collect(),
        }
    }

    pub fn has_gaps(&self) -> bool {
        !self.gaps.is_empty()
    }

    pub fn add_gap(&mut self, gap: String) {
        self.gaps.insert(gap);
    }
}

impl Default for GapSet {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug, Error)]
pub enum GateError {
    #[error("clearance not granted: {0}")]
    ClearanceDenied(String),

    #[error("open gaps remain: {0}")]
    OpenGaps(String),

    #[error("evidence recording failed: {0}")]
    RecordingFailed(String),

    #[error("ledger error: {0}")]
    LedgerError(String),

    #[error("z3 evaluation failed: {0}")]
    Z3Error(String),

    #[error("io error: {0}")]
    IoError(#[from] std::io::Error),

    #[error("serialization error: {0}")]
    SerializationError(#[from] serde_json::Error),
}

pub type GateResult<T> = Result<T, GateError>;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_clearance_allowed() {
        let c = Clearance::allowed();
        assert!(c.permitted);
        assert!(c.reason.is_none());
    }

    #[test]
    fn test_clearance_denied() {
        let c = Clearance::denied("test reason");
        assert!(!c.permitted);
        assert_eq!(c.reason, Some("test reason".to_string()));
    }

    #[test]
    fn test_gap_set_operations() {
        let mut gaps = GapSet::new();
        assert!(!gaps.has_gaps());

        gaps.add_gap("gap1".to_string());
        assert!(gaps.has_gaps());
        assert!(gaps.gaps.contains("gap1"));

        let gaps2 = GapSet::with_gaps(vec!["gap2".to_string(), "gap3".to_string()]);
        assert!(gaps2.has_gaps());
        assert_eq!(gaps2.gaps.len(), 2);
    }
}
