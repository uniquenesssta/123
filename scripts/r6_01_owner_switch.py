from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]


def write(path: str, content: str) -> None:
    target = ROOT / path
    target.parent.mkdir(parents=True, exist_ok=True)
    target.write_text(content.strip() + "\n", encoding="utf-8")


def remove_between(text: str, start: str, end: str) -> str:
    start_index = text.find(start)
    if start_index < 0:
        raise SystemExit(f"missing start marker: {start}")
    end_index = text.find(end, start_index)
    if end_index < 0:
        raise SystemExit(f"missing end marker: {end}")
    return text[:start_index] + text[end_index:]


write(
    "crates/persistence-postgres/src/adapters/catalog/mod.rs",
    r'''
pub(crate) mod teams;
''',
)

write(
    "crates/persistence-postgres/src/adapters/catalog/teams/mod.rs",
    r'''
mod detail;
mod directory;
''',
)

write(
    "crates/persistence-postgres/src/adapters/catalog/teams/directory/mod.rs",
    r'''
mod create_team;
mod list_mapper;
mod list_row;
mod list_team_options;
mod list_teams;
mod name_policy;
mod option_mapper;
mod option_row;
mod update_team;

pub(super) use list_mapper::map_team_list_row;
pub(super) use list_row::TeamListRow;
pub(super) use name_policy::normalize_team_name;
pub(super) use option_mapper::map_team_option_row;
pub(super) use option_row::TeamOptionRow;
''',
)

write(
    "crates/persistence-postgres/src/adapters/catalog/teams/directory/name_policy.rs",
    r'''
pub(super) fn normalize_team_name(value: &str) -> String {
    value
        .trim()
        .to_lowercase()
        .split_whitespace()
        .collect::<Vec<_>>()
        .join(" ")
}
''',
)

write(
    "crates/persistence-postgres/src/adapters/catalog/teams/directory/create_team.rs",
    r'''
use super::super::detail::{map_team_record, TeamRecordRow};
use super::normalize_team_name;
use crate::{PersistenceError, PersistenceResult, PostgresStore};
use football_domain::{TeamDraft, TeamRecord};
use serde_json::json;
use uuid::Uuid;

impl PostgresStore {
    pub async fn create_team(&self, draft: &TeamDraft) -> PersistenceResult<TeamRecord> {
        let canonical_name = draft.canonical_name.trim();
        if canonical_name.is_empty() {
            return Err(PersistenceError::InvalidState(
                "球队名称不能为空".to_string(),
            ));
        }
        let normalized_name = normalize_team_name(canonical_name);
        let id = Uuid::new_v4();
        let mut tx = self.pool.begin().await?;
        let row = sqlx::query_as::<_, TeamRecordRow>(
            r#"
            INSERT INTO football.teams (
                id, canonical_name, normalized_name, country_code, metadata
            ) VALUES ($1, $2, $3, $4, $5)
            RETURNING id, canonical_name, normalized_name, country_code, is_active, created_at
            "#,
        )
        .bind(id)
        .bind(canonical_name)
        .bind(&normalized_name)
        .bind(
            draft
                .country_code
                .as_deref()
                .map(str::trim)
                .filter(|value| !value.is_empty()),
        )
        .bind(&draft.metadata)
        .fetch_one(&mut *tx)
        .await?;
        crate::write_audit_event(
            &mut tx,
            "team_created",
            "team",
            id.to_string(),
            json!({"canonical_name": canonical_name}),
        )
        .await?;
        tx.commit().await?;
        map_team_record(row)
    }
}
''',
)

write(
    "crates/persistence-postgres/src/adapters/catalog/teams/directory/update_team.rs",
    r'''
use super::super::detail::{map_team_record, TeamRecordRow};
use super::normalize_team_name;
use crate::{PersistenceError, PersistenceResult, PostgresStore};
use football_domain::{TeamDraft, TeamRecord};
use serde_json::json;
use uuid::Uuid;

impl PostgresStore {
    pub async fn update_team(
        &self,
        team_id: Uuid,
        draft: &TeamDraft,
    ) -> PersistenceResult<TeamRecord> {
        let canonical_name = draft.canonical_name.trim();
        if canonical_name.is_empty() {
            return Err(PersistenceError::InvalidState(
                "球队名称不能为空".to_string(),
            ));
        }
        let mut tx = self.pool.begin().await?;
        let row = sqlx::query_as::<_, TeamRecordRow>(
            r#"
            UPDATE football.teams
            SET canonical_name = $2, normalized_name = $3, country_code = $4,
                metadata = metadata || $5, updated_at = now()
            WHERE id = $1
            RETURNING id, canonical_name, normalized_name, country_code, is_active, created_at
            "#,
        )
        .bind(team_id)
        .bind(canonical_name)
        .bind(normalize_team_name(canonical_name))
        .bind(
            draft
                .country_code
                .as_deref()
                .map(str::trim)
                .filter(|value| !value.is_empty()),
        )
        .bind(&draft.metadata)
        .fetch_optional(&mut *tx)
        .await?
        .ok_or_else(|| PersistenceError::InvalidState("球队不存在".to_string()))?;
        crate::write_audit_event(
            &mut tx,
            "team_updated",
            "team",
            team_id.to_string(),
            json!({"canonical_name": canonical_name, "source": "manual"}),
        )
        .await?;
        tx.commit().await?;
        map_team_record(row)
    }
}
''',
)

