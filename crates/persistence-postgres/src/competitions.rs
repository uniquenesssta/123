use super::{parse_competition_kind, PersistenceError, PersistenceResult, PostgresStore};
use football_domain::{CompetitionKind, ResolvedCompetitionContext};
use sqlx::Row;
use uuid::Uuid;

impl PostgresStore {
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
