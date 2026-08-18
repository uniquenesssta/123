use football_domain::{EntityMatchCandidate, EntityMatchResult};
use uuid::Uuid;

pub(super) fn exact_match(id: Uuid, reason: &str) -> EntityMatchResult {
    EntityMatchResult {
        status: "exact".to_string(),
        matched_id: Some(id),
        candidates: vec![EntityMatchCandidate {
            id,
            label: id.to_string(),
            reason: reason.to_string(),
            score: 1.0,
        }],
    }
}

pub(super) fn ambiguous(ids: Vec<Uuid>, reason: &str) -> EntityMatchResult {
    EntityMatchResult {
        status: "ambiguous".to_string(),
        matched_id: None,
        candidates: ids
            .into_iter()
            .map(|id| EntityMatchCandidate {
                id,
                label: id.to_string(),
                reason: reason.to_string(),
                score: 1.0,
            })
            .collect(),
    }
}

pub(super) fn from_candidates(candidates: Vec<EntityMatchCandidate>) -> EntityMatchResult {
    match candidates.len() {
        0 => EntityMatchResult {
            status: "no_match".to_string(),
            matched_id: None,
            candidates,
        },
        1 => EntityMatchResult {
            status: "exact".to_string(),
            matched_id: Some(candidates[0].id),
            candidates,
        },
        _ => EntityMatchResult {
            status: "ambiguous".to_string(),
            matched_id: None,
            candidates,
        },
    }
}

pub(super) fn no_match() -> EntityMatchResult {
    from_candidates(Vec::new())
}
