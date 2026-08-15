use crate::{PersistenceError, PersistenceResult, PostgresStore};
use football_domain::{
    BulkDeleteBlockedItem, BulkDeleteResult, TeamNameDraft, TeamNameRecord, TeamProfileDraft,
    TeamProfileRecord,
};
use serde_json::json;
use sqlx::Row;
use uuid::Uuid;

impl PostgresStore {
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
        crate::write_audit_event(
            &mut tx,
            "team_name_added",
            "team",
            draft.team_id.to_string(),
            json!({"name": name, "language_code": draft.language_code, "source": "manual"}),
        )
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
        crate::write_audit_event(
            &mut tx,
            "team_profile_updated",
            "team",
            team_id.to_string(),
            json!({"source": draft.metadata.get("source").cloned().unwrap_or(json!("manual"))}),
        )
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
        crate::write_audit_event(
            &mut tx,
            "team_deleted",
            "team",
            team_id.to_string(),
            json!({"canonical_name": team_name}),
        )
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
