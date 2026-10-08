use crate::adapters::p4::idempotency::validate_idempotency_key;
use crate::{sha256_json, PersistenceError, PersistenceResult};
use football_domain::{EvidenceClaimDraft, EvidenceConflictDraft};
use serde_json::json;
use std::collections::BTreeSet;
use uuid::Uuid;

pub(super) fn validate_evidence_claim(draft: &EvidenceClaimDraft) -> PersistenceResult<()> {
    validate_idempotency_key(&draft.idempotency_key)?;
    if draft.entity_type.trim().is_empty()
        || draft.field_key.trim().is_empty()
        || draft.source_tier.trim().is_empty()
        || draft.timezone.trim().is_empty()
        || draft.schema_version.trim().is_empty()
    {
        return Err(PersistenceError::InvalidState(
            "证据实体、字段、来源等级、时区和Schema版本不能为空".to_string(),
        ));
    }
    if draft.verification_state.requires_source()
        && (draft.source_url.as_deref().is_none_or(str::is_empty)
            || draft.source_title.as_deref().is_none_or(str::is_empty)
            || draft.source_domain.as_deref().is_none_or(str::is_empty))
    {
        return Err(PersistenceError::InvalidState(
            "有事实来源的证据必须保存URL、标题和域名".to_string(),
        ));
    }
    if draft.retrieved_at < draft.observed_at {
        return Err(PersistenceError::InvalidState(
            "retrieved_at不能早于observed_at".to_string(),
        ));
    }
    Ok(())
}

pub(super) fn evidence_claim_fingerprint(
    draft: &EvidenceClaimDraft,
    content_sha256: &str,
) -> PersistenceResult<String> {
    sha256_json(&json!({
        "match_id": draft.match_id,
        "entity_type": draft.entity_type,
        "entity_id": draft.entity_id,
        "field_key": draft.field_key,
        "verification_state": draft.verification_state.as_str(),
        "source_tier": draft.source_tier,
        "source_document_id": draft.source_document_id,
        "source_url": draft.source_url,
        "source_title": draft.source_title,
        "source_domain": draft.source_domain,
        "published_at": draft.published_at,
        "observed_at": draft.observed_at,
        "effective_at": draft.effective_at,
        "retrieved_at": draft.retrieved_at,
        "timezone": draft.timezone,
        "independent_source_count": draft.independent_source_count,
        "conflict_group_id": draft.conflict_group_id,
        "content_sha256": content_sha256,
        "research_run_id": draft.research_run_id,
        "prompt_version_id": draft.prompt_version_id,
        "prompt_version": draft.prompt_version,
        "schema_version_id": draft.schema_version_id,
        "schema_version": draft.schema_version,
        "metadata": draft.metadata,
    }))
}

pub(super) fn prepared_conflict(
    draft: &EvidenceConflictDraft,
) -> PersistenceResult<(BTreeSet<Uuid>, String)> {
    validate_idempotency_key(&draft.conflict_key)?;
    let evidence_ids = draft.evidence_ids.iter().copied().collect::<BTreeSet<_>>();
    if evidence_ids.len() < 2 {
        return Err(PersistenceError::InvalidState(
            "冲突组至少需要两条不同证据".to_string(),
        ));
    }
    let conflict_fingerprint = sha256_json(&json!({
        "match_id": draft.match_id,
        "entity_type": draft.entity_type,
        "entity_id": draft.entity_id,
        "field_key": draft.field_key,
        "evidence_ids": evidence_ids,
        "trace_id": draft.trace_id,
    }))?;
    Ok((evidence_ids, conflict_fingerprint))
}
