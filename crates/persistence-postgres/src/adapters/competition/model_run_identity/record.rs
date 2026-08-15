use super::record_row::ModelRunIdentityRow;
use uuid::Uuid;

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct ModelRunIdentityRecord {
    pub(crate) id: Uuid,
    pub(crate) model_key: String,
    pub(crate) model_version: String,
    pub(crate) parameter_version: String,
    pub(crate) rule_package_id: Option<Uuid>,
    pub(crate) rule_package_key: Option<String>,
    pub(crate) rule_package_version: Option<String>,
    pub(crate) rule_package_name: Option<String>,
    pub(crate) route_binding_id: Option<Uuid>,
}

impl From<ModelRunIdentityRow> for ModelRunIdentityRecord {
    fn from(row: ModelRunIdentityRow) -> Self {
        Self {
            id: row.id,
            model_key: row.model_key,
            model_version: row.model_version,
            parameter_version: row.parameter_version,
            rule_package_id: row.rule_package_id,
            rule_package_key: row.rule_package_key,
            rule_package_version: row.rule_package_version,
            rule_package_name: row.rule_package_name,
            route_binding_id: row.route_binding_id,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn id(value: u128) -> Uuid {
        Uuid::from_u128(value)
    }

    #[test]
    fn maps_complete_model_run_identity() {
        let record = ModelRunIdentityRecord::from(ModelRunIdentityRow {
            id: id(1),
            model_key: "poisson-v10".to_string(),
            model_version: "10.8.3".to_string(),
            parameter_version: "p-2026-08".to_string(),
            rule_package_id: Some(id(2)),
            rule_package_key: Some("league-default".to_string()),
            rule_package_version: Some("1".to_string()),
            rule_package_name: Some("League Default".to_string()),
            route_binding_id: Some(id(3)),
        });

        assert_eq!(record.id, id(1));
        assert_eq!(record.model_key, "poisson-v10");
        assert_eq!(record.rule_package_id, Some(id(2)));
        assert_eq!(record.route_binding_id, Some(id(3)));
    }

    #[test]
    fn preserves_nullable_rule_package_and_binding_relationships() {
        let record = ModelRunIdentityRecord::from(ModelRunIdentityRow {
            id: id(10),
            model_key: "shadow".to_string(),
            model_version: "1".to_string(),
            parameter_version: "default".to_string(),
            rule_package_id: None,
            rule_package_key: None,
            rule_package_version: None,
            rule_package_name: None,
            route_binding_id: None,
        });

        assert_eq!(record.rule_package_id, None);
        assert_eq!(record.rule_package_key, None);
        assert_eq!(record.rule_package_version, None);
        assert_eq!(record.rule_package_name, None);
        assert_eq!(record.route_binding_id, None);
    }
}
