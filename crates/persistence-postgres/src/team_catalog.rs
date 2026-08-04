use crate::{
    name_search::{push_name_search, NameSearch, NameSearchColumns},
    PersistenceError, PersistenceResult, PostgresStore,
};
use football_domain::{
    AvailabilityStatus, BulkDeleteBlockedItem, BulkDeleteResult, FormationDistributionQuery,
    FormationUsageListQuery, MatchStatus, TeamDetail, TeamDraft, TeamListItem, TeamListPage,
    TeamListQuery, TeamNameDraft, TeamNameRecord, TeamProfileDraft, TeamProfileRecord,
    TeamRecentMatch, TeamRecord, TeamSquadPlayer,
};
use serde_json::json;
use sqlx::{Postgres, QueryBuilder, Row};
use uuid::Uuid;

impl PostgresStore {
    pub async fn list_teams(&self, query: &TeamListQuery) -> PersistenceResult<TeamListPage> {
        if query.cursor_name.is_some() != query.cursor_id.is_some() {
            return Err(PersistenceError::InvalidState(
                "球队分页游标必须同时包含名称和 ID".to_string(),
            ));
        }
        let limit = query.limit.clamp(1, 200);
        let mut builder: QueryBuilder<Postgres> = QueryBuilder::new(
            r#"
            SELECT team.id, team.canonical_name, team.normalized_name, team.country_code,
                   COALESCE(profile.team_type, 'other') AS team_type,
                   current_coach.coach_name AS current_coach_name,
                   team.is_active,
                   COALESCE(squad.current_player_count, 0)::bigint AS current_player_count,
                   COALESCE(squad.unavailable_player_count, 0)::bigint AS unavailable_player_count,
                   squad.squad_ability_average,
                   profile.data_confidence AS profile_confidence
            FROM football.teams team
            LEFT JOIN football.team_profiles profile ON profile.team_id = team.id
            LEFT JOIN LATERAL (
                SELECT coach.canonical_name AS coach_name
                FROM football.team_coach_periods period
                JOIN football.coaches coach ON coach.id = period.coach_id
                WHERE period.team_id = team.id
                  AND period.valid_from <= current_date
                  AND (period.valid_to IS NULL OR period.valid_to >= current_date)
                ORDER BY CASE period.role WHEN 'head_coach' THEN 0 WHEN 'interim_head_coach' THEN 1 ELSE 2 END,
                         period.valid_from DESC, period.id DESC
                LIMIT 1
            ) current_coach ON true
            LEFT JOIN LATERAL (
                SELECT count(*)::bigint AS current_player_count,
                       count(*) FILTER (WHERE availability.status IN ('injured','suspended','doubtful','rested'))::bigint AS unavailable_player_count,
                       avg(ability.average_value) AS squad_ability_average
                FROM football.player_team_periods period
                JOIN football.players player ON player.id = period.player_id
                LEFT JOIN feature.player_ability_profiles ability ON ability.player_id = player.id
                LEFT JOIN LATERAL (
                    SELECT status
                    FROM football.player_availability item
                    WHERE item.player_id = player.id
                      AND item.valid_from <= now()
                      AND (item.valid_to IS NULL OR item.valid_to >= now())
                    ORDER BY item.valid_from DESC, item.created_at DESC
                    LIMIT 1
                ) availability ON true
                WHERE period.team_id = team.id
                  AND period.valid_from <= current_date
                  AND (period.valid_to IS NULL OR period.valid_to >= current_date)
                  AND period.registration_status IN ('registered','loan','trial')
                  AND player.status = 'active'
            ) squad ON true
            WHERE 1 = 1
            "#,
        );
        if query.active_only {
            builder.push(" AND team.is_active");
        }
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
        if let Some(country) = query
            .country_code
            .as_deref()
            .map(str::trim)
            .filter(|value| !value.is_empty())
        {
            builder.push(" AND upper(COALESCE(team.country_code,'')) = ");
            builder.push_bind(country.to_uppercase());
        }
        if let Some(team_type) = query
            .team_type
            .as_deref()
            .map(str::trim)
            .filter(|value| !value.is_empty())
        {
            builder.push(" AND COALESCE(profile.team_type, 'other') = ");
            builder.push_bind(team_type.to_ascii_lowercase());
        }
        if let (Some(cursor_name), Some(cursor_id)) = (&query.cursor_name, query.cursor_id) {
            builder.push(" AND (team.normalized_name, team.id) > (");
            builder.push_bind(cursor_name);
            builder.push(", ");
            builder.push_bind(cursor_id);
            builder.push(")");
        }
        builder.push(" ORDER BY team.normalized_name, team.id LIMIT ");
        builder.push_bind(i64::from(limit) + 1);
        let rows = builder.build().fetch_all(&self.pool).await?;
        let has_more = rows.len() > limit as usize;
        let items = rows
            .iter()
            .take(limit as usize)
            .map(team_list_item_from_row)
            .collect::<PersistenceResult<Vec<_>>>()?;
        let next = items.last().filter(|_| has_more);
        Ok(TeamListPage {
            next_cursor_name: next.map(|item| item.normalized_name.clone()),
            next_cursor_id: next.map(|item| item.id),
            items,
            has_more,
        })
    }