write(
    "crates/persistence-postgres/src/adapters/catalog/teams/directory/option_row.rs",
    r'''
use uuid::Uuid;

#[derive(Debug, sqlx::FromRow)]
pub(super) struct TeamOptionRow {
    pub(super) id: Uuid,
    pub(super) canonical_name: String,
    pub(super) country_code: Option<String>,
    pub(super) team_type: String,
}
''',
)

write(
    "crates/persistence-postgres/src/adapters/catalog/teams/directory/option_mapper.rs",
    r'''
use super::TeamOptionRow;
use football_domain::TeamOption;

pub(super) fn map_team_option_row(row: TeamOptionRow) -> TeamOption {
    TeamOption {
        id: row.id,
        canonical_name: row.canonical_name,
        country_code: row.country_code,
        team_type: row.team_type,
    }
}
''',
)

write(
    "crates/persistence-postgres/src/adapters/catalog/teams/directory/list_team_options.rs",
    r'''
use super::{map_team_option_row, TeamOptionRow};
use crate::{
    name_search::{push_name_search, NameSearch, NameSearchColumns},
    PersistenceResult, PostgresStore,
};
use football_domain::TeamOption;
use sqlx::{Postgres, QueryBuilder};

impl PostgresStore {
    pub async fn list_team_options(
        &self,
        search: Option<&str>,
        limit: u32,
    ) -> PersistenceResult<Vec<TeamOption>> {
        let safe_limit = limit.clamp(1, 500) as i64;
        let mut builder: QueryBuilder<Postgres> = QueryBuilder::new(
            r#"
            SELECT football.teams.id, football.teams.canonical_name, football.teams.country_code,
                   COALESCE(profile.team_type, 'other') AS team_type
            FROM football.teams
            LEFT JOIN football.team_profiles profile ON profile.team_id = football.teams.id
            WHERE football.teams.is_active
            "#,
        );
        if let Some(search) = NameSearch::parse(search) {
            push_name_search(
                &mut builder,
                &search,
                NameSearchColumns {
                    primary_normalized: "football.teams.normalized_name",
                    primary_display: "football.teams.canonical_name",
                    alias_table: "football.team_names",
                    alias_owner: "alias.team_id",
                    owner_id: "football.teams.id",
                    alias_normalized: "alias.normalized_name",
                    alias_display: "alias.name",
                },
            );
        }
        builder.push(" ORDER BY football.teams.normalized_name, football.teams.id LIMIT ");
        builder.push_bind(safe_limit);
        let rows = builder
            .build_query_as::<TeamOptionRow>()
            .fetch_all(&self.pool)
            .await?;
        Ok(rows.into_iter().map(map_team_option_row).collect())
    }
}
''',
)

write(
    "crates/persistence-postgres/src/adapters/catalog/teams/directory/list_row.rs",
    r'''
use uuid::Uuid;

#[derive(Debug, sqlx::FromRow)]
pub(super) struct TeamListRow {
    pub(super) id: Uuid,
    pub(super) canonical_name: String,
    pub(super) normalized_name: String,
    pub(super) country_code: Option<String>,
    pub(super) team_type: String,
    pub(super) current_coach_name: Option<String>,
    pub(super) is_active: bool,
    pub(super) current_player_count: i64,
    pub(super) unavailable_player_count: i64,
    pub(super) squad_ability_average: Option<f64>,
    pub(super) profile_confidence: Option<f64>,
}
''',
)

write(
    "crates/persistence-postgres/src/adapters/catalog/teams/directory/list_mapper.rs",
    r'''
use super::TeamListRow;
use football_domain::TeamListItem;

pub(super) fn map_team_list_row(row: TeamListRow) -> TeamListItem {
    TeamListItem {
        id: row.id,
        canonical_name: row.canonical_name,
        normalized_name: row.normalized_name,
        country_code: row.country_code,
        team_type: row.team_type,
        current_coach_name: row.current_coach_name,
        is_active: row.is_active,
        current_player_count: row.current_player_count,
        unavailable_player_count: row.unavailable_player_count,
        squad_ability_average: row.squad_ability_average,
        profile_confidence: row.profile_confidence,
    }
}
''',
)

