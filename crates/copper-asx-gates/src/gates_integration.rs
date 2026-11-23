// INTEGRATION GUIDE: Adding copper-asx-gates to copper-mcp
//
// File: crates/copper-mcp/src/gates_integration.rs
// Status: Reference implementation for Phase I.1
//
// To integrate gates into the existing MCP server:
//
// 1. Add to copper-mcp/Cargo.toml:
//    ```toml
//    [dependencies]
//    copper-asx-gates = { path = "../copper-asx-gates" }
//    ```
//
// 2. Add to crates/copper-mcp/src/lib.rs:
//    ```rust
//    mod gates_integration;
//    pub use gates_integration::MutableGateContext;
//    ```
//
// 3. In server.rs, before tool execution:
//    ```rust
//    use copper_asx_gates::{EvidenceRecorder, GateDecision};
//    
//    let mut recorder = EvidenceRecorder::new(".copper/evidence_ledger.jsonl")?;
//    let (decision, reason) = gate_executor.get_decision_for_tool(&tool_name);
//    
//    recorder.record_gate_decision(decision, &tool_name, &reason)?;
//    
//    if decision != GateDecision::Allow {
//        return Err(format!("Gate: {}", reason));
//    }
//    ```
//
// 4. See reference implementation below for full example.

use crate::{Clearance, EvidenceRecorder, GateDecision, GateExecutor, GapSet};
use std::path::Path;

pub struct MutableGateContext {
    executor: GateExecutor,
    recorder: EvidenceRecorder,
}

impl MutableGateContext {
    pub fn new_phase_one<P: AsRef<Path>>(ledger_path: P) -> Result<Self, Box<dyn std::error::Error>> {
        let executor = GateExecutor::new_with_phase_one_config();
        let recorder = EvidenceRecorder::new(ledger_path)?;

        Ok(MutableGateContext { executor, recorder })
    }

    pub fn new_custom<P: AsRef<Path>>(
        clearance: Clearance,
        gaps: GapSet,
        ledger_path: P,
    ) -> Result<Self, Box<dyn std::error::Error>> {
        let executor = GateExecutor::new(clearance, gaps);
        let recorder = EvidenceRecorder::new(ledger_path)?;

        Ok(MutableGateContext { executor, recorder })
    }

    pub fn evaluate_tool_call(&mut self, tool_name: &str) -> Result<GateDecision, String> {
        let (decision, reason) = self.executor.get_decision_for_tool(tool_name);

        self.recorder
            .record_gate_decision(decision.clone(), tool_name, &reason)
            .map_err(|e| format!("recording failed: {}", e))?;

        match decision {
            GateDecision::Allow => Ok(GateDecision::Allow),
            GateDecision::Blocked | GateDecision::Reject => Err(reason),
        }
    }

    pub fn get_ledger_entry_count(&self) -> Result<usize, Box<dyn std::error::Error>> {
        Ok(self.recorder.get_ledger_entry_count()?)
    }

    pub fn verify_ledger_integrity(&self) -> Result<bool, Box<dyn std::error::Error>> {
        Ok(self.recorder.verify_ledger_integrity()?)
    }

    pub fn set_clearance(&mut self, clearance: Clearance) {
        self.executor.set_clearance(clearance);
    }

    pub fn set_open_gaps(&mut self, gaps: GapSet) {
        self.executor.set_open_gaps(gaps);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::NamedTempFile;

    #[test]
    fn test_gate_context_phase_one() {
        let temp_file = NamedTempFile::new().unwrap();
        let mut ctx = MutableGateContext::new_phase_one(temp_file.path()).unwrap();

        let result = ctx.evaluate_tool_call("fetch_web");
        assert!(result.is_err());

        let count = ctx.get_ledger_entry_count().unwrap();
        assert_eq!(count, 1);
    }

    #[test]
    fn test_gate_context_multiple_calls() {
        let temp_file = NamedTempFile::new().unwrap();
        let temp_path = temp_file.path().to_path_buf();
        
        let mut ctx = MutableGateContext::new_phase_one(&temp_path).unwrap();

        ctx.evaluate_tool_call("tool1").ok();
        ctx.evaluate_tool_call("tool2").ok();
        ctx.evaluate_tool_call("tool3").ok();

        let count = ctx.get_ledger_entry_count().unwrap();
        assert_eq!(count, 3);

        let valid = ctx.verify_ledger_integrity().unwrap();
        assert!(valid);
    }

    #[test]
    fn test_gate_context_allowed_path() {
        let temp_file = NamedTempFile::new().unwrap();
        let temp_path = temp_file.path().to_path_buf();
        
        let mut ctx = MutableGateContext::new_custom(
            Clearance::allowed(),
            GapSet::new(),
            &temp_path,
        )
        .unwrap();

        let result = ctx.evaluate_tool_call("fetch_web");
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), GateDecision::Allow);
    }
}
