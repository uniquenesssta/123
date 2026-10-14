use crate::{PersistenceError, PersistenceResult};
use football_domain::{
    ReleaseAcceptanceCheck, ReleaseAcceptanceRun, ReleaseAcceptanceRunSummary,
    ReleaseAcceptanceStatus,
};
use serde_json::Value;
use sqlx::Row;

pub(super) fn run_from_row(
    row: &sqlx::postgres::PgRow,
    checks: Vec<ReleaseAcceptanceCheck>,
) -> PersistenceResult<ReleaseAcceptanceRun> {
    Ok(ReleaseAcceptanceRun {
        id: row.try_get("id")?,
        app_version: row.try_get("app_version")?,
        contract_version: row.try_get("contract_version")?,
        fixture_version: row.try_get("fixture_version")?,
        overall_status: parse_status(row.try_get::<String, _>("overall_status")?.as_str())?,
        started_at: row.try_get("started_at")?,
        completed_at: row.try_get("completed_at")?,
        requested_by: row.try_get("requested_by")?,
        report_sha256: row.try_get("report_sha256")?,
        passed_count: report_count(row.try_get::<i32, _>("passed_count")?),
        warning_count: report_count(row.try_get::<i32, _>("warning_count")?),
        blocked_count: report_count(row.try_get::<i32, _>("blocked_count")?),
        category_summaries: serde_json::from_value(row.try_get::<Value, _>("category_summaries")?)?,
        performance: serde_json::from_value(row.try_get::<Value, _>("performance_summary")?)?,
        cost: serde_json::from_value(row.try_get::<Value, _>("cost_summary")?)?,
        checks,
    })
}

pub(super) fn run_summary_from_row(
    row: &sqlx::postgres::PgRow,
) -> PersistenceResult<ReleaseAcceptanceRunSummary> {
    Ok(ReleaseAcceptanceRunSummary {
        id: row.try_get("id")?,
        app_version: row.try_get("app_version")?,
        overall_status: parse_status(row.try_get::<String, _>("overall_status")?.as_str())?,
        completed_at: row.try_get("completed_at")?,
        requested_by: row.try_get("requested_by")?,
        passed_count: report_count(row.try_get::<i32, _>("passed_count")?),
        warning_count: report_count(row.try_get::<i32, _>("warning_count")?),
        blocked_count: report_count(row.try_get::<i32, _>("blocked_count")?),
        report_sha256: row.try_get("report_sha256")?,
    })
}

pub(super) fn check_from_row(
    row: &sqlx::postgres::PgRow,
) -> PersistenceResult<ReleaseAcceptanceCheck> {
    Ok(ReleaseAcceptanceCheck {
        id: row.try_get("id")?,
        run_id: row.try_get("run_id")?,
        sequence_no: row.try_get("sequence_no")?,
        category: row.try_get("category")?,
        check_code: row.try_get("check_code")?,
        title: row.try_get("title")?,
        status: parse_status(row.try_get::<String, _>("status")?.as_str())?,
        summary: row.try_get("summary")?,
        remediation: row.try_get("remediation")?,
        evidence: row.try_get("evidence")?,
        duration_ms: row.try_get("duration_ms")?,
    })
}

pub(super) fn parse_status(value: &str) -> PersistenceResult<ReleaseAcceptanceStatus> {
    match value {
        "pass" => Ok(ReleaseAcceptanceStatus::Pass),
        "warning" => Ok(ReleaseAcceptanceStatus::Warning),
        "blocked" => Ok(ReleaseAcceptanceStatus::Blocked),
        other => Err(PersistenceError::InvalidState(format!(
            "未知发布验收状态：{other}"
        ))),
    }
}

pub(super) fn report_count(value: i32) -> u32 {
    u32::try_from(value).unwrap_or_default()
}
