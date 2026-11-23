use crate::GateResult;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::fs::{File, OpenOptions};
use std::io::{BufRead, BufReader, Write};
use std::path::Path;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum GateDecision {
    Allow,
    Reject,
    Blocked,
}

impl std::fmt::Display for GateDecision {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            GateDecision::Allow => write!(f, "allow"),
            GateDecision::Reject => write!(f, "reject"),
            GateDecision::Blocked => write!(f, "blocked"),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GateDecisionEvent {
    pub timestamp: DateTime<Utc>,
    pub event_type: String,
    pub tool_name: String,
    pub decision: GateDecision,
    pub reason: String,
    pub evidence_hash: String,
    pub prev_hash: String,
    pub packet_id: String,
}

pub struct EvidenceLedger {
    path: std::path::PathBuf,
    last_hash: String,
}

impl EvidenceLedger {
    pub fn new<P: AsRef<Path>>(path: P) -> GateResult<Self> {
        let path = path.as_ref().to_path_buf();

        if !path.parent().map(|p| p.exists()).unwrap_or(true) {
            std::fs::create_dir_all(path.parent().unwrap())?;
        }

        let last_hash = if path.exists() {
            Self::read_last_hash(&path)?
        } else {
            "0000000000000000000000000000000000000000000000000000000000000000".to_string()
        };

        Ok(EvidenceLedger { path, last_hash })
    }

    fn read_last_hash(path: &std::path::Path) -> GateResult<String> {
        let file = File::open(path)?;
        let reader = BufReader::new(file);
        let mut last_line = String::new();

        for line in reader.lines() {
            last_line = line?;
        }

        if last_line.is_empty() {
            return Ok("0000000000000000000000000000000000000000000000000000000000000000".to_string());
        }

        let event: GateDecisionEvent = serde_json::from_str(&last_line)
            .map_err(|e| crate::GateError::LedgerError(format!("parse error: {}", e)))?;

        Ok(event.evidence_hash)
    }

    pub fn record_decision(&mut self, decision: GateDecision, tool_name: &str, reason: &str, packet_id: &str) -> GateResult<()> {
        let prev_hash = self.last_hash.clone();
        let event = GateDecisionEvent {
            timestamp: Utc::now(),
            event_type: "gate_decision".to_string(),
            tool_name: tool_name.to_string(),
            decision: decision.clone(),
            reason: reason.to_string(),
            evidence_hash: String::new(),
            prev_hash: prev_hash.clone(),
            packet_id: packet_id.to_string(),
        };

        let evidence_text = format!(
            "{}|{}|{}|{}|{}|{}",
            event.timestamp.to_rfc3339(),
            event.tool_name,
            event.decision,
            event.reason,
            event.prev_hash,
            event.packet_id
        );

        let mut hasher = Sha256::new();
        hasher.update(evidence_text.as_bytes());
        let evidence_hash = format!("sha256:{}", hex::encode(hasher.finalize()));

        let mut event = event;
        event.evidence_hash = evidence_hash.clone();

        let mut file = OpenOptions::new()
            .create(true)
            .append(true)
            .open(&self.path)?;

        writeln!(file, "{}", serde_json::to_string(&event)?)?;

        self.last_hash = evidence_hash;

        Ok(())
    }

    pub fn verify_chain(&self) -> GateResult<bool> {
        if !self.path.exists() {
            return Ok(true);
        }

        let file = File::open(&self.path)?;
        let reader = BufReader::new(file);
        let mut prev_hash = "0000000000000000000000000000000000000000000000000000000000000000".to_string();

        for line in reader.lines() {
            let line = line?;
            if line.trim().is_empty() {
                continue;
            }

            let event: GateDecisionEvent = serde_json::from_str(&line)
                .map_err(|e| crate::GateError::LedgerError(format!("parse error: {}", e)))?;

            if event.prev_hash != prev_hash {
                return Ok(false);
            }

            let evidence_text = format!(
                "{}|{}|{}|{}|{}|{}",
                event.timestamp.to_rfc3339(),
                event.tool_name,
                event.decision,
                event.reason,
                event.prev_hash,
                event.packet_id
            );

            let mut hasher = Sha256::new();
            hasher.update(evidence_text.as_bytes());
            let computed_hash = format!("sha256:{}", hex::encode(hasher.finalize()));

            if computed_hash != event.evidence_hash {
                return Ok(false);
            }

            prev_hash = event.evidence_hash;
        }

        Ok(true)
    }

    pub fn get_entries_count(&self) -> GateResult<usize> {
        if !self.path.exists() {
            return Ok(0);
        }

        let file = File::open(&self.path)?;
        let reader = BufReader::new(file);
        let count = reader.lines().filter(|l| l.is_ok()).count();
        Ok(count)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::NamedTempFile;

    #[test]
    fn test_ledger_creation() {
        let temp_file = NamedTempFile::new().unwrap();
        let ledger = EvidenceLedger::new(temp_file.path()).unwrap();
        assert_eq!(ledger.last_hash, "0000000000000000000000000000000000000000000000000000000000000000");
    }

    #[test]
    fn test_record_decision() {
        let temp_file = NamedTempFile::new().unwrap();
        let temp_path = temp_file.path().to_path_buf();
        
        let mut ledger = EvidenceLedger::new(&temp_path).unwrap();
        ledger.record_decision(GateDecision::Blocked, "fetch_web", "clearance=false", "pkt-001").unwrap();

        let count = ledger.get_entries_count().unwrap();
        assert_eq!(count, 1);
    }

    #[test]
    fn test_chain_verification() {
        let temp_file = NamedTempFile::new().unwrap();
        let temp_path = temp_file.path().to_path_buf();
        
        let mut ledger = EvidenceLedger::new(&temp_path).unwrap();
        ledger.record_decision(GateDecision::Blocked, "tool1", "reason1", "pkt-001").unwrap();
        ledger.record_decision(GateDecision::Reject, "tool2", "reason2", "pkt-002").unwrap();

        let valid = ledger.verify_chain().unwrap();
        assert!(valid);
    }

    #[test]
    fn test_hash_chain_integrity() {
        let temp_file = NamedTempFile::new().unwrap();
        let temp_path = temp_file.path().to_path_buf();
        
        let mut ledger = EvidenceLedger::new(&temp_path).unwrap();
        ledger.record_decision(GateDecision::Blocked, "fetch", "denied", "pkt-001").unwrap();
        
        let first_hash = ledger.last_hash.clone();
        assert!(first_hash.starts_with("sha256:"));

        ledger.record_decision(GateDecision::Reject, "tool2", "gaps", "pkt-002").unwrap();
        
        let second_hash = ledger.last_hash.clone();
        assert!(second_hash.starts_with("sha256:"));
        assert_ne!(first_hash, second_hash);
    }
}
