use crate::{ports::analytics::ParameterLifecyclePort, ApplicationError, ApplicationResult};
use football_domain::{ParameterLifecycleReadiness, ParameterLifecycleReadinessRequest};

pub(crate) async fn execute<P>(
    port: &P,
    request: ParameterLifecycleReadinessRequest,
) -> ApplicationResult<ParameterLifecycleReadiness>
where
    P: ParameterLifecyclePort + ?Sized,
{
    validate_parameter_horizon(&request.snapshot_type)?;
    if request.minimum_sample_size < 20 {
        return Err(ApplicationError::Validation(
            "最低样本量不能少于 20 场".to_string(),
        ));
    }
    Ok(port.readiness(&request).await?)
}

fn validate_parameter_horizon(snapshot_type: &str) -> ApplicationResult<()> {
    if matches!(snapshot_type, "T-N" | "T-24h" | "T-6h" | "T-1h") {
        Ok(())
    } else {
        Err(ApplicationError::Validation(format!(
            "P4 参数收敛只允许 T-N、T-24h、T-6h 或 T-1h，收到：{snapshot_type}"
        )))
    }
}
