use crate::ports::prediction::P4FreezeExecutionPort;
use crate::ApplicationResult;
use football_domain::{PrematchSnapshotBundle, PrematchSnapshotDraft, PrematchSnapshotRecord};
use uuid::Uuid;

pub(crate) async fn freeze<P: P4FreezeExecutionPort + ?Sized>(
    port: &P,
    draft: PrematchSnapshotDraft,
) -> ApplicationResult<PrematchSnapshotRecord> {
    Ok(port.freeze_snapshot(&draft).await?)
}

pub(crate) async fn read<P: P4FreezeExecutionPort + ?Sized>(
    port: &P,
    snapshot_id: Uuid,
) -> ApplicationResult<PrematchSnapshotBundle> {
    Ok(port.read_snapshot(snapshot_id).await?)
}
