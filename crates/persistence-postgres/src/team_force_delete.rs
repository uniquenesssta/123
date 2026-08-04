use crate::{PersistenceError, PersistenceResult, PostgresStore};
use football_domain::{
    EntityReferenceCount, TeamForceDeletePreview, TeamForceDeleteRequest, TeamForceDeleteResult,
};
use serde_json::json;
use sqlx::{Postgres, Row, Transaction};
use std::collections::BTreeMap;
use uuid::Uuid;

impl PostgresStore {
    pub async fn preview_force_delete_team(
        &self,
        team_id: Uuid,
    ) -> PersistenceResult<TeamForceDeletePreview> {
        let mut tx = self.pool.begin().await?;
        let label = lock_team(&mut tx, team_id).await?;
        prepare_force_delete_targets(&mut tx, team_id, &label).await?;
        let references = force_delete_counts(&mut tx, team_id).await?;
        let total_rows = references.iter().map(|item| item.count.max(0) as u64).sum();
        tx.rollback().await?;

        Ok(TeamForceDeletePreview {
            team_id,
            label: label.clone(),
            confirmation_text: label,
            total_rows,
            references,
            warning: "该操作会永久删除球队、关联球员与教练、相关比赛、P4 快照与运行、评分、动态状态、导入批次及可追溯历史，无法撤销。"
                .to_string(),
        })
    }

    pub async fn force_delete_team(
        &self,
        request: &TeamForceDeleteRequest,
    ) -> PersistenceResult<TeamForceDeleteResult> {
        let mut tx = self.pool.begin().await?;
        let label = lock_team(&mut tx, request.team_id).await?;
        if request.confirmation_text.trim() != label {
            return Err(PersistenceError::InvalidState(format!(
                "确认文字不匹配；请输入完整球队名称：{label}"
            )));
        }

        prepare_force_delete_targets(&mut tx, request.team_id, &label).await?;
        let deleted_match_ids = temp_ids(&mut tx, "purge_matches").await?;
        let deleted_player_ids = temp_ids(&mut tx, "purge_players").await?;
        let deleted_coach_ids = temp_ids(&mut tx, "purge_coaches").await?;
        let deleted_import_batch_ids = temp_ids(&mut tx, "purge_import_batches").await?;
        let deleted_counts = force_delete_counts(&mut tx, request.team_id)
            .await?
            .into_iter()
            .map(|item| (item.relation, item.count.max(0) as u64))
            .collect::<BTreeMap<_, _>>();

        sqlx::query_scalar::<_, String>("SELECT set_config('football.force_purge', 'on', true)")
            .fetch_one(&mut *tx)
            .await?;

        execute_force_delete(&mut tx, request.team_id).await?;

        sqlx::query(
            r#"
            INSERT INTO audit.events (id, event_type, entity_type, entity_id, payload)
            VALUES ($1, 'team_force_deleted', 'team_purge', $2, $3)
            "#,
        )
        .bind(Uuid::new_v4())
        .bind(request.team_id.to_string())
        .bind(json!({
            "team_name": label,
            "deleted_counts": &deleted_counts,
            "deleted_match_ids": &deleted_match_ids,
            "deleted_player_ids": &deleted_player_ids,
            "deleted_coach_ids": &deleted_coach_ids,
            "deleted_import_batch_ids": &deleted_import_batch_ids,
        }))
        .execute(&mut *tx)
        .await?;

        tx.commit().await?;
        Ok(TeamForceDeleteResult {
            team_id: request.team_id,
            label,
            deleted_match_ids,
            deleted_player_ids,
            deleted_coach_ids,
            deleted_import_batch_ids,
            deleted_counts,
        })
    }
}

async fn lock_team(tx: &mut Transaction<'_, Postgres>, team_id: Uuid) -> PersistenceResult<String> {
    sqlx::query_scalar::<_, String>(
        "SELECT canonical_name FROM football.teams WHERE id=$1 FOR UPDATE",
    )
    .bind(team_id)
    .fetch_optional(&mut **tx)
    .await?
    .ok_or_else(|| PersistenceError::InvalidState("球队不存在或已经被删除".to_string()))
}

