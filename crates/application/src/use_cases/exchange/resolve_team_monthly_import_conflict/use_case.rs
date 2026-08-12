use crate::{ports::exchange::MonthlyWorkbookPort, ApplicationResult};
use football_domain::{SpreadsheetImportPreview, SpreadsheetImportResolution};
use std::future::Future;
use uuid::Uuid;

pub(crate) async fn execute<P, F>(
    session: F,
    batch_id: Uuid,
    resolution: SpreadsheetImportResolution,
) -> ApplicationResult<SpreadsheetImportPreview>
where
    P: MonthlyWorkbookPort,
    F: Future<Output = ApplicationResult<P>>,
{
    let port = session.await?;
    Ok(port.resolve_conflict(batch_id, &resolution).await?)
}
