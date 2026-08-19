use crate::PersistenceResult;
use sqlx::{Postgres, Transaction};
use uuid::Uuid;

pub(super) async fn execute_force_delete(
    tx: &mut Transaction<'_, Postgres>,
    team_id: Uuid,
) -> PersistenceResult<()> {
    sqlx::query_scalar::<_, String>("SELECT set_config('football.force_purge', 'on', true)")
        .fetch_one(&mut **tx)
        .await?;

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