async fn prepare_force_delete_targets(
    tx: &mut Transaction<'_, Postgres>,
    team_id: Uuid,
    label: &str,
) -> PersistenceResult<()> {
    sqlx::raw_sql(
        r#"
        CREATE TEMP TABLE purge_matches(id uuid PRIMARY KEY) ON COMMIT DROP;
        CREATE TEMP TABLE purge_players(id uuid PRIMARY KEY) ON COMMIT DROP;
        CREATE TEMP TABLE purge_coaches(id uuid PRIMARY KEY) ON COMMIT DROP;
        CREATE TEMP TABLE purge_snapshots(id uuid PRIMARY KEY) ON COMMIT DROP;
        CREATE TEMP TABLE purge_model_runs(id uuid PRIMARY KEY) ON COMMIT DROP;
        CREATE TEMP TABLE purge_research_runs(id uuid PRIMARY KEY) ON COMMIT DROP;
        CREATE TEMP TABLE purge_conflicts(id uuid PRIMARY KEY) ON COMMIT DROP;
        CREATE TEMP TABLE purge_evidence(id uuid PRIMARY KEY) ON COMMIT DROP;
        CREATE TEMP TABLE purge_routes(id uuid PRIMARY KEY) ON COMMIT DROP;
        CREATE TEMP TABLE purge_tasks(id uuid PRIMARY KEY) ON COMMIT DROP;
        CREATE TEMP TABLE purge_jobs(id uuid PRIMARY KEY) ON COMMIT DROP;
        CREATE TEMP TABLE purge_match_reviews(id uuid PRIMARY KEY) ON COMMIT DROP;
        CREATE TEMP TABLE purge_player_reviews(id uuid PRIMARY KEY) ON COMMIT DROP;
        CREATE TEMP TABLE purge_settlements(id uuid PRIMARY KEY) ON COMMIT DROP;
        CREATE TEMP TABLE purge_candidates(id uuid PRIMARY KEY) ON COMMIT DROP;
        CREATE TEMP TABLE purge_import_batches(id uuid PRIMARY KEY) ON COMMIT DROP;
        CREATE TEMP TABLE purge_source_documents(id uuid PRIMARY KEY) ON COMMIT DROP;
        CREATE TEMP TABLE purge_ai_sessions(id uuid PRIMARY KEY) ON COMMIT DROP;
        CREATE TEMP TABLE purge_all_entity_ids(id uuid PRIMARY KEY) ON COMMIT DROP;
        "#,
    )
    .execute(&mut **tx)
    .await?;

    sqlx::query(
        r#"
        INSERT INTO purge_players(id)
        SELECT player_id FROM football.player_team_periods WHERE team_id=$1
        UNION SELECT player_id FROM football.player_availability WHERE team_id=$1
        UNION SELECT lp.player_id
              FROM football.lineup_players lp
              JOIN football.lineups lineup ON lineup.id=lp.lineup_id
              WHERE lineup.team_id=$1
        UNION SELECT player_out_id FROM football.substitutions
              WHERE team_id=$1 AND player_out_id IS NOT NULL
        UNION SELECT player_in_id FROM football.substitutions
              WHERE team_id=$1 AND player_in_id IS NOT NULL
        UNION SELECT player_id FROM review.player_match_observations WHERE team_id=$1
        UNION SELECT player_id FROM review.match_events WHERE team_id=$1 AND player_id IS NOT NULL
        UNION SELECT related_player_id FROM review.match_events WHERE team_id=$1 AND related_player_id IS NOT NULL
        UNION SELECT player_id FROM review.player_match_reviews WHERE team_id=$1
        ON CONFLICT DO NOTHING
        "#,
    )
    .bind(team_id)
    .execute(&mut **tx)
    .await?;

    sqlx::query(
        r#"
        INSERT INTO purge_coaches(id)
        SELECT coach_id FROM football.team_coach_periods WHERE team_id=$1
        UNION SELECT coach_id FROM feature.formation_usage_observations
              WHERE team_id=$1 AND coach_id IS NOT NULL
        UNION SELECT coach_id FROM feature.team_tactical_observations
              WHERE team_id=$1 AND coach_id IS NOT NULL
        UNION SELECT coach_id FROM football.lineups
              WHERE team_id=$1 AND coach_id IS NOT NULL
        ON CONFLICT DO NOTHING
        "#,
    )
    .bind(team_id)
    .execute(&mut **tx)
    .await?;

    sqlx::query(
        r#"
        INSERT INTO purge_matches(id)
        SELECT id FROM football.matches WHERE home_team_id=$1 OR away_team_id=$1
        UNION SELECT match_id FROM football.lineups WHERE team_id=$1
        UNION SELECT match_id FROM football.substitutions WHERE team_id=$1
        UNION SELECT match_id FROM review.player_match_observations WHERE team_id=$1
        UNION SELECT match_id FROM review.match_events WHERE team_id=$1
        UNION SELECT match_review.match_id
              FROM review.team_match_reviews team_review
              JOIN review.match_reviews match_review ON match_review.id=team_review.match_review_id
              WHERE team_review.team_id=$1
        UNION SELECT lineup.match_id
              FROM football.lineup_players player
              JOIN football.lineups lineup ON lineup.id=player.lineup_id
              WHERE player.player_id IN (SELECT id FROM purge_players)
        UNION SELECT match_id FROM football.substitutions
              WHERE player_out_id IN (SELECT id FROM purge_players)
                 OR player_in_id IN (SELECT id FROM purge_players)
        UNION SELECT match_id FROM review.player_match_observations
              WHERE player_id IN (SELECT id FROM purge_players)
        UNION SELECT match_id FROM review.match_events
              WHERE player_id IN (SELECT id FROM purge_players)
                 OR related_player_id IN (SELECT id FROM purge_players)
        UNION SELECT review.match_id
              FROM review.player_match_reviews player_review
              JOIN review.match_reviews review ON review.id=player_review.match_review_id
              WHERE player_review.player_id IN (SELECT id FROM purge_players)
        UNION SELECT match_id FROM feature.match_player_contributions
              WHERE player_id IN (SELECT id FROM purge_players)
        ON CONFLICT DO NOTHING
        "#,
    )
    .bind(team_id)
    .execute(&mut **tx)
    .await?;

    sqlx::raw_sql(
        r#"
        INSERT INTO purge_snapshots(id)
        SELECT id FROM feature.snapshots
        WHERE match_id IN (SELECT id FROM purge_matches)
        ON CONFLICT DO NOTHING;

        INSERT INTO purge_model_runs(id)
        SELECT id FROM model.runs
        WHERE match_id IN (SELECT id FROM purge_matches)
           OR feature_snapshot_id IN (SELECT id FROM purge_snapshots)
        ON CONFLICT DO NOTHING;

        INSERT INTO purge_research_runs(id)
        SELECT id FROM research.runs
        WHERE match_id IN (SELECT id FROM purge_matches)
        UNION SELECT research_run_id FROM feature.snapshots
              WHERE id IN (SELECT id FROM purge_snapshots)
                AND research_run_id IS NOT NULL
        ON CONFLICT DO NOTHING;

        INSERT INTO purge_match_reviews(id)
        SELECT id FROM review.match_reviews
        WHERE match_id IN (SELECT id FROM purge_matches)
           OR source_run_id IN (SELECT id FROM purge_model_runs)
        ON CONFLICT DO NOTHING;
        "#,
    )
    .execute(&mut **tx)
    .await?;

    sqlx::query(
        r#"
        INSERT INTO purge_player_reviews(id)
        SELECT id FROM review.player_match_reviews
        WHERE match_review_id IN (SELECT id FROM purge_match_reviews)
           OR player_id IN (SELECT id FROM purge_players)
           OR team_id=$1
        ON CONFLICT DO NOTHING
        "#,
    )
    .bind(team_id)
    .execute(&mut **tx)
    .await?;

    sqlx::raw_sql(
        r#"
        INSERT INTO purge_settlements(id)
        SELECT id FROM review.postmatch_settlements
        WHERE match_id IN (SELECT id FROM purge_matches)
           OR match_review_id IN (SELECT id FROM purge_match_reviews)
           OR model_run_id IN (SELECT id FROM purge_model_runs)
           OR feature_snapshot_id IN (SELECT id FROM purge_snapshots)
        ON CONFLICT DO NOTHING;

        INSERT INTO purge_model_runs(id)
        SELECT model_run_id FROM review.postmatch_settlements
        WHERE id IN (SELECT id FROM purge_settlements)
        ON CONFLICT DO NOTHING;

        INSERT INTO purge_snapshots(id)
        SELECT feature_snapshot_id FROM review.postmatch_settlements
        WHERE id IN (SELECT id FROM purge_settlements)
        ON CONFLICT DO NOTHING;

        INSERT INTO purge_match_reviews(id)
        SELECT match_review_id FROM review.postmatch_settlements
        WHERE id IN (SELECT id FROM purge_settlements)
        ON CONFLICT DO NOTHING;

        INSERT INTO purge_tasks(id)
        SELECT id FROM platform.p4_freeze_tasks
        WHERE match_id IN (SELECT id FROM purge_matches)
           OR snapshot_id IN (SELECT id FROM purge_snapshots)
           OR research_run_id IN (SELECT id FROM purge_research_runs)
        ON CONFLICT DO NOTHING;

        INSERT INTO purge_research_runs(id)
        SELECT research_run_id FROM platform.p4_freeze_tasks
        WHERE id IN (SELECT id FROM purge_tasks)
          AND research_run_id IS NOT NULL
        ON CONFLICT DO NOTHING;

        INSERT INTO purge_jobs(id)
        SELECT research_job_id FROM platform.p4_freeze_tasks
        WHERE id IN (SELECT id FROM purge_tasks) AND research_job_id IS NOT NULL
        UNION SELECT freeze_job_id FROM platform.p4_freeze_tasks
        WHERE id IN (SELECT id FROM purge_tasks) AND freeze_job_id IS NOT NULL
        ON CONFLICT DO NOTHING;

        INSERT INTO purge_conflicts(id)
        SELECT id FROM research.evidence_conflicts
        WHERE match_id IN (SELECT id FROM purge_matches)
        ON CONFLICT DO NOTHING;

        INSERT INTO purge_evidence(id)
        SELECT id FROM research.evidence_claims
        WHERE match_id IN (SELECT id FROM purge_matches)
           OR research_run_id IN (SELECT id FROM purge_research_runs)
           OR conflict_group_id IN (SELECT id FROM purge_conflicts)
        UNION SELECT evidence_id FROM review.evidence_scoring_items
              WHERE settlement_id IN (SELECT id FROM purge_settlements)
        ON CONFLICT DO NOTHING;

        INSERT INTO purge_conflicts(id)
        SELECT conflict_group_id FROM research.evidence_claims
        WHERE id IN (SELECT id FROM purge_evidence)
          AND conflict_group_id IS NOT NULL
        ON CONFLICT DO NOTHING;

        INSERT INTO purge_research_runs(id)
        SELECT research_run_id FROM research.evidence_claims
        WHERE id IN (SELECT id FROM purge_evidence)
        ON CONFLICT DO NOTHING;

        INSERT INTO purge_routes(id)
        SELECT id FROM research.evidence_routes
        WHERE match_id IN (SELECT id FROM purge_matches)
           OR research_run_id IN (SELECT id FROM purge_research_runs)
        ON CONFLICT DO NOTHING;

        INSERT INTO purge_tasks(id)
        SELECT id FROM platform.p4_freeze_tasks
        WHERE research_run_id IN (SELECT id FROM purge_research_runs)
        ON CONFLICT DO NOTHING;
        "#,
    )
    .execute(&mut **tx)
    .await?;

    sqlx::query(
        r#"
        INSERT INTO purge_candidates(id)
        SELECT id FROM review.ability_update_candidates
        WHERE player_id IN (SELECT id FROM purge_players)
           OR match_review_id IN (SELECT id FROM purge_match_reviews)
           OR player_match_review_id IN (SELECT id FROM purge_player_reviews)
        ON CONFLICT DO NOTHING
        "#,
    )
    .execute(&mut **tx)
    .await?;

    sqlx::query(
        r#"
        INSERT INTO purge_import_batches(id)
        SELECT DISTINCT row.batch_id
        FROM catalog.import_rows row
        WHERE row.matched_entity_id=$1
           OR row.matched_entity_id IN (SELECT id FROM purge_players)
           OR row.matched_entity_id IN (SELECT id FROM purge_coaches)
           OR row.matched_entity_id IN (SELECT id FROM purge_matches)
           OR row.payload::text LIKE '%' || $1::text || '%'
           OR lower(row.payload::text) LIKE '%' || lower($2) || '%'
        ON CONFLICT DO NOTHING
        "#,
    )
    .bind(team_id)
    .bind(label)
    .execute(&mut **tx)
    .await?;

    sqlx::query(
        r#"
        INSERT INTO purge_source_documents(id)
        SELECT source_document_id FROM catalog.import_batches
        WHERE id IN (SELECT id FROM purge_import_batches)
          AND source_document_id IS NOT NULL
        UNION SELECT source_document_id FROM football.player_positions
              WHERE player_id IN (SELECT id FROM purge_players)
                AND source_document_id IS NOT NULL
        UNION SELECT source_document_id FROM football.player_team_periods
              WHERE (team_id=$1 OR player_id IN (SELECT id FROM purge_players))
                AND source_document_id IS NOT NULL
        UNION SELECT source_document_id FROM football.match_results
              WHERE match_id IN (SELECT id FROM purge_matches)
                AND source_document_id IS NOT NULL
        UNION SELECT source_document_id FROM football.lineups
              WHERE match_id IN (SELECT id FROM purge_matches)
                AND source_document_id IS NOT NULL
        UNION SELECT source_document_id FROM football.player_availability
              WHERE (team_id=$1 OR player_id IN (SELECT id FROM purge_players))
                AND source_document_id IS NOT NULL
        UNION SELECT source_document_id FROM football.substitutions
              WHERE (match_id IN (SELECT id FROM purge_matches) OR team_id=$1)
                AND source_document_id IS NOT NULL
        UNION SELECT source_document_id FROM review.match_events
              WHERE (match_id IN (SELECT id FROM purge_matches) OR team_id=$1
                     OR player_id IN (SELECT id FROM purge_players)
                     OR related_player_id IN (SELECT id FROM purge_players))
                AND source_document_id IS NOT NULL
        UNION SELECT source_document_id FROM feature.player_ability_observations
              WHERE player_id IN (SELECT id FROM purge_players)
                AND source_document_id IS NOT NULL
        UNION SELECT source_document_id FROM feature.player_dynamic_tags
              WHERE (player_id IN (SELECT id FROM purge_players) OR opponent_team_id=$1)
                AND source_document_id IS NOT NULL
        UNION SELECT source_document_id FROM review.player_match_observations
              WHERE (match_id IN (SELECT id FROM purge_matches)
                     OR player_id IN (SELECT id FROM purge_players)
                     OR team_id=$1)
                AND source_document_id IS NOT NULL
        UNION SELECT source_document_id FROM football.team_coach_periods
              WHERE (team_id=$1 OR coach_id IN (SELECT id FROM purge_coaches))
                AND source_document_id IS NOT NULL
        UNION SELECT source_document_id FROM feature.formation_usage_observations
              WHERE (team_id=$1 OR coach_id IN (SELECT id FROM purge_coaches))
                AND source_document_id IS NOT NULL
        UNION SELECT source_document_id FROM research.evidence_claims
              WHERE id IN (SELECT id FROM purge_evidence)
                AND source_document_id IS NOT NULL
        UNION SELECT source_document_id FROM review.evidence_scoring_items
              WHERE settlement_id IN (SELECT id FROM purge_settlements)
                AND source_document_id IS NOT NULL
        ON CONFLICT DO NOTHING
        "#,
    )
    .bind(team_id)
    .execute(&mut **tx)
    .await?;

    sqlx::query("INSERT INTO purge_all_entity_ids(id) VALUES ($1) ON CONFLICT DO NOTHING")
        .bind(team_id)
        .execute(&mut **tx)
        .await?;
    sqlx::raw_sql(
        r#"
        INSERT INTO purge_all_entity_ids SELECT id FROM purge_matches ON CONFLICT DO NOTHING;
        INSERT INTO purge_all_entity_ids SELECT id FROM purge_players ON CONFLICT DO NOTHING;
        INSERT INTO purge_all_entity_ids SELECT id FROM purge_coaches ON CONFLICT DO NOTHING;
        INSERT INTO purge_all_entity_ids SELECT id FROM purge_snapshots ON CONFLICT DO NOTHING;
        INSERT INTO purge_all_entity_ids SELECT id FROM purge_model_runs ON CONFLICT DO NOTHING;
        INSERT INTO purge_all_entity_ids SELECT id FROM purge_research_runs ON CONFLICT DO NOTHING;
        INSERT INTO purge_all_entity_ids SELECT id FROM purge_conflicts ON CONFLICT DO NOTHING;
        INSERT INTO purge_all_entity_ids SELECT id FROM purge_evidence ON CONFLICT DO NOTHING;
        INSERT INTO purge_all_entity_ids SELECT id FROM purge_tasks ON CONFLICT DO NOTHING;
        INSERT INTO purge_all_entity_ids SELECT id FROM purge_match_reviews ON CONFLICT DO NOTHING;
        INSERT INTO purge_all_entity_ids SELECT id FROM purge_settlements ON CONFLICT DO NOTHING;
        INSERT INTO purge_all_entity_ids SELECT id FROM purge_routes ON CONFLICT DO NOTHING;
        INSERT INTO purge_all_entity_ids SELECT id FROM purge_jobs ON CONFLICT DO NOTHING;
        INSERT INTO purge_all_entity_ids SELECT id FROM purge_player_reviews ON CONFLICT DO NOTHING;
        INSERT INTO purge_all_entity_ids SELECT id FROM purge_candidates ON CONFLICT DO NOTHING;
        INSERT INTO purge_all_entity_ids SELECT id FROM purge_import_batches ON CONFLICT DO NOTHING;
        "#,
    )
    .execute(&mut **tx)
    .await?;

    sqlx::raw_sql(
        r#"
        INSERT INTO purge_jobs(id)
        SELECT job.id FROM platform.jobs job
        WHERE EXISTS (
            SELECT 1 FROM purge_all_entity_ids entity
            WHERE job.payload::text LIKE '%' || entity.id::text || '%'
               OR COALESCE(job.result::text, '') LIKE '%' || entity.id::text || '%'
        )
        ON CONFLICT DO NOTHING;

        INSERT INTO purge_ai_sessions(id)
        SELECT session.id FROM ai_workspace.sessions session
        WHERE session.match_id IN (SELECT id FROM purge_matches)
           OR EXISTS (
               SELECT 1 FROM purge_all_entity_ids entity
               WHERE session.metadata::text LIKE '%' || entity.id::text || '%'
           )
        ON CONFLICT DO NOTHING;

        INSERT INTO purge_all_entity_ids SELECT id FROM purge_jobs ON CONFLICT DO NOTHING;
        INSERT INTO purge_all_entity_ids SELECT id FROM purge_ai_sessions ON CONFLICT DO NOTHING;
        "#,
    )
    .execute(&mut **tx)
    .await?;

    Ok(())
}

