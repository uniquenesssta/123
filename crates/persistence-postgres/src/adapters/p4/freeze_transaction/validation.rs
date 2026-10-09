use crate::{PersistenceError, PersistenceResult};
use football_domain::{PrematchSnapshotDraft, SnapshotSourceKind};
use sqlx::{Postgres, Row, Transaction};
use std::collections::BTreeSet;
use uuid::Uuid;

pub(super) async fn validate_snapshot_references(
    tx: &mut Transaction<'_, Postgres>,
    draft: &PrematchSnapshotDraft,
) -> PersistenceResult<()> {
    let match_row =
        sqlx::query("SELECT external_key, $2::timestamptz < kickoff_time AS cutoff_before_kickoff FROM football.matches WHERE id = $1")
            .bind(draft.match_id)
            .bind(draft.data_cutoff_at)
            .fetch_optional(&mut **tx)
            .await?
            .ok_or_else(|| {
                PersistenceError::InvalidState("赛前快照引用的比赛不存在".to_string())
            })?;
    let external_key: String = match_row.try_get("external_key")?;
    let cutoff_before_kickoff: bool = match_row.try_get("cutoff_before_kickoff")?;
    if external_key != draft.match_key {
        return Err(PersistenceError::InvalidState(
            "match_key与比赛稳定外部键不一致".to_string(),
        ));
    }
    if !cutoff_before_kickoff {
        return Err(PersistenceError::InvalidState(
            "赛前快照data_cutoff_at必须早于开球时间".to_string(),
        ));
    }

    let parameter_model_version: Option<Uuid> =
        sqlx::query_scalar("SELECT model_version_id FROM model.parameter_sets WHERE id = $1")
            .bind(draft.parameter_set_id)
            .fetch_optional(&mut **tx)
            .await?;
    if parameter_model_version != Some(draft.model_version_id) {
        return Err(PersistenceError::InvalidState(
            "参数版本不属于快照声明的模型版本".to_string(),
        ));
    }

    let profile_exists: bool =
        sqlx::query_scalar("SELECT EXISTS(SELECT 1 FROM model.competition_profiles WHERE id = $1)")
            .bind(draft.competition_profile_id)
            .fetch_one(&mut **tx)
            .await?;
    if !profile_exists {
        return Err(PersistenceError::InvalidState(
            "赛前快照引用的赛事Profile版本不存在".to_string(),
        ));
    }

    let registered_schema_version: Option<String> =
        sqlx::query_scalar("SELECT version FROM research.schema_versions WHERE id = $1")
            .bind(draft.schema_version_id)
            .fetch_optional(&mut **tx)
            .await?;
    if registered_schema_version.as_deref() != Some(draft.schema_version.as_str()) {
        return Err(PersistenceError::InvalidState(
            "快照Schema版本ID与版本号不一致".to_string(),
        ));
    }

    if matches!(draft.source_kind, SnapshotSourceKind::Real) && draft.research_run_id.is_none() {
        return Err(PersistenceError::InvalidState(
            "真实联网快照必须关联研究任务".to_string(),
        ));
    }
    if let Some(research_run_id) = draft.research_run_id {
        let run = sqlx::query(
            r#"
            SELECT match_id, horizon, data_cutoff_at = $2::timestamptz AS cutoff_matches, trace_id
            FROM research.runs WHERE id = $1
            "#,
        )
        .bind(research_run_id)
        .bind(draft.data_cutoff_at)
        .fetch_optional(&mut **tx)
        .await?
        .ok_or_else(|| PersistenceError::InvalidState("快照关联的研究任务不存在".to_string()))?;
        let run_horizon: String = run.try_get("horizon")?;
        if run.try_get::<Uuid, _>("match_id")? != draft.match_id
            || run_horizon != draft.horizon.as_str()
            || !run.try_get::<bool, _>("cutoff_matches")?
            || run.try_get::<Uuid, _>("trace_id")? != draft.trace_id
        {
            return Err(PersistenceError::InvalidState(
                "研究任务与快照的比赛、时点、截止时间或追踪ID不一致".to_string(),
            ));
        }
    }
    Ok(())
}

pub(super) async fn validate_snapshot_evidence(
    tx: &mut Transaction<'_, Postgres>,
    draft: &PrematchSnapshotDraft,
) -> PersistenceResult<()> {
    let evidence_ids = draft
        .features
        .iter()
        .flat_map(|feature| feature.evidence_ids.iter().copied())
        .collect::<BTreeSet<_>>();
    if matches!(draft.source_kind, SnapshotSourceKind::SyntheticFixture) {
        if !evidence_ids.is_empty() {
            return Err(PersistenceError::InvalidState(
                "合成fixture不得链接真实证据声明".to_string(),
            ));
        }
        return Ok(());
    }
    if evidence_ids.is_empty() {
        return Ok(());
    }
    let rows = sqlx::query(
        r#"
        SELECT id, match_id,
               COALESCE(published_at > $2::timestamptz, false)
               OR COALESCE(effective_at > $2::timestamptz, false) AS after_cutoff
        FROM research.evidence_claims
        WHERE id = ANY($1)
        "#,
    )
    .bind(evidence_ids.iter().copied().collect::<Vec<_>>())
    .bind(draft.data_cutoff_at)
    .fetch_all(&mut **tx)
    .await?;
    if rows.len() != evidence_ids.len() {
        return Err(PersistenceError::InvalidState(
            "快照包含不存在的证据声明".to_string(),
        ));
    }
    for row in rows {
        let match_id: Uuid = row.try_get("match_id")?;
        let after_cutoff: bool = row.try_get("after_cutoff")?;
        if match_id != draft.match_id {
            return Err(PersistenceError::InvalidState(
                "快照证据必须属于同一比赛".to_string(),
            ));
        }
        if after_cutoff {
            return Err(PersistenceError::InvalidState(
                "晚于data_cutoff_at的证据不得进入赛前快照".to_string(),
            ));
        }
    }
    Ok(())
}
