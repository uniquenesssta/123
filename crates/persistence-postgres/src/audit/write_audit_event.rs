use super::{audit_event::AuditEvent, audit_payload::AuditPayload};
use crate::PersistenceResult;
use serde_json::Value;
use sqlx::{Postgres, Transaction};

pub(crate) async fn write_audit_event(
    tx: &mut Transaction<'_, Postgres>,
    event_type: &str,
    entity_type: &str,
    entity_id: impl Into<super::audit_event::AuditEntityId>,
    payload: Value,
) -> PersistenceResult<()> {
    let event = AuditEvent::new(
        event_type,
        entity_type,
        entity_id,
        AuditPayload::new(payload),
    );
    sqlx::query(
        r#"
        INSERT INTO audit.events (id, event_type, entity_type, entity_id, payload)
        VALUES ($1, $2, $3, $4, $5)
        "#,
    )
    .bind(event.id)
    .bind(event.event_type)
    .bind(event.entity_type)
    .bind(event.entity_id.into_option())
    .bind(event.payload.into_value())
    .execute(&mut **tx)
    .await?;
    Ok(())
}
