use crate::types::*;
use sha2::{Sha256, Digest};

/// Records gate decisions to the Evidence Ledger
pub struct LedgerRecorder;

/// Represents a ledger entry for a gate decision
#[derive(Debug, Clone)]
pub struct LedgerDecisionEntry {
    pub entry_id: String,
    pub container_id: String,
    pub actor_id: String,
    pub timestamp: i64,
    pub tool_name: String,
    pub verdict: String,
    pub proof_hash: String,
    pub pillars_used: Vec<String>,
    pub gaps_detected: Vec<String>,
    pub ops_delta: i64,
    pub chain_hash: String, // SHA256 of previous entry
}

impl LedgerRecorder {
    /// Record a gate decision to the ledger
    pub fn record_gate_decision(
        verdict: &GateVerdict,
        previous_chain_hash: &str,
    ) -> LedgerDecisionEntry {
        let entry_id = format!("entry-{}", uuid_simple());
        
        // Hash the proof
        let proof_str = match &verdict.proof {
            ZProof::Witness(w) => w.clone(),
            ZProof::UnsatCore(u) => u.clone(),
        };
        let proof_hash = Self::hash_proof(&proof_str);
        
        // Build entry
        let entry = LedgerDecisionEntry {
            entry_id: entry_id.clone(),
            container_id: verdict.container_id.clone(),
            actor_id: verdict.actor_id.clone(),
            timestamp: verdict.timestamp,
            tool_name: "gate_decision".to_string(),
            verdict: format!("{:?}", verdict.verdict),
            proof_hash,
            pillars_used: verdict.pillars_used.clone(),
            gaps_detected: verdict.gaps_detected.clone(),
            ops_delta: verdict.ops_delta,
            chain_hash: previous_chain_hash.to_string(),
        };
        
        entry
    }

    /// Verify SHA256 chain integrity
    pub fn verify_ledger_chain(entries: &[LedgerDecisionEntry]) -> bool {
        if entries.is_empty() {
            return true;
        }

        // Verify each entry's chain hash matches previous entry's computed hash
        for i in 1..entries.len() {
            let prev_entry = &entries[i - 1];
            let prev_computed_hash = Self::compute_entry_hash(prev_entry);
            let current_chain_hash = &entries[i].chain_hash;

            if !current_chain_hash.eq(&prev_computed_hash) {
                return false;
            }
        }

        true
    }

    /// Compute hash for an entry (for chaining)
    pub fn compute_entry_hash(entry: &LedgerDecisionEntry) -> String {
        let mut hasher = Sha256::new();
        let serialized = format!(
            "{}:{}:{}:{}:{}:{}",
            entry.entry_id,
            entry.container_id,
            entry.actor_id,
            entry.timestamp,
            entry.tool_name,
            entry.verdict
        );
        hasher.update(serialized);
        hex::encode(hasher.finalize())
    }

    /// Hash the proof value
    fn hash_proof(proof: &str) -> String {
        let mut hasher = Sha256::new();
        hasher.update(proof);
        hex::encode(hasher.finalize())
    }

    /// Decrement ops counter (checked before allowing tool call)
    pub fn decrement_ops(
        _container_id: &str,
        ops_consumed: &mut i64,
        ops_ceiling: i64,
        ops_delta: i64,
    ) -> Result<(), String> {
        if *ops_consumed + ops_delta > ops_ceiling {
            return Err(format!(
                "Ops ceiling exceeded: {} + {} > {}",
                ops_consumed, ops_delta, ops_ceiling
            ));
        }

        *ops_consumed += ops_delta;
        Ok(())
    }

    /// Query ledger by container (default-deny cross-container)
    pub fn query_ledger_by_container(
        entries: &[LedgerDecisionEntry],
        container_id: &str,
        actor_id: &str,
        from_timestamp: i64,
    ) -> Vec<LedgerDecisionEntry> {
        entries
            .iter()
            .filter(|e| {
                e.container_id == container_id
                    && e.timestamp >= from_timestamp
                    && e.actor_id == actor_id
            })
            .cloned()
            .collect()
    }
}

