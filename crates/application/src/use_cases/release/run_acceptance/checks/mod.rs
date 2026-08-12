mod chain;
mod cost;
mod performance;
mod release;
mod security;

use football_domain::{
    ReleaseAcceptanceCheck, ReleaseAcceptanceRequest, ReleaseAcceptanceRuntimeFacts,
    ReleaseAcceptanceStatus,
};
use serde_json::Value;
use uuid::Uuid;

pub(super) fn build(
    run_id: Uuid,
    facts: &ReleaseAcceptanceRuntimeFacts,
    request: &ReleaseAcceptanceRequest,
) -> Vec<ReleaseAcceptanceCheck> {
    let mut checks = Vec::new();
    checks.extend(chain::runtime_checks(run_id, facts));
    checks.extend(chain::fixture_checks(run_id));
    checks.extend(performance::checks(run_id, facts));
    checks.extend(security::checks(run_id, facts));
    checks.extend(cost::checks(run_id, facts, request));
    checks.extend(release::checks(run_id, facts));
    for (index, check) in checks.iter_mut().enumerate() {
        check.sequence_no = i32::try_from(index + 1).unwrap_or(i32::MAX);
    }
    checks
}

pub(super) fn check(
    run_id: Uuid,
    (category, code, title): (&str, &str, &str),
    status: ReleaseAcceptanceStatus,
    summary: impl Into<String>,
    remediation: Option<&str>,
    evidence: Value,
    duration_ms: i64,
) -> ReleaseAcceptanceCheck {
    ReleaseAcceptanceCheck {
        id: Uuid::new_v4(),
        run_id,
        sequence_no: 0,
        category: category.to_string(),
        check_code: code.to_string(),
        title: title.to_string(),
        status,
        summary: summary.into(),
        remediation: remediation.map(str::to_string),
        evidence,
        duration_ms: duration_ms.max(0),
    }
}
