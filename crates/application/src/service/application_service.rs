use crate::composition::ApplicationComposition;
use crate::model_registry::ModelRegistry;
use crate::services::{
    competition::CompetitionService, database::DatabaseService, players::PlayerService,
    rules::RulesService, teams::TeamService,
};
use std::sync::atomic::AtomicBool;
pub struct ApplicationService {
    pub(crate) registry: ModelRegistry,
    pub(crate) database: DatabaseService,
    pub(crate) competition: CompetitionService,
    pub(crate) rules: RulesService,
    pub(crate) teams: TeamService,
    pub(crate) players: PlayerService,
    pub(crate) p4_worker_running: AtomicBool,
}
impl ApplicationService {
    pub fn new() -> Self {
        let (registry, database, competition, rules, teams, players, p4_worker_running) =
            ApplicationComposition::new().into_parts();
        Self {
            registry,
            database,
            competition,
            rules,
            teams,
            players,
            p4_worker_running,
        }
    }
}
impl Default for ApplicationService {
    fn default() -> Self {
        Self::new()
    }
}
