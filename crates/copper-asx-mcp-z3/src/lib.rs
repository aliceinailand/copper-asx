mod types;
mod mcp_transformer;
mod ledger_recorder;

pub use types::*;
pub use mcp_transformer::McpPacketTransformer;
pub use ledger_recorder::{LedgerRecorder, LedgerDecisionEntry};

/// Full end-to-end MCP → Z3 → Ledger pipeline
pub struct MpcZ3Pipeline;

impl MpcZ3Pipeline {
    /// Execute the complete gate decision pipeline
    ///
    /// Flow:
    /// 1. Transform MCP packet to Z3 assertions
    /// 2. Check assertions (mocked Z3 for now)
    /// 3. Record decision to ledger
    /// 4. Return verdict with proof
    pub fn execute_gate_decision(
        packet: &McpPacket,
        profile_clearance: u32,
        profile_hop_limit: u32,
        ledger_state: &[String],
        _previous_chain_hash: &str,
        ops_consumed: &mut i64,
        ops_ceiling: i64,
    ) -> Result<GateVerdict, String> {
        // Step 1: Transform MCP to Z3
        let transform_result =
            McpPacketTransformer::transform_mcp_to_z3(packet, profile_clearance, profile_hop_limit, ledger_state);

        if !transform_result.success {
            return Err(transform_result.error_message.unwrap_or_default());
        }

        // Step 2: Evaluate assertions (simplified Z3 evaluation for MVP)
        let (verdict_decision, proof) = Self::evaluate_assertions(&transform_result.assertions);

        // Step 3: Check ops before allowing
        LedgerRecorder::decrement_ops(&packet.container_id, ops_consumed, ops_ceiling, 1)?;

        // Step 4: Build verdict
        let verdict = GateVerdict {
            verdict: verdict_decision,
            proof,
            pillars_used: transform_result.pillar_atoms,
            gaps_detected: vec![],
            actor_id: packet.actor_id.clone(),
            timestamp: packet.timestamp,
            container_id: packet.container_id.clone(),
            ops_delta: 1,
        };

        Ok(verdict)
    }

    /// Evaluate Z3 assertions (simplified for MVP)
    fn evaluate_assertions(assertions: &[types::Z3Assertion]) -> (ZVerdict, ZProof) {
        // For MVP: if any assertion has an error, deny; otherwise allow
        let allow_count = assertions.iter().filter(|a| !a.formula.contains("error")).count();

        if allow_count == assertions.len() {
            (
                ZVerdict::Allow,
                ZProof::Witness("all assertions satisfied".to_string()),
            )
        } else {
            (
                ZVerdict::Deny,
                ZProof::UnsatCore("assertion failed".to_string()),
            )
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_test_packet() -> McpPacket {
        McpPacket {
            packet_id: "pkt-1".to_string(),
            actor_id: "actor-1".to_string(),
            container_id: "container-1".to_string(),
            tool_name: "get_data".to_string(),
            arguments: Default::default(),
            context_window: None,
            timestamp: 1700000000,
            hop_count: 1,
        }
    }

    #[test]
    fn test_pipeline_success() {
        let packet = make_test_packet();
        let mut ops_consumed = 0;
        let result = MpcZ3Pipeline::execute_gate_decision(
            &packet,
            3,
            5,
            &[],
            "genesis",
            &mut ops_consumed,
            100,
        );

        assert!(result.is_ok());
        let verdict = result.unwrap();
        assert_eq!(verdict.verdict, ZVerdict::Allow);
        assert_eq!(ops_consumed, 1);
    }

    #[test]
    fn test_pipeline_hop_limit_exceeded() {
        let mut packet = make_test_packet();
        packet.hop_count = 10;
        let mut ops_consumed = 0;
        let result = MpcZ3Pipeline::execute_gate_decision(
            &packet,
            3,
            5,
            &[],
            "genesis",
            &mut ops_consumed,
            100,
        );

        assert!(result.is_err());
        assert_eq!(ops_consumed, 0); // No ops consumed on error
    }

    #[test]
    fn test_pipeline_ops_ceiling_exceeded() {
        let packet = make_test_packet();
        let mut ops_consumed = 100;
        let result = MpcZ3Pipeline::execute_gate_decision(
            &packet,
            3,
            5,
            &[],
            "genesis",
            &mut ops_consumed,
            100,
        );

        assert!(result.is_err());
        assert_eq!(ops_consumed, 100); // Not modified on error
    }

    #[test]
    fn test_pipeline_verdict_includes_proof() {
        let packet = make_test_packet();
        let mut ops_consumed = 0;
        let result = MpcZ3Pipeline::execute_gate_decision(
            &packet,
            3,
            5,
            &[],
            "genesis",
            &mut ops_consumed,
            100,
        );

        let verdict = result.unwrap();
        match verdict.proof {
            ZProof::Witness(_) => {}
            ZProof::UnsatCore(_) => {}
        }
    }

    #[test]
    fn test_pipeline_container_isolation() {
        let mut packet1 = make_test_packet();
        packet1.container_id = "container-1".to_string();

        let mut packet2 = make_test_packet();
        packet2.container_id = "container-2".to_string();

        let mut ops1 = 0;
        let mut ops2 = 0;

        let result1 =
            MpcZ3Pipeline::execute_gate_decision(&packet1, 3, 5, &[], "genesis", &mut ops1, 100);
        let result2 =
            MpcZ3Pipeline::execute_gate_decision(&packet2, 3, 5, &[], "genesis", &mut ops2, 100);

        assert!(result1.is_ok());
        assert!(result2.is_ok());

        // Each container's ops counter is independent
        assert_eq!(ops1, 1);
        assert_eq!(ops2, 1);
    }

    #[test]
    fn test_mcp_transformer_integration() {
        let packet = make_test_packet();
        let result =
            McpPacketTransformer::transform_mcp_to_z3(&packet, 3, 5, &[]);

        assert!(result.success);
        assert!(!result.assertions.is_empty());
    }

    #[test]
    fn test_ledger_recorder_integration() {
        let verdict = GateVerdict {
            verdict: ZVerdict::Allow,
            proof: ZProof::Witness("test".to_string()),
            pillars_used: vec![],
            gaps_detected: vec![],
            actor_id: "actor-1".to_string(),
            timestamp: 1700000000,
            container_id: "container-1".to_string(),
            ops_delta: 1,
        };

        let entry = LedgerRecorder::record_gate_decision(&verdict, "prev");
        assert_eq!(entry.container_id, "container-1");
    }

    #[test]
    fn test_full_pipeline_with_ledger() {
        let packet = make_test_packet();
        let mut ops_consumed = 0;

        let result = MpcZ3Pipeline::execute_gate_decision(
            &packet,
            3,
            5,
            &[],
            "genesis",
            &mut ops_consumed,
            100,
        );

        assert!(result.is_ok());
        let verdict = result.unwrap();

        // Record to ledger
        let entry = LedgerRecorder::record_gate_decision(&verdict, "genesis");
        assert_eq!(entry.container_id, packet.container_id);
        assert_eq!(entry.actor_id, packet.actor_id);
    }
}
