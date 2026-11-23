use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc};

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq)]
pub enum ContainerStatus {
    Active,
    Archived,
    Deleted,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Container {
    pub container_id: String,
    pub owner_id: String,
    pub display_name: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub ledger_root: String,
    pub ops_ceiling: u32,
    pub ops_consumed: u32,
    pub member_count: u32,
    pub authorization_scope: String,
    pub status: ContainerStatus,
}

impl Container {
    pub fn new(owner_id: String) -> Self {
        let now = Utc::now();
        Self {
            container_id: uuid::Uuid::new_v4().to_string(),
            owner_id,
            display_name: "New Container".to_string(),
            created_at: now,
            updated_at: now,
            ledger_root: format!("ledger_{}", uuid::Uuid::new_v4()),
            ops_ceiling: 1000,
            ops_consumed: 0,
            member_count: 0,
            authorization_scope: "PRIVATE".to_string(),
            status: ContainerStatus::Active,
        }
    }

    pub fn with_name(mut self, name: String) -> Self {
        self.display_name = name;
        self
    }

    pub fn with_ops_ceiling(mut self, ceiling: u32) -> Self {
        self.ops_ceiling = ceiling;
        self
    }

    pub fn consume_ops(&mut self, amount: u32) -> bool {
        if self.ops_consumed + amount <= self.ops_ceiling {
            self.ops_consumed += amount;
            true
        } else {
            false
        }
    }

    pub fn refund_ops(&mut self, amount: u32) {
        self.ops_consumed = self.ops_consumed.saturating_sub(amount);
    }

    pub fn archive(&mut self) {
        self.status = ContainerStatus::Archived;
        self.updated_at = Utc::now();
    }

    pub fn delete(&mut self) {
        self.status = ContainerStatus::Deleted;
        self.updated_at = Utc::now();
    }

    pub fn is_active(&self) -> bool {
        self.status == ContainerStatus::Active
    }

    pub fn ops_remaining(&self) -> u32 {
        self.ops_ceiling.saturating_sub(self.ops_consumed)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_container_creation() {
        let container = Container::new("user1".to_string());
        assert_eq!(container.owner_id, "user1");
        assert_eq!(container.status, ContainerStatus::Active);
        assert_eq!(container.display_name, "New Container");
        assert_eq!(container.ops_consumed, 0);
        assert_eq!(container.ops_ceiling, 1000);
    }

    #[test]
    fn test_container_with_name() {
        let container = Container::new("user1".to_string())
            .with_name("My Container".to_string());
        assert_eq!(container.display_name, "My Container");
    }

    #[test]
    fn test_container_with_ops_ceiling() {
        let container = Container::new("user1".to_string())
            .with_ops_ceiling(5000);
        assert_eq!(container.ops_ceiling, 5000);
    }

    #[test]
    fn test_consume_ops() {
        let mut container = Container::new("user1".to_string());
        assert!(container.consume_ops(100));
        assert_eq!(container.ops_consumed, 100);
    }

    #[test]
    fn test_consume_ops_exceeds_ceiling() {
        let mut container = Container::new("user1".to_string());
        assert!(!container.consume_ops(2000));
        assert_eq!(container.ops_consumed, 0);
    }

    #[test]
    fn test_refund_ops() {
        let mut container = Container::new("user1".to_string());
        container.consume_ops(100);
        container.refund_ops(50);
        assert_eq!(container.ops_consumed, 50);
    }

    #[test]
    fn test_archive() {
        let mut container = Container::new("user1".to_string());
        container.archive();
        assert_eq!(container.status, ContainerStatus::Archived);
        assert!(!container.is_active());
    }

    #[test]
    fn test_delete() {
        let mut container = Container::new("user1".to_string());
        container.delete();
        assert_eq!(container.status, ContainerStatus::Deleted);
        assert!(!container.is_active());
    }

    #[test]
    fn test_ops_remaining() {
        let mut container = Container::new("user1".to_string());
        container.consume_ops(300);
        assert_eq!(container.ops_remaining(), 700);
    }

    #[test]
    fn test_container_isolation() {
        let c1 = Container::new("user1".to_string());
        let c2 = Container::new("user2".to_string());
        assert_ne!(c1.container_id, c2.container_id);
        assert_ne!(c1.ledger_root, c2.ledger_root);
    }
}