    pub async fn read_team(&self, team_id: Uuid) -> PersistenceResult<TeamDetail> {
        let row = sqlx::query(
            r#"
            SELECT id, canonical_name, normalized_name, country_code, is_active, created_at
            FROM football.teams WHERE id = $1
            "#,
        )
        .bind(team_id)
        .fetch_optional(&self.pool)
        .await?
        .ok_or_else(|| PersistenceError::InvalidState("球队不存在".to_string()))?;
        let team = team_record_from_row(&row)?;
        let names = sqlx::query(
            r#"
            SELECT id, team_id, name, normalized_name, language_code, valid_from, valid_to
            FROM football.team_names
            WHERE team_id = $1
            ORDER BY valid_from DESC NULLS LAST, name, id
            "#,
        )
        .bind(team_id)
        .fetch_all(&self.pool)
        .await?
        .iter()
        .map(team_name_from_row)
        .collect::<PersistenceResult<Vec<_>>>()?;
        let profile = sqlx::query(
            r#"
            SELECT team_id, short_name, team_type, founded_year, city, stadium, head_coach,
                   default_formation, tactical_style, attack_rating, midfield_rating,
                   defence_rating, goalkeeper_rating, reputation, data_confidence,
                   notes, metadata, updated_at
            FROM football.team_profiles WHERE team_id = $1
            "#,
        )
        .bind(team_id)
        .fetch_optional(&self.pool)
        .await?
        .as_ref()
        .map(team_profile_from_row)
        .transpose()?;
        let squad = sqlx::query(
            r#"
            SELECT player.id AS player_id, player.canonical_name AS player_name,
                   localized_name.name AS localized_name,
                   position.position_code, position.default_role_code AS role_code,
                   period.squad_number, period.registration_status,
                   availability.status AS availability_status,
                   ability.average_value AS ability_average
            FROM football.player_team_periods period
            JOIN football.players player ON player.id = period.player_id
            LEFT JOIN LATERAL (
                SELECT alias.name
                FROM football.player_names alias
                WHERE alias.player_id = player.id
                  AND (
                    lower(COALESCE(alias.language_code, '')) IN ('zh-cn', 'zh-hans', 'zh')
                    OR alias.name ~ '[一-龥]'
                  )
                ORDER BY
                  CASE lower(COALESCE(alias.language_code, ''))
                    WHEN 'zh-cn' THEN 0 WHEN 'zh-hans' THEN 1 WHEN 'zh' THEN 2 ELSE 3
                  END,
                  alias.is_primary DESC,
                  alias.valid_from DESC NULLS LAST,
                  alias.id DESC
                LIMIT 1
            ) localized_name ON true
            LEFT JOIN LATERAL (
                SELECT item.position_code, item.default_role_code
                FROM football.player_positions item
                WHERE item.player_id = player.id
                  AND (item.valid_from IS NULL OR item.valid_from <= current_date)
                  AND (item.valid_to IS NULL OR item.valid_to >= current_date)
                ORDER BY item.is_primary DESC, item.proficiency DESC, item.position_code
                LIMIT 1
            ) position ON true
            LEFT JOIN LATERAL (
                SELECT item.status
                FROM football.player_availability item
                WHERE item.player_id = player.id
                  AND item.valid_from <= now()
                  AND (item.valid_to IS NULL OR item.valid_to >= now())
                ORDER BY item.valid_from DESC, item.created_at DESC
                LIMIT 1
            ) availability ON true
            LEFT JOIN feature.player_ability_profiles ability ON ability.player_id = player.id
            WHERE period.team_id = $1
              AND period.valid_from <= current_date
              AND (period.valid_to IS NULL OR period.valid_to >= current_date)
              AND period.registration_status IN ('registered','loan','trial')
              AND player.status = 'active'
            ORDER BY position.position_code NULLS LAST, period.squad_number NULLS LAST,
                     player.normalized_name, player.id
            "#,
        )
        .bind(team_id)
        .fetch_all(&self.pool)
        .await?
        .iter()
        .map(team_squad_player_from_row)
        .collect::<PersistenceResult<Vec<_>>>()?;
        let player_periods = self.list_team_player_periods(team_id).await?;
        let coach_periods = self.list_team_coach_periods(team_id).await?;
        let recent_matches = sqlx::query(
            r#"
            SELECT fixture.id AS match_id,
                   CASE WHEN fixture.home_team_id = $1 THEN fixture.away_team_id ELSE fixture.home_team_id END AS opponent_team_id,
                   CASE WHEN fixture.home_team_id = $1 THEN away.canonical_name ELSE home.canonical_name END AS opponent_team_name,
                   fixture.kickoff_time,
                   CASE WHEN fixture.home_team_id = $1 THEN 'home' ELSE 'away' END AS venue_side,
                   fixture.status,
                   CASE WHEN result.match_id IS NULL THEN NULL
                        WHEN fixture.home_team_id = $1 THEN result.home_goals_90 ELSE result.away_goals_90 END AS goals_for,
                   CASE WHEN result.match_id IS NULL THEN NULL
                        WHEN fixture.home_team_id = $1 THEN result.away_goals_90 ELSE result.home_goals_90 END AS goals_against
            FROM football.matches fixture
            JOIN football.teams home ON home.id = fixture.home_team_id
            JOIN football.teams away ON away.id = fixture.away_team_id
            LEFT JOIN football.match_results result ON result.match_id = fixture.id
            WHERE fixture.home_team_id = $1 OR fixture.away_team_id = $1
            ORDER BY fixture.kickoff_time DESC, fixture.id DESC
            LIMIT 20
            "#,
        )
        .bind(team_id)
        .fetch_all(&self.pool)
        .await?
        .iter()
        .map(team_recent_match_from_row)
        .collect::<PersistenceResult<Vec<_>>>()?;
        let formation_usage = self
            .list_formation_usage_distributions(&FormationUsageListQuery {
                team_id: Some(team_id),
                coach_id: None,
                competition_id: None,
                limit: 200,
            })
            .await?;
        let resolved_formation_distribution = self
            .resolve_formation_distribution(&FormationDistributionQuery {
                match_id: None,
                team_id,
                coach_id: None,
                competition_id: None,
                as_of: None,
            })
            .await?;
        Ok(TeamDetail {
            team,
            names,
            profile,
            squad,
            player_periods,
            coach_periods,
            recent_matches,
            formation_usage,
            resolved_formation_distribution,
        })
    }

