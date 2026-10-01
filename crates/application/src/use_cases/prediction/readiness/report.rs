use football_domain::{
    PredictionReadinessCheck, PredictionReadinessCheckStatus, PredictionReadinessLevel,
};
use serde_json::Value;

pub(super) fn readiness_check(
    (code, label): (&str, &str),
    status: PredictionReadinessCheckStatus,
    weight: u8,
    score: u8,
    summary: &str,
    details: Vec<String>,
    metadata: Value,
) -> PredictionReadinessCheck {
    PredictionReadinessCheck {
        code: code.to_string(),
        label: label.to_string(),
        status,
        weight,
        score: score.min(weight),
        summary: summary.to_string(),
        details,
        metadata,
    }
}

pub(super) struct Assessment {
    pub(super) blockers: Vec<String>,
    pub(super) warnings: Vec<String>,
    pub(super) score: u8,
    pub(super) level: PredictionReadinessLevel,
}

pub(super) fn summarize(
    checks: &[PredictionReadinessCheck],
    shadow_reasons: &[String],
) -> Assessment {
    let blockers = checks
        .iter()
        .filter(|check| check.status == PredictionReadinessCheckStatus::Blocked)
        .flat_map(|check| {
            if check.details.is_empty() {
                vec![format!("{}：{}", check.label, check.summary)]
            } else {
                check
                    .details
                    .iter()
                    .map(|detail| format!("{}：{detail}", check.label))
                    .collect()
            }
        })
        .collect::<Vec<_>>();
    let mut warnings = checks
        .iter()
        .filter(|check| check.status == PredictionReadinessCheckStatus::Warning)
        .flat_map(|check| {
            if check.details.is_empty() {
                vec![format!("{}：{}", check.label, check.summary)]
            } else {
                check
                    .details
                    .iter()
                    .map(|detail| format!("{}：{detail}", check.label))
                    .collect()
            }
        })
        .collect::<Vec<_>>();
    for reason in shadow_reasons {
        if !warnings.contains(reason) {
            warnings.push(reason.clone());
        }
    }
    let score = checks
        .iter()
        .map(|check| u16::from(check.score))
        .sum::<u16>()
        .min(100) as u8;
    let level = if !blockers.is_empty() {
        PredictionReadinessLevel::Blocked
    } else if !shadow_reasons.is_empty() {
        PredictionReadinessLevel::ShadowOnly
    } else if !warnings.is_empty() {
        PredictionReadinessLevel::ReadyWithWarnings
    } else {
        PredictionReadinessLevel::FormalReady
    };
    Assessment {
        blockers,
        warnings,
        score,
        level,
    }
}
