use super::{sha256_json, write_audit_event, PersistenceError, PersistenceResult, PostgresStore};
use football_domain::{
    OpenAiAttemptDraft, OpenAiAttemptRecord, OpenAiUsageTotals, WebCitationDraft, WebSourceDraft,
};
use serde_json::json;
use sqlx::Row;
use uuid::Uuid;

impl PostgresStore {
    pub async fn append_openai_attempt(
        &self,
        draft: &OpenAiAttemptDraft,
    ) -> PersistenceResult<OpenAiAttemptRecord> {
        validate_attempt(draft)?;
        let attempt_fingerprint = sha256_json(&json!({
            "research_run_id": draft.research_run_id,
            "attempt_number": draft.attempt_number,
            "model_id": draft.model_id,
            "request_fingerprint": draft.request_fingerprint,
            "request_payload": draft.request_payload,
            "response_id": draft.response_id,
            "provider_request_id": draft.provider_request_id,
            "provider_status": draft.provider_status,
            "status": draft.status,
            "token_usage": draft.token_usage,
            "latency_ms": draft.latency_ms,
            "search_call_count": draft.search_call_count,
            "estimated_cost_usd": draft.estimated_cost_usd,
            "raw_response": draft.raw_response,
            "error_category": draft.error_category,
            "error_message": draft.error_message,
            "retryable": draft.retryable,
            "started_at": draft.started_at,
            "finished_at": draft.finished_at,
        }))?;
        let mut tx = self.pool.begin().await?;
        sqlx::query("SELECT pg_advisory_xact_lock(hashtextextended($1, 0))")
            .bind(format!(
                "openai-attempt:{}:{}",
                draft.research_run_id, draft.attempt_number
            ))
            .execute(&mut *tx)
            .await?;

        let record =
            if let Some(row) = sqlx::query(
                r#"
            SELECT id, research_run_id, attempt_number, attempt_fingerprint,
                   response_id, status, created_at
            FROM research.openai_attempts
            WHERE research_run_id = $1 AND attempt_number = $2
            "#,
            )
            .bind(draft.research_run_id)
            .bind(i32::try_from(draft.attempt_number).map_err(|_| {
                PersistenceError::InvalidState("OpenAI尝试次数超出i32范围".to_string())
            })?)
            .fetch_optional(&mut *tx)
            .await?
            {
                let existing: String = row.try_get("attempt_fingerprint")?;
                if existing != attempt_fingerprint {
                    return Err(PersistenceError::InvalidState(format!(
                        "研究任务{}的第{}次OpenAI尝试已经存在，但载荷指纹不同",
                        draft.research_run_id, draft.attempt_number
                    )));
                }
                attempt_record(&row)?
            } else {
                let id = Uuid::new_v4();
                let row = sqlx::query(
                    r#"
                INSERT INTO research.openai_attempts (
                    id, research_run_id, attempt_number, model_id,
                    request_fingerprint, attempt_fingerprint, request_payload,
                    response_id, provider_request_id, provider_status, status,
                    token_usage, latency_ms, search_call_count, estimated_cost_usd,
                    raw_response, error_category, error_message, retryable,
                    started_at, finished_at
                ) VALUES (
                    $1, $2, $3, $4, $5, $6, $7,
                    $8, $9, $10, $11, $12, $13, $14, $15,
                    $16, $17, $18, $19, $20, $21
                )
                RETURNING id, research_run_id, attempt_number, attempt_fingerprint,
                          response_id, status, created_at
                "#,
                )
                .bind(id)
                .bind(draft.research_run_id)
                .bind(i32::try_from(draft.attempt_number).map_err(|_| {
                    PersistenceError::InvalidState("OpenAI尝试次数超出i32范围".to_string())
                })?)
                .bind(&draft.model_id)
                .bind(&draft.request_fingerprint)
                .bind(&attempt_fingerprint)
                .bind(&draft.request_payload)
                .bind(&draft.response_id)
                .bind(&draft.provider_request_id)
                .bind(draft.provider_status.map(i32::from))
                .bind(&draft.status)
                .bind(&draft.token_usage)
                .bind(i64::try_from(draft.latency_ms).map_err(|_| {
                    PersistenceError::InvalidState("OpenAI延迟超出i64范围".to_string())
                })?)
                .bind(i32::try_from(draft.search_call_count).map_err(|_| {
                    PersistenceError::InvalidState("Web Search调用次数超出i32范围".to_string())
                })?)
                .bind(draft.estimated_cost_usd)
                .bind(&draft.raw_response)
                .bind(&draft.error_category)
                .bind(&draft.error_message)
                .bind(draft.retryable)
                .bind(draft.started_at)
                .bind(draft.finished_at)
                .fetch_one(&mut *tx)
                .await?;
                write_audit_event(
                    &mut tx,
                    "openai_attempt_recorded",
                    "research_run",
                    Some(draft.research_run_id.to_string()),
                    json!({
                        "attempt_number": draft.attempt_number,
                        "model_id": draft.model_id,
                        "status": draft.status,
                        "response_id": draft.response_id,
                        "provider_request_id": draft.provider_request_id,
                        "request_fingerprint": draft.request_fingerprint,
                        "attempt_fingerprint": attempt_fingerprint,
                        "error_category": draft.error_category,
                        "retryable": draft.retryable
                    }),
                )
                .await?;
                attempt_record(&row)?
            };
        tx.commit().await?;
        Ok(record)
    }

