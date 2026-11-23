pub mod container;
pub mod metadata;
pub mod ui_surface;

pub use container::{Container, ContainerStatus};
pub use metadata::{ContainerMetadata, MemberMetadata, ReadingState, RelationshipType};
pub use ui_surface::ContainerUIBuilder;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_container_creation() {
        let container = Container::new("user1".to_string());
        assert_eq!(container.owner_id, "user1");
        assert_eq!(container.status, ContainerStatus::Active);
    }

    #[test]
    fn test_metadata_reading_state() {
        let state = ReadingState::Indexed;
        assert_eq!(state, ReadingState::Indexed);
    }

    #[test]
    fn test_metadata_relationship_type() {
        let rel = RelationshipType::Initial;
        assert_eq!(rel, RelationshipType::Initial);
    }

    #[test]
    fn test_container_metadata_creation() {
        let metadata = ContainerMetadata::new("c1".to_string(), "user1".to_string());
        assert_eq!(metadata.container_id, "c1");
        assert_eq!(metadata.owner_id, "user1");
    }

    #[test]
    fn test_member_metadata_creation() {
        let member = MemberMetadata::new("m1".to_string(), "c1".to_string());
        assert_eq!(member.member_id, "m1");
        assert_eq!(member.container_id, "c1");
    }

    #[test]
    fn test_ui_builder() {
        let builder = ContainerUIBuilder::new("c1".to_string());
        assert_eq!(builder.container_id, "c1");
    }

    #[test]
    fn test_version_proposal_pending() {
        let member = MemberMetadata::new("m1".to_string(), "c1".to_string());
        assert!(!member.version_proposal_confirmed);
    }
}
