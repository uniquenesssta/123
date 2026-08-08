use crate::{ports::player::PlayerCatalogPort, ApplicationError, ApplicationResult};
use football_domain::{PlayerTeamPeriodDraft, PlayerTeamPeriodRecord};
const ALLOWED_REGISTRATION_STATUSES: [&str; 5] =
    ["registered", "loan", "trial", "released", "unknown"];
pub(crate) async fn execute<P>(
    port: &P,
    draft: PlayerTeamPeriodDraft,
) -> ApplicationResult<PlayerTeamPeriodRecord>
where
    P: PlayerCatalogPort + ?Sized,
{
    validate_registration_status(&draft.registration_status)?;
    Ok(port.add_player_team_period(&draft).await?)
}
fn validate_registration_status(status: &str) -> ApplicationResult<()> {
    if ALLOWED_REGISTRATION_STATUSES.contains(&status) {
        return Ok(());
    }
    Err(ApplicationError::Validation(format!(
        "未知注册状态：{status}"
    )))
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn registration_status_contract_is_preserved() {
        for status in ALLOWED_REGISTRATION_STATUSES {
            assert!(validate_registration_status(status).is_ok());
        }
        assert_eq!(
            validate_registration_status("invalid")
                .unwrap_err()
                .to_string(),
            "赛事或规则包输入无效：未知注册状态：invalid"
        );
    }
}
