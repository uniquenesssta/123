use uuid::Uuid;

#[derive(Debug, sqlx::FromRow)]
pub(super) struct TeamListRow {
    pub(super) id: Uuid,
    pub(super) canonical_name: String,
    pub(super) normalized_name: String,
    pub(super) country_code: Option<String>,
    pub(super) team_type: String,
    pub(super) current_coach_name: Option<String>,
    pub(super) is_active: bool,
    pub(super) current_player_count: i64,
    pub(super) unavailable_player_count: i64,
    pub(super) squad_ability_average: Option<f64>,
    pub(super) profile_confidence: Option<f64>,
}
