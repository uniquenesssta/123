use crate::{
    adapters::catalog::references::validate_entity_type, write_audit_event, PersistenceError,
    PersistenceResult, PostgresStore,
};
use football_domain::{
    BulkArchiveFailedItem, BulkArchiveResult, EntityDeletionCheck, EntityReferenceCount,
    TeamPlayerPeriodRecord,
};
use serde_json::json;
use sqlx::{Postgres, Row, Transaction};
use uuid::Uuid;

impl PostgresStore {
    pub(crate) async fn list_team_player_periods(
        &self,
        team_id: Uuid,
    ) -> PersistenceResult<Vec<TeamPlayerPeriodRecord>> {
        sqlx::query(
            r#"
            SELECT period.id, period.team_id, team.canonical_name AS team_name,
                   period.player_id, player.canonical_name AS player_name,
                   period.season_id, season.name AS season_name, period.squad_number,
                   period.valid_from, period.valid_to, period.registration_status
            FROM football.player_team_periods period
            JOIN football.teams team ON team.id=period.team_id
            JOIN football.players player ON player.id=period.player_id
            LEFT JOIN football.seasons season ON season.id=period.season_id
            WHERE period.team_id=$1
            ORDER BY period.valid_from DESC, period.valid_to DESC NULLS FIRST,
                     player.normalized_name, period.id
            "#,
        )
        .bind(team_id)
        .fetch_all(&self.pool)
        .await?
        .iter()
        .map(team_player_period_from_row)
        .collect()
    }

    pub async fn check_entity_deletion(
        &self,
        entity_type: &str,
        entity_id: Uuid,
    ) -> PersistenceResult<EntityDeletionCheck> {
        validate_entity_type(entity_type)?;
        let label = self.entity_label(entity_type, entity_id).await?;
        let Some(label) = label else {
            return Ok(EntityDeletionCheck {
                entity_type: entity_type.to_string(),
                entity_id,
                label: entity_id.to_string(),
                exists: false,
                can_permanently_delete: false,
                must_archive: false,
                references: Vec::new(),
                reason: "实体不存在".to_string(),
            });
        };
        let references = match entity_type {
            "team" => self.team_reference_counts(entity_id).await?,
            "player" => self.player_reference_counts(entity_id).await?,
            "coach" => self.coach_reference_counts(entity_id).await?,
            _ => unreachable!(),
        };
        let total: i64 = references.iter().map(|item| item.count).sum();
        Ok(EntityDeletionCheck {
            entity_type: entity_type.to_string(),
            entity_id,
            label,
            exists: true,
            can_permanently_delete: total == 0,
            must_archive: total > 0,
            references,
            reason: if total == 0 {
                "没有历史引用，可以永久删除".to_string()
            } else {
                format!("存在 {total} 条历史或业务引用，只允许归档")
            },
        })
    }

    async fn entity_label(&self, entity_type: &str, id: Uuid) -> PersistenceResult<Option<String>> {
        let query = match entity_type {
            "team" => "SELECT canonical_name FROM football.teams WHERE id=$1",
            "player" => "SELECT canonical_name FROM football.players WHERE id=$1",
            "coach" => "SELECT canonical_name FROM football.coaches WHERE id=$1",
            _ => unreachable!(),
        };
        Ok(sqlx::query_scalar(query)
            .bind(id)
            .fetch_optional(&self.pool)
            .await?)
    }

