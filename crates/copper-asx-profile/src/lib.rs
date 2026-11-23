mod guest_model;
mod permissions;
mod profile;

pub use guest_model::{can_escalate_guest, get_host_profile, validate_guest_mutation, GuestError, GuestOverlay};
pub use permissions::{can_tool_call, clearance_level_check, hop_limit_check, PermissionError, PermissionResult, ToolPermission};
pub use profile::{load_profile, validate_profile, PersonaWeights, Profile, ProfileError};
