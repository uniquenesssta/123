use crate::{
    role_resolution::{
        metadata_with_role_resolution, resolve_default_tactical_role_in_tx,
        resolve_tactical_role,
    },
    write_audit_event, PersistenceError, PersistenceResult, PostgresStore,
};
use chrono::Utc;
use football_domain::{
    AvailabilityStatus, TeamLineupPresetApplicationPreview, TeamLineupPresetDraft,
    TeamLineupPresetMemberRecord, TeamLineupPresetRecord,
};
use serde_json::{json, Value};
use sqlx::{Postgres, Row, Transaction};
use std::collections::HashSet;
use uuid::Uuid;

fn parse_availability(value: Option<String>) -> PersistenceResult<Option<AvailabilityStatus>> {
    value
        .map(|status| match status.as_str() {
            "available" => Ok(AvailabilityStatus::Available),
            "doubtful" => Ok(AvailabilityStatus::Doubtful),
            "injured" => Ok(AvailabilityStatus::Injured),
            "suspended" => Ok(AvailabilityStatus::Suspended),
            "rested" => Ok(AvailabilityStatus::Rested),
            "returning" => Ok(AvailabilityStatus::Returning),
            "unavailable" => Ok(AvailabilityStatus::Unavailable),
            "unknown" => Ok(AvailabilityStatus::Unknown),
            other => Err(PersistenceError::InvalidState(format!(
                "未知球员可用状态：{other}"
            ))),
        })
        .transpose()
}

fn validate_preset(draft: &TeamLineupPresetDraft) -> PersistenceResult<()> {
    if draft.name.trim().is_empty() {
        return Err(PersistenceError::InvalidState("阵容预设名称不能为空".to_string()));
    }
    if let Some(probability) = draft.usage_probability {
        if !(0.0..=1.0).contains(&probability) {
            return Err(PersistenceError::InvalidState(
                "阵容预设使用概率必须在 0 到 1 之间".to_string(),
            ));
        }
    }
    if draft.members.len() < 11 {
        return Err(PersistenceError::InvalidState(
            "阵容预设至少需要 11 名球员".to_string(),
        ));
    }
    let starter_count = draft.members.iter().filter(|member| member.is_starter).count();
    if starter_count != 11 {
        return Err(PersistenceError::InvalidState(format!(
            "阵容预设必须恰好包含 11 名首发，当前为 {starter_count} 名"
        )));
    }
    let unique_players = draft
        .members
        .iter()
        .map(|member| member.player_id)
        .collect::<HashSet<_>>();
    if unique_players.len() != draft.members.len() {
        return Err(PersistenceError::InvalidState(
            "阵容预设不能包含重复球员".to_string(),
        ));
    }
    if draft.members.iter().filter(|member| member.is_captain).count() > 1 {
        return Err(PersistenceError::InvalidState(
            "阵容预设最多只能设置一名队长".to_string(),
        ));
    }
    Ok(())
}

async fn verify_membership_in_tx(
    tx: &mut Transaction<'_, Postgres>,
    team_id: Uuid,
    player_ids: &[Uuid],
) -> PersistenceResult<()> {
    for player_id in player_ids {
        let belongs: bool = sqlx::query_scalar(
            r#"
            SELECT EXISTS (
                SELECT 1
                FROM football.player_team_periods period
                WHERE period.player_id = $1
                  AND period.team_id = $2
                  AND period.valid_from <= current_date
                  AND (period.valid_to IS NULL OR period.valid_to >= current_date)
                  AND period.registration_status IN ('registered', 'loan', 'trial')
            )
            "#,
        )
        .bind(player_id)
        .bind(team_id)
        .fetch_one(&mut **tx)
        .await?;
        if !belongs {
            return Err(PersistenceError::InvalidState(format!(
                "球员 {player_id} 当前不属于该球队，不能保存到活动阵容预设"
            )));
        }
    }
    Ok(())
}

