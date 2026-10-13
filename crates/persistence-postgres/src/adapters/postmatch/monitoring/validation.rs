use crate::{postmatch::FORMAL_HORIZONS, PersistenceError, PersistenceResult};
use football_domain::PostmatchMonitoringRequest;

pub(super) fn validate_monitoring_request(
    request: &PostmatchMonitoringRequest,
) -> PersistenceResult<()> {
    validate_horizon(&request.horizon)?;
    if request.baseline_size < 5 || request.current_size < 5 {
        return Err(PersistenceError::InvalidState(
            "漂移基线窗口和当前窗口都不能少于 5 场".to_string(),
        ));
    }
    Ok(())
}

fn validate_horizon(value: &str) -> PersistenceResult<()> {
    if FORMAL_HORIZONS.contains(&value) {
        Ok(())
    } else {
        Err(PersistenceError::InvalidState(
            "正式赛后监控只支持 T-N、T-24h、T-6h、T-1h".to_string(),
        ))
    }
}
