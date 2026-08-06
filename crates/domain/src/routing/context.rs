use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::competition::CompetitionKind;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResolvedCompetitionContext {
    pub competition_id: Option<Uuid>,
    pub season_id: Option<Uuid>,
    pub stage_id: Option<Uuid>,
    pub competition_kind: CompetitionKind,
}
