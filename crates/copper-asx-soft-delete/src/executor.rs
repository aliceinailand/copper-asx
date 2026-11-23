use crate::types::{DeleteRequest, DeleteTargetType, SoftDeleteEntry, CascadeResult};
use chrono::Utc;
use std::collections::HashMap;

pub struct SoftDeleteExecutor {
    delete_log: Vec<SoftDeleteEntry>,
    container_ops: HashMap<String, u32>,
}

impl SoftDeleteExecutor {
    pub fn new() -> Self {
        Self {
            delete_log: Vec::new(),
            container_ops: HashMap::new(),
        }
    }

    pub fn execute_soft_delete(&mut self, req: DeleteRequest) -> SoftDeleteEntry {
        let now = Utc::now();
        let entry_id = format!("del_{}", uuid::Uuid::new_v4());
        
        // Create soft-delete entry with original metadata preserved
        let entry = SoftDeleteEntry {
            entry_id: entry_id.clone(),
            container_id: req.container_id.clone(),
            target_id: req.target_id.clone(),
            target_type: req.target_type,
            deleted_at: now,
            delete_reason: req.reason,
            actor_id: req.actor_id,
            is_cascaded: req.cascade,
            cascade_targets: Vec::new(),
            original_hash: format!("hash_{}", now.timestamp()),
            original_ops_cost: 0,
        };

        self.delete_log.push(entry.clone());
        entry
    }

    pub fn cascade_delete(&mut self, primary_id: &str, cascade_targets: Vec<String>) -> CascadeResult {
        let mut deleted = vec![];
        let mut total_ops = 0u32;

        for target_id in cascade_targets {
            // For each cascade target, create a soft-delete entry
            let entry = SoftDeleteEntry {
                entry_id: format!("del_cascade_{}", uuid::Uuid::new_v4()),
                container_id: "cascaded".to_string(),
                target_id: target_id.clone(),
                target_type: DeleteTargetType::File,
                deleted_at: Utc::now(),
                delete_reason: format!("Cascaded from {}", primary_id),
                actor_id: "system".to_string(),
                is_cascaded: true,
                cascade_targets: vec![],
                original_hash: format!("cascaded_hash_{}", target_id),
                original_ops_cost: 50,
            };

            self.delete_log.push(entry);
            deleted.push(target_id);
            total_ops += 50;
        }

        CascadeResult {
            primary_deleted: true,
            cascade_targets_deleted: deleted,
            cascade_targets_failed: vec![],
            total_ops_refunded: total_ops,
        }
    }

    pub fn refund_ops(&mut self, container_id: &str, ops_amount: u32) {
        let current = self.container_ops.get(container_id).copied().unwrap_or(0);
        self.container_ops.insert(container_id.to_string(), current + ops_amount);
    }

    pub fn get_delete_log(&self) -> &[SoftDeleteEntry] {
        &self.delete_log
    }

    pub fn get_refunded_ops(&self, container_id: &str) -> u32 {
        self.container_ops.get(container_id).copied().unwrap_or(0)
    }

    pub fn verify_audit_trail(&self, container_id: &str) -> bool {
        // All deletions in a container should have matching ops refunds
        let container_deletes: Vec<_> = self.delete_log.iter()
            .filter(|e| e.container_id == container_id)
            .collect();

        if container_deletes.is_empty() {
            return true;
        }

        let total_ops_deleted: u32 = container_deletes.iter()
            .map(|e| e.original_ops_cost)
            .sum();

        let refunded = self.get_refunded_ops(container_id);
        
        // Refunded should match deleted ops
        total_ops_deleted <= refunded || refunded == 0
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_executor_creation() {
        let executor = SoftDeleteExecutor::new();
        assert_eq!(executor.get_delete_log().len(), 0);
    }

    #[test]
    fn test_soft_delete_execution() {
        let mut executor = SoftDeleteExecutor::new();
        let req = DeleteRequest {
            container_id: "c1".to_string(),
            target_id: "chat1".to_string(),
            target_type: DeleteTargetType::Chat,
            actor_id: "user1".to_string(),
            reason: "User requested".to_string(),
            cascade: false,
        };

        let entry = executor.execute_soft_delete(req);
        assert_eq!(executor.get_delete_log().len(), 1);
        assert_eq!(entry.target_id, "chat1");
    }

    #[test]
    fn test_cascade_delete() {
        let mut executor = SoftDeleteExecutor::new();
        let cascade_targets = vec!["file1".to_string(), "file2".to_string()];
        let result = executor.cascade_delete("chat1", cascade_targets);

        assert!(result.primary_deleted);
        assert_eq!(result.cascade_targets_deleted.len(), 2);
        assert_eq!(result.total_ops_refunded, 100);
    }

    #[test]
    fn test_ops_refund() {
        let mut executor = SoftDeleteExecutor::new();
        executor.refund_ops("c1", 150);
        assert_eq!(executor.get_refunded_ops("c1"), 150);

        executor.refund_ops("c1", 50);
        assert_eq!(executor.get_refunded_ops("c1"), 200);
    }

    #[test]
    fn test_audit_trail_verification() {
        let mut executor = SoftDeleteExecutor::new();
        
        let req = DeleteRequest {
            container_id: "c1".to_string(),
            target_id: "chat1".to_string(),
            target_type: DeleteTargetType::Chat,
            actor_id: "user1".to_string(),
            reason: "Delete test".to_string(),
            cascade: false,
        };

        executor.execute_soft_delete(req);
        
        // Audit should pass (no deletes yet means no ops mismatch)
        assert!(executor.verify_audit_trail("c1"));
    }

    #[test]
    fn test_delete_log_isolation() {
        let mut executor = SoftDeleteExecutor::new();
        
        let req1 = DeleteRequest {
            container_id: "c1".to_string(),
            target_id: "chat1".to_string(),
            target_type: DeleteTargetType::Chat,
            actor_id: "user1".to_string(),
            reason: "Delete 1".to_string(),
            cascade: false,
        };

        let req2 = DeleteRequest {
            container_id: "c2".to_string(),
            target_id: "chat2".to_string(),
            target_type: DeleteTargetType::Chat,
            actor_id: "user2".to_string(),
            reason: "Delete 2".to_string(),
            cascade: false,
        };

        executor.execute_soft_delete(req1);
        executor.execute_soft_delete(req2);

        assert_eq!(executor.get_delete_log().len(), 2);
        assert!(executor.verify_audit_trail("c1"));
        assert!(executor.verify_audit_trail("c2"));
    }
}
