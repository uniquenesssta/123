use football_domain::{
    ReleaseAcceptanceCategorySummary, ReleaseAcceptanceCheck, ReleaseAcceptanceCostSummary,
    ReleaseAcceptancePerformanceSummary, ReleaseAcceptanceRequest, ReleaseAcceptanceRuntimeFacts,
    ReleaseAcceptanceStatus,
};
use std::collections::BTreeMap;

pub(super) struct RunSummary {
    pub(super) overall_status: ReleaseAcceptanceStatus,
    pub(super) passed_count: u32,
    pub(super) warning_count: u32,
    pub(super) blocked_count: u32,
    pub(super) category_summaries: Vec<ReleaseAcceptanceCategorySummary>,
    pub(super) performance: ReleaseAcceptancePerformanceSummary,
    pub(super) cost: ReleaseAcceptanceCostSummary,
}

pub(super) fn build(
    checks: &[ReleaseAcceptanceCheck],
    runtime: &ReleaseAcceptanceRuntimeFacts,
    request: &ReleaseAcceptanceRequest,
) -> RunSummary {
    let passed_count = u32::try_from(
        checks
            .iter()
            .filter(|check| check.status == ReleaseAcceptanceStatus::Pass)
            .count(),
    )
    .unwrap_or(u32::MAX);
    let warning_count = u32::try_from(
        checks
            .iter()
            .filter(|check| check.status == ReleaseAcceptanceStatus::Warning)
            .count(),
    )
    .unwrap_or(u32::MAX);
    let blocked_count = u32::try_from(
        checks
            .iter()
            .filter(|check| check.status == ReleaseAcceptanceStatus::Blocked)
            .count(),
    )
    .unwrap_or(u32::MAX);
    let overall_status = if blocked_count > 0 {
        ReleaseAcceptanceStatus::Blocked
    } else if warning_count > 0 {
        ReleaseAcceptanceStatus::Warning
    } else {
        ReleaseAcceptanceStatus::Pass
    };
    RunSummary {
        overall_status,
        passed_count,
        warning_count,
        blocked_count,
        category_summaries: summarize_categories(checks),
        performance: ReleaseAcceptancePerformanceSummary {
            database_latency_ms: u64::try_from(runtime.database_latency_ms).unwrap_or(u64::MAX),
            recent_model_run_count: u64::try_from(runtime.recent_model_run_count)
                .unwrap_or_default(),
            recent_model_run_p95_ms: runtime.recent_model_run_p95_ms,
            recent_model_failure_count: u64::try_from(runtime.recent_model_failure_count)
                .unwrap_or_default(),
            query_warning_count: u64::try_from(runtime.query_warning_count).unwrap_or_default(),
        },
        cost: ReleaseAcceptanceCostSummary {
            window_days: request.cost_window_days,
            completed_requests: u64::try_from(runtime.completed_requests).unwrap_or_default(),
            failed_requests: u64::try_from(runtime.failed_requests).unwrap_or_default(),
            search_calls: u64::try_from(runtime.search_calls).unwrap_or_default(),
            estimated_cost_usd: runtime.estimated_cost_usd,
            latest_day_cost_usd: runtime.latest_day_cost_usd,
            daily_budget_usd: request.daily_cost_budget_usd,
            monthly_budget_usd: request.monthly_cost_budget_usd,
        },
    }
}

fn summarize_categories(
    checks: &[ReleaseAcceptanceCheck],
) -> Vec<ReleaseAcceptanceCategorySummary> {
    let mut summaries: BTreeMap<String, ReleaseAcceptanceCategorySummary> = BTreeMap::new();
    for check in checks {
        let summary = summaries.entry(check.category.clone()).or_insert_with(|| {
            ReleaseAcceptanceCategorySummary {
                category: check.category.clone(),
                ..ReleaseAcceptanceCategorySummary::default()
            }
        });
        match check.status {
            ReleaseAcceptanceStatus::Pass => summary.passed += 1,
            ReleaseAcceptanceStatus::Warning => summary.warnings += 1,
            ReleaseAcceptanceStatus::Blocked => summary.blocked += 1,
        }
    }
    summaries.into_values().collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::use_cases::release::run_acceptance::checks::check;
    use serde_json::Value;
    use uuid::Uuid;

    #[test]
    fn category_summary_counts_each_status() {
        let run_id = Uuid::new_v4();
        let checks = vec![
            check(
                run_id,
                ("chain", "a", "a"),
                ReleaseAcceptanceStatus::Pass,
                "",
                None,
                Value::Null,
                0,
            ),
            check(
                run_id,
                ("chain", "b", "b"),
                ReleaseAcceptanceStatus::Warning,
                "",
                None,
                Value::Null,
                0,
            ),
            check(
                run_id,
                ("chain", "c", "c"),
                ReleaseAcceptanceStatus::Blocked,
                "",
                None,
                Value::Null,
                0,
            ),
        ];
        let summary = summarize_categories(&checks);
        assert_eq!(summary[0].passed, 1);
        assert_eq!(summary[0].warnings, 1);
        assert_eq!(summary[0].blocked, 1);
    }
}
