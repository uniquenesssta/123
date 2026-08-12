use super::check;
use football_domain::{
    ReleaseAcceptanceCheck, ReleaseAcceptanceRequest, ReleaseAcceptanceRuntimeFacts,
    ReleaseAcceptanceStatus,
};
use serde_json::json;
use uuid::Uuid;

pub(super) fn checks(
    run_id: Uuid,
    facts: &ReleaseAcceptanceRuntimeFacts,
    request: &ReleaseAcceptanceRequest,
) -> Vec<ReleaseAcceptanceCheck> {
    let daily_exceeded = request
        .daily_cost_budget_usd
        .is_some_and(|budget| facts.latest_day_cost_usd > budget);
    let period_exceeded = request
        .monthly_cost_budget_usd
        .is_some_and(|budget| facts.estimated_cost_usd > budget);
    let status = if daily_exceeded || period_exceeded {
        ReleaseAcceptanceStatus::Blocked
    } else if request.daily_cost_budget_usd.is_none() || request.monthly_cost_budget_usd.is_none() {
        ReleaseAcceptanceStatus::Warning
    } else {
        ReleaseAcceptanceStatus::Pass
    };
    vec![check(
        run_id,
        ("cost", "openai_cost_observability", "OpenAI 成本与预算"),
        status,
        if daily_exceeded || period_exceeded {
            "显式成本预算已经超限，发布验收被阻断。".to_string()
        } else if status == ReleaseAcceptanceStatus::Warning {
            format!(
                "{} 日窗口估算成本 ${:.4}，最新有用量日期成本 ${:.4}；至少一个预算未设置。",
                request.cost_window_days, facts.estimated_cost_usd, facts.latest_day_cost_usd
            )
        } else {
            format!(
                "成本在显式预算内：周期 ${:.4}，最新日 ${:.4}。",
                facts.estimated_cost_usd, facts.latest_day_cost_usd
            )
        },
        match status {
            ReleaseAcceptanceStatus::Blocked => Some("降低调用量或调整经批准的预算后重新验收。"),
            ReleaseAcceptanceStatus::Warning => {
                Some("在本页设置单日与周期预算，避免只监控不设闸门。")
            }
            ReleaseAcceptanceStatus::Pass => None,
        },
        json!({
            "window_days": request.cost_window_days,
            "completed_requests": facts.completed_requests,
            "failed_requests": facts.failed_requests,
            "search_calls": facts.search_calls,
            "estimated_cost_usd": facts.estimated_cost_usd,
            "latest_day_cost_usd": facts.latest_day_cost_usd,
            "daily_budget_usd": request.daily_cost_budget_usd,
            "period_budget_usd": request.monthly_cost_budget_usd
        }),
        0,
    )]
}