async fn execute_force_delete(
    tx: &mut Transaction<'_, Postgres>,
    team_id: Uuid,
) -> PersistenceResult<()> {
    sqlx::raw_sql(
        r#"
        DELETE FROM review.evidence_scoring_decisions
        WHERE item_id IN (
            SELECT id FROM review.evidence_scoring_items
            WHERE settlement_id IN (SELECT id FROM purge_settlements)
               OR evidence_id IN (SELECT id FROM purge_evidence)
        );
        DELETE FROM review.evidence_scoring_items
        WHERE settlement_id IN (SELECT id FROM purge_settlements)
           OR evidence_id IN (SELECT id FROM purge_evidence);
        DELETE FROM analytics.evaluation_samples
        WHERE settlement_id IN (SELECT id FROM purge_settlements)
           OR run_id IN (SELECT id FROM purge_model_runs)
           OR review_id IN (SELECT id FROM purge_match_reviews);
        DELETE FROM review.postmatch_settlements
        WHERE id IN (SELECT id FROM purge_settlements);

        DELETE FROM research.manual_route_overrides
        WHERE task_id IN (SELECT id FROM purge_tasks)
           OR research_run_id IN (SELECT id FROM purge_research_runs)
           OR conflict_id IN (SELECT id FROM purge_conflicts)
           OR original_route_id IN (SELECT id FROM purge_routes);
        DELETE FROM platform.p4_freeze_tasks WHERE id IN (SELECT id FROM purge_tasks);
        DELETE FROM platform.jobs WHERE id IN (SELECT id FROM purge_jobs);

        DELETE FROM feature.snapshot_evidence
        WHERE snapshot_id IN (SELECT id FROM purge_snapshots)
           OR evidence_id IN (SELECT id FROM purge_evidence);
        DELETE FROM model.snapshot_probabilities
        WHERE snapshot_id IN (SELECT id FROM purge_snapshots)
           OR model_run_id IN (SELECT id FROM purge_model_runs);
        DELETE FROM feature.snapshot_features
        WHERE snapshot_id IN (SELECT id FROM purge_snapshots);

        DELETE FROM research.conflict_evaluations
        WHERE conflict_id IN (SELECT id FROM purge_conflicts)
           OR research_run_id IN (SELECT id FROM purge_research_runs)
           OR match_id IN (SELECT id FROM purge_matches);
        DELETE FROM research.evidence_conflict_members
        WHERE conflict_id IN (SELECT id FROM purge_conflicts)
           OR evidence_id IN (SELECT id FROM purge_evidence);
        DELETE FROM research.evidence_conflict_events
        WHERE conflict_id IN (SELECT id FROM purge_conflicts);
        DELETE FROM research.entity_resolutions
        WHERE research_run_id IN (SELECT id FROM purge_research_runs)
           OR match_id IN (SELECT id FROM purge_matches);
        DELETE FROM research.time_audits
        WHERE research_run_id IN (SELECT id FROM purge_research_runs)
           OR match_id IN (SELECT id FROM purge_matches);
        DELETE FROM research.evidence_routes
        WHERE id IN (SELECT id FROM purge_routes)
           OR research_run_id IN (SELECT id FROM purge_research_runs)
           OR match_id IN (SELECT id FROM purge_matches);
        DELETE FROM research.evidence_claims WHERE id IN (SELECT id FROM purge_evidence);
        DELETE FROM research.evidence_conflicts WHERE id IN (SELECT id FROM purge_conflicts);

        DELETE FROM analytics.ai_suggestions
        WHERE linked_candidate_id IN (SELECT id FROM purge_candidates)
           OR EXISTS (
               SELECT 1 FROM purge_all_entity_ids entity
               WHERE analytics.ai_suggestions.scope::text LIKE '%' || entity.id::text || '%'
                  OR analytics.ai_suggestions.payload::text LIKE '%' || entity.id::text || '%'
                  OR analytics.ai_suggestions.evidence::text LIKE '%' || entity.id::text || '%'
           );
        DELETE FROM review.ability_update_decisions
        WHERE candidate_id IN (SELECT id FROM purge_candidates);
        DELETE FROM review.ability_update_candidates
        WHERE id IN (SELECT id FROM purge_candidates);
        DELETE FROM review.player_match_reviews
        WHERE id IN (SELECT id FROM purge_player_reviews)
           OR player_id IN (SELECT id FROM purge_players);
        DELETE FROM review.team_match_reviews
        WHERE match_review_id IN (SELECT id FROM purge_match_reviews);
        DELETE FROM review.match_reviews WHERE id IN (SELECT id FROM purge_match_reviews);

        DELETE FROM model.runs WHERE id IN (SELECT id FROM purge_model_runs);
        DELETE FROM feature.snapshots WHERE id IN (SELECT id FROM purge_snapshots);
        DELETE FROM research.runs WHERE id IN (SELECT id FROM purge_research_runs);
        DELETE FROM ai_workspace.sessions WHERE id IN (SELECT id FROM purge_ai_sessions);

        DELETE FROM analytics.data_quality_findings
        WHERE entity_id IN (SELECT id::text FROM purge_all_entity_ids)
           OR EXISTS (
               SELECT 1 FROM purge_all_entity_ids entity
               WHERE analytics.data_quality_findings.evidence::text LIKE '%' || entity.id::text || '%'
           );
        DELETE FROM review.player_match_observations
        WHERE match_id IN (SELECT id FROM purge_matches)
           OR player_id IN (SELECT id FROM purge_players);
        DELETE FROM feature.match_player_contributions
        WHERE match_id IN (SELECT id FROM purge_matches)
           OR player_id IN (SELECT id FROM purge_players);
        UPDATE football.lineups
        SET coach_id=NULL
        WHERE coach_id IN (SELECT id FROM purge_coaches)
          AND match_id NOT IN (SELECT id FROM purge_matches);
        DELETE FROM football.matches WHERE id IN (SELECT id FROM purge_matches);

        DELETE FROM feature.formation_usage_observations
        WHERE team_id IN (SELECT id FROM purge_all_entity_ids)
           OR coach_id IN (SELECT id FROM purge_coaches);
        DELETE FROM feature.team_tactical_observations
        WHERE team_id IN (SELECT id FROM purge_all_entity_ids)
           OR coach_id IN (SELECT id FROM purge_coaches);
        DELETE FROM feature.team_ability_observations
        WHERE team_id IN (SELECT id FROM purge_all_entity_ids);
        DELETE FROM football.player_availability
        WHERE team_id IN (SELECT id FROM purge_all_entity_ids)
           OR player_id IN (SELECT id FROM purge_players);
        DELETE FROM football.player_team_periods
        WHERE team_id IN (SELECT id FROM purge_all_entity_ids)
           OR player_id IN (SELECT id FROM purge_players);
        DELETE FROM football.team_coach_periods
        WHERE team_id IN (SELECT id FROM purge_all_entity_ids)
           OR coach_id IN (SELECT id FROM purge_coaches);
        DELETE FROM football.team_season_memberships
        WHERE team_id IN (SELECT id FROM purge_all_entity_ids);
        UPDATE football.team_lineup_presets
        SET coach_id=NULL, updated_at=now()
        WHERE coach_id IN (SELECT id FROM purge_coaches)
          AND team_id NOT IN (SELECT id FROM purge_all_entity_ids);
        DELETE FROM football.team_lineup_preset_members
        WHERE player_id IN (SELECT id FROM purge_players)
           OR preset_id IN (
               SELECT id FROM football.team_lineup_presets
               WHERE team_id IN (SELECT id FROM purge_all_entity_ids)
           );
        DELETE FROM football.team_lineup_presets
        WHERE team_id IN (SELECT id FROM purge_all_entity_ids);
        DELETE FROM feature.player_dynamic_tags
        WHERE player_id IN (SELECT id FROM purge_players)
           OR opponent_team_id IN (SELECT id FROM purge_all_entity_ids);
        DELETE FROM feature.player_ability_snapshots
        WHERE player_id IN (SELECT id FROM purge_players);
        DELETE FROM feature.player_ability_observations
        WHERE player_id IN (SELECT id FROM purge_players);
        DELETE FROM football.external_entity_ids
        WHERE (entity_type='team' AND entity_id IN (SELECT id FROM purge_all_entity_ids))
           OR (entity_type='player' AND entity_id IN (SELECT id FROM purge_players))
           OR (entity_type='coach' AND entity_id IN (SELECT id FROM purge_coaches))
           OR (entity_type='match' AND entity_id IN (SELECT id FROM purge_matches));

        DELETE FROM catalog.bulk_import_staging
        WHERE batch_id IN (SELECT id FROM purge_import_batches);
        DELETE FROM catalog.bulk_import_runs
        WHERE batch_id IN (SELECT id FROM purge_import_batches);
        DELETE FROM catalog.import_batches
        WHERE id IN (SELECT id FROM purge_import_batches);

        DELETE FROM audit.events
        WHERE entity_id IN (SELECT id::text FROM purge_all_entity_ids)
           OR EXISTS (
               SELECT 1 FROM purge_all_entity_ids entity
               WHERE audit.events.payload::text LIKE '%' || entity.id::text || '%'
           );

        DELETE FROM football.players WHERE id IN (SELECT id FROM purge_players);
        DELETE FROM football.coaches WHERE id IN (SELECT id FROM purge_coaches);
        "#,
    )
    .execute(&mut **tx)
    .await?;

    sqlx::query("DELETE FROM football.teams WHERE id=$1")
        .bind(team_id)
        .execute(&mut **tx)
        .await?;

    sqlx::raw_sql(
        r#"
        DELETE FROM catalog.source_documents document
        WHERE document.id IN (SELECT id FROM purge_source_documents)
          AND NOT EXISTS (SELECT 1 FROM catalog.import_batches WHERE source_document_id=document.id)
          AND NOT EXISTS (SELECT 1 FROM football.player_positions WHERE source_document_id=document.id)
          AND NOT EXISTS (SELECT 1 FROM football.player_team_periods WHERE source_document_id=document.id)
          AND NOT EXISTS (SELECT 1 FROM football.match_results WHERE source_document_id=document.id)
          AND NOT EXISTS (SELECT 1 FROM football.lineups WHERE source_document_id=document.id)
          AND NOT EXISTS (SELECT 1 FROM football.player_availability WHERE source_document_id=document.id)
          AND NOT EXISTS (SELECT 1 FROM football.substitutions WHERE source_document_id=document.id)
          AND NOT EXISTS (SELECT 1 FROM review.match_events WHERE source_document_id=document.id)
          AND NOT EXISTS (SELECT 1 FROM feature.player_ability_observations WHERE source_document_id=document.id)
          AND NOT EXISTS (SELECT 1 FROM feature.player_dynamic_tags WHERE source_document_id=document.id)
          AND NOT EXISTS (SELECT 1 FROM model.rule_packages WHERE source_document_id=document.id)
          AND NOT EXISTS (SELECT 1 FROM review.player_match_observations WHERE source_document_id=document.id)
          AND NOT EXISTS (SELECT 1 FROM football.team_coach_periods WHERE source_document_id=document.id)
          AND NOT EXISTS (SELECT 1 FROM feature.formation_usage_observations WHERE source_document_id=document.id)
          AND NOT EXISTS (SELECT 1 FROM research.evidence_claims WHERE source_document_id=document.id)
          AND NOT EXISTS (SELECT 1 FROM review.evidence_scoring_items WHERE source_document_id=document.id);
        "#,
    )
    .execute(&mut **tx)
    .await?;

    Ok(())
}