    pub async fn update_team(
        &self,
        team_id: Uuid,
        draft: &TeamDraft,
    ) -> PersistenceResult<TeamRecord> {
        let name = draft.canonical_name.trim();
        if name.is_empty() {
            return Err(PersistenceError::InvalidState(
                "球队名称不能为空".to_string(),
            ));
        }
        let mut tx = self.pool.begin().await?;
        let row = sqlx::query(
            r#"
            UPDATE football.teams
            SET canonical_name = $2, normalized_name = $3, country_code = $4,
                metadata = metadata || $5, updated_at = now()
            WHERE id = $1
            RETURNING id, canonical_name, normalized_name, country_code, is_active, created_at
            "#,
        )
        .bind(team_id)
        .bind(name)
        .bind(normalize_name(name))
        .bind(
            draft
                .country_code
                .as_deref()
                .map(str::trim)
                .filter(|v| !v.is_empty()),
        )
        .bind(&draft.metadata)
        .fetch_optional(&mut *tx)
        .await?
        .ok_or_else(|| PersistenceError::InvalidState("球队不存在".to_string()))?;
        sqlx::query(
            "INSERT INTO audit.events (id,event_type,entity_type,entity_id,payload) VALUES ($1,'team_updated','team',$2,$3)",
        )
        .bind(Uuid::new_v4())
        .bind(team_id.to_string())
        .bind(json!({"canonical_name": name, "source": "manual"}))
        .execute(&mut *tx)
        .await?;
        tx.commit().await?;
        team_record_from_row(&row)
    }