write(
    "crates/persistence-postgres/src/adapters/catalog/teams/directory/list_teams.rs",
    r'''
use super::{map_team_list_row, TeamListRow};
use crate::{
    name_search::{push_name_search, NameSearch, NameSearchColumns},
    PersistenceError, PersistenceResult, PostgresStore,
};
use football_domain::{TeamListPage, TeamListQuery};
use sqlx::{Postgres, QueryBuilder};

impl PostgresStore {
    pub async fn list_teams(&self, query: &TeamListQuery) -> PersistenceResult<TeamListPage> {
        if query.cursor_name.is_some() != query.cursor_id.is_some() {
            return Err(PersistenceError::InvalidState(
                "球队分页游标必须同时包含名称和 ID".to_string(),
            ));
        }
        let limit = query.limit.clamp(1, 200);
        let mut builder: QueryBuilder<Postgres> = QueryBuilder::new(
            r#"
            SELECT team.id, team.canonical_name, team.normalized_name, team.country_code,
                   COALESCE(profile.team_type, 'other') AS team_type,
                   current_coach.coach_name AS current_coach_name,
                   team.is_active,
                   COALESCE(squad.current_player_count, 0)::bigint AS current_player_count,
                   COALESCE(squad.unavailable_player_count, 0)::bigint AS unavailable_player_count,
                   squad.squad_ability_average,
                   profile.data_confidence AS profile_confidence
            FROM football.teams team
            LEFT JOIN football.team_profiles profile ON profile.team_id = team.id
            LEFT JOIN LATERAL (
                SELECT coach.canonical_name AS coach_name
                FROM football.team_coach_periods period
                JOIN football.coaches coach ON coach.id = period.coach_id
                WHERE period.team_id = team.id
                  AND period.valid_from <= current_date
                  AND (period.valid_to IS NULL OR period.valid_to >= current_date)
                ORDER BY CASE period.role WHEN 'head_coach' THEN 0 WHEN 'interim_head_coach' THEN 1 ELSE 2 END,
                         period.valid_from DESC, period.id DESC
                LIMIT 1
            ) current_coach ON true
            LEFT JOIN LATERAL (
                SELECT count(*)::bigint AS current_player_count,
                       count(*) FILTER (WHERE availability.status IN ('injured','suspended','doubtful','rested'))::bigint AS unavailable_player_count,
                       avg(ability.average_value) AS squad_ability_average
                FROM football.player_team_periods period
                JOIN football.players player ON player.id = period.player_id
                LEFT JOIN feature.player_ability_profiles ability ON ability.player_id = player.id
                LEFT JOIN LATERAL (
                    SELECT status
                    FROM football.player_availability item
                    WHERE item.player_id = player.id
                      AND item.valid_from <= now()
                      AND (item.valid_to IS NULL OR item.valid_to >= now())
                    ORDER BY item.valid_from DESC, item.created_at DESC
                    LIMIT 1
                ) availability ON true
                WHERE period.team_id = team.id
                  AND period.valid_from <= current_date
                  AND (period.valid_to IS NULL OR period.valid_to >= current_date)
                  AND period.registration_status IN ('registered','loan','trial')
                  AND player.status = 'active'
            ) squad ON true
            WHERE 1 = 1
            "#,
        );
        if query.active_only {
            builder.push(" AND team.is_active");
        }
        if let Some(search) = NameSearch::parse(query.search.as_deref()) {
            push_name_search(
                &mut builder,
                &search,
                NameSearchColumns {
                    primary_normalized: "team.normalized_name",
                    primary_display: "team.canonical_name",
                    alias_table: "football.team_names",
                    alias_owner: "alias.team_id",
                    owner_id: "team.id",
                    alias_normalized: "alias.normalized_name",
                    alias_display: "alias.name",
                },
            );
        }
        if let Some(country) = query
            .country_code
            .as_deref()
            .map(str::trim)
            .filter(|value| !value.is_empty())
        {
            builder.push(" AND upper(COALESCE(team.country_code,'')) = ");
            builder.push_bind(country.to_uppercase());
        }
        if let Some(team_type) = query
            .team_type
            .as_deref()
            .map(str::trim)
            .filter(|value| !value.is_empty())
        {
            builder.push(" AND COALESCE(profile.team_type, 'other') = ");
            builder.push_bind(team_type.to_ascii_lowercase());
        }
        if let (Some(cursor_name), Some(cursor_id)) = (&query.cursor_name, query.cursor_id) {
            builder.push(" AND (team.normalized_name, team.id) > (");
            builder.push_bind(cursor_name);
            builder.push(", ");
            builder.push_bind(cursor_id);
            builder.push(")");
        }
        builder.push(" ORDER BY team.normalized_name, team.id LIMIT ");
        builder.push_bind(i64::from(limit) + 1);
        let rows = builder
            .build_query_as::<TeamListRow>()
            .fetch_all(&self.pool)
            .await?;
        let has_more = rows.len() > limit as usize;
        let items = rows
            .into_iter()
            .take(limit as usize)
            .map(map_team_list_row)
            .collect::<Vec<_>>();
        let next = items.last().filter(|_| has_more);
        Ok(TeamListPage {
            next_cursor_name: next.map(|item| item.normalized_name.clone()),
            next_cursor_id: next.map(|item| item.id),
            items,
            has_more,
        })
    }
}
''',
)