    pub async fn openai_attempt_number_offset(
        &self,
        research_run_id: Uuid,
    ) -> PersistenceResult<u32> {
        let value: i32 = sqlx::query_scalar(
            r#"
            SELECT COALESCE(max(attempt_number), 0)::integer
            FROM research.openai_attempts
            WHERE research_run_id = $1
            "#,
        )
        .bind(research_run_id)
        .fetch_one(&self.pool)
        .await?;
        u32::try_from(value)
            .map_err(|_| PersistenceError::InvalidState("OpenAI尝试编号偏移量为负数".to_string()))
    }

    pub async fn append_web_references(
        &self,
        citations: &[WebCitationDraft],
        sources: &[WebSourceDraft],
    ) -> PersistenceResult<()> {
        let Some((research_run_id, response_id)) = reference_identity(citations, sources)? else {
            return Ok(());
        };
        let mut tx = self.pool.begin().await?;
        let completed_attempt_exists: bool = sqlx::query_scalar(
            r#"
            SELECT EXISTS (
                SELECT 1
                FROM research.openai_attempts
                WHERE research_run_id = $1
                  AND response_id = $2
                  AND status = 'completed'
            )
            "#,
        )
        .bind(research_run_id)
        .bind(response_id)
        .fetch_one(&mut *tx)
        .await?;
        if !completed_attempt_exists {
            return Err(PersistenceError::InvalidState(format!(
                "研究任务{research_run_id}的response_id={response_id}没有已完成OpenAI尝试，拒绝写入引用"
            )));
        }
        for citation in citations {
            validate_https(&citation.url)?;
            let fingerprint = sha256_json(&json!({
                "url": citation.url,
                "title": citation.title,
                "domain": citation.domain,
                "output_index": citation.output_index,
                "start_index": citation.start_index,
                "end_index": citation.end_index,
            }))?;
            sqlx::query(
                r#"
                INSERT INTO research.web_citations (
                    id, research_run_id, response_id, url, title, domain,
                    output_index, start_index, end_index, citation_fingerprint, retrieved_at
                ) VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11)
                ON CONFLICT (research_run_id, response_id, citation_fingerprint) DO NOTHING
                "#,
            )
            .bind(Uuid::new_v4())
            .bind(citation.research_run_id)
            .bind(&citation.response_id)
            .bind(&citation.url)
            .bind(&citation.title)
            .bind(&citation.domain)
            .bind(i32::try_from(citation.output_index).map_err(|_| {
                PersistenceError::InvalidState("引用输出位置超出i32范围".to_string())
            })?)
            .bind(
                citation
                    .start_index
                    .map(i32::try_from)
                    .transpose()
                    .map_err(|_| {
                        PersistenceError::InvalidState("引用起始位置超出i32范围".to_string())
                    })?,
            )
            .bind(
                citation
                    .end_index
                    .map(i32::try_from)
                    .transpose()
                    .map_err(|_| {
                        PersistenceError::InvalidState("引用结束位置超出i32范围".to_string())
                    })?,
            )
            .bind(&fingerprint)
            .bind(citation.retrieved_at)
            .execute(&mut *tx)
            .await?;
        }
        for source in sources {
            validate_https(&source.url)?;
            let fingerprint = sha256_json(&json!({
                "url": source.url,
                "title": source.title,
                "domain": source.domain,
            }))?;
            sqlx::query(
                r#"
                INSERT INTO research.web_sources (
                    id, research_run_id, response_id, url, title, domain,
                    source_fingerprint, retrieved_at
                ) VALUES ($1, $2, $3, $4, $5, $6, $7, $8)
                ON CONFLICT (research_run_id, response_id, source_fingerprint) DO NOTHING
                "#,
            )
            .bind(Uuid::new_v4())
            .bind(source.research_run_id)
            .bind(&source.response_id)
            .bind(&source.url)
            .bind(&source.title)
            .bind(&source.domain)
            .bind(&fingerprint)
            .bind(source.retrieved_at)
            .execute(&mut *tx)
            .await?;
        }
        write_audit_event(
            &mut tx,
            "openai_web_references_recorded",
            "research_run",
            Some(research_run_id.to_string()),
            json!({
                "response_id": response_id,
                "citation_count": citations.len(),
                "source_count": sources.len()
            }),
        )
        .await?;
        tx.commit().await?;
        Ok(())
    }

    pub async fn openai_usage_totals(&self) -> PersistenceResult<OpenAiUsageTotals> {
        let row = sqlx::query(
            r#"
            SELECT
                COALESCE(sum(estimated_cost_usd) FILTER (
                    WHERE created_at >= date_trunc('day', now() AT TIME ZONE 'UTC') AT TIME ZONE 'UTC'
                ), 0)::float8 AS today_cost_usd,
                COALESCE(sum(estimated_cost_usd) FILTER (
                    WHERE created_at >= date_trunc('month', now() AT TIME ZONE 'UTC') AT TIME ZONE 'UTC'
                ), 0)::float8 AS month_cost_usd,
                count(*) FILTER (
                    WHERE created_at >= date_trunc('day', now() AT TIME ZONE 'UTC') AT TIME ZONE 'UTC'
                )::bigint AS today_request_count,
                count(*) FILTER (
                    WHERE created_at >= date_trunc('month', now() AT TIME ZONE 'UTC') AT TIME ZONE 'UTC'
                )::bigint AS month_request_count
            FROM research.openai_attempts
            "#,
        )
        .fetch_one(&self.pool)
        .await?;
        Ok(OpenAiUsageTotals {
            today_cost_usd: row.try_get("today_cost_usd")?,
            month_cost_usd: row.try_get("month_cost_usd")?,
            today_request_count: u64::try_from(row.try_get::<i64, _>("today_request_count")?)
                .map_err(|_| {
                    PersistenceError::InvalidState("今日OpenAI请求数为负数".to_string())
                })?,
            month_request_count: u64::try_from(row.try_get::<i64, _>("month_request_count")?)
                .map_err(|_| {
                    PersistenceError::InvalidState("本月OpenAI请求数为负数".to_string())
                })?,
        })
    }
}

