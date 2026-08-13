use serde_json::Value;

#[derive(Debug)]
pub(super) struct AuditPayload(Value);

impl AuditPayload {
    pub(super) fn new(value: Value) -> Self {
        Self(value)
    }

    pub(super) fn into_value(self) -> Value {
        self.0
    }
}
