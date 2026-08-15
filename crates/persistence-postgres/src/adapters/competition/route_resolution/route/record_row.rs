use serde_json::Value;
use sqlx::FromRow;
use uuid::Uuid;

#[derive(Debug, FromRow)]
pub(super) struct RouteRow {
    pub(super) binding_id: Option<Uuid>,
    pub(super) rule_package_id: Uuid,
    pub(super) package_key: String,
    pub(super) package_version: String,
    pub(super) package_display_name: String,
    pub(super) profile: Value,
    pub(super) competition_profile_id: Uuid,
    pub(super) routing: Value,
    pub(super) feature_requirements: Value,
    pub(super) output_contract: Value,
    pub(super) priority: i32,
    pub(super) model_key: String,
    pub(super) model_version_id: Uuid,
    pub(super) model_version: String,
    pub(super) parameter_set_id: Uuid,
    pub(super) parameter_version: String,
    pub(super) parameters: Value,
    pub(super) competition_id: Option<Uuid>,
    pub(super) season_id: Option<Uuid>,
    pub(super) stage_id: Option<Uuid>,
    pub(super) competition_kind: Option<String>,
}
