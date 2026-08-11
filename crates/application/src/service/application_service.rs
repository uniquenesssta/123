use crate::composition::ApplicationComposition;
use crate::model_registry::ModelRegistry;
use crate::services::{
    analytics::AnalyticsService, competition::CompetitionService, database::DatabaseService,
    exchange::ExchangeService, lineups::LineupService, players::PlayerService,
    postmatch::PostmatchService, prediction::PredictionService, research::ResearchService,
    review::ReviewService, rules::RulesService, teams::TeamService,
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
            p4_worker_running: parts.p4_worker_running,
        }
    }
}

impl Default for ApplicationService {
    fn default() -> Self {
        Self::new()
    }
}
