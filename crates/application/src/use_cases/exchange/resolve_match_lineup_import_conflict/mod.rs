use crate::{ports::exchange::MatchLineupExchangePort, ApplicationResult};
use football_domain::{SpreadsheetImportPreview, SpreadsheetImportResolution};
use std::future::Future;
use uuid::Uuid;

pub(crate) async fn execute<P, F>(
    session: F,
    batch_id: Uuid,
    resolution: SpreadsheetImportResolution,
) -> ApplicationResult<SpreadsheetImportPreview>
where
    P: MatchLineupExchangePort,
    F: Future<Output = ApplicationResult<P>>,
{
    let port = session.await?;
    Ok(port.resolve_import_conflict(batch_id, resolution).await?)
}
