use super::{
    invalid,
    packet::{Packet, COMPETITION, SEASON},
};
use crate::PersistenceResult;
use sha2::{Digest, Sha256};
use sqlx::PgConnection;
use uuid::Uuid;

pub(super) const COMPETITION_CODE: &str = "EPL-CALIBRATION-RESEARCH";

pub(super) fn identity(kind: &str, key: impl std::fmt::Display) -> Uuid {
    let hash = Sha256::digest(format!("football-calibration-research/fpl/{kind}/{key}"));
    let mut bytes = [0; 16];
    bytes.copy_from_slice(&hash[..16]);
    bytes[6] = (bytes[6] & 0x0f) | 0x80;
    bytes[8] = (bytes[8] & 0x3f) | 0x80;
    Uuid::from_bytes(bytes)
}

pub(super) fn team_id(key: u32) -> Uuid {
    identity("team/2026-27", key)
}

pub(super) fn round_id(week: u8) -> Uuid {
    identity("round/2026-27", week)
}

pub(super) struct League {
    pub competition: Uuid,
    pub season: Uuid,
    pub stage: Uuid,
}

impl League {
    pub(super) async fn bind(db: &mut PgConnection, packet: &Packet) -> PersistenceResult<Self> {
        let competition = identity("competition", COMPETITION_CODE);
        let season = identity("season", SEASON);
        let stage = identity("stage", SEASON);

        sqlx::query(
            "INSERT INTO football.competitions (id, code, name, timezone, competition_kind) \
                    VALUES ($1, $2, $3, 'Europe/London', 'league') ON CONFLICT (id) DO NOTHING",
        )
        .bind(competition)
        .bind(COMPETITION_CODE)
        .bind(COMPETITION)
        .execute(&mut *db)
        .await?;
        sqlx::query(
            "INSERT INTO football.seasons (id, competition_id, name, status) \
                    VALUES ($1, $2, $3, 'active') ON CONFLICT (id) DO NOTHING",
        )
        .bind(season)
        .bind(competition)
        .bind(SEASON)
        .execute(&mut *db)
        .await?;
        sqlx::query(
            "INSERT INTO football.competition_stages (id, season_id, code, name, stage_kind) \
                    VALUES ($1, $2, 'LEAGUE', $3, 'league') ON CONFLICT (id) DO NOTHING",
        )
        .bind(stage)
        .bind(season)
        .bind(COMPETITION)
        .execute(&mut *db)
        .await?;
        let bound: bool = sqlx::query_scalar(
            "SELECT EXISTS (SELECT 1 FROM football.competitions c \
             JOIN football.seasons s ON s.competition_id = c.id \
             JOIN football.competition_stages st ON st.season_id = s.id \
             WHERE c.id = $1 AND c.code = $2 AND c.name = $3 AND c.competition_kind = 'league' \
             AND c.is_active AND c.timezone = 'Europe/London' \
             AND s.id = $4 AND s.name = $5 AND s.status = 'active' \
             AND st.id = $6 AND st.code = 'LEAGUE' AND st.stage_kind = 'league')",
        )
        .bind(competition)
        .bind(COMPETITION_CODE)
        .bind(COMPETITION)
        .bind(season)
        .bind(SEASON)
        .bind(stage)
        .fetch_one(&mut *db)
        .await?;
        if !bound {
            return Err(invalid("独立研究库的赛事、赛季或联赛阶段身份冲突"));
        }

        for (club, name) in &packet.clubs {
            bind_team(db, season, *club, name).await?;
        }
        let weeks: std::collections::BTreeSet<_> = packet
            .history
            .iter()
            .map(|row| row.fixture.matchweek)
            .chain(packet.scheduled.iter().map(|row| row.matchweek))
            .collect();
        for week in weeks {
            bind_round(db, stage, week).await?;
        }
        Ok(Self {
            competition,
            season,
            stage,
        })
    }
}

async fn bind_team(
    db: &mut PgConnection,
    season: Uuid,
    club: u32,
    name: &str,
) -> PersistenceResult<()> {
    let id = team_id(club);
    let source = serde_json::json!({"fpl_club_id": club, "fpl_season": SEASON});
    let normalized = name.to_lowercase();
    sqlx::query(
        "INSERT INTO football.teams (id, canonical_name, normalized_name, metadata) \
                 VALUES ($1, $2, $3, $4) ON CONFLICT (id) DO NOTHING",
    )
    .bind(id)
    .bind(name)
    .bind(&normalized)
    .bind(&source)
    .execute(&mut *db)
    .await?;
    let matches: bool = sqlx::query_scalar(
        "SELECT EXISTS (SELECT 1 FROM football.teams WHERE id = $1 AND canonical_name = $2 \
         AND normalized_name = $3 AND is_active AND metadata @> $4::jsonb)",
    )
    .bind(id)
    .bind(name)
    .bind(&normalized)
    .bind(&source)
    .fetch_one(&mut *db)
    .await?;
    if !matches {
        return Err(invalid(format!("FPL 球队 {club} 的既有身份冲突")));
    }
    sqlx::query(
        "INSERT INTO football.team_season_memberships (id, team_id, season_id) \
                 VALUES ($1, $2, $3) ON CONFLICT (team_id, season_id) DO NOTHING",
    )
    .bind(identity("team-membership/2026-27", club))
    .bind(id)
    .bind(season)
    .execute(&mut *db)
    .await?;
    let registered: bool = sqlx::query_scalar(
        "SELECT EXISTS (SELECT 1 FROM football.team_season_memberships \
         WHERE team_id = $1 AND season_id = $2 AND registration_status = 'registered')",
    )
    .bind(id)
    .bind(season)
    .fetch_one(db)
    .await?;
    if !registered {
        return Err(invalid(format!("FPL 球队 {club} 的参赛状态冲突")));
    }
    Ok(())
}

async fn bind_round(db: &mut PgConnection, stage: Uuid, week: u8) -> PersistenceResult<()> {
    let id = round_id(week);
    let code = format!("MW{week}");
    sqlx::query(
        "INSERT INTO football.rounds (id, stage_id, code, name, sequence_no) \
                 VALUES ($1, $2, $3, $4, $5) ON CONFLICT (id) DO NOTHING",
    )
    .bind(id)
    .bind(stage)
    .bind(&code)
    .bind(format!("Matchweek {week}"))
    .bind(i32::from(week))
    .execute(&mut *db)
    .await?;
    let matches: bool = sqlx::query_scalar(
        "SELECT EXISTS (SELECT 1 FROM football.rounds WHERE id = $1 AND stage_id = $2 AND code = $3 AND sequence_no = $4)",
    ).bind(id).bind(stage).bind(&code).bind(i32::from(week)).fetch_one(db).await?;
    if !matches {
        return Err(invalid(format!("FPL 第 {week} 轮身份冲突")));
    }
    Ok(())
}
