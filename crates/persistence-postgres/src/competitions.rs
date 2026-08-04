use super::{
    parse_competition_kind, write_audit_event, PersistenceError, PersistenceResult, PostgresStore,
};
use football_domain::{
    CompetitionDraft, CompetitionKind, CompetitionRecord, ResolvedCompetitionContext, RoundDraft,
    RoundRecord, SeasonDraft, SeasonRecord, StageDraft, StageRecord,
};
use serde_json::json;
use sqlx::Row;
use uuid::Uuid;

impl PostgresStore {
    pub async fn create_competition(
        &self,
        draft: &CompetitionDraft,
    ) -> PersistenceResult<CompetitionRecord> {
        let id = Uuid::new_v4();
        let row = sqlx::query(
            r#"
            INSERT INTO football.competitions (
                id, code, name, country_code, timezone, competition_kind, metadata
            ) VALUES ($1, $2, $3, $4, $5, $6, $7)
            RETURNING id, code, name, country_code, timezone,
                      competition_kind, is_active, metadata, created_at
            "#,
        )
        .bind(id)
        .bind(draft.code.trim())
        .bind(draft.name.trim())
        .bind(draft.country_code.as_deref())
        .bind(draft.timezone.trim())
        .bind(draft.competition_kind.as_str())
        .bind(&draft.metadata)
        .fetch_one(&self.pool)
        .await?;
        competition_record_from_row(&row)
    }

    pub async fn delete_competition(&self, id: Uuid) -> PersistenceResult<()> {
        let mut tx = self.pool.begin().await?;
        let competition_name: String = sqlx::query_scalar(
            "SELECT name FROM football.competitions WHERE id = $1 AND is_active = true FOR UPDATE",
        )
        .bind(id)
        .fetch_optional(&mut *tx)
        .await?
        .ok_or_else(|| PersistenceError::InvalidState("赛事不存在或已经删除".to_string()))?;
        sqlx::query(
            r#"
            UPDATE football.competitions
            SET is_active = false,
                code = code || '-DELETED-' || left(id::text, 8),
                metadata = metadata || jsonb_build_object(
                    'deleted_at', now(),
                    'original_code', code
                ),
                updated_at = now()
            WHERE id = $1
            "#,
        )
        .bind(id)
        .execute(&mut *tx)
        .await?;
        sqlx::query("DELETE FROM football.external_entity_ids WHERE entity_type = 'competition' AND entity_id = $1")
            .bind(id)
            .execute(&mut *tx)
            .await?;
        sqlx::query(
            "UPDATE model.competition_bindings SET is_active = false WHERE competition_id = $1",
        )
        .bind(id)
        .execute(&mut *tx)
        .await?;
        write_audit_event(
            &mut tx,
            "competition_deleted",
            "competition",
            Some(id.to_string()),
            json!({"name": competition_name, "deletion_mode": "soft_delete"}),
        )
        .await?;
        tx.commit().await?;
        Ok(())
    }

    pub async fn read_competition(&self, id: Uuid) -> PersistenceResult<CompetitionRecord> {
        let row = sqlx::query(
            r#"
            SELECT id, code, name, country_code, timezone,
                   competition_kind, is_active, metadata, created_at
            FROM football.competitions
            WHERE id = $1 AND is_active = true
            "#,
        )
        .bind(id)
        .fetch_one(&self.pool)
        .await?;
        competition_record_from_row(&row)
    }

