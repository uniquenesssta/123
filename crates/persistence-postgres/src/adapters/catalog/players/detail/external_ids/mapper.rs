use super::row::ExternalEntityIdRow;
use football_domain::ExternalEntityIdRecord;

pub(super) fn map_external_entity_id(row: ExternalEntityIdRow) -> ExternalEntityIdRecord {
    ExternalEntityIdRecord {
        id: row.id,
        provider_id: row.provider_id,
        provider_name: row.provider_name,
        entity_type: row.entity_type,
        entity_id: row.entity_id,
        external_id: row.external_id,
        metadata: row.metadata,
    }
}
