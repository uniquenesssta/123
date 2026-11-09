use super::{
    catalog::{identity, round_id, team_id, League},
    invalid,
    packet::{Fixture, HistoricalResult, Packet},
};
use crate::{PersistenceResult, PostgresStore};
use chrono::{DateTime, Utc};
use serde_json::{json, Value};
use sqlx::{FromRow, PgConnection};
use uuid::Uuid;

const IMPORT_TYPE: &str = "official_league_calibration_history";

#[derive(FromRow)]
struct ExistingFixture {
    id: Uuid,
    competition_id: Option<Uuid>,
    season_id: Option<Uuid>,
    stage_id: Option<Uuid>,
    round_id: Option<Uuid>,
    home_team_id: Option<Uuid>,
    away_team_id: Option<Uuid>,
    kickoff_time: DateTime<Utc>,
    status: String,
}

pub(super) async fn import(store: &PostgresStore, packet: Packet) -> PersistenceResult<Value> {
    let mut tx = store.pool.begin().await?;
    // Serialize overlapping packets and leave no source, batch, or fixture behind on conflict.
    sqlx::query("SELECT pg_advisory_xact_lock(714118, 202627)")
        .execute(&mut *tx)
        .await?;
    let started_at: DateTime<Utc> = sqlx::query_scalar("SELECT clock_timestamp()")
        .fetch_one(&mut *tx)
        .await?;
    if packet.verified_at > started_at {
        return Err(invalid("赛果核对时间晚于数据库当前时间"));
    }
    let league = League::bind(&mut tx, &packet).await?;
    let source_id = record_source(&mut tx, &packet).await?;
    let mut fixtures_inserted = 0i64;
    let mut results_inserted = 0i64;
    for outcome in &packet.history {
        let (id, inserted, status) =
            bind_fixture(&mut tx, &league, &outcome.fixture, true, started_at).await?;
        fixtures_inserted += i64::from(inserted);
        results_inserted +=
            i64::from(bind_result(&mut tx, &packet, outcome, id, source_id, &status).await?);
    }
    for fixture in &packet.scheduled {
        let (_, inserted, _) = bind_fixture(&mut tx, &league, fixture, false, started_at).await?;
        fixtures_inserted += i64::from(inserted);
    }
    let batch_id = identity("batch", &packet.digest);
    sqlx::query(
        "INSERT INTO catalog.import_batches (id, source_document_id, import_type, status, \
         inserted_count, skipped_count, started_at, finished_at, metadata) \
         VALUES ($1, $2, $3, 'succeeded', $4, $5, $6, clock_timestamp(), $7) ON CONFLICT (id) DO NOTHING",
    )
    .bind(batch_id).bind(source_id).bind(IMPORT_TYPE).bind(results_inserted)
    .bind(packet.history.len() as i64 - results_inserted).bind(started_at)
    .bind(json!({"fixtures_inserted": fixtures_inserted, "season_id": league.season, "usage": "calibration_history_only"}))
    .execute(&mut *tx).await?;
    tx.commit().await?;
    Ok(json!({
        "status": "IMPORTED", "packet_sha256": packet.digest, "batch_id": batch_id, "source_document_id": source_id,
        "competition_id": league.competition, "season_id": league.season,
        "clubs": packet.clubs.len(), "historical_results": packet.history.len(), "scheduled_fixtures": packet.scheduled.len(),
        "fixtures_inserted": fixtures_inserted, "results_inserted": results_inserted,
        "scheduled_match_ids": packet.scheduled.iter().map(|row| (row.fixture_id, match_id(row.fixture_id)))
            .collect::<std::collections::BTreeMap<_, _>>(),
    }))
}

fn match_id(fixture: u32) -> Uuid {
    identity("match/2026-27", fixture)
}

