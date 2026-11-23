use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Represents an MCP packet received from the user
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct McpPacket {
    pub packet_id: String,
    pub actor_id: String,
    pub container_id: String,
    pub tool_name: String,
    pub arguments: HashMap<String, String>,
    pub context_window: Option<String>,
    pub timestamp: i64,
    pub hop_count: u32,
}

/// Represents transformation metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TransformMetadata {
    pub packet_id: String,
    pub actor_id: String,
    pub container_id: String,
    pub timestamp: i64,
    pub tool_name: String,
    pub arguments_hash: String,
}

/// Represents a Z3 assertion to be evaluated
#[derive(Debug, Clone)]
pub struct Z3Assertion {
    pub formula: String,
    pub assertion_type: AssertionType,
    pub metadata: HashMap<String, String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AssertionType {
    ToolPermission,
    HopLimit,
    ClearanceRequired,
    IndependenceRequired,
    OpsLimit,
    CrossContainerAccess,
}

/// Represents the result of Z3 transformation
#[derive(Debug, Clone)]
pub struct TransformationResult {
    pub assertions: Vec<Z3Assertion>,
    pub metadata: TransformMetadata,
    pub pillar_atoms: Vec<String>,
    pub success: bool,
    pub error_message: Option<String>,
}

/// Represents a Z3 decision/verdict
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum ZVerdict {
    Allow,
    Deny,
    Escalate,
}

/// Represents proof or contradiction from Z3
#[derive(Debug, Clone)]
pub enum ZProof {
    Witness(String),     // Satisfiable proof
    UnsatCore(String),   // Unsatisfiable core (contradiction)
}

/// Gate verdict with proof and ledger metadata
#[derive(Debug, Clone)]
pub struct GateVerdict {
    pub verdict: ZVerdict,
    pub proof: ZProof,
    pub pillars_used: Vec<String>,
    pub gaps_detected: Vec<String>,
    pub actor_id: String,
    pub timestamp: i64,
    pub container_id: String,
    pub ops_delta: i64,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_mcp_packet_creation() {
        let mut args = HashMap::new();
        args.insert("tool".to_string(), "test_tool".to_string());

        let packet = McpPacket {
            packet_id: "pkt-1".to_string(),
            actor_id: "actor-1".to_string(),
            container_id: "container-1".to_string(),
            tool_name: "get_data".to_string(),
            arguments: args,
            context_window: Some("test context".to_string()),
            timestamp: 1700000000,
            hop_count: 1,
        };

        assert_eq!(packet.actor_id, "actor-1");
        assert_eq!(packet.tool_name, "get_data");
        assert_eq!(packet.hop_count, 1);
    }

    #[test]
    fn test_z3_assertion_creation() {
        let mut metadata = HashMap::new();
        metadata.insert("rule".to_string(), "tool_permission".to_string());

        let assertion = Z3Assertion {
            formula: "(and (tool_allowed \"get_data\") (clearance_level >= 3))".to_string(),
            assertion_type: AssertionType::ToolPermission,
            metadata,
        };

        assert_eq!(assertion.assertion_type, AssertionType::ToolPermission);
        assert!(assertion.formula.contains("tool_allowed"));
    }

    #[test]
    fn test_gate_verdict_creation() {
        let verdict = GateVerdict {
            verdict: ZVerdict::Allow,
            proof: ZProof::Witness("sat: all constraints satisfied".to_string()),
            pillars_used: vec!["pillar-1".to_string(), "pillar-2".to_string()],
            gaps_detected: vec![],
            actor_id: "actor-1".to_string(),
            timestamp: 1700000000,
            container_id: "container-1".to_string(),
            ops_delta: 1,
        };

        assert_eq!(verdict.verdict, ZVerdict::Allow);
        assert_eq!(verdict.pillars_used.len(), 2);
    }

    #[test]
    fn test_transformation_result() {
        let result = TransformationResult {
            assertions: vec![],
            metadata: TransformMetadata {
                packet_id: "pkt-1".to_string(),
                actor_id: "actor-1".to_string(),
                container_id: "container-1".to_string(),
                timestamp: 1700000000,
                tool_name: "get_data".to_string(),
                arguments_hash: "abc123".to_string(),
            },
            pillar_atoms: vec!["pillars_satisfied".to_string()],
            success: true,
            error_message: None,
        };

        assert!(result.success);
        assert_eq!(result.pillar_atoms.len(), 1);
    }
}
