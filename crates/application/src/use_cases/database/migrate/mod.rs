use crate::ports::{database::DatabaseLifecyclePort, PortResult};

pub(crate) async fn execute(port: &(impl DatabaseLifecyclePort + ?Sized)) -> PortResult<()> {
    port.migrate().await
}
