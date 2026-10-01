use super::validation::{validate_preset, verify_membership_in_tx};
use crate::{
    role_resolution::{
        metadata_with_role_resolution, resolve_default_tactical_role_in_tx, resolve_tactical_role,
    },
    write_audit_event,
};
use crate::{PersistenceError, PersistenceResult, PostgresStore};
use chrono::Utc;
use football_domain::{TeamLineupPresetDraft, TeamLineupPresetRecord};
use serde_json::json;
use sqlx::Row;
use uuid::Uuid;

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
            &draft
                .members
                .iter()
                .map(|member| member.player_id)
                .collect::<Vec<_>>(),
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
            .bind(
                draft
                    .notes
                    .as_deref()
                    .map(str::trim)
                    .filter(|value| !value.is_empty()),
            )
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
            .bind(
                draft
                    .notes
                    .as_deref()
                    .map(str::trim)
                    .filter(|value| !value.is_empty()),
            )
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
            let member_metadata = metadata_with_role_resolution(&member.metadata, &role_resolution);
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