write(
    "crates/persistence-postgres/src/adapters/catalog/teams/detail/mod.rs",
    r'''
mod name_mapper;
mod name_row;
mod profile_mapper;
mod profile_row;
mod read_names;
mod read_profile;
mod read_recent_matches;
mod read_squad;
mod read_team;
mod read_team_record;
mod recent_match_mapper;
mod recent_match_row;
mod record_mapper;
mod record_row;
mod squad_mapper;
mod squad_row;

pub(super) use record_mapper::map_team_record;
pub(super) use record_row::TeamRecordRow;
''',
)

write(
    "crates/persistence-postgres/src/adapters/catalog/teams/detail/record_row.rs",
    r'''
use chrono::{DateTime, Utc};
use uuid::Uuid;

#[derive(Debug, sqlx::FromRow)]
pub(super) struct TeamRecordRow {
    pub(super) id: Uuid,
    pub(super) canonical_name: String,
    pub(super) normalized_name: String,
    pub(super) country_code: Option<String>,
    pub(super) is_active: bool,
    pub(super) created_at: DateTime<Utc>,
}
''',
)

write(
    "crates/persistence-postgres/src/adapters/catalog/teams/detail/record_mapper.rs",
    r'''
use super::TeamRecordRow;
use crate::PersistenceResult;
use football_domain::TeamRecord;

pub(super) fn map_team_record(row: TeamRecordRow) -> PersistenceResult<TeamRecord> {
    Ok(TeamRecord {
        id: row.id,
        canonical_name: row.canonical_name,
        normalized_name: row.normalized_name,
        country_code: row.country_code,
        is_active: row.is_active,
        created_at: row.created_at,
    })
}
''',
)

write(
    "crates/persistence-postgres/src/adapters/catalog/teams/detail/read_team_record.rs",
    r'''
use super::{map_team_record, TeamRecordRow};
use crate::{PersistenceError, PersistenceResult};
use football_domain::TeamRecord;
use sqlx::PgPool;
use uuid::Uuid;

pub(super) async fn read_team_record(
    pool: &PgPool,
    team_id: Uuid,
) -> PersistenceResult<TeamRecord> {
    let row = sqlx::query_as::<_, TeamRecordRow>(
        r#"
        SELECT id, canonical_name, normalized_name, country_code, is_active, created_at
        FROM football.teams
        WHERE id = $1
        "#,
    )
    .bind(team_id)
    .fetch_optional(pool)
    .await?
    .ok_or_else(|| PersistenceError::InvalidState("球队不存在".to_string()))?;
    map_team_record(row)
}
''',
)

write(
    "crates/persistence-postgres/src/adapters/catalog/teams/detail/name_row.rs",
    r'''
use chrono::NaiveDate;
use uuid::Uuid;

#[derive(Debug, sqlx::FromRow)]
pub(super) struct TeamNameRow {
    pub(super) id: Uuid,
    pub(super) team_id: Uuid,
    pub(super) name: String,
    pub(super) normalized_name: String,
    pub(super) language_code: Option<String>,
    pub(super) valid_from: Option<NaiveDate>,
    pub(super) valid_to: Option<NaiveDate>,
}
''',
)

write(
    "crates/persistence-postgres/src/adapters/catalog/teams/detail/name_mapper.rs",
    r'''
use super::name_row::TeamNameRow;
use football_domain::TeamNameRecord;

pub(super) fn map_team_name(row: TeamNameRow) -> TeamNameRecord {
    TeamNameRecord {
        id: row.id,
        team_id: row.team_id,
        name: row.name,
        normalized_name: row.normalized_name,
        language_code: row.language_code,
        valid_from: row.valid_from,
        valid_to: row.valid_to,
    }
}
''',
)