    pub async fn add_team_name(&self, draft: &TeamNameDraft) -> PersistenceResult<TeamNameRecord> {
        let name = draft.name.trim();
        if name.is_empty() {
            return Err(PersistenceError::InvalidState(
                "球队别名不能为空".to_string(),
            ));
        }
        if let (Some(from), Some(to)) = (draft.valid_from, draft.valid_to) {
            if to < from {
                return Err(PersistenceError::InvalidState(
                    "球队别名结束日期早于开始日期".to_string(),
                ));
            }
        }
        let mut tx = self.pool.begin().await?;
        let row = sqlx::query(
            r#"
            INSERT INTO football.team_names (
                id, team_id, name, normalized_name, language_code, valid_from, valid_to
            ) VALUES ($1,$2,$3,$4,$5,$6,$7)
            RETURNING id, team_id, name, normalized_name, language_code, valid_from, valid_to
            "#,
        )
        .bind(Uuid::new_v4())
        .bind(draft.team_id)
        .bind(name)
        .bind(normalize_name(name))
        .bind(
            draft
                .language_code
                .as_deref()
                .map(str::trim)
                .filter(|v| !v.is_empty()),
        )
        .bind(draft.valid_from)
        .bind(draft.valid_to)
        .fetch_one(&mut *tx)
        .await?;
        sqlx::query(
            "INSERT INTO audit.events (id,event_type,entity_type,entity_id,payload) VALUES ($1,'team_name_added','team',$2,$3)",
        )
        .bind(Uuid::new_v4())
        .bind(draft.team_id.to_string())
        .bind(json!({"name": name, "language_code": draft.language_code, "source": "manual"}))
        .execute(&mut *tx)
        .await?;
        tx.commit().await?;
        team_name_from_row(&row)
    }

    pub async fn upsert_team_profile(
        &self,
        team_id: Uuid,
        draft: &TeamProfileDraft,
    ) -> PersistenceResult<TeamProfileRecord> {
        validate_team_profile(draft)?;
        let mut tx = self.pool.begin().await?;
        let row = sqlx::query(
            r#"
            INSERT INTO football.team_profiles (
                team_id, short_name, team_type, founded_year, city, stadium, head_coach,
                default_formation, tactical_style, attack_rating, midfield_rating,
                defence_rating, goalkeeper_rating, reputation, data_confidence, notes, metadata
            ) VALUES ($1,$2,$3,$4,$5,$6,$7,$8,$9,$10,$11,$12,$13,$14,$15,$16,$17)
            ON CONFLICT (team_id) DO UPDATE SET
                short_name=EXCLUDED.short_name, team_type=EXCLUDED.team_type,
                founded_year=EXCLUDED.founded_year, city=EXCLUDED.city,
                stadium=EXCLUDED.stadium, head_coach=football.team_profiles.head_coach,
                default_formation=EXCLUDED.default_formation,
                tactical_style=EXCLUDED.tactical_style,
                attack_rating=EXCLUDED.attack_rating, midfield_rating=EXCLUDED.midfield_rating,
                defence_rating=EXCLUDED.defence_rating, goalkeeper_rating=EXCLUDED.goalkeeper_rating,
                reputation=EXCLUDED.reputation, data_confidence=EXCLUDED.data_confidence,
                notes=EXCLUDED.notes, metadata=football.team_profiles.metadata || EXCLUDED.metadata,
                updated_at=now()
            RETURNING team_id, short_name, team_type, founded_year, city, stadium, head_coach,
                      default_formation, tactical_style, attack_rating, midfield_rating,
                      defence_rating, goalkeeper_rating, reputation, data_confidence,
                      notes, metadata, updated_at
            "#,
        )
        .bind(team_id)
        .bind(trim_option(&draft.short_name))
        .bind(draft.team_type.trim())
        .bind(draft.founded_year)
        .bind(trim_option(&draft.city))
        .bind(trim_option(&draft.stadium))
        .bind(None::<&str>)
        .bind(trim_option(&draft.default_formation))
        .bind(draft.tactical_style.trim())
        .bind(draft.attack_rating)
        .bind(draft.midfield_rating)
        .bind(draft.defence_rating)
        .bind(draft.goalkeeper_rating)
        .bind(draft.reputation)
        .bind(draft.data_confidence)
        .bind(trim_option(&draft.notes))
        .bind(&draft.metadata)
        .fetch_one(&mut *tx)
        .await?;
        sqlx::query(
            "INSERT INTO audit.events (id,event_type,entity_type,entity_id,payload) VALUES ($1,'team_profile_updated','team',$2,$3)",
        )
        .bind(Uuid::new_v4())
        .bind(team_id.to_string())
        .bind(json!({"source": draft.metadata.get("source").cloned().unwrap_or(json!("manual"))}))
        .execute(&mut *tx)
        .await?;
        tx.commit().await?;
        team_profile_from_row(&row)
    }