async fn bind_fixture(
    db: &mut PgConnection,
    league: &League,
    fixture: &Fixture,
    historical: bool,
    started_at: DateTime<Utc>,
) -> PersistenceResult<(Uuid, bool, String)> {
    let key = format!("FPL-EPL-2026-27-{}", fixture.fixture_id);
    let id = match_id(fixture.fixture_id);
    let round = round_id(fixture.matchweek);
    let home = team_id(fixture.home_club_id);
    let away = team_id(fixture.away_club_id);
    let initial_status = if historical { "finished" } else { "scheduled" };
    let inserted = if historical || fixture.kickoff_utc > started_at {
        sqlx::query(
            "INSERT INTO football.matches (id, external_key, competition_id, season_id, stage_id, round_id, \
             home_team_id, away_team_id, kickoff_time, status) \
             VALUES ($1,$2,$3,$4,$5,$6,$7,$8,$9,$10) ON CONFLICT (external_key) DO NOTHING",
        )
        .bind(id).bind(&key).bind(league.competition).bind(league.season).bind(league.stage).bind(round)
        .bind(home).bind(away).bind(fixture.kickoff_utc).bind(initial_status)
        .execute(&mut *db).await?.rows_affected() == 1
    } else {
        false
    };
    let row: ExistingFixture = sqlx::query_as(
        "SELECT id, competition_id, season_id, stage_id, round_id, home_team_id, away_team_id, kickoff_time, status \
         FROM football.matches WHERE external_key = $1 FOR UPDATE",
    ).bind(&key).fetch_optional(&mut *db).await?
        .ok_or_else(|| invalid(format!("{key} 已开球，不能补写为未来赛程")))?;
    if row.id != id
        || row.competition_id != Some(league.competition)
        || row.season_id != Some(league.season)
        || row.stage_id != Some(league.stage)
        || row.round_id != Some(round)
        || row.home_team_id != Some(home)
        || row.away_team_id != Some(away)
        || row.kickoff_time != fixture.kickoff_utc
        || !matches!(row.status.as_str(), "finished" | "scheduled")
    {
        return Err(invalid(format!("{key} 与既有比赛身份或赛程冲突")));
    }
    Ok((id, inserted, row.status))
}

async fn bind_result(
    db: &mut PgConnection,
    packet: &Packet,
    outcome: &HistoricalResult,
    id: Uuid,
    source_id: Uuid,
    status: &str,
) -> PersistenceResult<bool> {
    let known_at = outcome.known_at(packet.source.observed_at);
    let metadata = json!({
        "intake": IMPORT_TYPE, "fpl_fixture_id": outcome.fixture.fixture_id, "feed_finished": outcome.feed_finished,
        "feed_finished_provisional": outcome.feed_finished_provisional,
        "official_report": outcome.corroboration.as_ref().map(|report| json!({"url": report.url, "verified_at": report.verified_at})),
    });
    let added = sqlx::query(
        "INSERT INTO football.match_results (match_id, home_goals_90, away_goals_90, finalized_at, created_at, source_document_id, metadata) \
         VALUES ($1, $2, $3, $4, clock_timestamp(), $5, $6) ON CONFLICT (match_id) DO NOTHING",
    ).bind(id).bind(outcome.home_goals_90).bind(outcome.away_goals_90).bind(known_at).bind(source_id).bind(metadata)
        .execute(&mut *db).await?.rows_affected() == 1;
    if !added {
        let existing: bool = sqlx::query_scalar(
            "SELECT EXISTS (SELECT 1 FROM football.match_results WHERE match_id = $1 AND home_goals_90 = $2 \
             AND away_goals_90 = $3 AND source_document_id IS NOT NULL AND metadata->>'intake' = $4)",
        ).bind(id).bind(outcome.home_goals_90).bind(outcome.away_goals_90).bind(IMPORT_TYPE).fetch_one(&mut *db).await?;
        if !existing || status != "finished" {
            return Err(invalid(format!(
                "FPL 比赛 {} 与已保存赛果或来源冲突",
                outcome.fixture.fixture_id
            )));
        }
    } else if status == "scheduled" {
        sqlx::query("UPDATE football.matches SET status = 'finished', updated_at = clock_timestamp() WHERE id = $1")
            .bind(id).execute(db).await?;
    }
    Ok(added)
}

async fn record_source(db: &mut PgConnection, packet: &Packet) -> PersistenceResult<Uuid> {
    let id = identity("source-document", &packet.digest);
    sqlx::query(
        "INSERT INTO catalog.source_documents (id, source_type, source_uri, content_sha256, accessed_at, metadata) \
         VALUES ($1,$2,$3,$4,$5,$6) ON CONFLICT (id) DO NOTHING",
    ).bind(id).bind(IMPORT_TYPE).bind(&packet.source.url).bind(&packet.digest).bind(packet.source.observed_at)
        .bind(json!({"fixture_payload_sha256": packet.source.sha256, "verified_at": packet.verified_at}))
        .execute(&mut *db).await?;
    let existing: bool = sqlx::query_scalar(
        "SELECT EXISTS (SELECT 1 FROM catalog.source_documents WHERE id = $1 AND source_type = $2 AND content_sha256 = $3)",
    ).bind(id).bind(IMPORT_TYPE).bind(&packet.digest).fetch_one(db).await?;
    if !existing {
        return Err(invalid("赛果制品来源身份冲突"));
    }
    Ok(id)
}
