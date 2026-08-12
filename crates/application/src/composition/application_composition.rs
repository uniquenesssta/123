use super::PortRegistry;
use crate::model_registry::ModelRegistry;
use crate::model_shell::PublicModelStub;
use crate::services::{
    ai_workspace::AiWorkspaceService, analytics::AnalyticsService, competition::CompetitionService,
    database::DatabaseService, exchange::ExchangeService, lineups::LineupService,
    p4_orchestration::P4OrchestrationService, players::PlayerService, postmatch::PostmatchService,
    prediction::PredictionService, release::ReleaseService, research::ResearchService,
    review::ReviewService, rules::RulesService, teams::TeamService,
};
use std::sync::Arc;

pub(crate) struct ApplicationComposition {
    registry: ModelRegistry,
    database: DatabaseService,
    competition: CompetitionService,
    rules: RulesService,
    teams: TeamService,
    players: PlayerService,
    lineups: LineupService,
    p4_orchestration: P4OrchestrationService,
    prediction: PredictionService,
    research: ResearchService,
    review: ReviewService,
    postmatch: PostmatchService,
    analytics: AnalyticsService,
    exchange: ExchangeService,
    ai_workspace: AiWorkspaceService,
    release: ReleaseService,
}

pub(crate) struct ApplicationParts {
    pub(crate) registry: ModelRegistry,
    pub(crate) database: DatabaseService,
    pub(crate) competition: CompetitionService,
    pub(crate) rules: RulesService,
    pub(crate) teams: TeamService,
    pub(crate) players: PlayerService,
    pub(crate) lineups: LineupService,
    pub(crate) p4_orchestration: P4OrchestrationService,
    pub(crate) prediction: PredictionService,
    pub(crate) research: ResearchService,
    pub(crate) review: ReviewService,
    pub(crate) postmatch: PostmatchService,
    pub(crate) analytics: AnalyticsService,
    pub(crate) exchange: ExchangeService,
    pub(crate) ai_workspace: AiWorkspaceService,
    pub(crate) release: ReleaseService,
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
            lineups: LineupService::new(),
            p4_orchestration: P4OrchestrationService::new(),
            prediction: PredictionService::new(),
            research: ResearchService::new(),
            review: ReviewService::new(),
            postmatch: PostmatchService::new(),
            analytics: AnalyticsService::new(),
            exchange: ExchangeService::new(),
            ai_workspace: AiWorkspaceService::new(),
            release: ReleaseService::new(),
        }
    }

    pub(crate) fn into_parts(self) -> ApplicationParts {
        ApplicationParts {
            registry: self.registry,
            database: self.database,
            competition: self.competition,
            rules: self.rules,
            teams: self.teams,
            players: self.players,
            lineups: self.lineups,
            p4_orchestration: self.p4_orchestration,
            prediction: self.prediction,
            research: self.research,
            review: self.review,
            postmatch: self.postmatch,
            analytics: self.analytics,
            exchange: self.exchange,
            ai_workspace: self.ai_workspace,
            release: self.release,
        }
    }
}