    pub async fn bulk_delete_players(
        &self,
        player_ids: &[Uuid],
    ) -> PersistenceResult<BulkDeleteResult> {
        let mut deleted_ids = Vec::new();
        let mut blocked = Vec::new();
        for player_id in unique_ids(player_ids) {
            let label = sqlx::query_scalar::<_, String>(
                "SELECT canonical_name FROM football.players WHERE id = $1",
            )
            .bind(player_id)
            .fetch_optional(&self.pool)
            .await?
            .unwrap_or_else(|| player_id.to_string());
            match self.delete_player(player_id).await {
                Ok(()) => deleted_ids.push(player_id),
                Err(error) => blocked.push(BulkDeleteBlockedItem {
                    id: player_id,
                    label,
                    reason: error.to_string(),
                }),
            }
        }
        Ok(BulkDeleteResult {
            requested_count: unique_ids(player_ids).len() as u64,
            deleted_ids,
            blocked,
        })
    }

    pub async fn bulk_delete_teams(
        &self,
        team_ids: &[Uuid],
    ) -> PersistenceResult<BulkDeleteResult> {
        let ids = unique_ids(team_ids);
        let mut deleted_ids = Vec::new();
        let mut blocked = Vec::new();
        for team_id in &ids {
            match self.delete_team(*team_id).await {
                Ok(()) => deleted_ids.push(*team_id),
                Err(error) => {
                    let label = sqlx::query_scalar::<_, String>(
                        "SELECT canonical_name FROM football.teams WHERE id = $1",
                    )
                    .bind(team_id)
                    .fetch_optional(&self.pool)
                    .await?
                    .unwrap_or_else(|| team_id.to_string());
                    blocked.push(BulkDeleteBlockedItem {
                        id: *team_id,
                        label,
                        reason: error.to_string(),
                    });
                }
            }
        }
        Ok(BulkDeleteResult {
            requested_count: ids.len() as u64,
            deleted_ids,
            blocked,
        })
    }

