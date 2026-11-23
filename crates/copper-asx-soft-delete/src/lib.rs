pub mod types;
pub mod executor;
pub mod fms_cascade;

pub use types::{DeleteRequest, DeleteTargetType, SoftDeleteEntry, CascadeResult, RecoveryProposal};
pub use executor::SoftDeleteExecutor;
pub use fms_cascade::{FmsErasureCascade, ErasurePolicy};

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_soft_delete_pipeline_complete() {
        let mut executor = SoftDeleteExecutor::new();
        
        let req = DeleteRequest {
            container_id: "c1".to_string(),
            target_id: "chat1".to_string(),
            target_type: DeleteTargetType::Chat,
            actor_id: "user1".to_string(),
            reason: "User deletion".to_string(),
            cascade: true,
        };
        
        let entry = executor.execute_soft_delete(req);
        assert_eq!(executor.get_delete_log().len(), 1);
        assert_eq!(entry.target_id, "chat1");
    }

    #[test]
    fn test_cascade_integration() {
        let mut executor = SoftDeleteExecutor::new();
        
        let req = DeleteRequest {
            container_id: "c1".to_string(),
            target_id: "chat1".to_string(),
            target_type: DeleteTargetType::Chat,
            actor_id: "user1".to_string(),
            reason: "Delete with cascade".to_string(),
            cascade: true,
        };
        
        executor.execute_soft_delete(req);
        
        let cascade_targets = vec!["file1".to_string(), "file2".to_string(), "file3".to_string()];
        let result = executor.cascade_delete("chat1", cascade_targets);
        
        assert!(result.primary_deleted);
        assert_eq!(result.cascade_targets_deleted.len(), 3);
        assert_eq!(result.total_ops_refunded, 150);
    }

    #[test]
    fn test_fms_erasure_with_soft_delete() {
        let mut cascade = FmsErasureCascade::new(ErasurePolicy::Immediate);
        
        cascade.register_seal("seal:file1".to_string(), vec!["base:chat1".to_string()]);
        cascade.register_seal("seal:file2".to_string(), vec!["base:chat1".to_string()]);
        
        let erased = cascade.cascade_erase("base:chat1");
        assert_eq!(erased.len(), 1);
        assert!(erased.contains(&"base:chat1".to_string()));
    }

    #[test]
    fn test_container_isolation_in_soft_delete() {
        let mut executor = SoftDeleteExecutor::new();
        
        let req1 = DeleteRequest {
            container_id: "c1".to_string(),
            target_id: "chat1".to_string(),
            target_type: DeleteTargetType::Chat,
            actor_id: "user1".to_string(),
            reason: "Delete from c1".to_string(),
            cascade: false,
        };
        
        let req2 = DeleteRequest {
            container_id: "c2".to_string(),
            target_id: "chat2".to_string(),
            target_type: DeleteTargetType::Chat,
            actor_id: "user2".to_string(),
            reason: "Delete from c2".to_string(),
            cascade: false,
        };
        
        executor.execute_soft_delete(req1);
        executor.execute_soft_delete(req2);
        
        assert_eq!(executor.get_delete_log().len(), 2);
        assert!(executor.verify_audit_trail("c1"));
        assert!(executor.verify_audit_trail("c2"));
    }

    #[test]
    fn test_audit_trail_immutability() {
        let executor = SoftDeleteExecutor::new();
        
        let log = executor.get_delete_log();
        assert_eq!(log.len(), 0);
    }

    #[test]
    fn test_delayed_erasure_policy() {
        let cascade = FmsErasureCascade::new(ErasurePolicy::Delayed { days: 30 });
        assert!(!cascade.should_erase_immediately());
        assert_eq!(cascade.get_erasure_delay_days(), Some(30));
    }

    #[test]
    fn test_cross_container_isolation_in_erasure() {
        let mut cascade = FmsErasureCascade::new(ErasurePolicy::Immediate);
        
        cascade.register_seal("c1:seal1".to_string(), vec!["c1:base".to_string()]);
        cascade.register_seal("c2:seal1".to_string(), vec!["c2:base".to_string()]);
        
        cascade.cascade_erase("c1:base");
        
        let all_seals = vec!["c2:seal1".to_string(), "c2:base".to_string()];
        assert!(cascade.verify_no_dangling_references(&all_seals));
    }
}