    async fn team_reference_counts(
        &self,
        id: Uuid,
    ) -> PersistenceResult<Vec<EntityReferenceCount>> {
        count_relations(
            &self.pool,
            id,
            &[
                ("matches", "SELECT count(*)::bigint FROM football.matches WHERE home_team_id=$1 OR away_team_id=$1"),
                ("lineups", "SELECT count(*)::bigint FROM football.lineups WHERE team_id=$1"),
                ("team_lineup_presets", "SELECT count(*)::bigint FROM football.team_lineup_presets WHERE team_id=$1"),
                ("player_team_periods", "SELECT count(*)::bigint FROM football.player_team_periods WHERE team_id=$1"),
                ("team_coach_periods", "SELECT count(*)::bigint FROM football.team_coach_periods WHERE team_id=$1"),
                ("team_season_memberships", "SELECT count(*)::bigint FROM football.team_season_memberships WHERE team_id=$1"),
                ("player_availability", "SELECT count(*)::bigint FROM football.player_availability WHERE team_id=$1"),
                ("formation_usage", "SELECT count(*)::bigint FROM feature.formation_usage_observations WHERE team_id=$1"),
                ("team_tactical_observations", "SELECT count(*)::bigint FROM feature.team_tactical_observations WHERE team_id=$1"),
                ("team_ability_observations", "SELECT count(*)::bigint FROM feature.team_ability_observations WHERE team_id=$1"),
                ("substitutions", "SELECT count(*)::bigint FROM football.substitutions WHERE team_id=$1"),
                ("match_events", "SELECT count(*)::bigint FROM review.match_events WHERE team_id=$1"),
                ("dynamic_tag_opponents", "SELECT count(*)::bigint FROM feature.player_dynamic_tags WHERE opponent_team_id=$1"),
                ("team_match_reviews", "SELECT count(*)::bigint FROM review.team_match_reviews WHERE team_id=$1"),
                ("player_match_reviews", "SELECT count(*)::bigint FROM review.player_match_reviews WHERE team_id=$1"),
                ("player_match_observations", "SELECT count(*)::bigint FROM review.player_match_observations WHERE team_id=$1"),
            ],
        ).await
    }

    async fn player_reference_counts(
        &self,
        id: Uuid,
    ) -> PersistenceResult<Vec<EntityReferenceCount>> {
        count_relations(
            &self.pool,
            id,
            &[
                ("lineup_players", "SELECT count(*)::bigint FROM football.lineup_players WHERE player_id=$1"),
                ("team_lineup_preset_members", "SELECT count(*)::bigint FROM football.team_lineup_preset_members WHERE player_id=$1"),
                ("substitutions", "SELECT count(*)::bigint FROM football.substitutions WHERE player_out_id=$1 OR player_in_id=$1"),
                ("match_events", "SELECT count(*)::bigint FROM review.match_events WHERE player_id=$1 OR related_player_id=$1"),
                ("player_team_periods", "SELECT count(*)::bigint FROM football.player_team_periods WHERE player_id=$1"),
                ("player_availability", "SELECT count(*)::bigint FROM football.player_availability WHERE player_id=$1"),
                ("ability_observations", "SELECT count(*)::bigint FROM feature.player_ability_observations WHERE player_id=$1"),
                ("ability_snapshots", "SELECT count(*)::bigint FROM feature.player_ability_snapshots WHERE player_id=$1"),
                ("dynamic_tags", "SELECT count(*)::bigint FROM feature.player_dynamic_tags WHERE player_id=$1"),
                ("match_contributions", "SELECT count(*)::bigint FROM feature.match_player_contributions WHERE player_id=$1"),
                ("player_match_reviews", "SELECT count(*)::bigint FROM review.player_match_reviews WHERE player_id=$1"),
                ("player_match_observations", "SELECT count(*)::bigint FROM review.player_match_observations WHERE player_id=$1"),
                ("ability_candidates", "SELECT count(*)::bigint FROM review.ability_update_candidates WHERE player_id=$1"),
            ],
        ).await
    }

    async fn coach_reference_counts(
        &self,
        id: Uuid,
    ) -> PersistenceResult<Vec<EntityReferenceCount>> {
        count_relations(
            &self.pool,
            id,
            &[
                (
                    "team_coach_periods",
                    "SELECT count(*)::bigint FROM football.team_coach_periods WHERE coach_id=$1",
                ),
                (
                    "team_lineup_presets",
                    "SELECT count(*)::bigint FROM football.team_lineup_presets WHERE coach_id=$1",
                ),
            ],
        )
        .await
    }

