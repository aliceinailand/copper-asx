use crate::types::{IndependenceScore, Pillar};
use std::collections::{HashMap, HashSet};

pub struct IndependenceAnalyzer;

impl IndependenceAnalyzer {
    /// Analyze independence between all pillar pairs
    pub fn analyze(pillars: &[Pillar]) -> Vec<Vec<Pillar>> {
        if pillars.is_empty() {
            return vec![];
        }

        // Build adjacency for independent pairs
        let mut independent_pairs: HashSet<(usize, usize)> = HashSet::new();

        for i in 0..pillars.len() {
            for j in (i + 1)..pillars.len() {
                let score = Self::compute_independence_score(&pillars[i], &pillars[j]);
                if score.is_independent() {
                    independent_pairs.insert((i, j));
                    independent_pairs.insert((j, i));
                }
            }
        }

        // Group into independent sets (simple greedy clustering)
        let mut groups: Vec<Vec<Pillar>> = vec![];
        let mut processed = HashSet::new();

        for i in 0..pillars.len() {
            if processed.contains(&i) {
                continue;
            }

            let mut group = vec![pillars[i].clone()];
            processed.insert(i);

            for j in 0..pillars.len() {
                if i != j && !processed.contains(&j) {
                    if independent_pairs.contains(&(i, j)) {
                        group.push(pillars[j].clone());
                        processed.insert(j);
                    }
                }
            }

            groups.push(group);
        }

        groups
    }

    /// Compute independence score between two pillars (0.0 - 1.0)
    pub fn compute_independence_score(p1: &Pillar, p2: &Pillar) -> IndependenceScore {
        // Social independence: different actors
        let social = if p1.actor != p2.actor { 0.9 } else { 0.1 };

        // Temporal independence: different timestamps (at least 1 hour apart)
        let time_diff = (p2.timestamp - p1.timestamp).num_hours().abs();
        let temporal = if time_diff >= 1 { 0.8 } else { 0.3 };

        // Citation independence: minimal source overlap
        let p1_sources: HashSet<_> = p1.supporting_sources.iter().collect();
        let p2_sources: HashSet<_> = p2.supporting_sources.iter().collect();
        let overlap = p1_sources.intersection(&p2_sources).count();
        let total_sources = p1_sources.len() + p2_sources.len();
        let citation = if overlap == 0 && total_sources > 0 {
            0.9
        } else if overlap == 0 {
            0.5
        } else {
            (1.0 - (overlap as f32 / total_sources as f32)).max(0.0)
        };

        let total_score = (social + temporal + citation) / 3.0;

        IndependenceScore {
            pillar_a: p1.id.clone(),
            pillar_b: p2.id.clone(),
            social_independence: social,
            temporal_independence: temporal,
            citation_independence: citation,
            total_score,
        }
    }

    /// Find all independent groups of size >= min_size
    pub fn find_independent_groups(
        pillars: &[Pillar],
        min_size: usize,
    ) -> Vec<Vec<Pillar>> {
        Self::analyze(pillars)
            .into_iter()
            .filter(|g| g.len() >= min_size)
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Utc;

    #[test]
    fn test_independence_score_different_actors() {
        let now = Utc::now();
        let p1 = Pillar {
            id: "p1".to_string(),
            source_id: "src1".to_string(),
            actor: "alice".to_string(),
            timestamp: now,
            claim: "test".to_string(),
            supporting_sources: vec![],
            proof_level: 7,
        };

        let p2 = Pillar {
            id: "p2".to_string(),
            source_id: "src2".to_string(),
            actor: "bob".to_string(),
            timestamp: now,
            claim: "test".to_string(),
            supporting_sources: vec![],
            proof_level: 7,
        };

        let score = IndependenceAnalyzer::compute_independence_score(&p1, &p2);
        assert!(score.social_independence > 0.5);
        assert!(score.is_independent());
    }

    #[test]
    fn test_independence_score_same_actor() {
        let now = Utc::now();
        let p1 = Pillar {
            id: "p1".to_string(),
            source_id: "src1".to_string(),
            actor: "alice".to_string(),
            timestamp: now,
            claim: "test".to_string(),
            supporting_sources: vec![],
            proof_level: 7,
        };

        let p2 = Pillar {
            id: "p2".to_string(),
            source_id: "src2".to_string(),
            actor: "alice".to_string(),
            timestamp: now,
            claim: "test".to_string(),
            supporting_sources: vec![],
            proof_level: 7,
        };

        let score = IndependenceAnalyzer::compute_independence_score(&p1, &p2);
        assert!(score.social_independence < 0.5);
    }

    #[test]
    fn test_analyze_empty() {
        let result = IndependenceAnalyzer::analyze(&[]);
        assert_eq!(result.len(), 0);
    }

    #[test]
    fn test_analyze_single() {
        let p1 = Pillar::new(
            "src1".to_string(),
            "alice".to_string(),
            "claim".to_string(),
            7,
        );
        let result = IndependenceAnalyzer::analyze(&[p1.clone()]);
        assert_eq!(result.len(), 1);
        assert_eq!(result[0][0].id, p1.id);
    }

    #[test]
    fn test_find_independent_groups() {
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
                proof_level: 7,
            },
        ];

        let groups = IndependenceAnalyzer::find_independent_groups(&pillars, 2);
        assert!(groups.len() > 0);
    }
}