fn attempt_record(row: &sqlx::postgres::PgRow) -> PersistenceResult<OpenAiAttemptRecord> {
    let attempt_number: i32 = row.try_get("attempt_number")?;
    Ok(OpenAiAttemptRecord {
        id: row.try_get("id")?,
        research_run_id: row.try_get("research_run_id")?,
        attempt_number: u32::try_from(attempt_number)
            .map_err(|_| PersistenceError::InvalidState("OpenAI尝试次数为负数".to_string()))?,
        attempt_fingerprint: row.try_get("attempt_fingerprint")?,
        response_id: row.try_get("response_id")?,
        status: row.try_get("status")?,
        created_at: row.try_get("created_at")?,
    })
}

fn reference_identity<'a>(
    citations: &'a [WebCitationDraft],
    sources: &'a [WebSourceDraft],
) -> PersistenceResult<Option<(Uuid, &'a str)>> {
    let identity = citations
        .first()
        .map(|citation| (citation.research_run_id, citation.response_id.as_str()))
        .or_else(|| {
            sources
                .first()
                .map(|source| (source.research_run_id, source.response_id.as_str()))
        });
    let Some((research_run_id, response_id)) = identity else {
        return Ok(None);
    };
    if response_id.trim().is_empty() {
        return Err(PersistenceError::InvalidState(
            "Web Search引用缺少response_id".to_string(),
        ));
    }
    for citation in citations {
        if citation.research_run_id != research_run_id || citation.response_id != response_id {
            return Err(PersistenceError::InvalidState(
                "同一次引用写入包含不同研究任务或response_id".to_string(),
            ));
        }
        validate_reference_text(&citation.title, &citation.domain)?;
        if citation
            .end_index
            .zip(citation.start_index)
            .is_some_and(|(end, start)| end < start)
        {
            return Err(PersistenceError::InvalidState(
                "Web Search引用结束位置早于起始位置".to_string(),
            ));
        }
    }
    for source in sources {
        if source.research_run_id != research_run_id || source.response_id != response_id {
            return Err(PersistenceError::InvalidState(
                "同一次来源写入包含不同研究任务或response_id".to_string(),
            ));
        }
        validate_reference_text(
            source.title.as_deref().unwrap_or(&source.url),
            &source.domain,
        )?;
    }
    Ok(Some((research_run_id, response_id)))
}

