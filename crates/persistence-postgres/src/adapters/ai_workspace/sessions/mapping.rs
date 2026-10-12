use crate::PersistenceResult;
use football_domain::{
    ApiWorkspaceGeneratedFileRecord, ApiWorkspaceMessageRecord, ApiWorkspaceSessionRecord,
};
use sqlx::Row;

pub(super) fn session_from_row(
    row: &sqlx::postgres::PgRow,
) -> PersistenceResult<ApiWorkspaceSessionRecord> {
    Ok(ApiWorkspaceSessionRecord {
        id: row.try_get("id")?,
        profile_id: row.try_get("profile_id")?,
        preset_key: row.try_get("preset_key")?,
        title: row.try_get("title")?,
        match_id: row.try_get("match_id")?,
        match_label: row.try_get("match_label")?,
        metadata: row.try_get("metadata")?,
        status: row.try_get("status")?,
        message_count: row.try_get("message_count")?,
        pending_operation_count: row.try_get("pending_operation_count")?,
        created_at: row.try_get("created_at")?,
        updated_at: row.try_get("updated_at")?,
    })
}

pub(super) fn message_from_row(
    row: &sqlx::postgres::PgRow,
) -> PersistenceResult<ApiWorkspaceMessageRecord> {
    Ok(ApiWorkspaceMessageRecord {
        id: row.try_get("id")?,
        session_id: row.try_get("session_id")?,
        role: row.try_get("role")?,
        content: row.try_get("content")?,
        structured_payload: row.try_get("structured_payload")?,
        citations: row.try_get("citations")?,
        attachments: row.try_get("attachments")?,
        provider_response_id: row.try_get("provider_response_id")?,
        model_id: row.try_get("model_id")?,
        token_usage: row.try_get("token_usage")?,
        created_at: row.try_get("created_at")?,
    })
}

pub(super) fn file_from_row(
    row: &sqlx::postgres::PgRow,
) -> PersistenceResult<ApiWorkspaceGeneratedFileRecord> {
    Ok(ApiWorkspaceGeneratedFileRecord {
        id: row.try_get("id")?,
        session_id: row.try_get("session_id")?,
        message_id: row.try_get("message_id")?,
        filename: row.try_get("filename")?,
        media_type: row.try_get("media_type")?,
        content_sha256: row.try_get("content_sha256")?,
        size_bytes: row.try_get("size_bytes")?,
        created_at: row.try_get("created_at")?,
    })
}