    pub async fn bulk_archive_entities(
        &self,
        entity_type: &str,
        ids: &[Uuid],
    ) -> PersistenceResult<BulkArchiveResult> {
        validate_entity_type(entity_type)?;
        let ids = unique_ids(ids);
        let mut archived_ids = Vec::new();
        let mut already_archived_ids = Vec::new();
        let mut failed = Vec::new();
        for id in &ids {
            let label = self
                .entity_label(entity_type, *id)
                .await?
                .unwrap_or_else(|| id.to_string());
            match self.archive_entity(entity_type, *id).await {
                Ok(true) => archived_ids.push(*id),
                Ok(false) => already_archived_ids.push(*id),
                Err(error) => failed.push(BulkArchiveFailedItem {
                    id: *id,
                    label,
                    reason: error.to_string(),
                }),
            }
        }
        Ok(BulkArchiveResult {
            entity_type: entity_type.to_string(),
            requested_count: ids.len() as u64,
            archived_ids,
            already_archived_ids,
            failed,
        })
    }

    async fn archive_entity(&self, entity_type: &str, id: Uuid) -> PersistenceResult<bool> {
        let mut tx = self.pool.begin().await?;
        let changed = match entity_type {
            "team" => sqlx::query("UPDATE football.teams SET is_active=false, updated_at=now() WHERE id=$1 AND is_active")
                .bind(id).execute(&mut *tx).await?.rows_affected(),
            "player" => sqlx::query("UPDATE football.players SET status='inactive', updated_at=now() WHERE id=$1 AND status NOT IN ('inactive','retired')")
                .bind(id).execute(&mut *tx).await?.rows_affected(),
            "coach" => sqlx::query("UPDATE football.coaches SET status='inactive', updated_at=now() WHERE id=$1 AND status NOT IN ('inactive','retired')")
                .bind(id).execute(&mut *tx).await?.rows_affected(),
            _ => unreachable!(),
        };
        if changed > 0 {
            write_audit_event(
                &mut tx,
                &format!("{entity_type}_archived"),
                entity_type,
                Some(id.to_string()),
                json!({"source":"manual_bulk_archive"}),
            )
            .await?;
        } else if self
            .entity_label_in_tx(&mut tx, entity_type, id)
            .await?
            .is_none()
        {
            return Err(PersistenceError::InvalidState("实体不存在".to_string()));
        }
        tx.commit().await?;
        Ok(changed > 0)
    }

    async fn entity_label_in_tx(
        &self,
        tx: &mut Transaction<'_, Postgres>,
        entity_type: &str,
        id: Uuid,
    ) -> PersistenceResult<Option<String>> {
        let query = match entity_type {
            "team" => "SELECT canonical_name FROM football.teams WHERE id=$1",
            "player" => "SELECT canonical_name FROM football.players WHERE id=$1",
            "coach" => "SELECT canonical_name FROM football.coaches WHERE id=$1",
            _ => unreachable!(),
        };
        Ok(sqlx::query_scalar(query)
            .bind(id)
            .fetch_optional(&mut **tx)
            .await?)
    }
}

async fn count_relations(
    pool: &sqlx::PgPool,
    id: Uuid,
    relations: &[(&str, &str)],
) -> PersistenceResult<Vec<EntityReferenceCount>> {
    let mut output = Vec::new();
    for (relation, query) in relations {
        let count: i64 = sqlx::query_scalar(query).bind(id).fetch_one(pool).await?;
        if count > 0 {
            output.push(EntityReferenceCount {
                relation: (*relation).to_string(),
                count,
            });
        }
    }
    Ok(output)
}

fn team_player_period_from_row(
    row: &sqlx::postgres::PgRow,
) -> PersistenceResult<TeamPlayerPeriodRecord> {
    Ok(TeamPlayerPeriodRecord {
        id: row.try_get("id")?,
        team_id: row.try_get("team_id")?,
        team_name: row.try_get("team_name")?,
        player_id: row.try_get("player_id")?,
        player_name: row.try_get("player_name")?,
        season_id: row.try_get("season_id")?,
        season_name: row.try_get("season_name")?,
        squad_number: row.try_get("squad_number")?,
        valid_from: row.try_get("valid_from")?,
        valid_to: row.try_get("valid_to")?,
        registration_status: row.try_get("registration_status")?,
    })
}

fn unique_ids(ids: &[Uuid]) -> Vec<Uuid> {
    let mut output = Vec::new();
    for id in ids {
        if !output.contains(id) {
            output.push(*id);
        }
    }
    output
}
