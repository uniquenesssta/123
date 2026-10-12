use crate::{postmatch::round6, PersistenceError, PersistenceResult};
use football_domain::EvidenceVerdict;

pub(super) fn validate_evidence_status(status: Option<&str>) -> PersistenceResult<()> {
    if let Some(value) = status {
        if !matches!(value, "pending" | "scored" | "not_verifiable") {
            return Err(PersistenceError::InvalidState(
                "证据评分状态必须为 pending、scored 或 not_verifiable".to_string(),
            ));
        }
    }
    Ok(())
}

pub(super) fn validate_evidence_decision_note(note: &str) -> PersistenceResult<()> {
    if note.trim().len() < 8 {
        return Err(PersistenceError::InvalidState(
            "证据判定说明至少需要 8 个字符".to_string(),
        ));
    }
    Ok(())
}

pub(super) fn verdict_scores(
    verdict: EvidenceVerdict,
    verification_state: &str,
    source_tier: &str,
) -> (Option<f64>, Option<f64>) {
    if verdict == EvidenceVerdict::NotVerifiable {
        return (None, None);
    }
    let accuracy: f64 = match verdict {
        EvidenceVerdict::Correct => 1.0,
        EvidenceVerdict::Partial => 0.5,
        EvidenceVerdict::Incorrect => 0.0,
        EvidenceVerdict::NotVerifiable => unreachable!(),
    };
    let verification_quality: f64 = match verification_state {
        "CONFIRMED" => 1.0,
        "PROBABLE" => 0.8,
        "CONFLICT" => 0.35,
        "STALE" => 0.25,
        "NOT_FOUND" | "NOT_APPLICABLE" => 0.5,
        _ => 0.5,
    };
    let source_quality: f64 = match source_tier.to_ascii_lowercase().as_str() {
        "official" | "tier_1" | "tier1" => 1.0,
        "reliable" | "tier_2" | "tier2" => 0.8,
        "secondary" | "tier_3" | "tier3" => 0.6,
        _ => 0.5,
    };
    let reliability =
        (accuracy * 0.70 + verification_quality * 0.15 + source_quality * 0.15).clamp(0.0, 1.0);
    (Some(round6(accuracy)), Some(round6(reliability)))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn not_verifiable_does_not_fake_accuracy() {
        assert_eq!(
            verdict_scores(EvidenceVerdict::NotVerifiable, "CONFIRMED", "official"),
            (None, None)
        );
        let (accuracy, reliability) =
            verdict_scores(EvidenceVerdict::Correct, "CONFIRMED", "official");
        assert_eq!(accuracy, Some(1.0));
        assert_eq!(reliability, Some(1.0));
    }
}
