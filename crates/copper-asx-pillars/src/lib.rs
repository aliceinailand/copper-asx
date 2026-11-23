//! Phase I.4: Pillar Extraction & Gap Analysis
//!
//! Extracts evidence pillars (independent sources) from IR/dependency graphs,
//! detects open gaps, and feeds into Z3 solver for formal verification.

pub mod gap_detector;
pub mod independence;
pub mod types;

pub use gap_detector::GapDetector;
pub use independence::IndependenceAnalyzer;
pub use types::{
    GapRange, IndependenceScore, Pillar, PillarAnalysisResult, PillarSet, Z3AtomSet,
};

/// Orchestrator for complete pillar extraction workflow
pub struct PillarExtractor;

impl PillarExtractor {
    /// Extract pillars and analyze independence + gaps
    pub fn extract_and_analyze(pillars: &[Pillar]) -> PillarAnalysisResult {
        let independent_groups = IndependenceAnalyzer::analyze(pillars);
        let open_gaps = GapDetector::find_temporal_gaps(
            pillars,
            chrono::Duration::hours(24),
        );

        let singletons: Vec<Pillar> = pillars
            .iter()
            .filter(|p| {
                !independent_groups.iter().any(|g| g.iter().any(|gp| gp.id == p.id))
            })
            .cloned()
            .collect();

        let z3_atoms = Z3AtomSet::new(
            pillars.len(),
            independent_groups.len(),
            open_gaps.len(),
        );

        let pillar_set = PillarSet {
            pillars: pillars.to_vec(),
            independence_graph: vec![],
            independent_groups: independent_groups
                .iter()
                .map(|g| g.iter().map(|p| p.id.clone()).collect())
                .collect(),
        };

        PillarAnalysisResult {
            extracted_pillars: pillar_set,
            open_gaps,
            singletons,
            z3_atoms,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Utc;

    #[test]
    fn test_complete_workflow() {
        let now = Utc::now();
        let pillars = vec![
            Pillar {
                id: "p1".to_string(),
                source_id: "src1".to_string(),
                actor: "alice".to_string(),
                timestamp: now,
                claim: "test".to_string(),
                supporting_sources: vec![],
                proof_level: 7,
            },
            Pillar {
                id: "p2".to_string(),
                source_id: "src2".to_string(),
                actor: "bob".to_string(),
                timestamp: now + chrono::Duration::hours(2),
                claim: "test".to_string(),
                supporting_sources: vec![],
                proof_level: 8,
            },
        ];

        let result = PillarExtractor::extract_and_analyze(&pillars);
        assert_eq!(result.extracted_pillars.pillar_count(), 2);
        assert!(result.z3_atoms.pillars_satisfied);
    }
}