impl PostgresStore {
    pub async fn save_team_lineup_preset(
        &self,
        draft: &TeamLineupPresetDraft,
    ) -> PersistenceResult<TeamLineupPresetRecord> {
        validate_preset(draft)?;
        let mut tx = self.pool.begin().await?;
        let team_exists: bool = sqlx::query_scalar(
            "SELECT EXISTS (SELECT 1 FROM football.teams WHERE id = $1 AND is_active)",
        )
        .bind(draft.team_id)
        .fetch_one(&mut *tx)
        .await?;
        if !team_exists {
            return Err(PersistenceError::InvalidState(
                "阵容预设所属球队不存在或已停用".to_string(),
            ));
        }
        verify_membership_in_tx(
            &mut tx,
            draft.team_id,
            &draft.members.iter().map(|member| member.player_id).collect::<Vec<_>>(),
        )
        .await?;

        let preset_id = draft.id.unwrap_or_else(Uuid::new_v4);
        let current_version = if draft.id.is_some() {
            let row = sqlx::query(
                "SELECT team_id, version, status FROM football.team_lineup_presets WHERE id=$1 FOR UPDATE",
            )
            .bind(preset_id)
            .fetch_optional(&mut *tx)
            .await?
            .ok_or_else(|| PersistenceError::InvalidState("阵容预设不存在".to_string()))?;
            let existing_team_id: Uuid = row.try_get("team_id")?;
            let status: String = row.try_get("status")?;
            if existing_team_id != draft.team_id {
                return Err(PersistenceError::InvalidState(
                    "不能把阵容预设移动到其他球队".to_string(),
                ));
            }
            if status != "active" {
                return Err(PersistenceError::InvalidState(
                    "已归档阵容预设不能直接修改，请先复制为新预设".to_string(),
                ));
            }
            row.try_get::<i32, _>("version")? + 1
        } else {
            1
        };

        if draft.is_default {
            sqlx::query(
                "UPDATE football.team_lineup_presets SET is_default=false, updated_at=now() WHERE team_id=$1 AND status='active'",
            )
            .bind(draft.team_id)
            .execute(&mut *tx)
            .await?;
        }

        if draft.id.is_some() {
            sqlx::query(
                r#"
                UPDATE football.team_lineup_presets
                SET name=$2, formation_id=$3, coach_id=$4, usage_context=$5,
                    usage_probability=$6, is_default=$7, source_lineup_id=$8,
                    notes=$9, version=$10, updated_at=now()
                WHERE id=$1
                "#,
            )
            .bind(preset_id)
            .bind(draft.name.trim())
            .bind(draft.formation_id)
            .bind(draft.coach_id)
            .bind(draft.usage_context.trim())
            .bind(draft.usage_probability)
            .bind(draft.is_default)
            .bind(draft.source_lineup_id)
            .bind(draft.notes.as_deref().map(str::trim).filter(|value| !value.is_empty()))
            .bind(current_version)
            .execute(&mut *tx)
            .await?;
            sqlx::query("DELETE FROM football.team_lineup_preset_members WHERE preset_id=$1")
                .bind(preset_id)
                .execute(&mut *tx)
                .await?;
        } else {
            sqlx::query(
                r#"
                INSERT INTO football.team_lineup_presets (
                    id, team_id, name, formation_id, coach_id, usage_context,
                    usage_probability, is_default, source_lineup_id, notes, version
                ) VALUES ($1,$2,$3,$4,$5,$6,$7,$8,$9,$10,1)
                "#,
            )
            .bind(preset_id)
            .bind(draft.team_id)
            .bind(draft.name.trim())
            .bind(draft.formation_id)
            .bind(draft.coach_id)
            .bind(draft.usage_context.trim())
            .bind(draft.usage_probability)
            .bind(draft.is_default)
            .bind(draft.source_lineup_id)
            .bind(draft.notes.as_deref().map(str::trim).filter(|value| !value.is_empty()))
            .execute(&mut *tx)
            .await?;
        }

        let role_as_of = Utc::now().date_naive();
        for member in &draft.members {
            let position_code = member
                .position_code
                .as_deref()
                .map(str::trim)
                .filter(|value| !value.is_empty())
                .map(str::to_uppercase);
            let inherited_role = resolve_default_tactical_role_in_tx(
                &mut tx,
                member.player_id,
                position_code.as_deref(),
                role_as_of,
            )
            .await?;
            let role_resolution =
                resolve_tactical_role(member.role_code.as_deref(), inherited_role.as_ref());
            let member_metadata =
                metadata_with_role_resolution(&member.metadata, &role_resolution);
            sqlx::query(
                r#"
                INSERT INTO football.team_lineup_preset_members (
                    preset_id, player_id, position_code, role_code, is_starter,
                    shirt_number, expected_minutes, sequence_no, bench_order,
                    is_captain, metadata
                ) VALUES ($1,$2,$3,$4,$5,$6,$7,$8,$9,$10,$11)
                "#,
            )
            .bind(preset_id)
            .bind(member.player_id)
            .bind(position_code)
            .bind(role_resolution.role_code.as_deref())
            .bind(member.is_starter)
            .bind(member.shirt_number)
            .bind(member.expected_minutes)
            .bind(member.sequence_no)
            .bind(member.bench_order)
            .bind(member.is_captain)
            .bind(member_metadata)
            .execute(&mut *tx)
            .await?;
        }

        write_audit_event(
            &mut tx,
            if draft.id.is_some() {
                "team_lineup_preset_updated"
            } else {
                "team_lineup_preset_created"
            },
            "team_lineup_preset",
            Some(preset_id.to_string()),
            json!({
                "team_id": draft.team_id,
                "name": draft.name.trim(),
                "version": current_version,
                "member_count": draft.members.len(),
                "starter_count": draft.members.iter().filter(|member| member.is_starter).count(),
            }),
        )
        .await?;
        tx.commit().await?;
        self.read_team_lineup_preset(preset_id).await
    }