write(
    "crates/persistence-postgres/src/adapters/catalog/teams/detail/read_names.rs",
    r'''
use super::{name_mapper::map_team_name, name_row::TeamNameRow};
use crate::PersistenceResult;
use football_domain::TeamNameRecord;
use sqlx::PgPool;
use uuid::Uuid;

pub(super) async fn read_names(
    pool: &PgPool,
    team_id: Uuid,
) -> PersistenceResult<Vec<TeamNameRecord>> {
    let rows = sqlx::query_as::<_, TeamNameRow>(
        r#"
        SELECT id, team_id, name, normalized_name, language_code, valid_from, valid_to
        FROM football.team_names
        WHERE team_id = $1
        ORDER BY valid_from DESC NULLS LAST, name, id
        "#,
    )
    .bind(team_id)
    .fetch_all(pool)
    .await?;
    Ok(rows.into_iter().map(map_team_name).collect())
}
''',
)

write(
    "crates/persistence-postgres/src/adapters/catalog/teams/detail/profile_row.rs",
    r'''
use chrono::{DateTime, Utc};
use serde_json::Value;
use uuid::Uuid;

#[derive(Debug, sqlx::FromRow)]
pub(super) struct TeamProfileRow {
    pub(super) team_id: Uuid,
    pub(super) short_name: Option<String>,
    pub(super) team_type: String,
    pub(super) founded_year: Option<i16>,
    pub(super) city: Option<String>,
    pub(super) stadium: Option<String>,
    pub(super) head_coach: Option<String>,
    pub(super) default_formation: Option<String>,
    pub(super) tactical_style: String,
    pub(super) attack_rating: Option<f64>,
    pub(super) midfield_rating: Option<f64>,
    pub(super) defence_rating: Option<f64>,
    pub(super) goalkeeper_rating: Option<f64>,
    pub(super) reputation: Option<f64>,
    pub(super) data_confidence: f64,
    pub(super) notes: Option<String>,
    pub(super) metadata: Value,
    pub(super) updated_at: DateTime<Utc>,
}
''',
)

write(
    "crates/persistence-postgres/src/adapters/catalog/teams/detail/profile_mapper.rs",
    r'''
use super::profile_row::TeamProfileRow;
use football_domain::TeamProfileRecord;

pub(super) fn map_team_profile(row: TeamProfileRow) -> TeamProfileRecord {
    TeamProfileRecord {
        team_id: row.team_id,
        short_name: row.short_name,
        team_type: row.team_type,
        founded_year: row.founded_year,
        city: row.city,
        stadium: row.stadium,
        head_coach: row.head_coach,
        default_formation: row.default_formation,
        tactical_style: row.tactical_style,
        attack_rating: row.attack_rating,
        midfield_rating: row.midfield_rating,
        defence_rating: row.defence_rating,
        goalkeeper_rating: row.goalkeeper_rating,
        reputation: row.reputation,
        data_confidence: row.data_confidence,
        notes: row.notes,
        metadata: row.metadata,
        updated_at: row.updated_at,
    }
}
''',
)

write(
    "crates/persistence-postgres/src/adapters/catalog/teams/detail/read_profile.rs",
    r'''
use super::{profile_mapper::map_team_profile, profile_row::TeamProfileRow};
use crate::PersistenceResult;
use football_domain::TeamProfileRecord;
use sqlx::PgPool;
use uuid::Uuid;

pub(super) async fn read_profile(
    pool: &PgPool,
    team_id: Uuid,
) -> PersistenceResult<Option<TeamProfileRecord>> {
    let row = sqlx::query_as::<_, TeamProfileRow>(
        r#"
        SELECT team_id, short_name, team_type, founded_year, city, stadium, head_coach,
               default_formation, tactical_style, attack_rating, midfield_rating,
               defence_rating, goalkeeper_rating, reputation, data_confidence,
               notes, metadata, updated_at
        FROM football.team_profiles
        WHERE team_id = $1
        "#,
    )
    .bind(team_id)
    .fetch_optional(pool)
    .await?;
    Ok(row.map(map_team_profile))
}
''',
)

write(
    "crates/persistence-postgres/src/adapters/catalog/teams/detail/squad_row.rs",
    r'''
use uuid::Uuid;

#[derive(Debug, sqlx::FromRow)]
pub(super) struct TeamSquadRow {
    pub(super) player_id: Uuid,
    pub(super) player_name: String,
    pub(super) localized_name: Option<String>,
    pub(super) position_code: Option<String>,
    pub(super) role_code: Option<String>,
    pub(super) squad_number: Option<i16>,
    pub(super) registration_status: String,
    pub(super) availability_status: Option<String>,
    pub(super) ability_average: Option<f64>,
}
''',
)

