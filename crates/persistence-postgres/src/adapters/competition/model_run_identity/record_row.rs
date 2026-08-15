use sqlx::FromRow;
use uuid::Uuid;

#[derive(Debug, FromRow)]
pub(super) struct ModelRunIdentityRow {
    pub(super) id: Uuid,
    pub(super) model_key: String,
    pub(super) model_version: String,
    pub(super) parameter_version: String,
    pub(super) rule_package_id: Option<Uuid>,
    pub(super) rule_package_key: Option<String>,
    pub(super) rule_package_version: Option<String>,
    pub(super) rule_package_name: Option<String>,
    pub(super) route_binding_id: Option<Uuid>,
}
