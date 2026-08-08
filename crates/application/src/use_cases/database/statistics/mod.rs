use crate::ports::{
    database::{DatabaseObservabilityPort, DatabaseStatistics},
    PortResult,
};

pub(crate) async fn execute(
    port: &(impl DatabaseObservabilityPort + ?Sized),
) -> PortResult<DatabaseStatistics> {
    port.statistics().await
}