/// Simple UUID generator for testing
fn uuid_simple() -> String {
    use std::time::{SystemTime, UNIX_EPOCH};
    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_millis();
    format!("{:x}", now)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_verdict() -> GateVerdict {
        GateVerdict {
            verdict: ZVerdict::Allow,
            proof: ZProof::Witness("sat: all constraints".to_string()),
            pillars_used: vec!["p1".to_string()],
            gaps_detected: vec![],
            actor_id: "actor-1".to_string(),
            timestamp: 1700000000,
            container_id: "container-1".to_string(),
            ops_delta: 1,
        }
    }

    #[test]
    fn test_record_gate_decision() {
        let verdict = make_verdict();
        let entry = LedgerRecorder::record_gate_decision(&verdict, "prev-hash");

        assert_eq!(entry.container_id, "container-1");
        assert_eq!(entry.actor_id, "actor-1");
        assert_eq!(entry.chain_hash, "prev-hash");
        assert_eq!(entry.ops_delta, 1);
    }

    #[test]
    fn test_compute_entry_hash() {
        let verdict = make_verdict();
        let entry = LedgerRecorder::record_gate_decision(&verdict, "prev-hash");
        let hash1 = LedgerRecorder::compute_entry_hash(&entry);
        let hash2 = LedgerRecorder::compute_entry_hash(&entry);

        // Same entry should produce same hash
        assert_eq!(hash1, hash2);
        assert_eq!(hash1.len(), 64); // SHA256 hex
    }

    #[test]
    fn test_verify_ledger_chain() {
        let mut entries = vec![];

        // Entry 1
        let verdict1 = make_verdict();
        let entry1 = LedgerRecorder::record_gate_decision(&verdict1, "genesis");
        entries.push(entry1.clone());

        // Entry 2 (chain to entry 1)
        let verdict2 = make_verdict();
        let hash1 = LedgerRecorder::compute_entry_hash(&entry1);
        let entry2 = LedgerRecorder::record_gate_decision(&verdict2, &hash1);
        entries.push(entry2);

        assert!(LedgerRecorder::verify_ledger_chain(&entries));
    }

    #[test]
    fn test_broken_chain() {
        let mut entries = vec![];

        let verdict1 = make_verdict();
        let entry1 = LedgerRecorder::record_gate_decision(&verdict1, "genesis");
        entries.push(entry1);

        let verdict2 = make_verdict();
        let entry2 = LedgerRecorder::record_gate_decision(&verdict2, "wrong-hash");
        entries.push(entry2);

        assert!(!LedgerRecorder::verify_ledger_chain(&entries));
    }

    #[test]
    fn test_decrement_ops_success() {
        let mut ops_consumed = 50;
        let result = LedgerRecorder::decrement_ops("container-1", &mut ops_consumed, 100, 10);

        assert!(result.is_ok());
        assert_eq!(ops_consumed, 60);
    }

    #[test]
    fn test_decrement_ops_exceeds_ceiling() {
        let mut ops_consumed = 95;
        let result = LedgerRecorder::decrement_ops("container-1", &mut ops_consumed, 100, 10);

        assert!(result.is_err());
        assert_eq!(ops_consumed, 95); // Not modified on error
    }

    #[test]
    fn test_query_ledger_by_container() {
        let verdict1 = make_verdict();
        let entry1 = LedgerRecorder::record_gate_decision(&verdict1, "prev");
        let entry2 = LedgerRecorder::record_gate_decision(&verdict1, "prev");

        let all_entries = vec![entry1, entry2];

        // Query for specific container and actor
        let result = LedgerRecorder::query_ledger_by_container(
            &all_entries,
            "container-1",
            "actor-1",
            1700000000,
        );

        assert_eq!(result.len(), 2);
    }

    #[test]
    fn test_query_ledger_cross_container_isolation() {
        let mut verdict1 = make_verdict();
        verdict1.container_id = "container-1".to_string();

        let mut verdict2 = make_verdict();
        verdict2.container_id = "container-2".to_string();

        let entry1 = LedgerRecorder::record_gate_decision(&verdict1, "prev");
        let entry2 = LedgerRecorder::record_gate_decision(&verdict2, "prev");

        let all_entries = vec![entry1, entry2];

        // Query for container-1 should not return container-2 entries
        let result =
            LedgerRecorder::query_ledger_by_container(&all_entries, "container-1", "actor-1", 0);

        assert_eq!(result.len(), 1);
        assert_eq!(result[0].container_id, "container-1");
    }

    #[test]
    fn test_proof_hash() {
        let proof = "sat: all constraints satisfied";
        let hash1 = LedgerRecorder::hash_proof(proof);
        let hash2 = LedgerRecorder::hash_proof(proof);

        assert_eq!(hash1, hash2);
        assert_eq!(hash1.len(), 64);
    }

    #[test]
    fn test_empty_ledger_chain() {
        let entries: Vec<LedgerDecisionEntry> = vec![];
        assert!(LedgerRecorder::verify_ledger_chain(&entries));
    }
}