    pub async fn resolve_competition_context(
        &self,
        competition_id: Option<Uuid>,
        season_id: Option<Uuid>,
        stage_id: Option<Uuid>,
        fallback_kind: CompetitionKind,
    ) -> PersistenceResult<ResolvedCompetitionContext> {
        if let Some(stage_id_value) = stage_id {
            let row = sqlx::query(
                r#"
                SELECT
                    c.id AS competition_id, s.id AS season_id, st.id AS stage_id,
                    st.stage_kind
                FROM football.competition_stages st
                JOIN football.seasons s ON s.id = st.season_id
                JOIN football.competitions c ON c.id = s.competition_id
                WHERE st.id = $1 AND c.is_active = true
                "#,
            )
            .bind(stage_id_value)
            .fetch_optional(&self.pool)
            .await?
            .ok_or_else(|| {
                PersistenceError::InvalidState(format!(
                    "赛事阶段不存在或所属赛事已停用：{stage_id_value}"
                ))
            })?;
            let resolved_competition_id: Uuid = row.try_get("competition_id")?;
            let resolved_season_id: Uuid = row.try_get("season_id")?;
            ensure_scope_id("赛事", competition_id, resolved_competition_id)?;
            ensure_scope_id("赛季", season_id, resolved_season_id)?;
            return Ok(ResolvedCompetitionContext {
                competition_id: Some(resolved_competition_id),
                season_id: Some(resolved_season_id),
                stage_id: Some(stage_id_value),
                competition_kind: parse_competition_kind(&row.try_get::<String, _>("stage_kind")?)?,
            });
        }

        if let Some(season_id_value) = season_id {
            let row = sqlx::query(
                r#"
                SELECT c.id AS competition_id, s.id AS season_id, c.competition_kind
                FROM football.seasons s
                JOIN football.competitions c ON c.id = s.competition_id
                WHERE s.id = $1 AND c.is_active = true
                "#,
            )
            .bind(season_id_value)
            .fetch_optional(&self.pool)
            .await?
            .ok_or_else(|| {
                PersistenceError::InvalidState(format!(
                    "赛季不存在或所属赛事已停用：{season_id_value}"
                ))
            })?;
            let resolved_competition_id: Uuid = row.try_get("competition_id")?;
            ensure_scope_id("赛事", competition_id, resolved_competition_id)?;
            return Ok(ResolvedCompetitionContext {
                competition_id: Some(resolved_competition_id),
                season_id: Some(season_id_value),
                stage_id: None,
                competition_kind: parse_competition_kind(
                    &row.try_get::<String, _>("competition_kind")?,
                )?,
            });
        }

        if let Some(competition_id_value) = competition_id {
            let competition = self.read_competition(competition_id_value).await?;
            return Ok(ResolvedCompetitionContext {
                competition_id: Some(competition.id),
                season_id: None,
                stage_id: None,
                competition_kind: competition.competition_kind,
            });
        }

        Ok(ResolvedCompetitionContext {
            competition_id: None,
            season_id: None,
            stage_id: None,
            competition_kind: fallback_kind,
        })
    }

    pub async fn list_competitions(&self) -> PersistenceResult<Vec<CompetitionRecord>> {
        let rows = sqlx::query(
            r#"
            SELECT id, code, name, country_code, timezone,
                   competition_kind, is_active, metadata, created_at
            FROM football.competitions
            WHERE is_active = true
            ORDER BY COALESCE((metadata->>'sort_order')::integer, 999999), name, code
            "#,
        )
        .fetch_all(&self.pool)
        .await?;
        rows.iter().map(competition_record_from_row).collect()
    }

    pub async fn create_season(&self, draft: &SeasonDraft) -> PersistenceResult<SeasonRecord> {
        let id = Uuid::new_v4();
        sqlx::query(
            r#"
            INSERT INTO football.seasons (
                id, competition_id, name, starts_on, ends_on, status, metadata
            ) VALUES ($1, $2, $3, $4, $5, $6, $7)
            "#,
        )
        .bind(id)
        .bind(draft.competition_id)
        .bind(draft.name.trim())
        .bind(draft.starts_on)
        .bind(draft.ends_on)
        .bind(draft.status.trim())
        .bind(&draft.metadata)
        .execute(&self.pool)
        .await?;
        self.read_season(id).await
    }