    pub async fn list_team_lineup_presets(
        &self,
        team_id: Uuid,
        include_archived: bool,
    ) -> PersistenceResult<Vec<TeamLineupPresetRecord>> {
        let ids = sqlx::query_scalar::<_, Uuid>(
            r#"
            SELECT id
            FROM football.team_lineup_presets
            WHERE team_id=$1 AND ($2 OR status='active')
            ORDER BY is_default DESC, status, updated_at DESC, lower(name), id
            LIMIT 200
            "#,
        )
        .bind(team_id)
        .bind(include_archived)
        .fetch_all(&self.pool)
        .await?;
        let mut result = Vec::with_capacity(ids.len());
        for id in ids {
            result.push(self.read_team_lineup_preset(id).await?);
        }
        Ok(result)
    }

    pub async fn read_team_lineup_preset(
        &self,
        preset_id: Uuid,
    ) -> PersistenceResult<TeamLineupPresetRecord> {
        let row = sqlx::query(
            r#"
            SELECT preset.id, preset.team_id, team.canonical_name AS team_name,
                   preset.name, preset.formation_id, formation.code AS formation_code,
                   formation.name AS formation_name, preset.coach_id,
                   coach.canonical_name AS coach_name, preset.usage_context,
                   preset.usage_probability, preset.is_default, preset.status,
                   preset.version, preset.source_lineup_id, preset.notes,
                   preset.created_at, preset.updated_at,
                   count(member.player_id) AS member_count,
                   count(member.player_id) FILTER (WHERE member.is_starter) AS starter_count
            FROM football.team_lineup_presets preset
            JOIN football.teams team ON team.id=preset.team_id
            LEFT JOIN football.formations formation ON formation.id=preset.formation_id
            LEFT JOIN football.coaches coach ON coach.id=preset.coach_id
            LEFT JOIN football.team_lineup_preset_members member ON member.preset_id=preset.id
            WHERE preset.id=$1
            GROUP BY preset.id, team.canonical_name, formation.code, formation.name,
                     coach.canonical_name
            "#,
        )
        .bind(preset_id)
        .fetch_optional(&self.pool)
        .await?
        .ok_or_else(|| PersistenceError::InvalidState("阵容预设不存在".to_string()))?;

        let member_rows = sqlx::query(
            r#"
            SELECT member.player_id, player.canonical_name AS player_name,
                   alternate_name.name AS alternate_name, member.position_code,
                   COALESCE(NULLIF(btrim(member.role_code), ''), inherited_role.default_role_code)
                       AS role_code,
                   CASE
                     WHEN member.metadata->>'role_origin' IN (
                       'lineup_override', 'player_position_default', 'missing'
                     ) THEN member.metadata->>'role_origin'
                     WHEN NULLIF(btrim(member.role_code), '') IS NOT NULL THEN 'lineup_override'
                     WHEN inherited_role.default_role_code IS NOT NULL THEN 'player_position_default'
                     ELSE 'missing'
                   END AS role_origin,
                   CASE
                     WHEN member.metadata->>'role_origin' = 'player_position_default'
                       THEN COALESCE(
                         NULLIF(btrim(member.metadata->>'role_source_position_code'), ''),
                         inherited_role.position_code
                       )
                     WHEN member.metadata->>'role_origin' IN ('lineup_override', 'missing')
                       THEN NULL
                     WHEN NULLIF(btrim(member.role_code), '') IS NOT NULL THEN NULL
                     WHEN inherited_role.default_role_code IS NOT NULL
                       THEN inherited_role.position_code
                     ELSE NULL
                   END AS role_source_position_code,
                   member.is_starter, member.shirt_number,
                   member.expected_minutes, member.sequence_no, member.bench_order,
                   member.is_captain, player.status AS player_status, member.metadata,
                   current_team.team_id AS current_team_id,
                   current_team.team_name AS current_team_name,
                   current_availability.status AS availability_status
            FROM football.team_lineup_preset_members member
            JOIN football.team_lineup_presets preset ON preset.id=member.preset_id
            JOIN football.players player ON player.id=member.player_id
            LEFT JOIN LATERAL (
                SELECT alias.name
                FROM football.player_names alias
                WHERE alias.player_id=player.id
                  AND alias.name <> player.canonical_name
                  AND NOT (alias.name ~ '[一-龥]')
                ORDER BY CASE lower(COALESCE(alias.language_code,''))
                    WHEN 'en' THEN 0 WHEN 'pt' THEN 1 WHEN 'es' THEN 2 ELSE 3 END,
                    alias.is_primary DESC, alias.id DESC
                LIMIT 1
            ) alternate_name ON true
            LEFT JOIN LATERAL (
                SELECT position.default_role_code, position.position_code
                FROM football.player_positions position
                WHERE position.player_id = member.player_id
                  AND position.default_role_code IS NOT NULL
                  AND btrim(position.default_role_code) <> ''
                  AND (position.valid_from IS NULL OR position.valid_from <= current_date)
                  AND (position.valid_to IS NULL OR position.valid_to >= current_date)
                ORDER BY
                  CASE
                    WHEN member.position_code IS NOT NULL
                     AND upper(position.position_code) = upper(member.position_code) THEN 0
                    WHEN position.is_primary THEN 1
                    ELSE 2
                  END,
                  position.proficiency DESC,
                  position.valid_from DESC NULLS LAST,
                  position.id DESC
                LIMIT 1
            ) inherited_role ON true
            LEFT JOIN LATERAL (
                SELECT period.team_id, team.canonical_name AS team_name
                FROM football.player_team_periods period
                JOIN football.teams team ON team.id=period.team_id
                WHERE period.player_id=player.id
                  AND period.team_id=preset.team_id
                  AND period.valid_from <= current_date
                  AND (period.valid_to IS NULL OR period.valid_to >= current_date)
                  AND period.registration_status IN ('registered','loan','trial')
                ORDER BY period.valid_from DESC, period.id DESC
                LIMIT 1
            ) current_team ON true
            LEFT JOIN LATERAL (
                SELECT availability.status
                FROM football.player_availability availability
                WHERE availability.player_id=player.id
                  AND (availability.team_id IS NULL OR availability.team_id=current_team.team_id)
                  AND availability.valid_from <= now()
                  AND (availability.valid_to IS NULL OR availability.valid_to >= now())
                ORDER BY availability.valid_from DESC, availability.created_at DESC
                LIMIT 1
            ) current_availability ON true
            WHERE member.preset_id=$1
            ORDER BY member.is_starter DESC, member.sequence_no,
                     member.bench_order NULLS LAST, player.normalized_name
            "#,
        )
        .bind(preset_id)
        .fetch_all(&self.pool)
        .await?;

        let members = member_rows
            .into_iter()
            .map(|member| {
                Ok(TeamLineupPresetMemberRecord {
                    player_id: member.try_get("player_id")?,
                    player_name: member.try_get("player_name")?,
                    alternate_name: member.try_get("alternate_name")?,
                    position_code: member.try_get("position_code")?,
                    role_code: member.try_get("role_code")?,
                    role_origin: member.try_get("role_origin")?,
                    role_source_position_code: member.try_get("role_source_position_code")?,
                    is_starter: member.try_get("is_starter")?,
                    shirt_number: member.try_get("shirt_number")?,
                    expected_minutes: member.try_get("expected_minutes")?,
                    sequence_no: member.try_get("sequence_no")?,
                    bench_order: member.try_get("bench_order")?,
                    is_captain: member.try_get("is_captain")?,
                    current_team_id: member.try_get("current_team_id")?,
                    current_team_name: member.try_get("current_team_name")?,
                    player_status: member.try_get("player_status")?,
                    availability_status: parse_availability(member.try_get("availability_status")?)?,
                    metadata: member.try_get::<Value, _>("metadata")?,
                })
            })
            .collect::<PersistenceResult<Vec<_>>>()?;

        Ok(TeamLineupPresetRecord {
            id: row.try_get("id")?,
            team_id: row.try_get("team_id")?,
            team_name: row.try_get("team_name")?,
            name: row.try_get("name")?,
            formation_id: row.try_get("formation_id")?,
            formation_code: row.try_get("formation_code")?,
            formation_name: row.try_get("formation_name")?,
            coach_id: row.try_get("coach_id")?,
            coach_name: row.try_get("coach_name")?,
            usage_context: row.try_get("usage_context")?,
            usage_probability: row.try_get("usage_probability")?,
            is_default: row.try_get("is_default")?,
            status: row.try_get("status")?,
            version: row.try_get("version")?,
            source_lineup_id: row.try_get("source_lineup_id")?,
            notes: row.try_get("notes")?,
            starter_count: row.try_get("starter_count")?,
            member_count: row.try_get("member_count")?,
            members,
            created_at: row.try_get("created_at")?,
            updated_at: row.try_get("updated_at")?,
        })
    }

