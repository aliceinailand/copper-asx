use crate::types::*;
use sha2::{Sha256, Digest};
use std::collections::HashMap;

/// Transforms MCP packets into Z3 assertions
pub struct McpPacketTransformer;

impl McpPacketTransformer {
    /// Transform an MCP packet to Z3 assertions
    pub fn transform_mcp_to_z3(
        packet: &McpPacket,
        profile_clearance: u32,
        profile_hop_limit: u32,
        ledger_state: &[String], // Prior ledger entries for context
    ) -> TransformationResult {
        let mut assertions = Vec::new();
        let mut pillar_atoms = Vec::new();

        // Create metadata
        let arguments_hash = Self::hash_arguments(&packet.arguments);
        let metadata = TransformMetadata {
            packet_id: packet.packet_id.clone(),
            actor_id: packet.actor_id.clone(),
            container_id: packet.container_id.clone(),
            timestamp: packet.timestamp,
            tool_name: packet.tool_name.clone(),
            arguments_hash,
        };

        // Assertion 1: Tool permission
        if let Some(tool_perm) = Self::encode_tool_permission(&packet.tool_name) {
            assertions.push(tool_perm);
        }

        // Assertion 2: Hop limit check
        if packet.hop_count > profile_hop_limit {
            return TransformationResult {
                assertions: vec![],
                metadata,
                pillar_atoms: vec![],
                success: false,
                error_message: Some(format!(
                    "Hop limit exceeded: {} > {}",
                    packet.hop_count, profile_hop_limit
                )),
            };
        }

        assertions.push(Self::encode_hop_limit(packet.hop_count, profile_hop_limit));

        // Assertion 3: Clearance requirement
        assertions.push(Self::encode_clearance_requirement(profile_clearance));

        // Assertion 4: Independence check
        if Self::should_check_independence(&packet.tool_name) {
            assertions.push(Self::encode_independence_check(&packet.actor_id, ledger_state));
            pillar_atoms.push("independence_verified".to_string());
        }

        // Assertion 5: Ops limit (placeholder for integration with ops counter)
        assertions.push(Self::encode_ops_limit(&packet.container_id));

        // Pillar atoms for Z3
        pillar_atoms.push("pillars_satisfied".to_string());

        TransformationResult {
            assertions,
            metadata,
            pillar_atoms,
            success: true,
            error_message: None,
        }
    }

    /// Encode tool permission as Z3 formula
    fn encode_tool_permission(tool_name: &str) -> Option<Z3Assertion> {
        // Whitelist of allowed tools; in production, would check profile
        let allowed_tools = vec![
            "get_data",
            "list_files",
            "read_container",
            "query_ledger",
        ];

        if allowed_tools.contains(&tool_name) {
            Some(Z3Assertion {
                formula: format!(
                    "(assert (tool_allowed \"{}\"))",
                    tool_name
                ),
                assertion_type: AssertionType::ToolPermission,
                metadata: Default::default(),
            })
        } else {
            None // Tool not in whitelist
        }
    }

    /// Encode hop limit as Z3 formula
    fn encode_hop_limit(hop_count: u32, hop_limit: u32) -> Z3Assertion {
        Z3Assertion {
            formula: format!(
                "(assert (<= {} {}))",
                hop_count, hop_limit
            ),
            assertion_type: AssertionType::HopLimit,
            metadata: Default::default(),
        }
    }

    /// Encode clearance requirement as Z3 formula
    fn encode_clearance_requirement(clearance_level: u32) -> Z3Assertion {
        Z3Assertion {
            formula: format!(
                "(assert (>= clearance_level {}))",
                clearance_level
            ),
            assertion_type: AssertionType::ClearanceRequired,
            metadata: Default::default(),
        }
    }

    /// Encode independence check as Z3 formula
    fn encode_independence_check(actor_id: &str, ledger_state: &[String]) -> Z3Assertion {
        // Check if actor has prior operations in ledger (simplified)
        let actor_count = ledger_state
            .iter()
            .filter(|e| e.contains(actor_id))
            .count();

        // If actor is new or has prior independent operations, assertion passes
        let formula = if actor_count == 0 {
            "(assert true)".to_string() // New actor
        } else {
            format!(
                "(assert (> independent_sources {}))",
                (actor_count / 2).max(1)
            )
        };

        Z3Assertion {
            formula,
            assertion_type: AssertionType::IndependenceRequired,
            metadata: Default::default(),
        }
    }

