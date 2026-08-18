use crate::{
    name_search::{push_name_search, NameSearch, NameSearchColumns},
    write_audit_event, PersistenceError, PersistenceResult, PostgresStore,
};
use football_domain::{
    BulkArchiveFailedItem, BulkArchiveResult, EntityDeletionCheck, EntityMatchCandidate,
    EntityMatchRequest, EntityMatchResult, EntityReferenceCount, EntityReferenceQuery,
    EntityReferenceRecord, TeamPlayerPeriodRecord,
};
use serde_json::json;
use sqlx::{Postgres, QueryBuilder, Row, Transaction};
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

    pub async fn list_entity_references(
        &self,
        query: &EntityReferenceQuery,
    ) -> PersistenceResult<Vec<EntityReferenceRecord>> {
        match query.entity_type.as_str() {
            "team" => self.list_team_references(query).await,
            "player" => self.list_player_references(query).await,
            "coach" => self.list_coach_references(query).await,
            other => Err(PersistenceError::InvalidState(format!(
                "不支持的实体类型：{other}"
            ))),
        }
    }

    async fn list_team_references(
        &self,
        query: &EntityReferenceQuery,
    ) -> PersistenceResult<Vec<EntityReferenceRecord>> {
        let mut builder: QueryBuilder<Postgres> = QueryBuilder::new(
            r#"
            SELECT team.id, team.canonical_name, team.normalized_name, team.country_code,
                   team.is_active,
                   COALESCE((SELECT array_agg(alias.name ORDER BY alias.name) FROM football.team_names alias WHERE alias.team_id=team.id), ARRAY[]::text[]) AS aliases,
                   COALESCE((SELECT array_agg(external.external_id ORDER BY external.external_id) FROM football.external_entity_ids external WHERE external.entity_type='team' AND external.entity_id=team.id), ARRAY[]::text[]) AS external_ids
            FROM football.teams team
            WHERE 1=1
            "#,
        );
        if let Some(search) = NameSearch::parse(query.search.as_deref()) {
            push_name_search(
                &mut builder,
                &search,
                NameSearchColumns {
                    primary_normalized: "team.normalized_name",
                    primary_display: "team.canonical_name",
                    alias_table: "football.team_names",
                    alias_owner: "alias.team_id",
                    owner_id: "team.id",
                    alias_normalized: "alias.normalized_name",
                    alias_display: "alias.name",
                },
            );
        }
        if query.active_only {
            builder.push(" AND team.is_active");
        }
        builder.push(" ORDER BY team.normalized_name, team.id LIMIT ");
        builder.push_bind(i64::from(query.limit.clamp(1, 500)));
        builder
            .build()
            .fetch_all(&self.pool)
            .await?
            .iter()
            .map(team_reference_from_row)
            .collect()
    }

    async fn list_player_references(
        &self,
        query: &EntityReferenceQuery,
    ) -> PersistenceResult<Vec<EntityReferenceRecord>> {
        let mut builder: QueryBuilder<Postgres> = QueryBuilder::new(
            r#"
            SELECT player.id, player.canonical_name, player.normalized_name,
                   player.date_of_birth, player.nationality_code, player.status,
                   COALESCE((SELECT array_agg(alias.name ORDER BY alias.name) FROM football.player_names alias WHERE alias.player_id=player.id), ARRAY[]::text[]) AS aliases,
                   COALESCE((SELECT array_agg(external.external_id ORDER BY external.external_id) FROM football.external_entity_ids external WHERE external.entity_type='player' AND external.entity_id=player.id), ARRAY[]::text[]) AS external_ids
            FROM football.players player
            WHERE 1=1
            "#,
        );
        if let Some(search) = NameSearch::parse(query.search.as_deref()) {
            push_name_search(
                &mut builder,
                &search,
                NameSearchColumns {
                    primary_normalized: "player.normalized_name",
                    primary_display: "player.canonical_name",
                    alias_table: "football.player_names",
                    alias_owner: "alias.player_id",
                    owner_id: "player.id",
                    alias_normalized: "alias.normalized_name",
                    alias_display: "alias.name",
                },
            );
        }
        if query.active_only {
            builder.push(" AND player.status='active'");
        }
        builder.push(" ORDER BY player.normalized_name, player.id LIMIT ");
        builder.push_bind(i64::from(query.limit.clamp(1, 500)));
        builder
            .build()
            .fetch_all(&self.pool)
            .await?
            .iter()
            .map(player_reference_from_row)
            .collect()
    }

    async fn list_coach_references(
        &self,
        query: &EntityReferenceQuery,
    ) -> PersistenceResult<Vec<EntityReferenceRecord>> {
        let mut builder: QueryBuilder<Postgres> = QueryBuilder::new(
            r#"
            SELECT coach.id, coach.canonical_name, coach.normalized_name,
                   coach.nationality_code, coach.status,
                   COALESCE((SELECT array_agg(alias.name ORDER BY alias.name) FROM football.coach_names alias WHERE alias.coach_id=coach.id), ARRAY[]::text[]) AS aliases,
                   COALESCE((SELECT array_agg(external.external_id ORDER BY external.external_id) FROM football.external_entity_ids external WHERE external.entity_type='coach' AND external.entity_id=coach.id), ARRAY[]::text[]) AS external_ids
            FROM football.coaches coach
            WHERE 1=1
            "#,
        );
        if let Some(search) = NameSearch::parse(query.search.as_deref()) {
            push_name_search(
                &mut builder,
                &search,
                NameSearchColumns {
                    primary_normalized: "coach.normalized_name",
                    primary_display: "coach.canonical_name",
                    alias_table: "football.coach_names",
                    alias_owner: "alias.coach_id",
                    owner_id: "coach.id",
                    alias_normalized: "alias.normalized_name",
                    alias_display: "alias.name",
                },
            );
        }
        if query.active_only {
            builder.push(" AND coach.status='active'");
        }
        builder.push(" ORDER BY coach.normalized_name, coach.id LIMIT ");
        builder.push_bind(i64::from(query.limit.clamp(1, 500)));
        builder
            .build()
            .fetch_all(&self.pool)
            .await?
            .iter()
            .map(coach_reference_from_row)
            .collect()
    }

    pub async fn resolve_entity_reference(
        &self,
        request: &EntityMatchRequest,
    ) -> PersistenceResult<EntityMatchResult> {
        validate_entity_type(&request.entity_type)?;
        if let Some(id) = request.entity_id {
            if self.entity_exists(&request.entity_type, id).await? {
                return Ok(exact_match(id, "稳定实体 ID 精确匹配"));
            }
        }
        if let (Some(provider_id), Some(external_id)) = (
            request.provider_id,
            request
                .external_id
                .as_deref()
                .map(str::trim)
                .filter(|v| !v.is_empty()),
        ) {
            let ids = sqlx::query_scalar::<_, Uuid>(
                "SELECT entity_id FROM football.external_entity_ids WHERE provider_id=$1 AND entity_type=$2 AND external_id=$3",
            )
            .bind(provider_id)
            .bind(&request.entity_type)
            .bind(external_id)
            .fetch_all(&self.pool)
            .await?;
            if ids.len() == 1 {
                return Ok(exact_match(ids[0], "受信数据源外部 ID 精确匹配"));
            }
            if ids.len() > 1 {
                return Ok(ambiguous(ids, "外部 ID 对应多条实体"));
            }
        }
        let Some(name) = request
            .canonical_name
            .as_deref()
            .map(normalize_name)
            .filter(|value| !value.is_empty())
        else {
            return Ok(EntityMatchResult {
                status: "no_match".to_string(),
                matched_id: None,
                candidates: Vec::new(),
            });
        };
        let candidates = match request.entity_type.as_str() {
            "team" => {
                self.match_teams_by_name(&name, request.country_code.as_deref())
                    .await?
            }
            "player" => {
                self.match_players_by_name(&name, request.date_of_birth)
                    .await?
            }
            "coach" => {
                self.match_coaches_by_name(&name, request.nationality_code.as_deref())
                    .await?
            }
            _ => unreachable!(),
        };
        match candidates.len() {
            0 => Ok(EntityMatchResult {
                status: "no_match".to_string(),
                matched_id: None,
                candidates,
            }),
            1 => Ok(EntityMatchResult {
                status: "exact".to_string(),
                matched_id: Some(candidates[0].id),
                candidates,
            }),
            _ => Ok(EntityMatchResult {
                status: "ambiguous".to_string(),
                matched_id: None,
                candidates,
            }),
        }
    }

    async fn match_teams_by_name(
        &self,
        normalized_name: &str,
        country_code: Option<&str>,
    ) -> PersistenceResult<Vec<EntityMatchCandidate>> {
        let rows = sqlx::query(
            r#"
            SELECT DISTINCT team.id, team.canonical_name,
                   CASE WHEN team.normalized_name=$1 THEN '正式名称' ELSE '球队别名' END AS reason
            FROM football.teams team
            LEFT JOIN football.team_names alias ON alias.team_id=team.id
            WHERE (team.normalized_name=$1 OR alias.normalized_name=$1)
              AND ($2::text IS NULL OR upper(COALESCE(team.country_code,''))=upper($2))
            ORDER BY team.canonical_name, team.id
            "#,
        )
        .bind(normalized_name)
        .bind(
            country_code
                .map(str::trim)
                .filter(|value| !value.is_empty()),
        )
        .fetch_all(&self.pool)
        .await?;
        candidate_rows(&rows, 0.95)
    }

    async fn match_players_by_name(
        &self,
        normalized_name: &str,
        date_of_birth: Option<chrono::NaiveDate>,
    ) -> PersistenceResult<Vec<EntityMatchCandidate>> {
        let rows = sqlx::query(
            r#"
            SELECT DISTINCT player.id, player.canonical_name,
                   CASE WHEN player.normalized_name=$1 THEN '规范姓名与出生日期' ELSE '球员别名与出生日期' END AS reason
            FROM football.players player
            LEFT JOIN football.player_names alias ON alias.player_id=player.id
            WHERE (player.normalized_name=$1 OR alias.normalized_name=$1)
              AND ($2::date IS NULL OR player.date_of_birth=$2)
            ORDER BY player.canonical_name, player.id
            "#,
        )
        .bind(normalized_name)
        .bind(date_of_birth)
        .fetch_all(&self.pool)
        .await?;
        candidate_rows(&rows, if date_of_birth.is_some() { 1.0 } else { 0.7 })
    }

    async fn match_coaches_by_name(
        &self,
        normalized_name: &str,
        nationality_code: Option<&str>,
    ) -> PersistenceResult<Vec<EntityMatchCandidate>> {
        let rows = sqlx::query(
            r#"
            SELECT DISTINCT coach.id, coach.canonical_name,
                   CASE WHEN coach.normalized_name=$1 THEN '规范姓名与国籍' ELSE '教练别名与国籍' END AS reason
            FROM football.coaches coach
            LEFT JOIN football.coach_names alias ON alias.coach_id=coach.id
            WHERE (coach.normalized_name=$1 OR alias.normalized_name=$1)
              AND ($2::text IS NULL OR upper(COALESCE(coach.nationality_code,''))=upper($2))
            ORDER BY coach.canonical_name, coach.id
            "#,
        )
        .bind(normalized_name)
        .bind(nationality_code.map(str::trim).filter(|value| !value.is_empty()))
        .fetch_all(&self.pool)
        .await?;
        candidate_rows(&rows, 0.95)
    }

    async fn entity_exists(&self, entity_type: &str, id: Uuid) -> PersistenceResult<bool> {
        let query = match entity_type {
            "team" => "SELECT EXISTS(SELECT 1 FROM football.teams WHERE id=$1)",
            "player" => "SELECT EXISTS(SELECT 1 FROM football.players WHERE id=$1)",
            "coach" => "SELECT EXISTS(SELECT 1 FROM football.coaches WHERE id=$1)",
            other => {
                return Err(PersistenceError::InvalidState(format!(
                    "不支持的实体类型：{other}"
                )))
            }
        };
        Ok(sqlx::query_scalar(query)
            .bind(id)
            .fetch_one(&self.pool)
            .await?)
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

fn validate_entity_type(value: &str) -> PersistenceResult<()> {
    if matches!(value, "team" | "player" | "coach") {
        Ok(())
    } else {
        Err(PersistenceError::InvalidState(format!(
            "不支持的实体类型：{value}"
        )))
    }
}

fn exact_match(id: Uuid, reason: &str) -> EntityMatchResult {
    EntityMatchResult {
        status: "exact".to_string(),
        matched_id: Some(id),
        candidates: vec![EntityMatchCandidate {
            id,
            label: id.to_string(),
            reason: reason.to_string(),
            score: 1.0,
        }],
    }
}

fn ambiguous(ids: Vec<Uuid>, reason: &str) -> EntityMatchResult {
    EntityMatchResult {
        status: "ambiguous".to_string(),
        matched_id: None,
        candidates: ids
            .into_iter()
            .map(|id| EntityMatchCandidate {
                id,
                label: id.to_string(),
                reason: reason.to_string(),
                score: 1.0,
            })
            .collect(),
    }
}

fn candidate_rows(
    rows: &[sqlx::postgres::PgRow],
    score: f64,
) -> PersistenceResult<Vec<EntityMatchCandidate>> {
    rows.iter()
        .map(|row| {
            Ok(EntityMatchCandidate {
                id: row.try_get("id")?,
                label: row.try_get("canonical_name")?,
                reason: row.try_get("reason")?,
                score,
            })
        })
        .collect()
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

fn team_reference_from_row(
    row: &sqlx::postgres::PgRow,
) -> PersistenceResult<EntityReferenceRecord> {
    Ok(EntityReferenceRecord {
        entity_type: "team".to_string(),
        id: row.try_get("id")?,
        canonical_name: row.try_get("canonical_name")?,
        normalized_name: row.try_get("normalized_name")?,
        country_code: row.try_get("country_code")?,
        nationality_code: None,
        date_of_birth: None,
        status: if row.try_get::<bool, _>("is_active")? {
            "active".to_string()
        } else {
            "inactive".to_string()
        },
        aliases: row.try_get("aliases")?,
        external_ids: row.try_get("external_ids")?,
    })
}

fn player_reference_from_row(
    row: &sqlx::postgres::PgRow,
) -> PersistenceResult<EntityReferenceRecord> {
    Ok(EntityReferenceRecord {
        entity_type: "player".to_string(),
        id: row.try_get("id")?,
        canonical_name: row.try_get("canonical_name")?,
        normalized_name: row.try_get("normalized_name")?,
        country_code: None,
        nationality_code: row.try_get("nationality_code")?,
        date_of_birth: row.try_get("date_of_birth")?,
        status: row.try_get("status")?,
        aliases: row.try_get("aliases")?,
        external_ids: row.try_get("external_ids")?,
    })
}

fn coach_reference_from_row(
    row: &sqlx::postgres::PgRow,
) -> PersistenceResult<EntityReferenceRecord> {
    Ok(EntityReferenceRecord {
        entity_type: "coach".to_string(),
        id: row.try_get("id")?,
        canonical_name: row.try_get("canonical_name")?,
        normalized_name: row.try_get("normalized_name")?,
        country_code: None,
        nationality_code: row.try_get("nationality_code")?,
        date_of_birth: None,
        status: row.try_get("status")?,
        aliases: row.try_get("aliases")?,
        external_ids: row.try_get("external_ids")?,
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

fn normalize_name(value: &str) -> String {
    value
        .split_whitespace()
        .collect::<Vec<_>>()
        .join(" ")
        .to_lowercase()
}