    pub async fn preview_team_lineup_preset_application(
        &self,
        preset_id: Uuid,
    ) -> PersistenceResult<TeamLineupPresetApplicationPreview> {
        let preset = self.read_team_lineup_preset(preset_id).await?;
        let mut blockers = Vec::new();
        let mut warnings = Vec::new();
        if preset.status != "active" {
            blockers.push("阵容预设已经归档".to_string());
        }
        if preset.starter_count != 11 {
            blockers.push(format!(
                "预设必须恰好包含 11 名首发，当前为 {} 名",
                preset.starter_count
            ));
        }
        for member in &preset.members {
            if member.player_status != "active" {
                blockers.push(format!("{} 已不是活动球员", member.player_name));
            }
            if member.current_team_id != Some(preset.team_id) {
                blockers.push(format!(
                    "{} 当前不再属于 {}",
                    member.player_name, preset.team_name
                ));
            }
            if matches!(
                member.availability_status,
                Some(AvailabilityStatus::Injured)
                    | Some(AvailabilityStatus::Suspended)
                    | Some(AvailabilityStatus::Unavailable)
                    | Some(AvailabilityStatus::Doubtful)
            ) {
                warnings.push(format!(
                    "{} 当前状态为 {}",
                    member.player_name,
                    member
                        .availability_status
                        .map(AvailabilityStatus::as_str)
                        .unwrap_or("unknown")
                ));
            }
        }
        blockers.sort();
        blockers.dedup();
        warnings.sort();
        warnings.dedup();
        Ok(TeamLineupPresetApplicationPreview {
            can_apply: blockers.is_empty(),
            preset,
            blockers,
            warnings,
        })
    }

