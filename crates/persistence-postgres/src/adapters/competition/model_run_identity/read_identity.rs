use super::{record::ModelRunIdentityRecord, record_row::ModelRunIdentityRow};
use crate::{PersistenceResult, PostgresStore};
use uuid::Uuid;

pub(crate) async fn read_model_run_identity(
    store: &PostgresStore,
    run_id: Uuid,
) -> PersistenceResult<ModelRunIdentityRecord> {
    let row = sqlx::query_as::<_, ModelRunIdentityRow>(
        r#"
        SELECT
            r.id,
            d.model_key,
            v.version AS model_version,
            p.parameter_version,
            rp.id AS rule_package_id,
            rp.package_key AS rule_package_key,
            rp.version AS rule_package_version,
            rp.display_name AS rule_package_name,
            r.route_binding_id
        FROM model.runs r
        JOIN model.versions v ON v.id = r.model_version_id
        JOIN model.definitions d ON d.id = v.model_id
        JOIN model.parameter_sets p ON p.id = r.parameter_set_id
        LEFT JOIN model.rule_packages rp ON rp.id = r.rule_package_id
        WHERE r.id = $1
        "#,
    )
    .bind(run_id)
    .fetch_one(&store.pool)
    .await?;

    Ok(row.into())
}

#[cfg(test)]
mod tests {
    use crate::PersistenceError;

    #[test]
    fn sql_errors_keep_persistence_sqlx_semantics() {
        let error = PersistenceError::from(sqlx::Error::RowNotFound);
        assert!(matches!(
            error,
            PersistenceError::Sqlx(sqlx::Error::RowNotFound)
        ));
    }
}
