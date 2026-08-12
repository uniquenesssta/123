use super::summary::RunSummary;
use crate::ApplicationResult;
use chrono::{DateTime, Utc};
use football_domain::{
    ReleaseAcceptanceCheck, ReleaseAcceptanceRequest, RELEASE_ACCEPTANCE_CONTRACT_VERSION,
    RELEASE_ACCEPTANCE_FIXTURE_VERSION,
};
use serde_json::json;
use sha2::{Digest, Sha256};
use uuid::Uuid;

pub(super) fn hash(
    run_id: Uuid,
    started_at: DateTime<Utc>,
    completed_at: DateTime<Utc>,
    request: &ReleaseAcceptanceRequest,
    summary: &RunSummary,
    checks: &[ReleaseAcceptanceCheck],
) -> ApplicationResult<String> {
    let value = json!({
        "run_id": run_id,
        "app_version": env!("CARGO_PKG_VERSION"),
        "contract_version": RELEASE_ACCEPTANCE_CONTRACT_VERSION,
        "fixture_version": RELEASE_ACCEPTANCE_FIXTURE_VERSION,
        "overall_status": summary.overall_status.as_str(),
        "started_at": started_at,
        "completed_at": completed_at,
        "requested_by": request.requested_by.as_deref(),
        "passed_count": summary.passed_count,
        "warning_count": summary.warning_count,
        "blocked_count": summary.blocked_count,
        "category_summaries": &summary.category_summaries,
        "performance": &summary.performance,
        "cost": &summary.cost,
        "checks": checks,
    });
    let bytes = serde_json::to_vec(&value)?;
    let mut hasher = Sha256::new();
    hasher.update(bytes);
    Ok(hex::encode(hasher.finalize()))
}