    pub async fn archive_team_lineup_preset(
        &self,
        preset_id: Uuid,
    ) -> PersistenceResult<TeamLineupPresetRecord> {
        let mut tx = self.pool.begin().await?;
        let updated = sqlx::query(
            r#"
            UPDATE football.team_lineup_presets
            SET status='archived', is_default=false, updated_at=now()
            WHERE id=$1 AND status='active'
            RETURNING team_id, name
            "#,
        )
        .bind(preset_id)
        .fetch_optional(&mut *tx)
        .await?
        .ok_or_else(|| PersistenceError::InvalidState("阵容预设不存在或已经归档".to_string()))?;
        let team_id: Uuid = updated.try_get("team_id")?;
        let name: String = updated.try_get("name")?;
        write_audit_event(
            &mut tx,
            "team_lineup_preset_archived",
            "team_lineup_preset",
            Some(preset_id.to_string()),
            json!({"team_id": team_id, "name": name}),
        )
        .await?;
        tx.commit().await?;
        self.read_team_lineup_preset(preset_id).await
    }

    pub async fn delete_team_lineup_preset(&self, preset_id: Uuid) -> PersistenceResult<()> {
        let mut tx = self.pool.begin().await?;
        let row = sqlx::query(
            r#"
            SELECT team_id, name, status,
                   (SELECT count(*)::bigint
                    FROM football.team_lineup_preset_members member
                    WHERE member.preset_id = preset.id) AS member_count
            FROM football.team_lineup_presets preset
            WHERE id = $1
            FOR UPDATE
            "#,
        )
        .bind(preset_id)
        .fetch_optional(&mut *tx)
        .await?
        .ok_or_else(|| PersistenceError::InvalidState("阵容预设不存在或已经删除".to_string()))?;
        let team_id: Uuid = row.try_get("team_id")?;
        let name: String = row.try_get("name")?;
        let status: String = row.try_get("status")?;
        let member_count: i64 = row.try_get("member_count")?;

        sqlx::query("DELETE FROM football.team_lineup_presets WHERE id=$1")
            .bind(preset_id)
            .execute(&mut *tx)
            .await?;
        write_audit_event(
            &mut tx,
            "team_lineup_preset_deleted",
            "team_lineup_preset",
            Some(preset_id.to_string()),
            json!({
                "team_id": team_id,
                "name": name,
                "previous_status": status,
                "member_count": member_count,
            }),
        )
        .await?;
        tx.commit().await?;
        Ok(())
    }