write(
    "crates/persistence-postgres/src/adapters/catalog/teams/detail/squad_mapper.rs",
    r'''
use super::squad_row::TeamSquadRow;
use crate::{PersistenceError, PersistenceResult};
use football_domain::{AvailabilityStatus, TeamSquadPlayer};

pub(super) fn map_team_squad_row(row: TeamSquadRow) -> PersistenceResult<TeamSquadPlayer> {
    Ok(TeamSquadPlayer {
        player_id: row.player_id,
        player_name: row.player_name,
        localized_name: row.localized_name,
        position_code: row.position_code,
        role_code: row.role_code,
        squad_number: row.squad_number,
        registration_status: row.registration_status,
        availability_status: row
            .availability_status
            .as_deref()
            .map(parse_availability)
            .transpose()?,
        ability_average: row.ability_average,
    })
}

fn parse_availability(value: &str) -> PersistenceResult<AvailabilityStatus> {
    match value {
        "available" => Ok(AvailabilityStatus::Available),
        "doubtful" => Ok(AvailabilityStatus::Doubtful),
        "unavailable" => Ok(AvailabilityStatus::Unavailable),
        "injured" => Ok(AvailabilityStatus::Injured),
        "suspended" => Ok(AvailabilityStatus::Suspended),
        "rested" => Ok(AvailabilityStatus::Rested),
        "returning" => Ok(AvailabilityStatus::Returning),
        "unknown" => Ok(AvailabilityStatus::Unknown),
        other => Err(PersistenceError::InvalidState(format!(
            "未知球员可用状态：{other}"
        ))),
    }
}
''',
)

write(
    "crates/persistence-postgres/src/adapters/catalog/teams/detail/read_squad.rs",
    r'''
use super::{squad_mapper::map_team_squad_row, squad_row::TeamSquadRow};
use crate::PersistenceResult;
use football_domain::TeamSquadPlayer;
use sqlx::PgPool;
use uuid::Uuid;

pub(super) async fn read_squad(
    pool: &PgPool,
    team_id: Uuid,
) -> PersistenceResult<Vec<TeamSquadPlayer>> {
    let rows = sqlx::query_as::<_, TeamSquadRow>(
        r#"
        SELECT player.id AS player_id, player.canonical_name AS player_name,
               localized_name.name AS localized_name,
               position.position_code, position.default_role_code AS role_code,
               period.squad_number, period.registration_status,
               availability.status AS availability_status,
               ability.average_value AS ability_average
        FROM football.player_team_periods period
        JOIN football.players player ON player.id = period.player_id
        LEFT JOIN LATERAL (
            SELECT alias.name
            FROM football.player_names alias
            WHERE alias.player_id = player.id
              AND (
                lower(COALESCE(alias.language_code, '')) IN ('zh-cn', 'zh-hans', 'zh')
                OR alias.name ~ '[一-龥]'
              )
            ORDER BY
              CASE lower(COALESCE(alias.language_code, ''))
                WHEN 'zh-cn' THEN 0 WHEN 'zh-hans' THEN 1 WHEN 'zh' THEN 2 ELSE 3
              END,
              alias.is_primary DESC,
              alias.valid_from DESC NULLS LAST,
              alias.id DESC
            LIMIT 1
        ) localized_name ON true
        LEFT JOIN LATERAL (
            SELECT item.position_code, item.default_role_code
            FROM football.player_positions item
            WHERE item.player_id = player.id
              AND (item.valid_from IS NULL OR item.valid_from <= current_date)
              AND (item.valid_to IS NULL OR item.valid_to >= current_date)
            ORDER BY item.is_primary DESC, item.proficiency DESC, item.position_code
            LIMIT 1
        ) position ON true
        LEFT JOIN LATERAL (
            SELECT item.status
            FROM football.player_availability item
            WHERE item.player_id = player.id
              AND item.valid_from <= now()
              AND (item.valid_to IS NULL OR item.valid_to >= now())
            ORDER BY item.valid_from DESC, item.created_at DESC
            LIMIT 1
        ) availability ON true
        LEFT JOIN feature.player_ability_profiles ability ON ability.player_id = player.id
        WHERE period.team_id = $1
          AND period.valid_from <= current_date
          AND (period.valid_to IS NULL OR period.valid_to >= current_date)
          AND period.registration_status IN ('registered','loan','trial')
          AND player.status = 'active'
        ORDER BY position.position_code NULLS LAST, period.squad_number NULLS LAST,
                 player.normalized_name, player.id
        "#,
    )
    .bind(team_id)
    .fetch_all(pool)
    .await?;
    rows.into_iter().map(map_team_squad_row).collect()
}
''',
)

