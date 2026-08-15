use football_domain::CompetitionKind;
use sqlx::{FromRow, Postgres, Transaction};
use uuid::Uuid;

use crate::{parse_competition_kind, PersistenceError, PersistenceResult};

#[derive(Debug, FromRow)]
struct PackageRouteMetadataRow {
    model_version_id: Option<Uuid>,
    parameter_set_id: Option<Uuid>,
    competition_kind: Option<String>,
}

pub(in crate::adapters::competition::bindings) async fn package_route_metadata(
    tx: &mut Transaction<'_, Postgres>,
    package_id: Uuid,
) -> PersistenceResult<(Uuid, Uuid, CompetitionKind)> {
    let row = sqlx::query_as::<_, PackageRouteMetadataRow>(
        r#"
        SELECT model_version_id, parameter_set_id, competition_kind
        FROM model.rule_packages
        WHERE id = $1 AND status = 'active'
        "#,
    )
    .bind(package_id)
    .fetch_one(&mut **tx)
    .await?;

    let model_version_id = row
        .model_version_id
        .ok_or_else(|| PersistenceError::InvalidState("规则包缺少模型版本".to_string()))?;
    let parameter_set_id = row
        .parameter_set_id
        .ok_or_else(|| PersistenceError::InvalidState("规则包缺少参数版本".to_string()))?;
    let competition_kind = row
        .competition_kind
        .ok_or_else(|| PersistenceError::InvalidState("规则包缺少赛事类型".to_string()))?;

    Ok((
        model_version_id,
        parameter_set_id,
        parse_competition_kind(&competition_kind)?,
    ))
}