    pub async fn duplicate_team_lineup_preset(
        &self,
        preset_id: Uuid,
        name: &str,
    ) -> PersistenceResult<TeamLineupPresetRecord> {
        let source = self.read_team_lineup_preset(preset_id).await?;
        let draft = TeamLineupPresetDraft {
            id: None,
            team_id: source.team_id,
            name: name.trim().to_string(),
            formation_id: source.formation_id,
            coach_id: source.coach_id,
            usage_context: source.usage_context,
            usage_probability: source.usage_probability,
            is_default: false,
            source_lineup_id: source.source_lineup_id,
            notes: source.notes,
            members: source
                .members
                .into_iter()
                .map(|member| football_domain::TeamLineupPresetMemberDraft {
                    player_id: member.player_id,
                    position_code: member.position_code,
                    role_code: member.role_code,
                    is_starter: member.is_starter,
                    shirt_number: member.shirt_number,
                    expected_minutes: member.expected_minutes,
                    sequence_no: member.sequence_no,
                    bench_order: member.bench_order,
                    is_captain: member.is_captain,
                    metadata: member.metadata,
                })
                .collect(),
        };
        self.save_team_lineup_preset(&draft).await
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use football_domain::TeamLineupPresetMemberDraft;

    fn member(index: u128, starter: bool) -> TeamLineupPresetMemberDraft {
        TeamLineupPresetMemberDraft {
            player_id: Uuid::from_u128(index + 1),
            position_code: None,
            role_code: None,
            is_starter: starter,
            shirt_number: None,
            expected_minutes: Some(if starter { 90 } else { 20 }),
            sequence_no: index as i16,
            bench_order: if starter { None } else { Some(index as i16) },
            is_captain: index == 0,
            metadata: json!({}),
        }
    }

    fn draft(starter_count: usize, member_count: usize) -> TeamLineupPresetDraft {
        TeamLineupPresetDraft {
            id: None,
            team_id: Uuid::from_u128(10_000),
            name: "主力阵容".to_string(),
            formation_id: None,
            coach_id: None,
            usage_context: "general".to_string(),
            usage_probability: Some(0.6),
            is_default: true,
            source_lineup_id: None,
            notes: None,
            members: (0..member_count)
                .map(|index| member(index as u128, index < starter_count))
                .collect(),
        }
    }

    #[test]
    fn preset_requires_exactly_eleven_starters() {
        assert!(validate_preset(&draft(11, 18)).is_ok());
        assert!(validate_preset(&draft(10, 18)).is_err());
        assert!(validate_preset(&draft(12, 18)).is_err());
    }

    #[test]
    fn preset_rejects_duplicate_players() {
        let mut value = draft(11, 18);
        value.members[17].player_id = value.members[0].player_id;
        assert!(validate_preset(&value).is_err());
    }
}