    pub async fn list_seasons(&self) -> PersistenceResult<Vec<SeasonRecord>> {
        let rows = sqlx::query(
            r#"
            SELECT
                s.id, s.competition_id, c.name AS competition_name,
                s.name, s.starts_on, s.ends_on, s.status
            FROM football.seasons s
            JOIN football.competitions c ON c.id = s.competition_id
            WHERE c.is_active = true
            ORDER BY c.name, s.starts_on DESC NULLS LAST, s.name
            "#,
        )
        .fetch_all(&self.pool)
        .await?;
        rows.iter().map(season_record_from_row).collect()
    }

    pub async fn create_stage(&self, draft: &StageDraft) -> PersistenceResult<StageRecord> {
        let id = Uuid::new_v4();
        sqlx::query(
            r#"
            INSERT INTO football.competition_stages (
                id, season_id, code, name, stage_kind, sequence_no, rules
            ) VALUES ($1, $2, $3, $4, $5, $6, $7)
            "#,
        )
        .bind(id)
        .bind(draft.season_id)
        .bind(draft.code.trim())
        .bind(draft.name.trim())
        .bind(draft.stage_kind.as_str())
        .bind(draft.sequence_no)
        .bind(&draft.rules)
        .execute(&self.pool)
        .await?;
        self.read_stage(id).await
    }

    pub async fn list_stages(&self) -> PersistenceResult<Vec<StageRecord>> {
        let rows = sqlx::query(
            r#"
            SELECT
                st.id, st.season_id, s.name AS season_name,
                s.competition_id, c.name AS competition_name,
                st.code, st.name, st.stage_kind, st.sequence_no
            FROM football.competition_stages st
            JOIN football.seasons s ON s.id = st.season_id
            JOIN football.competitions c ON c.id = s.competition_id
            WHERE c.is_active = true
            ORDER BY c.name, s.name, st.sequence_no, st.name
            "#,
        )
        .fetch_all(&self.pool)
        .await?;
        rows.iter().map(stage_record_from_row).collect()
    }

    pub async fn create_round(&self, draft: &RoundDraft) -> PersistenceResult<RoundRecord> {
        let id = Uuid::new_v4();
        sqlx::query(
            r#"
            INSERT INTO football.rounds (
                id, stage_id, code, name, sequence_no, starts_at, ends_at
            ) VALUES ($1, $2, $3, $4, $5, $6, $7)
            "#,
        )
        .bind(id)
        .bind(draft.stage_id)
        .bind(draft.code.trim())
        .bind(draft.name.trim())
        .bind(draft.sequence_no)
        .bind(draft.starts_at)
        .bind(draft.ends_at)
        .execute(&self.pool)
        .await?;
        self.read_round(id).await
    }

    pub async fn list_rounds(&self) -> PersistenceResult<Vec<RoundRecord>> {
        let rows = sqlx::query(
            r#"
            SELECT
                r.id, r.stage_id, st.name AS stage_name,
                r.code, r.name, r.sequence_no, r.starts_at, r.ends_at
            FROM football.rounds r
            JOIN football.competition_stages st ON st.id = r.stage_id
            JOIN football.seasons s ON s.id = st.season_id
            JOIN football.competitions c ON c.id = s.competition_id
            WHERE c.is_active = true
            ORDER BY st.name, r.sequence_no, r.starts_at NULLS LAST
            "#,
        )
        .fetch_all(&self.pool)
        .await?;
        rows.iter().map(round_record_from_row).collect()
    }

    async fn read_season(&self, id: Uuid) -> PersistenceResult<SeasonRecord> {
        let row = sqlx::query(
            r#"
            SELECT
                s.id, s.competition_id, c.name AS competition_name,
                s.name, s.starts_on, s.ends_on, s.status
            FROM football.seasons s
            JOIN football.competitions c ON c.id = s.competition_id
            WHERE s.id = $1
            "#,
        )
        .bind(id)
        .fetch_one(&self.pool)
        .await?;
        season_record_from_row(&row)
    }

