use crate::{ports::player::PlayerSignalPort, ApplicationError, ApplicationResult};
use football_domain::{PlayerAbilityObservationDraft, PlayerAbilityObservationRecord};
pub(crate) async fn execute<P>(
    port: &P,
    draft: PlayerAbilityObservationDraft,
) -> ApplicationResult<PlayerAbilityObservationRecord>
where
    P: PlayerSignalPort + ?Sized,
{
    validate_calculation_version(&draft.calculation_version)?;
    Ok(port.add_ability_observation(&draft).await?)
}
fn validate_calculation_version(version: &str) -> ApplicationResult<()> {
    if !version.trim().is_empty() {
        return Ok(());
    }
    Err(ApplicationError::Validation(
        "能力观察必须提供 calculation_version".to_string(),
    ))
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn calculation_version_remains_required() {
        assert!(validate_calculation_version("v1").is_ok());
        assert_eq!(
            validate_calculation_version("   ").unwrap_err().to_string(),
            "赛事或规则包输入无效：能力观察必须提供 calculation_version"
        );
    }
}
