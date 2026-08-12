use crate::{ApplicationError, ApplicationResult};
use football_domain::ReleaseAcceptanceRequest;

pub(super) fn normalize_request(
    mut request: ReleaseAcceptanceRequest,
) -> ApplicationResult<ReleaseAcceptanceRequest> {
    request.performance_window_days = request.performance_window_days.clamp(1, 365);
    request.cost_window_days = request.cost_window_days.clamp(1, 365);
    validate_budget(request.daily_cost_budget_usd, "单日成本预算")?;
    validate_budget(request.monthly_cost_budget_usd, "周期成本预算")?;
    Ok(request)
}

fn validate_budget(value: Option<f64>, label: &str) -> ApplicationResult<()> {
    if value.is_some_and(|number| !number.is_finite() || number < 0.0) {
        return Err(ApplicationError::Validation(format!(
            "{label}必须是大于或等于 0 的有限数值"
        )));
    }
    Ok(())
}
