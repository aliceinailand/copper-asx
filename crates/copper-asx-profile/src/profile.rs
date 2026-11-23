use serde::{Deserialize, Serialize};
use thiserror::Error;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct PersonaWeights {
    pub reasoning: f64,
    pub caution: f64,
    pub speed: f64,
    pub innovation: f64,
}

impl PersonaWeights {
    pub fn new(reasoning: f64, caution: f64, speed: f64, innovation: f64) -> Result<Self, ProfileError> {
        let sum = reasoning + caution + speed + innovation;
        if (sum - 1.0).abs() > 1e-6 {
            return Err(ProfileError::InvalidPersonaWeights(format!(
                "weights must sum to 1.0, got {}",
                sum
            )));
        }

        if reasoning < 0.0 || caution < 0.0 || speed < 0.0 || innovation < 0.0 {
            return Err(ProfileError::InvalidPersonaWeights(
                "weights must be non-negative".to_string(),
            ));
        }

        Ok(PersonaWeights {
            reasoning,
            caution,
            speed,
            innovation,
        })
    }

    pub fn sum(&self) -> f64 {
        self.reasoning + self.caution + self.speed + self.innovation
    }

    pub fn clarity_score(&self) -> f64 {
        self.caution + self.reasoning
    }

    pub fn speed_score(&self) -> f64 {
        self.speed
    }

    pub fn caution_score(&self) -> f64 {
        self.caution
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Profile {
    pub id: String,
    pub name: String,
    pub persona: PersonaWeights,
    pub tool_surface: Vec<String>,
    pub hop_limit: u32,
    pub clearance_required: String,
    pub escalation_threshold: String,
    pub allow_guest: bool,
}

impl Profile {
    pub fn new(
        id: String,
        name: String,
        persona: PersonaWeights,
        tool_surface: Vec<String>,
        hop_limit: u32,
        clearance_required: String,
        escalation_threshold: String,
        allow_guest: bool,
    ) -> Result<Self, ProfileError> {
        if hop_limit == 0 || hop_limit > 10 {
            return Err(ProfileError::InvalidHopLimit(format!(
                "hop_limit must be between 1 and 10, got {}",
                hop_limit
            )));
        }

        if tool_surface.is_empty() {
            return Err(ProfileError::InvalidToolSurface(
                "tool_surface cannot be empty".to_string(),
            ));
        }

        if id.is_empty() || name.is_empty() {
            return Err(ProfileError::InvalidProfile(
                "id and name cannot be empty".to_string(),
            ));
        }

        Ok(Profile {
            id,
            name,
            persona,
            tool_surface,
            hop_limit,
            clearance_required,
            escalation_threshold,
            allow_guest,
        })
    }

    pub fn is_guest_profile(&self) -> bool {
        false
    }
}

#[derive(Debug, Error)]
pub enum ProfileError {
    #[error("Invalid persona weights: {0}")]
    InvalidPersonaWeights(String),

    #[error("Invalid hop limit: {0}")]
    InvalidHopLimit(String),

    #[error("Invalid tool surface: {0}")]
    InvalidToolSurface(String),

    #[error("Invalid profile: {0}")]
    InvalidProfile(String),

    #[error("IO error: {0}")]
    IoError(#[from] std::io::Error),

    #[error("JSON error: {0}")]
    JsonError(#[from] serde_json::Error),

    #[error("Profile not found: {0}")]
    NotFound(String),

    #[error("Permission denied: {0}")]
    PermissionDenied(String),
}

pub fn validate_profile(profile: &Profile) -> Result<(), ProfileError> {
    if profile.persona.sum() - 1.0 > 1e-6 {
        return Err(ProfileError::InvalidPersonaWeights(format!(
            "persona weights must sum to 1.0, got {}",
            profile.persona.sum()
        )));
    }

    if profile.hop_limit == 0 || profile.hop_limit > 10 {
        return Err(ProfileError::InvalidHopLimit(format!(
            "hop_limit must be between 1 and 10, got {}",
            profile.hop_limit
        )));
    }

    if profile.tool_surface.is_empty() {
        return Err(ProfileError::InvalidToolSurface(
            "tool_surface cannot be empty".to_string(),
        ));
    }

    Ok(())
}

pub fn load_profile(id: &str) -> Result<Profile, ProfileError> {
    Err(ProfileError::NotFound(format!(
        "Profile loading from storage not yet implemented for id: {}",
        id
    )))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_valid_persona_weights() {
        let weights = PersonaWeights::new(0.3, 0.4, 0.2, 0.1);
        assert!(weights.is_ok());
        let w = weights.unwrap();
        assert_eq!(w.reasoning, 0.3);
        assert_eq!(w.caution, 0.4);
    }

    #[test]
    fn test_invalid_persona_weights_sum() {
        let weights = PersonaWeights::new(0.3, 0.4, 0.2, 0.0);
        assert!(weights.is_err());
    }

    #[test]
    fn test_negative_persona_weights() {
        let weights = PersonaWeights::new(-0.1, 0.5, 0.3, 0.3);
        assert!(weights.is_err());
    }

    #[test]
    fn test_clarity_score() {
        let weights = PersonaWeights::new(0.3, 0.4, 0.2, 0.1).unwrap();
        assert!((weights.clarity_score() - 0.7).abs() < 1e-6);
    }

    #[test]
    fn test_valid_profile_creation() {
        let weights = PersonaWeights::new(0.3, 0.4, 0.2, 0.1).unwrap();
        let profile = Profile::new(
            "operator-123".to_string(),
            "Alice Operator".to_string(),
            weights,
            vec!["fetch_web_page".to_string(), "execute_code".to_string()],
            3,
            "moderate".to_string(),
            "high".to_string(),
            true,
        );
        assert!(profile.is_ok());
    }

    #[test]
    fn test_profile_invalid_hop_limit() {
        let weights = PersonaWeights::new(0.3, 0.4, 0.2, 0.1).unwrap();
        let profile = Profile::new(
            "operator-123".to_string(),
            "Alice Operator".to_string(),
            weights,
            vec!["fetch_web_page".to_string()],
            0,
            "moderate".to_string(),
            "high".to_string(),
            true,
        );
        assert!(profile.is_err());
    }

    #[test]
    fn test_validate_profile() {
        let weights = PersonaWeights::new(0.3, 0.4, 0.2, 0.1).unwrap();
        let profile = Profile::new(
            "operator-123".to_string(),
            "Alice Operator".to_string(),
            weights,
            vec!["fetch_web_page".to_string()],
            3,
            "moderate".to_string(),
            "high".to_string(),
            true,
        )
        .unwrap();
        assert!(validate_profile(&profile).is_ok());
    }

    #[test]
    fn test_is_guest_profile() {
        let weights = PersonaWeights::new(0.3, 0.4, 0.2, 0.1).unwrap();
        let profile = Profile::new(
            "operator-123".to_string(),
            "Alice Operator".to_string(),
            weights,
            vec!["fetch_web_page".to_string()],
            3,
            "moderate".to_string(),
            "high".to_string(),
            true,
        )
        .unwrap();
        assert!(!profile.is_guest_profile());
    }
}
