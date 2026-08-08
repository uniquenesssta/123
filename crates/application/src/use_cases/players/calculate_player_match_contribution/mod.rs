use crate::{ports::player::PlayerSignalPort, ApplicationResult};
use football_domain::{PlayerMatchContribution, PlayerMatchContributionRequest};
pub(crate) async fn execute<P>(
    port: &P,
    request: PlayerMatchContributionRequest,
) -> ApplicationResult<PlayerMatchContribution>
where
    P: PlayerSignalPort + ?Sized,
{
    Ok(port.calculate_match_contribution(&request).await?)
}
