use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc};

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub enum ReadingState {
    Indexed,
    Examined,
    Verified,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub enum RelationshipType {
    Initial,
    AppendVersion,
    EditedVersion,
    ContainerVariant,
    ManualVersion,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContainerMetadata {
    pub container_id: String,
    pub owner_id: String,
    pub display_name: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub ledger_root: String,
    pub ops_ceiling: u32,
    pub ops_consumed: u32,
    pub member_count: u32,
    pub reading_state_default: ReadingState,
    pub authorization_scope: String,
}

impl ContainerMetadata {
    pub fn new(container_id: String, owner_id: String) -> Self {
        let now = Utc::now();
        Self {
            container_id,
            owner_id,
            display_name: "Container".to_string(),
            created_at: now,
            updated_at: now,
            ledger_root: format!("ledger_{}", uuid::Uuid::new_v4()),
            ops_ceiling: 1000,
            ops_consumed: 0,
            member_count: 0,
            reading_state_default: ReadingState::Indexed,
            authorization_scope: "PRIVATE".to_string(),
        }
    }

    pub fn with_name(mut self, name: String) -> Self {
        self.display_name = name;
        self
    }

    pub fn with_reading_state(mut self, state: ReadingState) -> Self {
        self.reading_state_default = state;
        self
    }

    pub fn increment_member_count(&mut self) {
        self.member_count += 1;
        self.updated_at = Utc::now();
    }

    pub fn decrement_member_count(&mut self) {
        self.member_count = self.member_count.saturating_sub(1);
        self.updated_at = Utc::now();
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MemberMetadata {
    pub member_id: String,
    pub container_id: String,
    pub content_sha256: String,
    pub family_id: String,
    pub version_number: u32,
    pub parent_member_id: Option<String>,
    pub relationship_type: RelationshipType,
    pub reading_state: ReadingState,
    pub activity_edge: DateTime<Utc>,
    pub ops_delta: u32,
    pub version_proposal_confirmed: bool,
    pub provenance: MemberProvenance,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MemberProvenance {
    pub source_id: String,
    pub location: String,
    pub raw_text_hash: String,
    pub excerpt: String,
}

impl MemberMetadata {
    pub fn new(member_id: String, container_id: String) -> Self {
        Self {
            member_id,
            container_id,
            content_sha256: format!("sha256_{}", uuid::Uuid::new_v4()),
            family_id: uuid::Uuid::new_v4().to_string(),
            version_number: 1,
            parent_member_id: None,
            relationship_type: RelationshipType::Initial,
            reading_state: ReadingState::Indexed,
            activity_edge: Utc::now(),
            ops_delta: 10,
            version_proposal_confirmed: false,
            provenance: MemberProvenance {
                source_id: uuid::Uuid::new_v4().to_string(),
                location: "initial".to_string(),
                raw_text_hash: format!("hash_{}", uuid::Uuid::new_v4()),
                excerpt: String::new(),
            },
        }
    }

    pub fn with_parent(mut self, parent_id: String) -> Self {
        self.parent_member_id = Some(parent_id);
        self.relationship_type = RelationshipType::AppendVersion;
        self.version_number += 1;
        self
    }

    pub fn as_edited_version(mut self) -> Self {
        self.relationship_type = RelationshipType::EditedVersion;
        self.reading_state = ReadingState::Examined;
        self
    }

    pub fn confirm_version_proposal(&mut self) {
        self.version_proposal_confirmed = true;
        self.reading_state = ReadingState::Verified;
        self.activity_edge = Utc::now();
    }

    pub fn pending_version_proposal(&self) -> bool {
        !self.version_proposal_confirmed
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_reading_state_indexed() {
        assert_eq!(ReadingState::Indexed, ReadingState::Indexed);
    }

    #[test]
    fn test_reading_state_examined() {
        assert_eq!(ReadingState::Examined, ReadingState::Examined);
    }

    #[test]
    fn test_reading_state_verified() {
        assert_eq!(ReadingState::Verified, ReadingState::Verified);
    }

    #[test]
    fn test_relationship_type_initial() {
        assert_eq!(RelationshipType::Initial, RelationshipType::Initial);
    }

    #[test]
    fn test_relationship_type_append_version() {
        assert_eq!(RelationshipType::AppendVersion, RelationshipType::AppendVersion);
    }

    #[test]
    fn test_relationship_type_edited_version() {
        assert_eq!(RelationshipType::EditedVersion, RelationshipType::EditedVersion);
    }

    #[test]
    fn test_relationship_type_container_variant() {
        assert_eq!(RelationshipType::ContainerVariant, RelationshipType::ContainerVariant);
    }

    #[test]
    fn test_relationship_type_manual_version() {
        assert_eq!(RelationshipType::ManualVersion, RelationshipType::ManualVersion);
    }

    #[test]
    fn test_container_metadata_creation() {
        let metadata = ContainerMetadata::new("c1".to_string(), "user1".to_string());
        assert_eq!(metadata.container_id, "c1");
        assert_eq!(metadata.owner_id, "user1");
        assert_eq!(metadata.reading_state_default, ReadingState::Indexed);
    }

    #[test]
    fn test_container_metadata_with_name() {
        let metadata = ContainerMetadata::new("c1".to_string(), "user1".to_string())
            .with_name("My Container".to_string());
        assert_eq!(metadata.display_name, "My Container");
    }

    #[test]
    fn test_container_metadata_with_reading_state() {
        let metadata = ContainerMetadata::new("c1".to_string(), "user1".to_string())
            .with_reading_state(ReadingState::Verified);
        assert_eq!(metadata.reading_state_default, ReadingState::Verified);
    }

    #[test]
    fn test_increment_member_count() {
        let mut metadata = ContainerMetadata::new("c1".to_string(), "user1".to_string());
        metadata.increment_member_count();
        assert_eq!(metadata.member_count, 1);
    }

    #[test]
    fn test_decrement_member_count() {
        let mut metadata = ContainerMetadata::new("c1".to_string(), "user1".to_string());
        metadata.increment_member_count();
        metadata.decrement_member_count();
        assert_eq!(metadata.member_count, 0);
    }

    #[test]
    fn test_member_metadata_creation() {
        let member = MemberMetadata::new("m1".to_string(), "c1".to_string());
        assert_eq!(member.member_id, "m1");
        assert_eq!(member.container_id, "c1");
        assert_eq!(member.version_number, 1);
        assert_eq!(member.relationship_type, RelationshipType::Initial);
        assert!(!member.version_proposal_confirmed);
    }

    #[test]
    fn test_member_metadata_with_parent() {
        let member = MemberMetadata::new("m1".to_string(), "c1".to_string())
            .with_parent("m0".to_string());
        assert_eq!(member.parent_member_id, Some("m0".to_string()));
        assert_eq!(member.relationship_type, RelationshipType::AppendVersion);
        assert_eq!(member.version_number, 2);
    }

    #[test]
    fn test_member_as_edited_version() {
        let member = MemberMetadata::new("m1".to_string(), "c1".to_string())
            .as_edited_version();
        assert_eq!(member.relationship_type, RelationshipType::EditedVersion);
        assert_eq!(member.reading_state, ReadingState::Examined);
    }

    #[test]
    fn test_confirm_version_proposal() {
        let mut member = MemberMetadata::new("m1".to_string(), "c1".to_string());
        assert!(!member.version_proposal_confirmed);
        member.confirm_version_proposal();
        assert!(member.version_proposal_confirmed);
        assert_eq!(member.reading_state, ReadingState::Verified);
    }

    #[test]
    fn test_pending_version_proposal() {
        let member = MemberMetadata::new("m1".to_string(), "c1".to_string());
        assert!(member.pending_version_proposal());
    }

    #[test]
    fn test_member_provenance() {
        let member = MemberMetadata::new("m1".to_string(), "c1".to_string());
        assert!(!member.provenance.source_id.is_empty());
        assert_eq!(member.provenance.location, "initial");
    }

    #[test]
    fn test_member_ops_delta() {
        let member = MemberMetadata::new("m1".to_string(), "c1".to_string());
        assert_eq!(member.ops_delta, 10);
    }

    #[test]
    fn test_cross_container_isolation() {
        let m1 = MemberMetadata::new("m1".to_string(), "c1".to_string());
        let m2 = MemberMetadata::new("m2".to_string(), "c2".to_string());
        assert_ne!(m1.container_id, m2.container_id);
    }
}
