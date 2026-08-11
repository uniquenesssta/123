use crate::{ports::exchange::MatchLineupExchangePort, ApplicationResult};
use football_domain::SpreadsheetImportPreview;
use std::future::Future;
use uuid::Uuid;

pub(crate) async fn execute<P, F>(
    session: F,
    batch_id: Uuid,
) -> ApplicationResult<SpreadsheetImportPreview>
where
    P: MatchLineupExchangePort,
    F: Future<Output = ApplicationResult<P>>,
{
    let port = session.await?;
    Ok(port.read_import_preview(batch_id).await?)
}