    async fn delete_team(&self, team_id: Uuid) -> PersistenceResult<()> {
        let check = self.check_entity_deletion("team", team_id).await?;
        if !check.can_permanently_delete {
            return Err(PersistenceError::InvalidState(check.reason));
        }
        let mut tx = self.pool.begin().await?;
        let team_name = sqlx::query_scalar::<_, String>(
            "SELECT canonical_name FROM football.teams WHERE id = $1 FOR UPDATE",
        )
        .bind(team_id)
        .fetch_optional(&mut *tx)
        .await?
        .ok_or_else(|| PersistenceError::InvalidState("球队不存在".to_string()))?;
        let match_count: i64 = sqlx::query_scalar(
            "SELECT count(*)::bigint FROM football.matches WHERE home_team_id=$1 OR away_team_id=$1",
        )
        .bind(team_id)
        .fetch_one(&mut *tx)
        .await?;
        if match_count > 0 {
            return Err(PersistenceError::InvalidState(format!(
                "球队已关联 {match_count} 场比赛，为保留历史赛果不能永久删除"
            )));
        }
        let review_count: i64 = sqlx::query_scalar(
            r#"
            SELECT
                (SELECT count(*)::bigint FROM review.team_match_reviews WHERE team_id=$1)
              + (SELECT count(*)::bigint FROM review.player_match_reviews WHERE team_id=$1)
            "#,
        )
        .bind(team_id)
        .fetch_one(&mut *tx)
        .await?;
        if review_count > 0 {
            return Err(PersistenceError::InvalidState(format!(
                "球队已关联 {review_count} 条球队或球员赛后复盘，为保留历史记录不能永久删除"
            )));
        }
        sqlx::query(
            "DELETE FROM football.external_entity_ids WHERE entity_type='team' AND entity_id=$1",
        )
        .bind(team_id)
        .execute(&mut *tx)
        .await?;
        sqlx::query("DELETE FROM football.teams WHERE id=$1")
            .bind(team_id)
            .execute(&mut *tx)
            .await?;
        sqlx::query(
            "INSERT INTO audit.events (id,event_type,entity_type,entity_id,payload) VALUES ($1,'team_deleted','team',$2,$3)",
        )
        .bind(Uuid::new_v4())
        .bind(team_id.to_string())
        .bind(json!({"canonical_name": team_name}))
        .execute(&mut *tx)
        .await?;
        tx.commit().await?;
        Ok(())
    }
}

