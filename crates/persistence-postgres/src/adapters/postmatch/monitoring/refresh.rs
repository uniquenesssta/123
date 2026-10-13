use super::validation::validate_monitoring_request;
use crate::{PersistenceResult, PostgresStore};
use football_domain::{PostmatchMonitoringRequest, PostmatchOverview};

impl PostgresStore {
    pub async fn refresh_postmatch_monitoring(
        &self,
        request: &PostmatchMonitoringRequest,
    ) -> PersistenceResult<PostmatchOverview> {
        validate_monitoring_request(request)?;
        let partition = self
            .active_postmatch_partition(request.competition_id, &request.horizon)
            .await?;
        self.refresh_provider_scores_for_partition(&partition)
            .await?;
        self.refresh_drift_for_partition(&partition, request.baseline_size, request.current_size)
            .await?;
        self.postmatch_overview(100).await
    }
}
