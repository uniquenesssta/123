use crate::PersistenceResult;
use football_domain::EntityMatchCandidate;
use sqlx::{PgPool, Row};

pub(super) async fn match_teams_by_name(
    pool: &PgPool,
    normalized_name: &str,
    country_code: Option<&str>,
) -> PersistenceResult<Vec<EntityMatchCandidate>> {
    let rows = sqlx::query(
        r#"
        SELECT DISTINCT team.id, team.canonical_name,
               CASE WHEN team.normalized_name=$1 THEN '正式名称' ELSE '球队别名' END AS reason
        FROM football.teams team
        LEFT JOIN football.team_names alias ON alias.team_id=team.id
        WHERE (team.normalized_name=$1 OR alias.normalized_name=$1)
          AND ($2::text IS NULL OR upper(COALESCE(team.country_code,''))=upper($2))
        ORDER BY team.canonical_name, team.id
        "#,
    )
    .bind(normalized_name)
    .bind(
        country_code
            .map(str::trim)
            .filter(|value| !value.is_empty()),
    )
    .fetch_all(pool)
    .await?;
    candidate_rows(&rows, 0.95)
}

pub(super) async fn match_players_by_name(
    pool: &PgPool,
    normalized_name: &str,
    date_of_birth: Option<chrono::NaiveDate>,
) -> PersistenceResult<Vec<EntityMatchCandidate>> {
    let rows = sqlx::query(
        r#"
        SELECT DISTINCT player.id, player.canonical_name,
               CASE WHEN player.normalized_name=$1 THEN '规范姓名与出生日期' ELSE '球员别名与出生日期' END AS reason
        FROM football.players player
        LEFT JOIN football.player_names alias ON alias.player_id=player.id
        WHERE (player.normalized_name=$1 OR alias.normalized_name=$1)
          AND ($2::date IS NULL OR player.date_of_birth=$2)
        ORDER BY player.canonical_name, player.id
        "#,
    )
    .bind(normalized_name)
    .bind(date_of_birth)
    .fetch_all(pool)
    .await?;
    candidate_rows(&rows, if date_of_birth.is_some() { 1.0 } else { 0.7 })
}

pub(super) async fn match_coaches_by_name(
    pool: &PgPool,
    normalized_name: &str,
    nationality_code: Option<&str>,
) -> PersistenceResult<Vec<EntityMatchCandidate>> {
    let rows = sqlx::query(
        r#"
        SELECT DISTINCT coach.id, coach.canonical_name,
               CASE WHEN coach.normalized_name=$1 THEN '规范姓名与国籍' ELSE '教练别名与国籍' END AS reason
        FROM football.coaches coach
        LEFT JOIN football.coach_names alias ON alias.coach_id=coach.id
        WHERE (coach.normalized_name=$1 OR alias.normalized_name=$1)
          AND ($2::text IS NULL OR upper(COALESCE(coach.nationality_code,''))=upper($2))
        ORDER BY coach.canonical_name, coach.id
        "#,
    )
    .bind(normalized_name)
    .bind(
        nationality_code
            .map(str::trim)
            .filter(|value| !value.is_empty()),
    )
    .fetch_all(pool)
    .await?;
    candidate_rows(&rows, 0.95)
}

fn candidate_rows(
    rows: &[sqlx::postgres::PgRow],
    score: f64,
) -> PersistenceResult<Vec<EntityMatchCandidate>> {
    rows.iter()
        .map(|row| {
            Ok(EntityMatchCandidate {
                id: row.try_get("id")?,
                label: row.try_get("canonical_name")?,
                reason: row.try_get("reason")?,
                score,
            })
        })
        .collect()
}
