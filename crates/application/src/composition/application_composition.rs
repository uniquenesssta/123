use super::PortRegistry;
use crate::model_registry::ModelRegistry;
use crate::model_shell::PublicModelStub;
use crate::services::{
    competition::CompetitionService, database::DatabaseService, players::PlayerService,
    rules::RulesService, teams::TeamService,
};
use std::sync::{atomic::AtomicBool, Arc};
pub(crate) struct ApplicationComposition {
    registry: ModelRegistry,
    database: DatabaseService,
    competition: CompetitionService,
    rules: RulesService,
    teams: TeamService,
    players: PlayerService,
    p4_worker_running: AtomicBool,
}
impl ApplicationComposition {
    pub(crate) fn new() -> Self {
        let mut registry = ModelRegistry::new();
        for model in PublicModelStub::built_in_models() {
            registry.register(Arc::new(model));
        }
        let database = DatabaseService::new(PortRegistry::new());
        Self {
            registry,
            database,
            competition: CompetitionService::new(),
            rules: RulesService::new(),
            teams: TeamService::new(),
            players: PlayerService::new(),
            p4_worker_running: AtomicBool::new(false),
        }
    }
    pub(crate) fn into_parts(
        self,
    ) -> (
        ModelRegistry,
        DatabaseService,
        CompetitionService,
        RulesService,
        TeamService,
        PlayerService,
        AtomicBool,
    ) {
        (
            self.registry,
            self.database,
            self.competition,
            self.rules,
            self.teams,
            self.players,
            self.p4_worker_running,
        )
    }
}
