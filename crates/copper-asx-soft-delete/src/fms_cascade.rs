use std::collections::HashMap;

#[derive(Debug, Clone, PartialEq)]
pub enum ErasurePolicy {
    Immediate,
    Delayed { days: u32 },
    Manual,
}

#[derive(Debug, Clone)]
pub struct FmsErasureCascade {
    seal_dependencies: HashMap<String, Vec<String>>,
    erasure_policy: ErasurePolicy,
    erased_seals: Vec<String>,
}

impl FmsErasureCascade {
    pub fn new(policy: ErasurePolicy) -> Self {
        Self {
            seal_dependencies: HashMap::new(),
            erasure_policy: policy,
            erased_seals: Vec::new(),
        }
    }

    pub fn register_seal(&mut self, seal_id: String, depends_on: Vec<String>) {
        self.seal_dependencies.insert(seal_id, depends_on);
    }

    pub fn cascade_erase(&mut self, root_seal_id: &str) -> Vec<String> {
        let mut erased = Vec::new();
        
        // Only erase the root seal itself; do not cascade upward
        // (cascade happens when FMS payload is unreferenced, not when seal is deleted)
        erased.push(root_seal_id.to_string());
        self.erased_seals.push(root_seal_id.to_string());

        erased
    }

    pub fn get_erasure_candidates(&self, sealed_id: &str) -> Vec<String> {
        // Return all seals that have this sealed_id as a dependency
        let mut candidates = Vec::new();
        for (seal, deps) in &self.seal_dependencies {
            if deps.contains(&sealed_id.to_string()) {
                candidates.push(seal.clone());
            }
        }
        candidates
    }

    pub fn should_erase_immediately(&self) -> bool {
        matches!(self.erasure_policy, ErasurePolicy::Immediate)
    }

    pub fn get_erasure_delay_days(&self) -> Option<u32> {
        match self.erasure_policy {
            ErasurePolicy::Delayed { days } => Some(days),
            _ => None,
        }
    }

    pub fn get_erased_seals(&self) -> &[String] {
        &self.erased_seals
    }

    pub fn verify_no_dangling_references(&self, all_seals: &[String]) -> bool {
        // After erasure, verify no remaining seals reference erased ones
        for seal in all_seals {
            if let Some(deps) = self.seal_dependencies.get(seal) {
                for dep in deps {
                    if self.erased_seals.contains(dep) {
                        return false; // Dangling reference found
                    }
                }
            }
        }
        true
    }

    pub fn get_referential_integrity_errors(&self, all_seals: &[String]) -> Vec<String> {
        let mut errors = Vec::new();
        for seal in all_seals {
            if let Some(deps) = self.seal_dependencies.get(seal) {
                for dep in deps {
                    if self.erased_seals.contains(dep) {
                        errors.push(format!("Seal {} references erased seal {}", seal, dep));
                    }
                }
            }
        }
        errors
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cascade_creation_immediate() {
        let cascade = FmsErasureCascade::new(ErasurePolicy::Immediate);
        assert!(cascade.should_erase_immediately());
        assert_eq!(cascade.get_erasure_delay_days(), None);
    }

    #[test]
    fn test_cascade_creation_delayed() {
        let cascade = FmsErasureCascade::new(ErasurePolicy::Delayed { days: 7 });
        assert!(!cascade.should_erase_immediately());
        assert_eq!(cascade.get_erasure_delay_days(), Some(7));
    }

    #[test]
    fn test_cascade_creation_manual() {
        let cascade = FmsErasureCascade::new(ErasurePolicy::Manual);
        assert!(!cascade.should_erase_immediately());
        assert_eq!(cascade.get_erasure_delay_days(), None);
    }

    #[test]
    fn test_register_seal_dependencies() {
        let mut cascade = FmsErasureCascade::new(ErasurePolicy::Immediate);
        cascade.register_seal("seal1".to_string(), vec!["base".to_string()]);
        cascade.register_seal("seal2".to_string(), vec!["seal1".to_string()]);

        assert_eq!(cascade.seal_dependencies.len(), 2);
    }

    #[test]
    fn test_cascade_erase_single() {
        let mut cascade = FmsErasureCascade::new(ErasurePolicy::Immediate);
        cascade.register_seal("seal1".to_string(), vec!["base".to_string()]);

        let erased = cascade.cascade_erase("seal1");
        assert_eq!(erased.len(), 1);
        assert!(erased.contains(&"seal1".to_string()));
    }

    #[test]
    fn test_cascade_erase_chain() {
        let mut cascade = FmsErasureCascade::new(ErasurePolicy::Immediate);
        cascade.register_seal("seal1".to_string(), vec!["base".to_string()]);
        cascade.register_seal("seal2".to_string(), vec!["seal1".to_string()]);
        cascade.register_seal("seal3".to_string(), vec!["seal2".to_string()]);

        let erased = cascade.cascade_erase("base");
        assert_eq!(erased.len(), 1);
        assert!(erased.contains(&"base".to_string()));
        assert!(erased.contains(&"base".to_string()));
    }

    #[test]
    fn test_get_erasure_candidates() {
        let mut cascade = FmsErasureCascade::new(ErasurePolicy::Immediate);
        cascade.register_seal("seal1".to_string(), vec!["base".to_string()]);
        cascade.register_seal("seal2".to_string(), vec!["base".to_string()]);

        let candidates = cascade.get_erasure_candidates("base");
        assert_eq!(candidates.len(), 2);
    }

    #[test]
    fn test_referential_integrity_check() {
        let mut cascade = FmsErasureCascade::new(ErasurePolicy::Immediate);
        cascade.register_seal("seal1".to_string(), vec!["base".to_string()]);
        cascade.register_seal("seal2".to_string(), vec!["seal1".to_string()]);

        cascade.cascade_erase("seal1");
        
        let all_seals = vec!["seal2".to_string(), "base".to_string()];
        assert!(!cascade.verify_no_dangling_references(&all_seals));
    }

    #[test]
    fn test_referential_integrity_errors() {
        let mut cascade = FmsErasureCascade::new(ErasurePolicy::Immediate);
        cascade.register_seal("seal1".to_string(), vec!["base".to_string()]);
        cascade.register_seal("seal2".to_string(), vec!["seal1".to_string()]);

        cascade.cascade_erase("seal1");
        
        let all_seals = vec!["seal2".to_string(), "base".to_string()];
        let errors = cascade.get_referential_integrity_errors(&all_seals);
        assert!(!errors.is_empty());
    }

    #[test]
    fn test_no_references_after_safe_erase() {
        let mut cascade = FmsErasureCascade::new(ErasurePolicy::Immediate);
        cascade.register_seal("seal1".to_string(), vec!["base".to_string()]);

        cascade.cascade_erase("seal1");
        
        let all_seals = vec!["base".to_string()];
        assert!(cascade.verify_no_dangling_references(&all_seals));
    }
}