    async fn read_stage(&self, id: Uuid) -> PersistenceResult<StageRecord> {
        let row = sqlx::query(
            r#"
            SELECT
                st.id, st.season_id, s.name AS season_name,
                s.competition_id, c.name AS competition_name,
                st.code, st.name, st.stage_kind, st.sequence_no
            FROM football.competition_stages st
            JOIN football.seasons s ON s.id = st.season_id
            JOIN football.competitions c ON c.id = s.competition_id
            WHERE st.id = $1
            "#,
        )
        .bind(id)
        .fetch_one(&self.pool)
        .await?;
        stage_record_from_row(&row)
    }

    async fn read_round(&self, id: Uuid) -> PersistenceResult<RoundRecord> {
        let row = sqlx::query(
            r#"
            SELECT
                r.id, r.stage_id, st.name AS stage_name,
                r.code, r.name, r.sequence_no, r.starts_at, r.ends_at
            FROM football.rounds r
            JOIN football.competition_stages st ON st.id = r.stage_id
            WHERE r.id = $1
            "#,
        )
        .bind(id)
        .fetch_one(&self.pool)
        .await?;
        round_record_from_row(&row)
    }
}

fn competition_record_from_row(
    row: &sqlx::postgres::PgRow,
) -> PersistenceResult<CompetitionRecord> {
    Ok(CompetitionRecord {
        id: row.try_get("id")?,
        code: row.try_get("code")?,
        name: row.try_get("name")?,
        country_code: row.try_get("country_code")?,
        timezone: row.try_get("timezone")?,
        competition_kind: parse_competition_kind(&row.try_get::<String, _>("competition_kind")?)?,
        is_active: row.try_get("is_active")?,
        metadata: row.try_get("metadata")?,
        created_at: row.try_get("created_at")?,
    })
}

fn season_record_from_row(row: &sqlx::postgres::PgRow) -> PersistenceResult<SeasonRecord> {
    Ok(SeasonRecord {
        id: row.try_get("id")?,
        competition_id: row.try_get("competition_id")?,
        competition_name: row.try_get("competition_name")?,
        name: row.try_get("name")?,
        starts_on: row.try_get("starts_on")?,
        ends_on: row.try_get("ends_on")?,
        status: row.try_get("status")?,
    })
}

fn stage_record_from_row(row: &sqlx::postgres::PgRow) -> PersistenceResult<StageRecord> {
    Ok(StageRecord {
        id: row.try_get("id")?,
        season_id: row.try_get("season_id")?,
        season_name: row.try_get("season_name")?,
        competition_id: row.try_get("competition_id")?,
        competition_name: row.try_get("competition_name")?,
        code: row.try_get("code")?,
        name: row.try_get("name")?,
        stage_kind: parse_competition_kind(&row.try_get::<String, _>("stage_kind")?)?,
        sequence_no: row.try_get("sequence_no")?,
    })
}

fn round_record_from_row(row: &sqlx::postgres::PgRow) -> PersistenceResult<RoundRecord> {
    Ok(RoundRecord {
        id: row.try_get("id")?,
        stage_id: row.try_get("stage_id")?,
        stage_name: row.try_get("stage_name")?,
        code: row.try_get("code")?,
        name: row.try_get("name")?,
        sequence_no: row.try_get("sequence_no")?,
        starts_at: row.try_get("starts_at")?,
        ends_at: row.try_get("ends_at")?,
    })
}

fn ensure_scope_id(label: &str, supplied: Option<Uuid>, resolved: Uuid) -> PersistenceResult<()> {
    if let Some(supplied_id) = supplied {
        if supplied_id != resolved {
            return Err(PersistenceError::InvalidState(format!(
                "{label}层级不一致：提交 {supplied_id}，实际所属 {resolved}"
            )));
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn competition_kind_round_trip_is_complete() {
        for kind in CompetitionKind::ALL {
            assert_eq!(parse_competition_kind(kind.as_str()).unwrap(), kind);
        }
    }

    #[test]
    fn scope_id_validation_rejects_mismatch() {
        let actual = Uuid::new_v4();
        assert!(ensure_scope_id("赛事", Some(actual), actual).is_ok());
        assert!(ensure_scope_id("赛事", Some(Uuid::new_v4()), actual).is_err());
    }
}
