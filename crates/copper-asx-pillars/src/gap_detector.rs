use crate::types::{GapRange, Pillar};
use chrono::{Duration, Utc};

pub struct GapDetector;

impl GapDetector {
    /// Find temporal gaps (time spans with no evidence)
    pub fn find_temporal_gaps(
        pillars: &[Pillar],
        time_window: Duration,
    ) -> Vec<GapRange> {
        if pillars.is_empty() {
            return vec![];
        }

        // Sort pillars by timestamp
        let mut sorted = pillars.to_vec();
        sorted.sort_by_key(|p| p.timestamp);

        let mut gaps = vec![];

        // Check for gap before first pillar
        let now = Utc::now();
        if sorted[0].timestamp > now - time_window {
            gaps.push(GapRange {
                start: now - time_window,
                end: sorted[0].timestamp,
                claim_uncovered: "evidence before earliest known source".to_string(),
                severity: "medium".to_string(),
            });
        }

        // Check for gaps between pillars
        for i in 0..sorted.len() - 1 {
            let gap_between = sorted[i + 1].timestamp - sorted[i].timestamp;
            if gap_between > time_window {
                gaps.push(GapRange {
                    start: sorted[i].timestamp,
                    end: sorted[i + 1].timestamp,
                    claim_uncovered: format!(
                        "evidence gap between {} and {}",
                        sorted[i].source_id, sorted[i + 1].source_id
                    ),
                    severity: if gap_between > time_window * 2 {
                        "high".to_string()
                    } else {
                        "medium".to_string()
                    },
                });
            }
        }

        // Check for gap after last pillar
        if sorted[sorted.len() - 1].timestamp < now - Duration::hours(1) {
            gaps.push(GapRange {
                start: sorted[sorted.len() - 1].timestamp,
                end: now,
                claim_uncovered: "evidence after last known source".to_string(),
                severity: "low".to_string(),
            });
        }

        gaps
    }

    /// Find semantic gaps (claims with no supporting sources)
    pub fn find_semantic_gaps(pillars: &[Pillar], claims: &[String]) -> Vec<GapRange> {
        let mut gaps = vec![];

        for claim in claims {
            let supporting = pillars.iter().filter(|p| p.claim == *claim).count();
            if supporting == 0 {
                gaps.push(GapRange {
                    start: Utc::now(),
                    end: Utc::now(),
                    claim_uncovered: format!("claim unsupported: {}", claim),
                    severity: "high".to_string(),
                });
            }
        }

        gaps
    }

    /// Check if claim is fully covered (at least min_pillars support it)
    pub fn is_claim_covered(pillars: &[Pillar], claim: &str, min_pillars: usize) -> bool {
        let supporting = pillars.iter().filter(|p| p.claim == claim).count();
        supporting >= min_pillars
    }

    /// Get coverage percentage for a claim
    pub fn coverage_percentage(pillars: &[Pillar], claim: &str) -> f32 {
        if pillars.is_empty() {
            return 0.0;
        }
        let supporting = pillars.iter().filter(|p| p.claim == claim).count() as f32;
        (supporting / pillars.len() as f32) * 100.0
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Utc;

    #[test]
    fn test_no_gaps_empty() {
        let gaps = GapDetector::find_temporal_gaps(&[], Duration::hours(24));
        assert_eq!(gaps.len(), 0);
    }

    #[test]
    fn test_single_pillar_no_temporal_gaps() {
        let now = Utc::now();
        let p = Pillar {
            id: "p1".to_string(),
            source_id: "src1".to_string(),
            actor: "alice".to_string(),
            timestamp: now,
            claim: "test".to_string(),
            supporting_sources: vec![],
            proof_level: 7,
        };

        let gaps = GapDetector::find_temporal_gaps(&[p], Duration::hours(24));
        // Should have gap before and after
        assert!(gaps.len() >= 1);
    }

    #[test]
    fn test_find_semantic_gaps() {
        let now = Utc::now();
        let pillars = vec![
            Pillar {
                id: "p1".to_string(),
                source_id: "src1".to_string(),
                actor: "alice".to_string(),
                timestamp: now,
                claim: "claim_a".to_string(),
                supporting_sources: vec![],
                proof_level: 7,
            },
        ];

        let claims = vec!["claim_a".to_string(), "claim_b".to_string()];
        let gaps = GapDetector::find_semantic_gaps(&pillars, &claims);

        assert!(gaps.len() > 0);
        assert!(gaps.iter().any(|g| g.claim_uncovered.contains("claim_b")));
    }

    #[test]
    fn test_is_claim_covered() {
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
                timestamp: now + Duration::hours(1),
                claim: "test".to_string(),
                supporting_sources: vec![],
                proof_level: 7,
            },
        ];

        assert!(GapDetector::is_claim_covered(&pillars, "test", 2));
        assert!(!GapDetector::is_claim_covered(&pillars, "other", 1));
    }

    #[test]
    fn test_coverage_percentage() {
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
                timestamp: now,
                claim: "other".to_string(),
                supporting_sources: vec![],
                proof_level: 7,
            },
        ];

        let coverage = GapDetector::coverage_percentage(&pillars, "test");
        assert_eq!(coverage, 50.0);
    }
}