async fn force_delete_counts(
    tx: &mut Transaction<'_, Postgres>,
    team_id: Uuid,
) -> PersistenceResult<Vec<EntityReferenceCount>> {
    let rows = sqlx::query(
        r#"
        SELECT relation, count FROM (
            SELECT 'teams'::text relation, 1::bigint count
            UNION ALL SELECT 'matches', count(*)::bigint FROM purge_matches
            UNION ALL SELECT 'players', count(*)::bigint FROM purge_players
            UNION ALL SELECT 'coaches', count(*)::bigint FROM purge_coaches
            UNION ALL SELECT 'feature_snapshots', count(*)::bigint FROM purge_snapshots
            UNION ALL SELECT 'model_runs', count(*)::bigint FROM purge_model_runs
            UNION ALL SELECT 'research_runs', count(*)::bigint FROM purge_research_runs
            UNION ALL SELECT 'p4_freeze_tasks', count(*)::bigint FROM purge_tasks
            UNION ALL SELECT 'postmatch_settlements', count(*)::bigint FROM purge_settlements
            UNION ALL SELECT 'match_reviews', count(*)::bigint FROM purge_match_reviews
            UNION ALL SELECT 'ability_update_candidates', count(*)::bigint FROM purge_candidates
            UNION ALL SELECT 'import_batches', count(*)::bigint FROM purge_import_batches
            UNION ALL SELECT 'ai_workspace_sessions', count(*)::bigint FROM purge_ai_sessions
            UNION ALL SELECT 'match_events', count(*)::bigint
                FROM review.match_events
                WHERE match_id IN (SELECT id FROM purge_matches)
                   OR team_id=$1
                   OR player_id IN (SELECT id FROM purge_players)
                   OR related_player_id IN (SELECT id FROM purge_players)
            UNION ALL SELECT 'team_lineup_presets', count(*)::bigint
                FROM football.team_lineup_presets
                WHERE team_id=$1
            UNION ALL SELECT 'team_lineup_preset_members', count(*)::bigint
                FROM football.team_lineup_preset_members
                WHERE preset_id IN (SELECT id FROM football.team_lineup_presets WHERE team_id=$1)
                   OR player_id IN (SELECT id FROM purge_players)
            UNION ALL SELECT 'player_team_periods', count(*)::bigint
                FROM football.player_team_periods
                WHERE team_id=$1 OR player_id IN (SELECT id FROM purge_players)
            UNION ALL SELECT 'player_ability_observations', count(*)::bigint
                FROM feature.player_ability_observations
                WHERE player_id IN (SELECT id FROM purge_players)
            UNION ALL SELECT 'player_dynamic_tags', count(*)::bigint
                FROM feature.player_dynamic_tags
                WHERE player_id IN (SELECT id FROM purge_players) OR opponent_team_id=$1
            UNION ALL SELECT 'formation_usage_observations', count(*)::bigint
                FROM feature.formation_usage_observations
                WHERE team_id=$1 OR coach_id IN (SELECT id FROM purge_coaches)
            UNION ALL SELECT 'team_tactical_observations', count(*)::bigint
                FROM feature.team_tactical_observations
                WHERE team_id=$1 OR coach_id IN (SELECT id FROM purge_coaches)
            UNION ALL SELECT 'team_ability_observations', count(*)::bigint
                FROM feature.team_ability_observations WHERE team_id=$1
        ) summary
        WHERE count > 0
        ORDER BY relation
        "#,
    )
    .bind(team_id)
    .fetch_all(&mut **tx)
    .await?;

    rows.iter()
        .map(|row| {
            Ok(EntityReferenceCount {
                relation: row.try_get("relation")?,
                count: row.try_get("count")?,
            })
        })
        .collect()
}

async fn temp_ids(tx: &mut Transaction<'_, Postgres>, table: &str) -> PersistenceResult<Vec<Uuid>> {
    let sql = format!("SELECT id FROM {table} ORDER BY id");
    Ok(sqlx::query_scalar::<_, Uuid>(&sql)
        .fetch_all(&mut **tx)
        .await?)
}
