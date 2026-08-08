use crate::{ports::team::TeamCatalogPort, ApplicationResult};
use football_domain::TeamOption;
pub(crate) async fn execute<P>(
    port: &P,
    search: Option<String>,
    limit: u32,
) -> ApplicationResult<Vec<TeamOption>>
where
    P: TeamCatalogPort + ?Sized,
{
    Ok(port.list_team_options(search.as_deref(), limit).await?)
}