fn validate_reference_text(title: &str, domain: &str) -> PersistenceResult<()> {
    if title.trim().is_empty() || domain.trim().is_empty() {
        return Err(PersistenceError::InvalidState(
            "Web Search引用标题和域名不能为空".to_string(),
        ));
    }
    reject_secret_text(title)?;
    reject_secret_text(domain)
}

fn validate_attempt(draft: &OpenAiAttemptDraft) -> PersistenceResult<()> {
    if draft.attempt_number == 0
        || draft.model_id.trim().is_empty()
        || draft.request_fingerprint.len() != 64
        || draft.finished_at < draft.started_at
        || !matches!(
            draft.status.as_str(),
            "queued" | "in_progress" | "completed" | "failed" | "cancelled" | "incomplete"
        )
    {
        return Err(PersistenceError::InvalidState(
            "OpenAI尝试记录包含无效次数、模型、指纹、状态或时间".to_string(),
        ));
    }
    reject_secret_text(&draft.request_payload.to_string())?;
    if let Some(raw) = &draft.raw_response {
        reject_secret_text(&raw.to_string())?;
    }
    if draft
        .estimated_cost_usd
        .is_some_and(|value| !value.is_finite() || value < 0.0)
    {
        return Err(PersistenceError::InvalidState(
            "OpenAI估算成本必须是非负有限数".to_string(),
        ));
    }
    Ok(())
}

fn validate_https(value: &str) -> PersistenceResult<()> {
    if value.starts_with("https://") {
        Ok(())
    } else {
        Err(PersistenceError::InvalidState(format!(
            "Web Search来源必须使用HTTPS：{value}"
        )))
    }
}

fn reject_secret_text(value: &str) -> PersistenceResult<()> {
    let lower = value.to_lowercase();
    let contains_long_openai_key = lower
        .split(|character: char| {
            !character.is_ascii_alphanumeric() && character != '-' && character != '_'
        })
        .any(|token| token.starts_with("sk-") && token.len() >= 23);
    if lower.contains("authorization") || lower.contains("bearer ") || contains_long_openai_key {
        return Err(PersistenceError::InvalidState(
            "OpenAI审计载荷疑似包含API密钥或Authorization头，已拒绝写入".to_string(),
        ));
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::{Duration, Utc};

    #[test]
    fn attempt_validation_rejects_secrets_and_invalid_cost() {
        let now = Utc::now();
        let mut draft = OpenAiAttemptDraft {
            research_run_id: Uuid::new_v4(),
            attempt_number: 1,
            model_id: "configured-model".to_string(),
            request_fingerprint: "a".repeat(64),
            request_payload: json!({"input":"facts only"}),
            response_id: None,
            provider_request_id: None,
            provider_status: Some(429),
            status: "failed".to_string(),
            token_usage: json!({}),
            latency_ms: 10,
            search_call_count: 0,
            estimated_cost_usd: None,
            raw_response: None,
            error_category: Some("rate_limit".to_string()),
            error_message: Some("limited".to_string()),
            retryable: true,
            started_at: now,
            finished_at: now + Duration::milliseconds(10),
        };
        assert!(validate_attempt(&draft).is_ok());
        draft.request_payload = json!({"Authorization":"Bearer credential-material"});
        assert!(validate_attempt(&draft).is_err());
        draft.request_payload = json!({});
        draft.estimated_cost_usd = Some(f64::NAN);
        assert!(validate_attempt(&draft).is_err());
    }

    #[test]
    fn secret_validation_does_not_reject_short_sk_team_names() {
        reject_secret_text("SK-Brann official team news").expect("team name must be allowed");
        let synthetic_key = format!("sk-proj-{}", "x".repeat(32));
        assert!(reject_secret_text(&synthetic_key).is_err());
    }
}
