use crate::{ApplicationResult, ApplicationService};
use serde_json::Value;
use uuid::Uuid;

impl ApplicationService {
    pub async fn api_workspace_context(
        &self,
        match_id: Option<Uuid>,
        entity_type: Option<&str>,
        entity_id: Option<Uuid>,
    ) -> ApplicationResult<Value> {
        self.ai_workspace
            .context(
                self.ai_workspace_session(),
                match_id,
                entity_type,
                entity_id,
            )
            .await
    }
}
