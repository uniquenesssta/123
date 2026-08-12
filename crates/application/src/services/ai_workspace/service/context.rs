use super::AiWorkspaceService;
use crate::{
    ports::{
        exchange::MatchLineupExchangePort, player::PlayerCatalogPort, team::TeamCatalogPort,
    },
    use_cases::ai_workspace,
    ApplicationResult,
};
use serde_json::Value;
use std::future::Future;
use uuid::Uuid;

impl AiWorkspaceService {
    pub(crate) async fn context<P, F>(
        &self,
        session: F,
        match_id: Option<Uuid>,
        entity_type: Option<&str>,
        entity_id: Option<Uuid>,
    ) -> ApplicationResult<Value>
    where
        P: MatchLineupExchangePort + PlayerCatalogPort + TeamCatalogPort,
        F: Future<Output = ApplicationResult<P>>,
    {
        ai_workspace::context::execute(session, match_id, entity_type, entity_id).await
    }
}
