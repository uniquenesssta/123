use crate::PersistenceResult;
use football_domain::MatchReviewEventDraft;
use serde_json::{json, Value};
use sha2::{Digest, Sha256};
use uuid::Uuid;

pub(super) fn match_review_package_id(metadata: &Value) -> Option<Uuid> {
    metadata
        .get("package_id")
        .and_then(Value::as_str)
        .and_then(|value| Uuid::parse_str(value).ok())
}

pub(super) fn match_event_key(
    event: &MatchReviewEventDraft,
    match_id: Uuid,
    index: usize,
) -> PersistenceResult<String> {
    if let Some(value) = event
        .event_key
        .as_deref()
        .map(str::trim)
        .filter(|value| !value.is_empty())
    {
        return Ok(value.to_string());
    }
    let identity = json!({
        "match_id": match_id,
        "sequence_no": event.sequence_no.unwrap_or(index as i32 + 1),
        "event_type": event.event_type.as_str(),
        "team_id": event.team_id,
        "player_id": event.player_id,
        "related_player_id": event.related_player_id,
        "minute": event.minute,
        "stoppage_minute": event.stoppage_minute,
        "period": event.period.trim(),
    });
    let digest = Sha256::digest(serde_json::to_vec(&identity)?);
    Ok(format!("generated:{}", hex::encode(digest)))
}
