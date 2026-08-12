use crate::{
    use_cases::application_facade::bootstrap, ApplicationResult, ApplicationService, BootstrapData,
};

impl ApplicationService {
    pub async fn bootstrap(&self) -> ApplicationResult<BootstrapData> {
        bootstrap::execute(self).await
    }
}
