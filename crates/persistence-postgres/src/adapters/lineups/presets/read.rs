use crate::{PersistenceError, PersistenceResult, PostgresStore};
use football_domain::{AvailabilityStatus, TeamLineupPresetMemberRecord, TeamLineupPresetRecord};
use serde_json::Value;
use sqlx::Row;
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

impl PostgresStore {
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
                    availability_status: parse_availability(
                        member.try_get("availability_status")?,
                    )?,
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
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn preset_availability_mapping_preserves_null_and_rejects_unknown_codes() {
        assert!(parse_availability(None).unwrap().is_none());
        for status in [
            "available",
            "doubtful",
            "injured",
            "suspended",
            "rested",
            "returning",
            "unavailable",
            "unknown",
        ] {
            assert_eq!(
                parse_availability(Some(status.into()))
                    .unwrap()
                    .unwrap()
                    .as_str(),
                status
            );
        }
        assert!(
            matches!(parse_availability(Some("invalid".into())), Err(PersistenceError::InvalidState(message)) if message == "未知球员可用状态：invalid")
        );
    }
}
