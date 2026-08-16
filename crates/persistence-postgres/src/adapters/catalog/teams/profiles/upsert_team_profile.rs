use super::input_policy::{trimmed_optional_text, validate_team_profile};
use super::mapper::map_team_profile;
use super::row::TeamProfileRow;
use crate::{PersistenceResult, PostgresStore};
use football_domain::{TeamProfileDraft, TeamProfileRecord};
use serde_json::json;
use uuid::Uuid;

impl PostgresStore {
    pub async fn upsert_team_profile(
        &self,
        team_id: Uuid,
        draft: &TeamProfileDraft,
    ) -> PersistenceResult<TeamProfileRecord> {
        validate_team_profile(draft)?;
        let mut tx = self.pool.begin().await?;
        let row = sqlx::query_as::<_, TeamProfileRow>(
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
        .bind(trimmed_optional_text(&draft.short_name))
        .bind(draft.team_type.trim())
        .bind(draft.founded_year)
        .bind(trimmed_optional_text(&draft.city))
        .bind(trimmed_optional_text(&draft.stadium))
        .bind(None::<&str>)
        .bind(trimmed_optional_text(&draft.default_formation))
        .bind(draft.tactical_style.trim())
        .bind(draft.attack_rating)
        .bind(draft.midfield_rating)
        .bind(draft.defence_rating)
        .bind(draft.goalkeeper_rating)
        .bind(draft.reputation)
        .bind(draft.data_confidence)
        .bind(trimmed_optional_text(&draft.notes))
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
        Ok(map_team_profile(row))
    }
}