fn validate_team_profile(draft: &TeamProfileDraft) -> PersistenceResult<()> {
    if draft
        .founded_year
        .is_some_and(|year| !(1850..=2100).contains(&year))
    {
        return Err(PersistenceError::InvalidState(
            "球队成立年份必须在1850到2100之间".to_string(),
        ));
    }
    if !matches!(
        draft.team_type.trim(),
        "club" | "national" | "reserve" | "youth" | "women" | "other"
    ) {
        return Err(PersistenceError::InvalidState("球队类型无效".to_string()));
    }
    if !matches!(
        draft.tactical_style.trim(),
        "balanced" | "possession" | "direct" | "counter" | "pressing" | "defensive" | "custom"
    ) {
        return Err(PersistenceError::InvalidState("战术风格无效".to_string()));
    }
    for (label, value) in [
        ("进攻评分", draft.attack_rating),
        ("中场评分", draft.midfield_rating),
        ("防守评分", draft.defence_rating),
        ("门将评分", draft.goalkeeper_rating),
        ("声望", draft.reputation),
    ] {
        if value.is_some_and(|value| !(0.0..=100.0).contains(&value)) {
            return Err(PersistenceError::InvalidState(format!(
                "{label}必须在0到100之间"
            )));
        }
    }
    if !(0.0..=1.0).contains(&draft.data_confidence) {
        return Err(PersistenceError::InvalidState(
            "球队资料可信度必须在0到1之间".to_string(),
        ));
    }
    Ok(())
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

fn trim_option(value: &Option<String>) -> Option<&str> {
    value
        .as_deref()
        .map(str::trim)
        .filter(|value| !value.is_empty())
}

fn normalize_name(value: &str) -> String {
    value
        .split_whitespace()
        .collect::<Vec<_>>()
        .join(" ")
        .to_lowercase()
}

fn team_record_from_row(row: &sqlx::postgres::PgRow) -> PersistenceResult<TeamRecord> {
    Ok(TeamRecord {
        id: row.try_get("id")?,
        canonical_name: row.try_get("canonical_name")?,
        normalized_name: row.try_get("normalized_name")?,
        country_code: row.try_get("country_code")?,
        is_active: row.try_get("is_active")?,
        created_at: row.try_get("created_at")?,
    })
}

fn team_list_item_from_row(row: &sqlx::postgres::PgRow) -> PersistenceResult<TeamListItem> {
    Ok(TeamListItem {
        id: row.try_get("id")?,
        canonical_name: row.try_get("canonical_name")?,
        normalized_name: row.try_get("normalized_name")?,
        country_code: row.try_get("country_code")?,
        team_type: row.try_get("team_type")?,
        current_coach_name: row.try_get("current_coach_name")?,
        is_active: row.try_get("is_active")?,
        current_player_count: row.try_get("current_player_count")?,
        unavailable_player_count: row.try_get("unavailable_player_count")?,
        squad_ability_average: row.try_get("squad_ability_average")?,
        profile_confidence: row.try_get("profile_confidence")?,
    })
}

fn team_name_from_row(row: &sqlx::postgres::PgRow) -> PersistenceResult<TeamNameRecord> {
    Ok(TeamNameRecord {
        id: row.try_get("id")?,
        team_id: row.try_get("team_id")?,
        name: row.try_get("name")?,
        normalized_name: row.try_get("normalized_name")?,
        language_code: row.try_get("language_code")?,
        valid_from: row.try_get("valid_from")?,
        valid_to: row.try_get("valid_to")?,
    })
}

fn team_profile_from_row(row: &sqlx::postgres::PgRow) -> PersistenceResult<TeamProfileRecord> {
    Ok(TeamProfileRecord {
        team_id: row.try_get("team_id")?,
        short_name: row.try_get("short_name")?,
        team_type: row.try_get("team_type")?,
        founded_year: row.try_get("founded_year")?,
        city: row.try_get("city")?,
        stadium: row.try_get("stadium")?,
        head_coach: row.try_get("head_coach")?,
        default_formation: row.try_get("default_formation")?,
        tactical_style: row.try_get("tactical_style")?,
        attack_rating: row.try_get("attack_rating")?,
        midfield_rating: row.try_get("midfield_rating")?,
        defence_rating: row.try_get("defence_rating")?,
        goalkeeper_rating: row.try_get("goalkeeper_rating")?,
        reputation: row.try_get("reputation")?,
        data_confidence: row.try_get("data_confidence")?,
        notes: row.try_get("notes")?,
        metadata: row.try_get("metadata")?,
        updated_at: row.try_get("updated_at")?,
    })
}

fn team_squad_player_from_row(row: &sqlx::postgres::PgRow) -> PersistenceResult<TeamSquadPlayer> {
    let availability: Option<String> = row.try_get("availability_status")?;
    Ok(TeamSquadPlayer {
        player_id: row.try_get("player_id")?,
        player_name: row.try_get("player_name")?,
        localized_name: row.try_get("localized_name")?,
        position_code: row.try_get("position_code")?,
        role_code: row.try_get("role_code")?,
        squad_number: row.try_get("squad_number")?,
        registration_status: row.try_get("registration_status")?,
        availability_status: availability
            .as_deref()
            .map(parse_availability)
            .transpose()?,
        ability_average: row.try_get("ability_average")?,
    })
}

fn team_recent_match_from_row(row: &sqlx::postgres::PgRow) -> PersistenceResult<TeamRecentMatch> {
    let status: String = row.try_get("status")?;
    Ok(TeamRecentMatch {
        match_id: row.try_get("match_id")?,
        opponent_team_id: row.try_get("opponent_team_id")?,
        opponent_team_name: row.try_get("opponent_team_name")?,
        kickoff_time: row.try_get("kickoff_time")?,
        venue_side: row.try_get("venue_side")?,
        status: parse_match_status(&status)?,
        goals_for: row.try_get("goals_for")?,
        goals_against: row.try_get("goals_against")?,
    })
}

fn parse_availability(value: &str) -> PersistenceResult<AvailabilityStatus> {
    match value {
        "available" => Ok(AvailabilityStatus::Available),
        "doubtful" => Ok(AvailabilityStatus::Doubtful),
        "unavailable" => Ok(AvailabilityStatus::Unavailable),
        "injured" => Ok(AvailabilityStatus::Injured),
        "suspended" => Ok(AvailabilityStatus::Suspended),
        "rested" => Ok(AvailabilityStatus::Rested),
        "returning" => Ok(AvailabilityStatus::Returning),
        "unknown" => Ok(AvailabilityStatus::Unknown),
        other => Err(PersistenceError::InvalidState(format!(
            "未知球员可用状态：{other}"
        ))),
    }
}

fn parse_match_status(value: &str) -> PersistenceResult<MatchStatus> {
    match value {
        "scheduled" => Ok(MatchStatus::Scheduled),
        "live" => Ok(MatchStatus::Live),
        "finished" => Ok(MatchStatus::Finished),
        "postponed" => Ok(MatchStatus::Postponed),
        "cancelled" => Ok(MatchStatus::Cancelled),
        other => Err(PersistenceError::InvalidState(format!(
            "未知比赛状态：{other}"
        ))),
    }
}
