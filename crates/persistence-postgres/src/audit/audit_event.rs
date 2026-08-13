use super::audit_payload::AuditPayload;
use uuid::Uuid;

#[derive(Debug)]
pub(crate) struct AuditEntityId(Option<String>);

impl AuditEntityId {
    pub(super) fn into_option(self) -> Option<String> {
        self.0
    }
}

impl From<String> for AuditEntityId {
    fn from(value: String) -> Self {
        Self(Some(value))
    }
}

impl From<Option<String>> for AuditEntityId {
    fn from(value: Option<String>) -> Self {
        Self(value)
    }
}

#[derive(Debug)]
pub(super) struct AuditEvent<'a> {
    pub(super) id: Uuid,
    pub(super) event_type: &'a str,
    pub(super) entity_type: &'a str,
    pub(super) entity_id: AuditEntityId,
    pub(super) payload: AuditPayload,
}

impl<'a> AuditEvent<'a> {
    pub(super) fn new(
        event_type: &'a str,
        entity_type: &'a str,
        entity_id: impl Into<AuditEntityId>,
        payload: AuditPayload,
    ) -> Self {
        Self {
            id: Uuid::new_v4(),
            event_type,
            entity_type,
            entity_id: entity_id.into(),
            payload,
        }
    }
}