write(
    "crates/persistence-postgres/src/adapters/catalog/teams/detail/recent_match_row.rs",
    r'''
use chrono::{DateTime, Utc};
use uuid::Uuid;

#[derive(Debug, sqlx::FromRow)]
pub(super) struct TeamRecentMatchRow {
    pub(super) match_id: Uuid,
    pub(super) opponent_team_id: Uuid,
    pub(super) opponent_team_name: String,
    pub(super) kickoff_time: DateTime<Utc>,
    pub(super) venue_side: String,
    pub(super) status: String,
    pub(super) goals_for: Option<i16>,
    pub(super) goals_against: Option<i16>,
}
''',
)

write(
    "crates/persistence-postgres/src/adapters/catalog/teams/detail/recent_match_mapper.rs",
    r'''
use super::recent_match_row::TeamRecentMatchRow;
use crate::{PersistenceError, PersistenceResult};
use football_domain::{MatchStatus, TeamRecentMatch};

pub(super) fn map_team_recent_match(
    row: TeamRecentMatchRow,
) -> PersistenceResult<TeamRecentMatch> {
    Ok(TeamRecentMatch {
        match_id: row.match_id,
        opponent_team_id: row.opponent_team_id,
        opponent_team_name: row.opponent_team_name,
        kickoff_time: row.kickoff_time,
        venue_side: row.venue_side,
        status: parse_match_status(&row.status)?,
        goals_for: row.goals_for,
        goals_against: row.goals_against,
    })
}

fn parse_match_status(value: &str) -> PersistenceResult<MatchStatus> {
    match value {
        "scheduled" => Ok(MatchStatus::Scheduled),
        "live" => Ok(MatchStatus::Live),
        "finished" => Ok(MatchStatus::Finished),
        "postponed" => Ok(MatchStatus::Postponed),
        "cancelled" => Ok(MatchStatus::Cancelled),
        other => Err(PersistenceError::InvalidState(format!(
            "未知比赛状态：{other}"
        ))),
    }
}
''',
)

write(
    "crates/persistence-postgres/src/adapters/catalog/teams/detail/read_recent_matches.rs",
    r'''
use super::{recent_match_mapper::map_team_recent_match, recent_match_row::TeamRecentMatchRow};
use crate::PersistenceResult;
use football_domain::TeamRecentMatch;
use sqlx::PgPool;
use uuid::Uuid;

pub(super) async fn read_recent_matches(
    pool: &PgPool,
    team_id: Uuid,
) -> PersistenceResult<Vec<TeamRecentMatch>> {
    let rows = sqlx::query_as::<_, TeamRecentMatchRow>(
        r#"
        SELECT fixture.id AS match_id,
               CASE WHEN fixture.home_team_id = $1 THEN fixture.away_team_id ELSE fixture.home_team_id END AS opponent_team_id,
               CASE WHEN fixture.home_team_id = $1 THEN away.canonical_name ELSE home.canonical_name END AS opponent_team_name,
               fixture.kickoff_time,
               CASE WHEN fixture.home_team_id = $1 THEN 'home' ELSE 'away' END AS venue_side,
               fixture.status,
               CASE WHEN result.match_id IS NULL THEN NULL
                    WHEN fixture.home_team_id = $1 THEN result.home_goals_90 ELSE result.away_goals_90 END AS goals_for,
               CASE WHEN result.match_id IS NULL THEN NULL
                    WHEN fixture.home_team_id = $1 THEN result.away_goals_90 ELSE result.home_goals_90 END AS goals_against
        FROM football.matches fixture
        JOIN football.teams home ON home.id = fixture.home_team_id
        JOIN football.teams away ON away.id = fixture.away_team_id
        LEFT JOIN football.match_results result ON result.match_id = fixture.id
        WHERE fixture.home_team_id = $1 OR fixture.away_team_id = $1
        ORDER BY fixture.kickoff_time DESC, fixture.id DESC
        LIMIT 20
        "#,
    )
    .bind(team_id)
    .fetch_all(pool)
    .await?;
    rows.into_iter().map(map_team_recent_match).collect()
}
''',
)

