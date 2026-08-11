use crate::{ports::exchange::MatchLineupExchangePort, ApplicationResult};
use football_domain::SpreadsheetImportCommitResult;
use std::future::Future;
use uuid::Uuid;

pub(crate) async fn execute<P, F>(
    session: F,
    batch_id: Uuid,
) -> ApplicationResult<SpreadsheetImportCommitResult>
where
    P: MatchLineupExchangePort,
    F: Future<Output = ApplicationResult<P>>,
{
    let port = session.await?;
    Ok(port.commit_import(batch_id).await?)
}