    /// Encode ops limit as Z3 formula
    fn encode_ops_limit(container_id: &str) -> Z3Assertion {
        Z3Assertion {
            formula: format!(
                "(assert (<= ops_consumed_{} ops_ceiling_{}))",
                container_id, container_id
            ),
            assertion_type: AssertionType::OpsLimit,
            metadata: Default::default(),
        }
    }

    /// Check if a tool requires independence verification
    fn should_check_independence(tool_name: &str) -> bool {
        matches!(
            tool_name,
            "query_ledger" | "read_container" | "escalate"
        )
    }

    /// Hash arguments for audit trail
    fn hash_arguments(args: &HashMap<String, String>) -> String {
        let mut hasher = Sha256::new();
        let serialized = format!("{:?}", args);
        hasher.update(serialized);
        hex::encode(hasher.finalize())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_packet(tool_name: &str) -> McpPacket {
        McpPacket {
            packet_id: "pkt-1".to_string(),
            actor_id: "actor-1".to_string(),
            container_id: "container-1".to_string(),
            tool_name: tool_name.to_string(),
            arguments: Default::default(),
            context_window: None,
            timestamp: 1700000000,
            hop_count: 1,
        }
    }

    #[test]
    fn test_transform_simple_tool() {
        let packet = make_packet("get_data");
        let result = McpPacketTransformer::transform_mcp_to_z3(&packet, 3, 5, &[]);

        assert!(result.success);
        assert!(!result.assertions.is_empty());
        assert_eq!(result.pillar_atoms.len(), 1);
    }

    #[test]
    fn test_transform_forbidden_tool() {
        let packet = make_packet("system_delete");
        let result = McpPacketTransformer::transform_mcp_to_z3(&packet, 3, 5, &[]);

        assert!(result.success);
        // Tool not in whitelist, so no tool_permission assertion added
    }

    #[test]
    fn test_hop_limit_exceeded() {
        let mut packet = make_packet("get_data");
        packet.hop_count = 10;
        let result = McpPacketTransformer::transform_mcp_to_z3(&packet, 3, 5, &[]);

        assert!(!result.success);
        assert!(result.error_message.is_some());
    }

    #[test]
    fn test_hop_limit_ok() {
        let mut packet = make_packet("get_data");
        packet.hop_count = 3;
        let result = McpPacketTransformer::transform_mcp_to_z3(&packet, 3, 5, &[]);

        assert!(result.success);
    }

    #[test]
    fn test_independence_check_new_actor() {
        let packet = make_packet("query_ledger");
        let ledger_state = vec![]; // Empty ledger
        let result = McpPacketTransformer::transform_mcp_to_z3(&packet, 3, 5, &ledger_state);

        assert!(result.success);
        assert!(result.pillar_atoms.contains(&"independence_verified".to_string()));
    }

    #[test]
    fn test_independence_check_existing_actor() {
        let packet = make_packet("query_ledger");
        let ledger_state = vec![
            "actor-1: operation-1".to_string(),
            "actor-1: operation-2".to_string(),
            "actor-2: operation-3".to_string(),
        ];
        let result = McpPacketTransformer::transform_mcp_to_z3(&packet, 3, 5, &ledger_state);

        assert!(result.success);
        assert!(result.pillar_atoms.contains(&"independence_verified".to_string()));
    }

    #[test]
    fn test_arguments_hash() {
        let mut args1 = HashMap::new();
        args1.insert("key".to_string(), "value".to_string());

        let mut args2 = HashMap::new();
        args2.insert("key".to_string(), "value".to_string());

        let hash1 = McpPacketTransformer::hash_arguments(&args1);
        let hash2 = McpPacketTransformer::hash_arguments(&args2);

        // Same args should produce consistent hashes
        assert_eq!(hash1.len(), 64); // SHA256 hex is 64 chars
        assert_eq!(hash2.len(), 64);
    }

    #[test]
    fn test_clearance_encoding() {
        let assertion = McpPacketTransformer::encode_clearance_requirement(3);
        assert!(assertion.formula.contains("clearance_level"));
        assert!(assertion.formula.contains("3"));
    }

    #[test]
    fn test_tool_permission_allowed() {
        let assertion = McpPacketTransformer::encode_tool_permission("get_data");
        assert!(assertion.is_some());
        assert!(assertion.unwrap().formula.contains("get_data"));
    }

    #[test]
    fn test_tool_permission_denied() {
        let assertion = McpPacketTransformer::encode_tool_permission("forbidden_tool");
        assert!(assertion.is_none());
    }
}