write(
    "crates/persistence-postgres/src/adapters/catalog/teams/detail/read_team.rs",
    r'''
use super::{
    read_names::read_names, read_profile::read_profile, read_recent_matches::read_recent_matches,
    read_squad::read_squad, read_team_record::read_team_record,
};
use crate::{PersistenceResult, PostgresStore};
use football_domain::{
    FormationDistributionQuery, FormationUsageListQuery, TeamDetail,
};
use uuid::Uuid;

impl PostgresStore {
    pub async fn read_team(&self, team_id: Uuid) -> PersistenceResult<TeamDetail> {
        let team = read_team_record(&self.pool, team_id).await?;
        let names = read_names(&self.pool, team_id).await?;
        let profile = read_profile(&self.pool, team_id).await?;
        let squad = read_squad(&self.pool, team_id).await?;
        let player_periods = self.list_team_player_periods(team_id).await?;
        let coach_periods = self.list_team_coach_periods(team_id).await?;
        let recent_matches = read_recent_matches(&self.pool, team_id).await?;
        let formation_usage = self
            .list_formation_usage_distributions(&FormationUsageListQuery {
                team_id: Some(team_id),
                coach_id: None,
                competition_id: None,
                limit: 200,
            })
            .await?;
        let resolved_formation_distribution = self
            .resolve_formation_distribution(&FormationDistributionQuery {
                match_id: None,
                team_id,
                coach_id: None,
                competition_id: None,
                as_of: None,
            })
            .await?;
        Ok(TeamDetail {
            team,
            names,
            profile,
            squad,
            player_periods,
            coach_periods,
            recent_matches,
            formation_usage,
            resolved_formation_distribution,
        })
    }
}
''',
)

player_path = ROOT / "crates/persistence-postgres/src/player_catalog.rs"
player = player_path.read_text(encoding="utf-8")
player = remove_between(
    player,
    "    pub async fn create_team(&self, draft: &TeamDraft) -> PersistenceResult<TeamRecord> {",
    "    pub async fn create_data_provider(",
)
player = player.replace(
    "    PreferredFoot, SeasonTeamMembershipOption, TeamDraft, TeamOption, TeamRecord,\n",
    "    PreferredFoot, SeasonTeamMembershipOption,\n",
    1,
)
player = remove_between(
    player,
    "\nfn team_record_from_row(row: &sqlx::postgres::PgRow) -> PersistenceResult<TeamRecord> {",
    "\nfn data_provider_from_row(row: &sqlx::postgres::PgRow) -> PersistenceResult<DataProviderRecord> {",
)
player_path.write_text(player, encoding="utf-8")

team_path = ROOT / "crates/persistence-postgres/src/team_catalog.rs"
team = team_path.read_text(encoding="utf-8")
impl_marker = "impl PostgresStore {"
impl_index = team.find(impl_marker)
if impl_index < 0:
    raise SystemExit("team_catalog impl marker missing")
team = (
    "use crate::{PersistenceError, PersistenceResult, PostgresStore};\n"
    "use football_domain::{\n"
    "    BulkDeleteBlockedItem, BulkDeleteResult, TeamNameDraft, TeamNameRecord, TeamProfileDraft,\n"
    "    TeamProfileRecord,\n"
    "};\n"
    "use serde_json::json;\n"
    "use sqlx::Row;\n"
    "use uuid::Uuid;\n\n"
    + team[impl_index:]
)
team = remove_between(
    team,
    "    pub async fn list_teams(&self, query: &TeamListQuery) -> PersistenceResult<TeamListPage> {",
    "    pub async fn add_team_name(&self, draft: &TeamNameDraft) -> PersistenceResult<TeamNameRecord> {",
)
team = remove_between(
    team,
    "\nfn team_record_from_row(row: &sqlx::postgres::PgRow) -> PersistenceResult<TeamRecord> {",
    "\nfn team_name_from_row(row: &sqlx::postgres::PgRow) -> PersistenceResult<TeamNameRecord> {",
)
squad_marker = "\nfn team_squad_player_from_row(row: &sqlx::postgres::PgRow) -> PersistenceResult<TeamSquadPlayer> {"
squad_index = team.find(squad_marker)
if squad_index < 0:
    raise SystemExit("team squad mapper marker missing")
team = team[:squad_index] + "\n"
team_path.write_text(team, encoding="utf-8")

adapters_path = ROOT / "crates/persistence-postgres/src/adapters/mod.rs"
adapters = adapters_path.read_text(encoding="utf-8")
if "pub(crate) mod catalog;" not in adapters:
    adapters = adapters.replace(
        "pub(crate) mod competition;\n",
        "pub(crate) mod catalog;\npub(crate) mod competition;\n",
        1,
    )
adapters_path.write_text(adapters, encoding="utf-8")

# This helper and its workflow are transient and must not survive the owner-switch commit.
(ROOT / ".github/workflows/r6-01-owner-switch.yml").unlink(missing_ok=True)
Path(__file__).unlink(missing_ok=True)
