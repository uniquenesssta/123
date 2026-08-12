use crate::composition::ApplicationComposition;
use crate::model_registry::ModelRegistry;
use crate::services::{
    ai_workspace::AiWorkspaceService, analytics::AnalyticsService, competition::CompetitionService,
    database::DatabaseService, exchange::ExchangeService, lineups::LineupService,
    players::PlayerService, postmatch::PostmatchService, prediction::PredictionService,
    release::ReleaseService, research::ResearchService, review::ReviewService, rules::RulesService,
    teams::TeamService,
};
use std::sync::atomic::AtomicBool;

pub struct ApplicationService {
    pub(crate) registry: ModelRegistry,
    pub(crate) database: DatabaseService,
    pub(crate) competition: CompetitionService,
    pub(crate) rules: RulesService,
    pub(crate) teams: TeamService,
    pub(crate) players: PlayerService,
    pub(crate) lineups: LineupService,
    pub(crate) prediction: PredictionService,
    pub(crate) research: ResearchService,
    pub(crate) review: ReviewService,
    pub(crate) postmatch: PostmatchService,
    pub(crate) analytics: AnalyticsService,
    pub(crate) exchange: ExchangeService,
    pub(crate) ai_workspace: AiWorkspaceService,
    pub(crate) release: ReleaseService,
    pub(crate) p4_worker_running: AtomicBool,
}

impl ApplicationService {
    pub fn new() -> Self {
        let parts = ApplicationComposition::new().into_parts();
        Self {
            registry: parts.registry,
            database: parts.database,
            competition: parts.competition,
            rules: parts.rules,
            teams: parts.teams,
            players: parts.players,
            lineups: parts.lineups,
            prediction: parts.prediction,
            research: parts.research,
            review: parts.review,
            postmatch: parts.postmatch,
            analytics: parts.analytics,
            exchange: parts.exchange,
            ai_workspace: parts.ai_workspace,
            release: parts.release,
            p4_worker_running: parts.p4_worker_running,
        }
    }
}

impl Default for ApplicationService {
    fn default() -> Self {
        Self::new()
    }
}
