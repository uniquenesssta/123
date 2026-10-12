use super::policy::{validate_evidence_decision_note, validate_evidence_status, verdict_scores};
use crate::PersistenceError;
use football_domain::EvidenceVerdict;

#[test]
fn status_filter_accepts_none_and_original_three_states() {
    for status in [
        None,
        Some("pending"),
        Some("scored"),
        Some("not_verifiable"),
    ] {
        assert!(validate_evidence_status(status).is_ok());
    }
}

#[test]
fn status_filter_rejects_unknown_case_and_whitespace() {
    for status in ["", "Pending", " pending", "scored ", "accepted", "未知"] {
        assert!(
            matches!(validate_evidence_status(Some(status)), Err(PersistenceError::InvalidState(message)) if message == "证据评分状态必须为 pending、scored 或 not_verifiable")
        );
    }
}

#[test]
fn decision_note_accepts_original_utf8_byte_boundary_after_trim() {
    for note in [
        "12345678",
        "  12345678\n",
        "三字长",
        "\u{3000}三字长\u{2003}",
    ] {
        assert!(validate_evidence_decision_note(note).is_ok());
    }
}

#[test]
fn decision_note_rejects_short_or_blank_with_original_error() {
    for note in ["", "1234567", " \t\n ", "两字", "\u{3000}\u{2003}"] {
        assert!(
            matches!(validate_evidence_decision_note(note), Err(PersistenceError::InvalidState(message)) if message == "证据判定说明至少需要 8 个字符")
        );
    }
}

#[test]
fn verdict_accuracy_preserves_three_manual_values() {
    for (verdict, accuracy, reliability) in [
        (EvidenceVerdict::Correct, 1.0, 1.0),
        (EvidenceVerdict::Partial, 0.5, 0.65),
        (EvidenceVerdict::Incorrect, 0.0, 0.3),
    ] {
        assert_eq!(
            verdict_scores(verdict, "CONFIRMED", "official"),
            (Some(accuracy), Some(reliability))
        );
    }
}

#[test]
fn reliability_preserves_verification_quality_and_case_policy() {
    for (state, expected) in [
        ("CONFIRMED", 1.0),
        ("PROBABLE", 0.97),
        ("CONFLICT", 0.9025),
        ("STALE", 0.8875),
        ("NOT_FOUND", 0.925),
        ("NOT_APPLICABLE", 0.925),
        ("confirmed", 0.925),
        ("", 0.925),
    ] {
        assert_eq!(
            verdict_scores(EvidenceVerdict::Correct, state, "official"),
            (Some(1.0), Some(expected))
        );
    }
}

#[test]
fn reliability_preserves_source_aliases_case_and_untrimmed_fallback() {
    for (tier, expected) in [
        ("official", 1.0),
        ("OFFICIAL", 1.0),
        ("tier_1", 1.0),
        ("tier1", 1.0),
        ("reliable", 0.97),
        ("TIER_2", 0.97),
        ("tier2", 0.97),
        ("secondary", 0.94),
        ("tier_3", 0.94),
        ("TIER3", 0.94),
        (" official ", 0.925),
        ("", 0.925),
    ] {
        assert_eq!(
            verdict_scores(EvidenceVerdict::Correct, "CONFIRMED", tier),
            (Some(1.0), Some(expected))
        );
    }
}

#[test]
fn not_verifiable_has_no_scores_for_any_quality_context() {
    for state in [
        "CONFIRMED",
        "PROBABLE",
        "CONFLICT",
        "STALE",
        "NOT_FOUND",
        "unknown",
        "",
    ] {
        for tier in ["official", "reliable", "secondary", "unknown", ""] {
            assert_eq!(
                verdict_scores(EvidenceVerdict::NotVerifiable, state, tier),
                (None, None)
            );
        }
    }
}
