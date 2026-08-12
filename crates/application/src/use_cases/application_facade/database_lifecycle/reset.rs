use super::connect;
use crate::composition::{DatabaseHealth, DatabaseOptions};
use crate::{ApplicationResult, ApplicationService};
use std::sync::Arc;

pub(crate) async fn execute(
    application: &Arc<ApplicationService>,
    options: DatabaseOptions,
    confirmation: String,
) -> ApplicationResult<DatabaseHealth> {
    if let Err(error) = application
        .database
        .reset_to_pristine(&options, &confirmation)
        .await
    {
        if !application.database.is_connected().await {
            let _ = connect::execute(application, options.clone()).await;
        }
        return Err(error.into());
    }
    connect::execute(application, options).await
}
