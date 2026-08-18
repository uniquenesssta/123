use crate::PersistenceResult;
use football_domain::EntityReferenceRecord;
use sqlx::Row;

pub(super) fn team_reference_from_row(
    row: &sqlx::postgres::PgRow,
) -> PersistenceResult<EntityReferenceRecord> {
    Ok(EntityReferenceRecord {
        entity_type: "team".to_string(),
        id: row.try_get("id")?,
        canonical_name: row.try_get("canonical_name")?,
        normalized_name: row.try_get("normalized_name")?,
        country_code: row.try_get("country_code")?,
        nationality_code: None,
        date_of_birth: None,
        status: if row.try_get::<bool, _>("is_active")? {
            "active".to_string()
        } else {
            "inactive".to_string()
        },
        aliases: row.try_get("aliases")?,
        external_ids: row.try_get("external_ids")?,
    })
}

pub(super) fn player_reference_from_row(
    row: &sqlx::postgres::PgRow,
) -> PersistenceResult<EntityReferenceRecord> {
    Ok(EntityReferenceRecord {
        entity_type: "player".to_string(),
        id: row.try_get("id")?,
        canonical_name: row.try_get("canonical_name")?,
        normalized_name: row.try_get("normalized_name")?,
        country_code: None,
        nationality_code: row.try_get("nationality_code")?,
        date_of_birth: row.try_get("date_of_birth")?,
        status: row.try_get("status")?,
        aliases: row.try_get("aliases")?,
        external_ids: row.try_get("external_ids")?,
    })
}

pub(super) fn coach_reference_from_row(
    row: &sqlx::postgres::PgRow,
) -> PersistenceResult<EntityReferenceRecord> {
    Ok(EntityReferenceRecord {
        entity_type: "coach".to_string(),
        id: row.try_get("id")?,
        canonical_name: row.try_get("canonical_name")?,
        normalized_name: row.try_get("normalized_name")?,
        country_code: None,
        nationality_code: row.try_get("nationality_code")?,
        date_of_birth: None,
        status: row.try_get("status")?,
        aliases: row.try_get("aliases")?,
        external_ids: row.try_get("external_ids")?,
    })
}
