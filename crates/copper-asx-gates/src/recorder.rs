use crate::{EvidenceLedger, GateDecision, GateResult};
use std::path::Path;

pub struct EvidenceRecorder {
    ledger: EvidenceLedger,
    packet_id_counter: u64,
}

impl EvidenceRecorder {
    pub fn new<P: AsRef<Path>>(ledger_path: P) -> GateResult<Self> {
        let ledger = EvidenceLedger::new(ledger_path)?;
        Ok(EvidenceRecorder {
            ledger,
            packet_id_counter: 0,
        })
    }

    pub fn record_gate_decision(
        &mut self,
        decision: GateDecision,
        tool_name: &str,
        reason: &str,
    ) -> GateResult<String> {
        self.packet_id_counter += 1;
        let packet_id = format!("pkt-{:06}", self.packet_id_counter);

        self.ledger.record_decision(decision, tool_name, reason, &packet_id)?;

        Ok(packet_id)
    }

    pub fn verify_ledger_integrity(&self) -> GateResult<bool> {
        self.ledger.verify_chain()
    }

    pub fn get_ledger_entry_count(&self) -> GateResult<usize> {
        self.ledger.get_entries_count()
    }

    pub fn get_packet_id_counter(&self) -> u64 {
        self.packet_id_counter
    }

    pub fn reset_packet_counter(&mut self) {
        self.packet_id_counter = 0;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::NamedTempFile;

    #[test]
    fn test_recorder_creation() {
        let temp_file = NamedTempFile::new().unwrap();
        let recorder = EvidenceRecorder::new(temp_file.path()).unwrap();
        assert_eq!(recorder.packet_id_counter, 0);
    }

    #[test]
    fn test_record_multiple_decisions() {
        let temp_file = NamedTempFile::new().unwrap();
        let temp_path = temp_file.path().to_path_buf();
        
        let mut recorder = EvidenceRecorder::new(&temp_path).unwrap();

        let pkt1 = recorder.record_gate_decision(GateDecision::Blocked, "fetch_web", "clearance=false").unwrap();
        assert_eq!(pkt1, "pkt-000001");

        let pkt2 = recorder.record_gate_decision(GateDecision::Reject, "run_script", "open_gaps=gap1").unwrap();
        assert_eq!(pkt2, "pkt-000002");

        let count = recorder.get_ledger_entry_count().unwrap();
        assert_eq!(count, 2);
    }

    #[test]
    fn test_ledger_integrity_after_records() {
        let temp_file = NamedTempFile::new().unwrap();
        let temp_path = temp_file.path().to_path_buf();
        
        let mut recorder = EvidenceRecorder::new(&temp_path).unwrap();
        recorder.record_gate_decision(GateDecision::Allow, "tool1", "permitted").unwrap();
        recorder.record_gate_decision(GateDecision::Blocked, "tool2", "denied").unwrap();

        let valid = recorder.verify_ledger_integrity().unwrap();
        assert!(valid);
    }

    #[test]
    fn test_packet_id_counter_increment() {
        let temp_file = NamedTempFile::new().unwrap();
        let temp_path = temp_file.path().to_path_buf();
        
        let mut recorder = EvidenceRecorder::new(&temp_path).unwrap();
        assert_eq!(recorder.get_packet_id_counter(), 0);

        recorder.record_gate_decision(GateDecision::Blocked, "tool", "reason").unwrap();
        assert_eq!(recorder.get_packet_id_counter(), 1);

        recorder.record_gate_decision(GateDecision::Blocked, "tool", "reason").unwrap();
        assert_eq!(recorder.get_packet_id_counter(), 2);
    }

    #[test]
    fn test_reset_packet_counter() {
        let temp_file = NamedTempFile::new().unwrap();
        let temp_path = temp_file.path().to_path_buf();
        
        let mut recorder = EvidenceRecorder::new(&temp_path).unwrap();
        recorder.record_gate_decision(GateDecision::Blocked, "tool", "reason").unwrap();
        assert_eq!(recorder.get_packet_id_counter(), 1);

        recorder.reset_packet_counter();
        assert_eq!(recorder.get_packet_id_counter(), 0);
    }
}
