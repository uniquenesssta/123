use crate::{PersistenceError, PersistenceResult};
use sqlx::{Postgres, Transaction};
use uuid::Uuid;

pub(super) async fn lock_team(
    tx: &mut Transaction<'_, Postgres>,
    team_id: Uuid,
) -> PersistenceResult<String> {
    sqlx::query_scalar::<_, String>(
        "SELECT canonical_name FROM football.teams WHERE id=$1 FOR UPDATE",
    )
    .bind(team_id)
    .fetch_optional(&mut **tx)
    .await?
    .ok_or_else(|| PersistenceError::InvalidState("球队不存在或已经被删除".to_string()))
}

pub(super) async fn prepare_force_delete_targets(
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

pub(super) async fn temp_ids(
    tx: &mut Transaction<'_, Postgres>,
    table: &str,
) -> PersistenceResult<Vec<Uuid>> {
    let sql = format!("SELECT id FROM {table} ORDER BY id");
    Ok(sqlx::query_scalar::<_, Uuid>(&sql)
        .fetch_all(&mut **tx)
        .await?)
}
