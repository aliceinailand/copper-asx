use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct DeleteRequest {
    pub container_id: String,
    pub target_id: String,
    pub target_type: DeleteTargetType,
    pub actor_id: String,
    pub reason: String,
    pub cascade: bool,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq)]
pub enum DeleteTargetType {
    Chat,
    File,
    Construct,
    Container,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct SoftDeleteEntry {
    pub entry_id: String,
    pub container_id: String,
    pub target_id: String,
    pub target_type: DeleteTargetType,
    pub deleted_at: DateTime<Utc>,
    pub delete_reason: String,
    pub actor_id: String,
    pub is_cascaded: bool,
    pub cascade_targets: Vec<String>,
    pub original_hash: String,
    pub original_ops_cost: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct CascadeResult {
    pub primary_deleted: bool,
    pub cascade_targets_deleted: Vec<String>,
    pub cascade_targets_failed: Vec<String>,
    pub total_ops_refunded: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct RecoveryProposal {
    pub entry_id: String,
    pub container_id: String,
    pub target_id: String,
    pub deleted_at: DateTime<Utc>,
    pub can_recover: bool,
    pub reason_unrecoverable: Option<String>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_delete_request_creation() {
        let req = DeleteRequest {
            container_id: "c1".to_string(),
            target_id: "chat1".to_string(),
            target_type: DeleteTargetType::Chat,
            actor_id: "user1".to_string(),
            reason: "User requested deletion".to_string(),
            cascade: true,
        };
        assert_eq!(req.target_type, DeleteTargetType::Chat);
        assert!(req.cascade);
    }

    #[test]
    fn test_soft_delete_entry_creation() {
        let now = Utc::now();
        let entry = SoftDeleteEntry {
            entry_id: "del1".to_string(),
            container_id: "c1".to_string(),
            target_id: "chat1".to_string(),
            target_type: DeleteTargetType::Chat,
            deleted_at: now,
            delete_reason: "User requested".to_string(),
            actor_id: "user1".to_string(),
            is_cascaded: false,
            cascade_targets: vec![],
            original_hash: "abc123".to_string(),
            original_ops_cost: 100,
        };
        assert_eq!(entry.target_type, DeleteTargetType::Chat);
        assert!(!entry.is_cascaded);
        assert_eq!(entry.cascade_targets.len(), 0);
    }

    #[test]
    fn test_cascade_result_creation() {
        let result = CascadeResult {
            primary_deleted: true,
            cascade_targets_deleted: vec!["file1".to_string(), "file2".to_string()],
            cascade_targets_failed: vec![],
            total_ops_refunded: 250,
        };
        assert!(result.primary_deleted);
        assert_eq!(result.cascade_targets_deleted.len(), 2);
        assert_eq!(result.total_ops_refunded, 250);
    }

    #[test]
    fn test_recovery_proposal_creation() {
        let now = Utc::now();
        let proposal = RecoveryProposal {
            entry_id: "del1".to_string(),
            container_id: "c1".to_string(),
            target_id: "chat1".to_string(),
            deleted_at: now,
            can_recover: true,
            reason_unrecoverable: None,
        };
        assert!(proposal.can_recover);
        assert!(proposal.reason_unrecoverable.is_none());
    }
}
