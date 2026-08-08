use crate::ports::{
    database::{DatabaseHealthSnapshot, DatabaseObservabilityPort},
    PortResult,
};

pub(crate) async fn execute(
    port: &(impl DatabaseObservabilityPort + ?Sized),
) -> PortResult<DatabaseHealthSnapshot> {
    port.health().await
}
