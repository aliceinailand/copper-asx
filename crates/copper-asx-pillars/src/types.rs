use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub struct Pillar {
    pub id: String,                      // sha256(source + actor + timestamp)
    pub source_id: String,               // unique source identifier
    pub actor: String,                   // human author / system identity
    pub timestamp: DateTime<Utc>,
    pub claim: String,                   // what this pillar evidences
    pub supporting_sources: Vec<String>, // other source IDs (for citation tracking)
    pub proof_level: u8,                 // 1–10 (confidence score)
}

impl Pillar {
    pub fn new(
        source_id: String,
        actor: String,
        claim: String,
        proof_level: u8,
    ) -> Self {
        let id = format!(
            "pillar:{}:{}:{}",
            source_id,
            actor,
            chrono::Utc::now().timestamp()
        );
        Self {
            id,
            source_id,
            actor,
            timestamp: Utc::now(),
            claim,
            supporting_sources: vec![],
            proof_level: proof_level.max(1).min(10),
        }
    }

    pub fn with_sources(mut self, sources: Vec<String>) -> Self {
        self.supporting_sources = sources;
        self
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PillarSet {
    pub pillars: Vec<Pillar>,
    pub independence_graph: Vec<(String, String)>, // edges: (pillar_id, related_pillar_id)
    pub independent_groups: Vec<Vec<String>>,      // each vec = mutually independent pillars
}

impl PillarSet {
    pub fn new() -> Self {
        Self {
            pillars: vec![],
            independence_graph: vec![],
            independent_groups: vec![],
        }
    }

    pub fn with_pillars(mut self, pillars: Vec<Pillar>) -> Self {
        self.pillars = pillars;
        self
    }

    pub fn pillar_count(&self) -> usize {
        self.pillars.len()
    }

    pub fn singletons(&self) -> Vec<&Pillar> {
        self.pillars
            .iter()
            .filter(|p| {
                self.independence_graph
                    .iter()
                    .filter(|(a, _) | a == &p.id)
                    .count()
                    == 0
            })
            .collect()
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GapRange {
    pub start: DateTime<Utc>,
    pub end: DateTime<Utc>,
    pub claim_uncovered: String, // what claim has no evidence in this span
    pub severity: String,        // "critical" / "high" / "medium" / "low"
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PillarAnalysisResult {
    pub extracted_pillars: PillarSet,
    pub open_gaps: Vec<GapRange>,
    pub singletons: Vec<Pillar>,   // pillars with 0 independent corroboration
    pub z3_atoms: Z3AtomSet,       // ready to feed to Z3
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Z3AtomSet {
    pub pillars_satisfied: bool,
    pub independence_verified: bool,
    pub no_open_gaps: bool,
    pub pillar_count: usize,
    pub independent_group_count: usize,
}

impl Z3AtomSet {
    pub fn new(
        pillar_count: usize,
        independent_groups: usize,
        gap_count: usize,
    ) -> Self {
        Self {
            pillars_satisfied: pillar_count >= 2,
            independence_verified: independent_groups > 0,
            no_open_gaps: gap_count == 0,
            pillar_count,
            independent_group_count: independent_groups,
        }
    }

    pub fn atoms_as_strings(&self) -> Vec<String> {
        vec![
            format!("pillars_satisfied:{}", self.pillars_satisfied),
            format!("independence_verified:{}", self.independence_verified),
            format!("no_open_gaps:{}", self.no_open_gaps),
            format!("pillar_count:{}", self.pillar_count),
            format!(
                "independent_group_count:{}",
                self.independent_group_count
            ),
        ]
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IndependenceScore {
    pub pillar_a: String,
    pub pillar_b: String,
    pub social_independence: f32,
    pub temporal_independence: f32,
    pub citation_independence: f32,
    pub total_score: f32,
}

impl IndependenceScore {
    pub fn is_independent(&self) -> bool {
        self.total_score >= 0.5
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_pillar_creation() {
        let pillar = Pillar::new(
            "src001".to_string(),
            "alice".to_string(),
            "evidence claim".to_string(),
            7,
        );

        assert_eq!(pillar.source_id, "src001");
        assert_eq!(pillar.actor, "alice");
        assert_eq!(pillar.proof_level, 7);
        assert!(pillar.id.starts_with("pillar:"));
    }

    #[test]
    fn test_pillar_proof_level_bounds() {
        let p1 = Pillar::new(
            "src".to_string(),
            "alice".to_string(),
            "claim".to_string(),
            0,
        );
        assert_eq!(p1.proof_level, 1);

        let p2 = Pillar::new(
            "src".to_string(),
            "alice".to_string(),
            "claim".to_string(),
            15,
        );
        assert_eq!(p2.proof_level, 10);
    }

    #[test]
    fn test_z3_atom_set_creation() {
        let atoms = Z3AtomSet::new(2, 1, 0);
        assert!(atoms.pillars_satisfied);
        assert!(atoms.independence_verified);
        assert!(atoms.no_open_gaps);
    }

    #[test]
    fn test_z3_atom_set_strings() {
        let atoms = Z3AtomSet::new(2, 1, 0);
        let strings = atoms.atoms_as_strings();
        assert_eq!(strings.len(), 5);
        assert!(strings[0].contains("pillars_satisfied:true"));
    }

    #[test]
    fn test_independence_score() {
        let score = IndependenceScore {
            pillar_a: "p1".to_string(),
            pillar_b: "p2".to_string(),
            social_independence: 0.8,
            temporal_independence: 0.7,
            citation_independence: 0.6,
            total_score: 0.7,
        };
        assert!(score.is_independent());
    }
}
