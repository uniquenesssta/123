from pathlib import Path
import re

ROOT = Path.cwd()


def write(path: str, content: str) -> None:
    p = ROOT / path
    p.parent.mkdir(parents=True, exist_ok=True)
    p.write_text(content.strip() + "\n", encoding="utf-8", newline="\n")


def replace_exact(path: str, old: str, new: str) -> None:
    p = ROOT / path
    text = p.read_text(encoding="utf-8")
    count = text.count(old)
    if count != 1:
        raise SystemExit(f"{path}: expected one anchor, found {count}: {old[:100]!r}")
    p.write_text(text.replace(old, new, 1), encoding="utf-8", newline="\n")


def scan_item_end(text: str, open_brace: int) -> int:
    depth = 0
    i = open_brace
    n = len(text)
    block_comment_depth = 0
    while i < n:
        if block_comment_depth:
            if text.startswith("/*", i):
                block_comment_depth += 1
                i += 2
                continue
            if text.startswith("*/", i):
                block_comment_depth -= 1
                i += 2
                continue
            i += 1
            continue
        if text.startswith("//", i):
            j = text.find("\n", i + 2)
            i = n if j < 0 else j + 1
            continue
        if text.startswith("/*", i):
            block_comment_depth = 1
            i += 2
            continue
        raw = re.match(r'(?:b)?r(#{0,16})"', text[i:])
        if raw:
            hashes = raw.group(1)
            end = '"' + hashes
            j = text.find(end, i + raw.end())
            if j < 0:
                raise SystemExit("unterminated Rust raw string")
            i = j + len(end)
            continue
        if text[i] == '"':
            i += 1
            while i < n:
                if text[i] == '\\':
                    i += 2
                elif text[i] == '"':
                    i += 1
                    break
                else:
                    i += 1
            continue
        if text[i] == "'":
            # Treat only a short, closed Rust char literal as a string-like token;
            # lifetimes such as 'a remain ordinary source text.
            j = i + 1
            if j < n and text[j] == '\\':
                j += 2
            else:
                j += 1
            if j < n and text[j] == "'":
                i = j + 1
                continue
        if text[i] == '{':
            depth += 1
        elif text[i] == '}':
            depth -= 1
            if depth == 0:
                return i + 1
        i += 1
    raise SystemExit("unbalanced Rust item")


def remove_fn(path: str, name: str) -> None:
    p = ROOT / path
    text = p.read_text(encoding="utf-8")
    pattern = re.compile(
        rf"(?m)^[ \t]*(?:pub(?:\([^)]*\))?[ \t]+)?(?:async[ \t]+)?fn[ \t]+{re.escape(name)}\b"
    )
    matches = list(pattern.finditer(text))
    if len(matches) != 1:
        raise SystemExit(f"{path}: expected one function {name}, found {len(matches)}")
    m = matches[0]
    open_brace = text.find("{", m.end())
    if open_brace < 0:
        raise SystemExit(f"{path}: no body for {name}")
    end = scan_item_end(text, open_brace)
    while end < len(text) and text[end] in " \t":
        end += 1
    if end < len(text) and text[end] == "\r":
        end += 1
    if end < len(text) and text[end] == "\n":
        end += 1
    # Remove one preceding blank line where present, but never remove the line
    # containing the previous closing brace/item.
    start = m.start()
    if start >= 1 and text[start - 1] == "\n":
        prev = text.rfind("\n", 0, start - 1)
        if prev >= 0 and text[prev + 1 : start - 1].strip() == "":
            start = prev + 1
    p.write_text(text[:start] + text[end:], encoding="utf-8", newline="\n")


# ---------------------------------------------------------------------------
# New R6-08 owners
# ---------------------------------------------------------------------------
write("crates/persistence-postgres/src/adapters/catalog/entity_matching/mod.rs", r'''
mod existence;
mod external_id;
mod name_candidates;
mod normalization;
mod outcome;
mod resolve;
''')

write("crates/persistence-postgres/src/adapters/catalog/entity_matching/normalization.rs", r'''
pub(super) fn normalize_name(value: &str) -> String {
    value
        .split_whitespace()
        .collect::<Vec<_>>()
        .join(" ")
        .to_lowercase()
}

#[cfg(test)]
mod tests {
    use super::normalize_name;

    #[test]
    fn matching_normalization_preserves_existing_case_and_spacing_semantics() {
        assert_eq!(normalize_name("  José   Mourinho  "), "josé mourinho");
    }
}
''')

write("crates/persistence-postgres/src/adapters/catalog/entity_matching/outcome.rs", r'''
use football_domain::{EntityMatchCandidate, EntityMatchResult};
use uuid::Uuid;

pub(super) fn exact_match(id: Uuid, reason: &str) -> EntityMatchResult {
    EntityMatchResult {
        status: "exact".to_string(),
        matched_id: Some(id),
        candidates: vec![EntityMatchCandidate {
            id,
            label: id.to_string(),
            reason: reason.to_string(),
            score: 1.0,
        }],
    }
}

pub(super) fn ambiguous(ids: Vec<Uuid>, reason: &str) -> EntityMatchResult {
    EntityMatchResult {
        status: "ambiguous".to_string(),
        matched_id: None,
        candidates: ids
            .into_iter()
            .map(|id| EntityMatchCandidate {
                id,
                label: id.to_string(),
                reason: reason.to_string(),
                score: 1.0,
            })
            .collect(),
    }
}

pub(super) fn from_candidates(candidates: Vec<EntityMatchCandidate>) -> EntityMatchResult {
    match candidates.len() {
        0 => EntityMatchResult {
            status: "no_match".to_string(),
            matched_id: None,
            candidates,
        },
        1 => EntityMatchResult {
            status: "exact".to_string(),
            matched_id: Some(candidates[0].id),
            candidates,
        },
        _ => EntityMatchResult {
            status: "ambiguous".to_string(),
            matched_id: None,
            candidates,
        },
    }
}

pub(super) fn no_match() -> EntityMatchResult {
    from_candidates(Vec::new())
}
''')

write("crates/persistence-postgres/src/adapters/catalog/entity_matching/existence.rs", r'''
use crate::{PersistenceError, PersistenceResult};
use sqlx::PgPool;
use uuid::Uuid;

pub(super) async fn entity_exists(
    pool: &PgPool,
    entity_type: &str,
    id: Uuid,
) -> PersistenceResult<bool> {
    let query = match entity_type {
        "team" => "SELECT EXISTS(SELECT 1 FROM football.teams WHERE id=$1)",
        "player" => "SELECT EXISTS(SELECT 1 FROM football.players WHERE id=$1)",
        "coach" => "SELECT EXISTS(SELECT 1 FROM football.coaches WHERE id=$1)",
        other => {
            return Err(PersistenceError::InvalidState(format!(
                "不支持的实体类型：{other}"
            )))
        }
    };
    Ok(sqlx::query_scalar(query).bind(id).fetch_one(pool).await?)
}
''')

write("crates/persistence-postgres/src/adapters/catalog/entity_matching/external_id.rs", r'''
use crate::PersistenceResult;
use sqlx::PgPool;
use uuid::Uuid;

pub(super) async fn matching_entity_ids(
    pool: &PgPool,
    provider_id: Uuid,
    entity_type: &str,
    external_id: &str,
) -> PersistenceResult<Vec<Uuid>> {
    Ok(sqlx::query_scalar::<_, Uuid>(
        "SELECT entity_id FROM football.external_entity_ids WHERE provider_id=$1 AND entity_type=$2 AND external_id=$3",
    )
    .bind(provider_id)
    .bind(entity_type)
    .bind(external_id)
    .fetch_all(pool)
    .await?)
}
''')

write("crates/persistence-postgres/src/adapters/catalog/entity_matching/name_candidates.rs", r'''
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
    .bind(country_code.map(str::trim).filter(|value| !value.is_empty()))
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
''')

write("crates/persistence-postgres/src/adapters/catalog/entity_matching/resolve.rs", r'''
use super::{
    existence::entity_exists,
    external_id::matching_entity_ids,
    name_candidates::{match_coaches_by_name, match_players_by_name, match_teams_by_name},
    normalization::normalize_name,
    outcome::{ambiguous, exact_match, from_candidates, no_match},
};
use crate::{
    adapters::catalog::references::validate_entity_type, PersistenceResult, PostgresStore,
};
use football_domain::{EntityMatchRequest, EntityMatchResult};

impl PostgresStore {
    pub async fn resolve_entity_reference(
        &self,
        request: &EntityMatchRequest,
    ) -> PersistenceResult<EntityMatchResult> {
        validate_entity_type(&request.entity_type)?;

        if let Some(id) = request.entity_id {
            if entity_exists(&self.pool, &request.entity_type, id).await? {
                return Ok(exact_match(id, "稳定实体 ID 精确匹配"));
            }
        }

        if let (Some(provider_id), Some(external_id)) = (
            request.provider_id,
            request
                .external_id
                .as_deref()
                .map(str::trim)
                .filter(|value| !value.is_empty()),
        ) {
            let ids = matching_entity_ids(
                &self.pool,
                provider_id,
                &request.entity_type,
                external_id,
            )
            .await?;
            if ids.len() == 1 {
                return Ok(exact_match(ids[0], "受信数据源外部 ID 精确匹配"));
            }
            if ids.len() > 1 {
                return Ok(ambiguous(ids, "外部 ID 对应多条实体"));
            }
        }

        let Some(name) = request
            .canonical_name
            .as_deref()
            .map(normalize_name)
            .filter(|value| !value.is_empty())
        else {
            return Ok(no_match());
        };

        let candidates = match request.entity_type.as_str() {
            "team" => match_teams_by_name(&self.pool, &name, request.country_code.as_deref()).await?,
            "player" => match_players_by_name(&self.pool, &name, request.date_of_birth).await?,
            "coach" => {
                match_coaches_by_name(&self.pool, &name, request.nationality_code.as_deref()).await?
            }
            _ => unreachable!(),
        };
        Ok(from_candidates(candidates))
    }
}
''')

write("crates/persistence-postgres/src/adapters/catalog/references/mod.rs", r'''
mod directory;
mod entity_type;
mod external_ids;
mod providers;

pub(crate) use entity_type::validate_entity_type;
''')

write("crates/persistence-postgres/src/adapters/catalog/references/entity_type.rs", r'''
use crate::{PersistenceError, PersistenceResult};

pub(crate) fn validate_entity_type(value: &str) -> PersistenceResult<()> {
    if matches!(value, "team" | "player" | "coach") {
        Ok(())
    } else {
        Err(PersistenceError::InvalidState(format!(
            "不支持的实体类型：{value}"
        )))
    }
}

#[cfg(test)]
mod tests {
    use super::validate_entity_type;

    #[test]
    fn reference_entity_type_contract_is_strict() {
        assert!(validate_entity_type("team").is_ok());
        assert!(validate_entity_type("player").is_ok());
        assert!(validate_entity_type("coach").is_ok());
        assert_eq!(
            validate_entity_type("match").unwrap_err().to_string(),
            "状态无效: 不支持的实体类型：match"
        );
    }
}
''')

write("crates/persistence-postgres/src/adapters/catalog/references/directory/mod.rs", r'''
mod list;
mod mapper;
mod read;
''')

write("crates/persistence-postgres/src/adapters/catalog/references/directory/list.rs", r'''
use super::read::{coach_references, player_references, team_references};
use crate::{
    adapters::catalog::references::validate_entity_type, PersistenceResult, PostgresStore,
};
use football_domain::{EntityReferenceQuery, EntityReferenceRecord};

impl PostgresStore {
    pub async fn list_entity_references(
        &self,
        query: &EntityReferenceQuery,
    ) -> PersistenceResult<Vec<EntityReferenceRecord>> {
        validate_entity_type(&query.entity_type)?;
        match query.entity_type.as_str() {
            "team" => team_references(&self.pool, query).await,
            "player" => player_references(&self.pool, query).await,
            "coach" => coach_references(&self.pool, query).await,
            _ => unreachable!(),
        }
    }
}
''')

write("crates/persistence-postgres/src/adapters/catalog/references/directory/read.rs", r'''
use super::mapper::{coach_reference_from_row, player_reference_from_row, team_reference_from_row};
use crate::{
    name_search::{push_name_search, NameSearch, NameSearchColumns},
    PersistenceResult,
};
use football_domain::{EntityReferenceQuery, EntityReferenceRecord};
use sqlx::{PgPool, Postgres, QueryBuilder};

pub(super) async fn team_references(
    pool: &PgPool,
    query: &EntityReferenceQuery,
) -> PersistenceResult<Vec<EntityReferenceRecord>> {
    let mut builder: QueryBuilder<Postgres> = QueryBuilder::new(
        r#"
        SELECT team.id, team.canonical_name, team.normalized_name, team.country_code,
               team.is_active,
               COALESCE((SELECT array_agg(alias.name ORDER BY alias.name) FROM football.team_names alias WHERE alias.team_id=team.id), ARRAY[]::text[]) AS aliases,
               COALESCE((SELECT array_agg(external.external_id ORDER BY external.external_id) FROM football.external_entity_ids external WHERE external.entity_type='team' AND external.entity_id=team.id), ARRAY[]::text[]) AS external_ids
        FROM football.teams team
        WHERE 1=1
        "#,
    );
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
    if query.active_only {
        builder.push(" AND team.is_active");
    }
    builder.push(" ORDER BY team.normalized_name, team.id LIMIT ");
    builder.push_bind(i64::from(query.limit.clamp(1, 500)));
    builder
        .build()
        .fetch_all(pool)
        .await?
        .iter()
        .map(team_reference_from_row)
        .collect()
}

pub(super) async fn player_references(
    pool: &PgPool,
    query: &EntityReferenceQuery,
) -> PersistenceResult<Vec<EntityReferenceRecord>> {
    let mut builder: QueryBuilder<Postgres> = QueryBuilder::new(
        r#"
        SELECT player.id, player.canonical_name, player.normalized_name,
               player.date_of_birth, player.nationality_code, player.status,
               COALESCE((SELECT array_agg(alias.name ORDER BY alias.name) FROM football.player_names alias WHERE alias.player_id=player.id), ARRAY[]::text[]) AS aliases,
               COALESCE((SELECT array_agg(external.external_id ORDER BY external.external_id) FROM football.external_entity_ids external WHERE external.entity_type='player' AND external.entity_id=player.id), ARRAY[]::text[]) AS external_ids
        FROM football.players player
        WHERE 1=1
        "#,
    );
    if let Some(search) = NameSearch::parse(query.search.as_deref()) {
        push_name_search(
            &mut builder,
            &search,
            NameSearchColumns {
                primary_normalized: "player.normalized_name",
                primary_display: "player.canonical_name",
                alias_table: "football.player_names",
                alias_owner: "alias.player_id",
                owner_id: "player.id",
                alias_normalized: "alias.normalized_name",
                alias_display: "alias.name",
            },
        );
    }
    if query.active_only {
        builder.push(" AND player.status='active'");
    }
    builder.push(" ORDER BY player.normalized_name, player.id LIMIT ");
    builder.push_bind(i64::from(query.limit.clamp(1, 500)));
    builder
        .build()
        .fetch_all(pool)
        .await?
        .iter()
        .map(player_reference_from_row)
        .collect()
}

pub(super) async fn coach_references(
    pool: &PgPool,
    query: &EntityReferenceQuery,
) -> PersistenceResult<Vec<EntityReferenceRecord>> {
    let mut builder: QueryBuilder<Postgres> = QueryBuilder::new(
        r#"
        SELECT coach.id, coach.canonical_name, coach.normalized_name,
               coach.nationality_code, coach.status,
               COALESCE((SELECT array_agg(alias.name ORDER BY alias.name) FROM football.coach_names alias WHERE alias.coach_id=coach.id), ARRAY[]::text[]) AS aliases,
               COALESCE((SELECT array_agg(external.external_id ORDER BY external.external_id) FROM football.external_entity_ids external WHERE external.entity_type='coach' AND external.entity_id=coach.id), ARRAY[]::text[]) AS external_ids
        FROM football.coaches coach
        WHERE 1=1
        "#,
    );
    if let Some(search) = NameSearch::parse(query.search.as_deref()) {
        push_name_search(
            &mut builder,
            &search,
            NameSearchColumns {
                primary_normalized: "coach.normalized_name",
                primary_display: "coach.canonical_name",
                alias_table: "football.coach_names",
                alias_owner: "alias.coach_id",
                owner_id: "coach.id",
                alias_normalized: "alias.normalized_name",
                alias_display: "alias.name",
            },
        );
    }
    if query.active_only {
        builder.push(" AND coach.status='active'");
    }
    builder.push(" ORDER BY coach.normalized_name, coach.id LIMIT ");
    builder.push_bind(i64::from(query.limit.clamp(1, 500)));
    builder
        .build()
        .fetch_all(pool)
        .await?
        .iter()
        .map(coach_reference_from_row)
        .collect()
}
''')

write("crates/persistence-postgres/src/adapters/catalog/references/directory/mapper.rs", r'''
use crate::PersistenceResult;
use football_domain::EntityReferenceRecord;
use sqlx::Row;

pub(super) fn team_reference_from_row(
    row: &sqlx::postgres::PgRow,
) -> PersistenceResult<EntityReferenceRecord> {
    Ok(EntityReferenceRecord {
        entity_type: "team".to_string(),
        id: row.try_get("id")?,
        canonical_name: row.try_get("canonical_name")?,
        normalized_name: row.try_get("normalized_name")?,
        country_code: row.try_get("country_code")?,
        nationality_code: None,
        date_of_birth: None,
        status: if row.try_get::<bool, _>("is_active")? {
            "active".to_string()
        } else {
            "inactive".to_string()
        },
        aliases: row.try_get("aliases")?,
        external_ids: row.try_get("external_ids")?,
    })
}

pub(super) fn player_reference_from_row(
    row: &sqlx::postgres::PgRow,
) -> PersistenceResult<EntityReferenceRecord> {
    Ok(EntityReferenceRecord {
        entity_type: "player".to_string(),
        id: row.try_get("id")?,
        canonical_name: row.try_get("canonical_name")?,
        normalized_name: row.try_get("normalized_name")?,
        country_code: None,
        nationality_code: row.try_get("nationality_code")?,
        date_of_birth: row.try_get("date_of_birth")?,
        status: row.try_get("status")?,
        aliases: row.try_get("aliases")?,
        external_ids: row.try_get("external_ids")?,
    })
}

pub(super) fn coach_reference_from_row(
    row: &sqlx::postgres::PgRow,
) -> PersistenceResult<EntityReferenceRecord> {
    Ok(EntityReferenceRecord {
        entity_type: "coach".to_string(),
        id: row.try_get("id")?,
        canonical_name: row.try_get("canonical_name")?,
        normalized_name: row.try_get("normalized_name")?,
        country_code: None,
        nationality_code: row.try_get("nationality_code")?,
        date_of_birth: None,
        status: row.try_get("status")?,
        aliases: row.try_get("aliases")?,
        external_ids: row.try_get("external_ids")?,
    })
}
''')

write("crates/persistence-postgres/src/adapters/catalog/references/providers/mod.rs", r'''
mod mapper;
mod read;
mod validation;
mod write;
''')

write("crates/persistence-postgres/src/adapters/catalog/references/providers/validation.rs", r'''
use crate::{PersistenceError, PersistenceResult};
use football_domain::DataProviderDraft;

pub(super) struct ValidatedProvider<'a> {
    pub code: String,
    pub name: &'a str,
    pub provider_type: &'a str,
    pub base_url: Option<&'a str>,
}

pub(super) fn validate_provider(draft: &DataProviderDraft) -> PersistenceResult<ValidatedProvider<'_>> {
    let code = draft.code.trim().to_lowercase();
    let name = draft.name.trim();
    let provider_type = draft.provider_type.trim();
    if code.is_empty() || name.is_empty() || provider_type.is_empty() {
        return Err(PersistenceError::InvalidState(
            "数据源代码、名称和类型不能为空".to_string(),
        ));
    }
    Ok(ValidatedProvider {
        code,
        name,
        provider_type,
        base_url: draft
            .base_url
            .as_deref()
            .map(str::trim)
            .filter(|value| !value.is_empty()),
    })
}
''')

write("crates/persistence-postgres/src/adapters/catalog/references/providers/mapper.rs", r'''
use crate::PersistenceResult;
use football_domain::DataProviderRecord;
use sqlx::Row;

pub(super) fn data_provider_from_row(
    row: &sqlx::postgres::PgRow,
) -> PersistenceResult<DataProviderRecord> {
    Ok(DataProviderRecord {
        id: row.try_get("id")?,
        code: row.try_get("code")?,
        name: row.try_get("name")?,
        provider_type: row.try_get("provider_type")?,
        base_url: row.try_get("base_url")?,
        is_active: row.try_get("is_active")?,
    })
}
''')

write("crates/persistence-postgres/src/adapters/catalog/references/providers/write.rs", r'''
use super::{mapper::data_provider_from_row, validation::validate_provider};
use crate::{PersistenceResult, PostgresStore};
use football_domain::{DataProviderDraft, DataProviderRecord};
use uuid::Uuid;

impl PostgresStore {
    pub async fn create_data_provider(
        &self,
        draft: &DataProviderDraft,
    ) -> PersistenceResult<DataProviderRecord> {
        let validated = validate_provider(draft)?;
        let generated_id = Uuid::new_v4();
        let row = sqlx::query(
            r#"
            INSERT INTO catalog.data_providers (
                id, code, name, provider_type, base_url, metadata
            ) VALUES ($1, $2, $3, $4, $5, $6)
            ON CONFLICT (code) DO UPDATE SET
                name = EXCLUDED.name,
                provider_type = EXCLUDED.provider_type,
                base_url = EXCLUDED.base_url,
                metadata = catalog.data_providers.metadata || EXCLUDED.metadata,
                is_active = true,
                updated_at = now()
            RETURNING id, code, name, provider_type, base_url, is_active
            "#,
        )
        .bind(generated_id)
        .bind(&validated.code)
        .bind(validated.name)
        .bind(validated.provider_type)
        .bind(validated.base_url)
        .bind(&draft.metadata)
        .fetch_one(&self.pool)
        .await?;
        data_provider_from_row(&row)
    }
}
''')

write("crates/persistence-postgres/src/adapters/catalog/references/providers/read.rs", r'''
use super::mapper::data_provider_from_row;
use crate::{PersistenceResult, PostgresStore};
use football_domain::DataProviderRecord;

impl PostgresStore {
    pub async fn list_data_providers(&self) -> PersistenceResult<Vec<DataProviderRecord>> {
        let rows = sqlx::query(
            r#"
            SELECT id, code, name, provider_type, base_url, is_active
            FROM catalog.data_providers
            WHERE is_active
            ORDER BY name, code
            "#,
        )
        .fetch_all(&self.pool)
        .await?;
        rows.iter().map(data_provider_from_row).collect()
    }
}
''')

write("crates/persistence-postgres/src/adapters/catalog/references/external_ids/mod.rs", r'''
mod mapper;
mod validation;
mod write;
''')

write("crates/persistence-postgres/src/adapters/catalog/references/external_ids/validation.rs", r'''
use crate::{PersistenceError, PersistenceResult};
use football_domain::ExternalEntityIdDraft;

pub(super) struct ValidatedExternalId<'a> {
    pub entity_type: &'a str,
    pub external_id: &'a str,
}

pub(super) fn validate_external_id(
    draft: &ExternalEntityIdDraft,
) -> PersistenceResult<ValidatedExternalId<'_>> {
    if !matches!(
        draft.entity_type.as_str(),
        "competition" | "season" | "team" | "player" | "coach" | "match"
    ) {
        return Err(PersistenceError::InvalidState(
            "外部 ID 实体类型无效".to_string(),
        ));
    }
    let external_id = draft.external_id.trim();
    if external_id.is_empty() {
        return Err(PersistenceError::InvalidState(
            "外部 ID 不能为空".to_string(),
        ));
    }
    Ok(ValidatedExternalId {
        entity_type: draft.entity_type.trim(),
        external_id,
    })
}
''')

write("crates/persistence-postgres/src/adapters/catalog/references/external_ids/mapper.rs", r'''
use crate::PersistenceResult;
use football_domain::ExternalEntityIdRecord;
use sqlx::Row;

pub(super) fn external_entity_id_from_row(
    row: &sqlx::postgres::PgRow,
) -> PersistenceResult<ExternalEntityIdRecord> {
    Ok(ExternalEntityIdRecord {
        id: row.try_get("id")?,
        provider_id: row.try_get("provider_id")?,
        provider_name: row.try_get("provider_name")?,
        entity_type: row.try_get("entity_type")?,
        entity_id: row.try_get("entity_id")?,
        external_id: row.try_get("external_id")?,
        metadata: row.try_get("metadata")?,
    })
}
''')

write("crates/persistence-postgres/src/adapters/catalog/references/external_ids/write.rs", r'''
use super::{mapper::external_entity_id_from_row, validation::validate_external_id};
use crate::{PersistenceResult, PostgresStore};
use football_domain::{ExternalEntityIdDraft, ExternalEntityIdRecord};
use uuid::Uuid;

impl PostgresStore {
    pub async fn add_external_entity_id(
        &self,
        draft: &ExternalEntityIdDraft,
    ) -> PersistenceResult<ExternalEntityIdRecord> {
        let validated = validate_external_id(draft)?;
        let row = sqlx::query(
            r#"
            WITH inserted AS (
                INSERT INTO football.external_entity_ids (
                    id, provider_id, entity_type, entity_id, external_id, metadata
                ) VALUES ($1, $2, $3, $4, $5, $6)
                ON CONFLICT (provider_id, entity_type, external_id) DO UPDATE SET
                    entity_id = EXCLUDED.entity_id,
                    metadata = football.external_entity_ids.metadata || EXCLUDED.metadata
                RETURNING *
            )
            SELECT inserted.id, inserted.provider_id,
                   provider.name AS provider_name, inserted.entity_type,
                   inserted.entity_id, inserted.external_id, inserted.metadata
            FROM inserted
            JOIN catalog.data_providers provider ON provider.id = inserted.provider_id
            "#,
        )
        .bind(Uuid::new_v4())
        .bind(draft.provider_id)
        .bind(validated.entity_type)
        .bind(draft.entity_id)
        .bind(validated.external_id)
        .bind(&draft.metadata)
        .fetch_one(&self.pool)
        .await?;
        external_entity_id_from_row(&row)
    }
}
''')

# ---------------------------------------------------------------------------
# Remove old R6-08 ownership while preserving R6-09 and other responsibilities.
# ---------------------------------------------------------------------------
entity_path = "crates/persistence-postgres/src/entity_catalog.rs"
for name in [
    "list_entity_references",
    "list_team_references",
    "list_player_references",
    "list_coach_references",
    "resolve_entity_reference",
    "match_teams_by_name",
    "match_players_by_name",
    "match_coaches_by_name",
    "entity_exists",
    "validate_entity_type",
    "exact_match",
    "ambiguous",
    "candidate_rows",
    "team_reference_from_row",
    "player_reference_from_row",
    "coach_reference_from_row",
    "normalize_name",
]:
    remove_fn(entity_path, name)

replace_exact(
    entity_path,
    "use crate::{\n    name_search::{push_name_search, NameSearch, NameSearchColumns},\n    write_audit_event, PersistenceError, PersistenceResult, PostgresStore,\n};",
    "use crate::{\n    adapters::catalog::references::validate_entity_type, write_audit_event, PersistenceError,\n    PersistenceResult, PostgresStore,\n};",
)
replace_exact(
    entity_path,
    "use football_domain::{\n    BulkArchiveFailedItem, BulkArchiveResult, EntityDeletionCheck, EntityMatchCandidate,\n    EntityMatchRequest, EntityMatchResult, EntityReferenceCount, EntityReferenceQuery,\n    EntityReferenceRecord, TeamPlayerPeriodRecord,\n};",
    "use football_domain::{\n    BulkArchiveFailedItem, BulkArchiveResult, EntityDeletionCheck, EntityReferenceCount,\n    TeamPlayerPeriodRecord,\n};",
)
replace_exact(
    entity_path,
    "use sqlx::{Postgres, QueryBuilder, Row, Transaction};",
    "use sqlx::{Postgres, Row, Transaction};",
)

player_path = "crates/persistence-postgres/src/player_catalog.rs"
for name in [
    "create_data_provider",
    "list_data_providers",
    "add_external_entity_id",
    "data_provider_from_row",
    "external_entity_id_from_row",
]:
    remove_fn(player_path, name)
replace_exact(
    player_path,
    "    AvailabilityStatus, DataProviderDraft, DataProviderRecord, ExternalEntityIdDraft,\n    ExternalEntityIdRecord, LineupDraft, LineupHistoryRemovalResult, LineupPairDraft,",
    "    AvailabilityStatus, LineupDraft, LineupHistoryRemovalResult, LineupPairDraft,",
)

replace_exact(
    "crates/persistence-postgres/src/adapters/catalog/mod.rs",
    "mod dynamic_tags;\nmod formations;\npub(crate) mod players;",
    "mod dynamic_tags;\nmod entity_matching;\nmod formations;\npub(crate) mod players;\npub(crate) mod references;",
)

# ---------------------------------------------------------------------------
# Contract test
# ---------------------------------------------------------------------------
write("crates/persistence-postgres/tests/entity_matching_references_repository_contract.rs", r'''
use chrono::NaiveDate;
use football_domain::{
    CoachDraft, CoachNameDraft, DataProviderDraft, EntityMatchRequest, EntityReferenceQuery,
    ExternalEntityIdDraft, PlayerDraft, PlayerNameDraft, PlayerStatus, PreferredFoot, TeamDraft,
    TeamNameDraft,
};
use football_persistence_postgres::{DatabaseOptions, PersistenceError, PostgresStore};
use serde_json::json;
use uuid::Uuid;

#[tokio::test]
#[ignore = "需要专用且可写的 PostgreSQL 测试数据库；设置 FOOTBALL_TEST_DATABASE_URL 后显式运行"]
async fn entity_matching_and_references_contract_is_preserved() {
    let url = std::env::var("FOOTBALL_TEST_DATABASE_URL").expect("设置 FOOTBALL_TEST_DATABASE_URL");
    let store = PostgresStore::connect(&DatabaseOptions {
        connection_url: url,
        max_connections: 4,
        connect_timeout_seconds: 10,
    })
    .await
    .expect("connect");
    store.migrate().await.expect("migrate");

    let token = Uuid::new_v4().simple().to_string();
    let team = store
        .create_team(&TeamDraft {
            canonical_name: format!("R6-08 Team {token}"),
            country_code: Some("PT".into()),
            metadata: json!({"contract":"r6-08"}),
        })
        .await
        .expect("create team");
    let team_alias = format!("Atlético Ref {token}");
    store
        .add_team_name(&TeamNameDraft {
            team_id: team.id,
            name: team_alias.clone(),
            language_code: Some("es".into()),
            valid_from: None,
            valid_to: None,
        })
        .await
        .expect("team alias");

    let dob = NaiveDate::from_ymd_opt(1995, 2, 3).unwrap();
    let player = store
        .create_player(&PlayerDraft {
            canonical_name: format!("R6-08 Player {token}"),
            date_of_birth: Some(dob),
            nationality_code: Some("BR".into()),
            preferred_foot: PreferredFoot::Right,
            height_cm: Some(181),
            status: PlayerStatus::Active,
            metadata: json!({"contract":"r6-08"}),
        })
        .await
        .expect("create player");
    let player_alias = format!("João Ref {token}");
    store
        .add_player_name(&PlayerNameDraft {
            player_id: player.id,
            name: player_alias.clone(),
            language_code: Some("pt".into()),
            is_primary: false,
            valid_from: None,
            valid_to: None,
        })
        .await
        .expect("player alias");

    let coach = store
        .create_coach(&CoachDraft {
            canonical_name: format!("R6-08 Coach {token}"),
            nationality_code: Some("PT".into()),
            status: "active".into(),
            metadata: json!({"contract":"r6-08"}),
        })
        .await
        .expect("create coach");
    let coach_alias = format!("José Ref {token}");
    store
        .add_coach_name(&CoachNameDraft {
            coach_id: coach.id,
            name: coach_alias.clone(),
            language_code: Some("pt".into()),
            is_primary: false,
            valid_from: None,
            valid_to: None,
        })
        .await
        .expect("coach alias");

    let provider = store
        .create_data_provider(&DataProviderDraft {
            code: format!("  R6_08_{token}  "),
            name: format!("  R6-08 Provider {token}  "),
            provider_type: "  official  ".into(),
            base_url: Some("  https://example.test/r6-08  ".into()),
            metadata: json!({"contract":"r6-08"}),
        })
        .await
        .expect("provider");
    assert_eq!(provider.code, format!("r6_08_{token}"));
    assert_eq!(provider.name, format!("R6-08 Provider {token}"));
    assert_eq!(provider.provider_type, "official");
    assert_eq!(provider.base_url.as_deref(), Some("https://example.test/r6-08"));
    assert!(store
        .list_data_providers()
        .await
        .expect("providers")
        .iter()
        .any(|item| item.id == provider.id));

    let external_value = format!("EXT-{token}");
    let external = store
        .add_external_entity_id(&ExternalEntityIdDraft {
            provider_id: provider.id,
            entity_type: "team".into(),
            entity_id: team.id,
            external_id: format!("  {external_value}  "),
            metadata: json!({"source":"contract"}),
        })
        .await
        .expect("external id");
    assert_eq!(external.entity_id, team.id);
    assert_eq!(external.external_id, external_value);
    assert_eq!(external.provider_name, provider.name);

    let team_refs = store
        .list_entity_references(&EntityReferenceQuery {
            entity_type: "team".into(),
            search: Some(format!("atletico ref {token}")),
            active_only: true,
            limit: 20,
        })
        .await
        .expect("team reference search");
    let team_ref = team_refs.iter().find(|item| item.id == team.id).expect("team ref");
    assert!(team_ref.aliases.iter().any(|value| value == &team_alias));
    assert!(team_ref.external_ids.iter().any(|value| value == &external_value));

    let player_refs = store
        .list_entity_references(&EntityReferenceQuery {
            entity_type: "player".into(),
            search: Some(format!("joao ref {token}")),
            active_only: true,
            limit: 20,
        })
        .await
        .expect("player reference search");
    assert!(player_refs.iter().any(|item| item.id == player.id));

    let coach_refs = store
        .list_entity_references(&EntityReferenceQuery {
            entity_type: "coach".into(),
            search: Some(format!("jose ref {token}")),
            active_only: true,
            limit: 20,
        })
        .await
        .expect("coach reference search");
    assert!(coach_refs.iter().any(|item| item.id == coach.id));

    let stable = store
        .resolve_entity_reference(&EntityMatchRequest {
            entity_type: "team".into(),
            entity_id: Some(team.id),
            provider_id: None,
            external_id: None,
            canonical_name: None,
            country_code: None,
            nationality_code: None,
            date_of_birth: None,
        })
        .await
        .expect("stable id");
    assert_eq!(stable.status, "exact");
    assert_eq!(stable.matched_id, Some(team.id));
    assert_eq!(stable.candidates[0].reason, "稳定实体 ID 精确匹配");

    let external_match = store
        .resolve_entity_reference(&EntityMatchRequest {
            entity_type: "team".into(),
            entity_id: None,
            provider_id: Some(provider.id),
            external_id: Some(external_value.clone()),
            canonical_name: Some("wrong fallback name".into()),
            country_code: None,
            nationality_code: None,
            date_of_birth: None,
        })
        .await
        .expect("external id match");
    assert_eq!(external_match.matched_id, Some(team.id));
    assert_eq!(external_match.candidates[0].reason, "受信数据源外部 ID 精确匹配");

    let team_name_match = store
        .resolve_entity_reference(&EntityMatchRequest {
            entity_type: "team".into(),
            entity_id: None,
            provider_id: None,
            external_id: None,
            canonical_name: Some(team_alias),
            country_code: Some("pt".into()),
            nationality_code: None,
            date_of_birth: None,
        })
        .await
        .expect("team name match");
    assert_eq!(team_name_match.matched_id, Some(team.id));
    assert_eq!(team_name_match.candidates[0].reason, "球队别名");
    assert!((team_name_match.candidates[0].score - 0.95).abs() < f64::EPSILON);

    let player_name_match = store
        .resolve_entity_reference(&EntityMatchRequest {
            entity_type: "player".into(),
            entity_id: None,
            provider_id: None,
            external_id: None,
            canonical_name: Some(player_alias),
            country_code: None,
            nationality_code: None,
            date_of_birth: Some(dob),
        })
        .await
        .expect("player name match");
    assert_eq!(player_name_match.matched_id, Some(player.id));
    assert_eq!(player_name_match.candidates[0].reason, "球员别名与出生日期");
    assert!((player_name_match.candidates[0].score - 1.0).abs() < f64::EPSILON);

    let coach_name_match = store
        .resolve_entity_reference(&EntityMatchRequest {
            entity_type: "coach".into(),
            entity_id: None,
            provider_id: None,
            external_id: None,
            canonical_name: Some(coach_alias),
            country_code: None,
            nationality_code: Some("pt".into()),
            date_of_birth: None,
        })
        .await
        .expect("coach name match");
    assert_eq!(coach_name_match.matched_id, Some(coach.id));
    assert_eq!(coach_name_match.candidates[0].reason, "教练别名与国籍");

    let no_match = store
        .resolve_entity_reference(&EntityMatchRequest {
            entity_type: "team".into(),
            entity_id: None,
            provider_id: None,
            external_id: None,
            canonical_name: Some(format!("missing {token}")),
            country_code: None,
            nationality_code: None,
            date_of_birth: None,
        })
        .await
        .expect("no match");
    assert_eq!(no_match.status, "no_match");
    assert!(no_match.matched_id.is_none());
    assert!(no_match.candidates.is_empty());

    let bad_type = store
        .list_entity_references(&EntityReferenceQuery {
            entity_type: "match".into(),
            search: None,
            active_only: true,
            limit: 20,
        })
        .await
        .expect_err("unsupported reference type");
    assert!(matches!(bad_type, PersistenceError::InvalidState(m) if m == "不支持的实体类型：match"));

    let bad_provider = store
        .create_data_provider(&DataProviderDraft {
            code: "  ".into(),
            name: "name".into(),
            provider_type: "official".into(),
            base_url: None,
            metadata: json!({}),
        })
        .await
        .expect_err("blank provider code");
    assert!(matches!(bad_provider, PersistenceError::InvalidState(m) if m == "数据源代码、名称和类型不能为空"));

    let bad_external_type = store
        .add_external_entity_id(&ExternalEntityIdDraft {
            provider_id: provider.id,
            entity_type: "formation".into(),
            entity_id: team.id,
            external_id: "x".into(),
            metadata: json!({}),
        })
        .await
        .expect_err("invalid external id entity type");
    assert!(matches!(bad_external_type, PersistenceError::InvalidState(m) if m == "外部 ID 实体类型无效"));

    let blank_external = store
        .add_external_entity_id(&ExternalEntityIdDraft {
            provider_id: provider.id,
            entity_type: "team".into(),
            entity_id: team.id,
            external_id: "   ".into(),
            metadata: json!({}),
        })
        .await
        .expect_err("blank external id");
    assert!(matches!(blank_external, PersistenceError::InvalidState(m) if m == "外部 ID 不能为空"));

    store.close().await;
}
''')

# ---------------------------------------------------------------------------
# Ownership verifier + legacy verifier retargeting
# ---------------------------------------------------------------------------
write("scripts/verify-entity-matching-references-persistence.mjs", r'''
import fs from "node:fs";
const read = (p) => fs.readFileSync(new URL(`../${p}`, import.meta.url), "utf8").replace(/\r\n?/g, "\n");
const exists = (p) => fs.existsSync(new URL(`../${p}`, import.meta.url));
const req = (ok, msg) => { if (!ok) throw new Error(msg); };
const entity = read("crates/persistence-postgres/src/entity_catalog.rs");
const player = read("crates/persistence-postgres/src/player_catalog.rs");
const catalog = read("crates/persistence-postgres/src/adapters/catalog/mod.rs");
const matchingRoot = "crates/persistence-postgres/src/adapters/catalog/entity_matching/";
const referencesRoot = "crates/persistence-postgres/src/adapters/catalog/references/";
const files = [];
const walk = (base, rel="") => { for (const e of fs.readdirSync(new URL(`../${base}${rel}`, import.meta.url), {withFileTypes:true})) e.isDirectory() ? walk(base, `${rel}${e.name}/`) : files.push(`${base}${rel}${e.name}`); };
walk(matchingRoot); walk(referencesRoot);
const all = files.filter(p => p.endsWith(".rs")).map(read).join("\n");
req(catalog.includes("mod entity_matching;") && catalog.includes("pub(crate) mod references;"), "R6-08 catalog modules missing");
for (const name of ["list_entity_references", "resolve_entity_reference", "create_data_provider", "list_data_providers", "add_external_entity_id"]) {
  req((all.match(new RegExp(`pub async fn ${name}\\b`, "g")) || []).length === 1, `R6-08 owner count invalid: ${name}`);
}
for (const name of ["list_entity_references", "resolve_entity_reference"]) req(!entity.includes(`pub async fn ${name}`), `entity_catalog legacy owner remains: ${name}`);
for (const name of ["create_data_provider", "list_data_providers", "add_external_entity_id"]) req(!player.includes(`pub async fn ${name}`), `player_catalog legacy owner remains: ${name}`);
req(entity.includes("pub async fn check_entity_deletion") && entity.includes("pub async fn bulk_archive_entities"), "R6-09 deletion/archive moved early");
req(entity.includes("team_reference_counts") && entity.includes("player_reference_counts") && entity.includes("coach_reference_counts"), "R6-09 reference-count owner moved early");
const resolve = read(`${matchingRoot}resolve.rs`);
const list = read(`${referencesRoot}directory/list.rs`);
req(!resolve.includes("sqlx::") && !resolve.includes("SELECT ") && !list.includes("sqlx::") && !list.includes("SELECT "), "R6-08 coordinator owns SQL");
for (const path of [
  `${matchingRoot}existence.rs`, `${matchingRoot}external_id.rs`, `${matchingRoot}name_candidates.rs`,
  `${referencesRoot}directory/read.rs`, `${referencesRoot}directory/mapper.rs`,
  `${referencesRoot}providers/read.rs`, `${referencesRoot}providers/write.rs`, `${referencesRoot}providers/validation.rs`,
  `${referencesRoot}external_ids/write.rs`, `${referencesRoot}external_ids/validation.rs`,
]) req(exists(path), `R6-08 responsibility owner missing: ${path}`);
const matching = files.filter(p => p.includes("/entity_matching/")).map(read).join("\n");
for (const token of ["稳定实体 ID 精确匹配", "受信数据源外部 ID 精确匹配", "外部 ID 对应多条实体", "球队别名", "球员别名与出生日期", "教练别名与国籍", "status: \"ambiguous\""]) req(matching.includes(token), `matching contract missing: ${token}`);
const directoryRead = read(`${referencesRoot}directory/read.rs`);
req(directoryRead.includes("NameSearch::parse") && directoryRead.includes("push_name_search"), "reference directory lost unified NameSearch");
req(directoryRead.includes("query.limit.clamp(1, 500)") && directoryRead.includes("ORDER BY team.normalized_name, team.id") && directoryRead.includes("ORDER BY player.normalized_name, player.id") && directoryRead.includes("ORDER BY coach.normalized_name, coach.id"), "reference paging/order contract changed");
const providerValidation = read(`${referencesRoot}providers/validation.rs`);
const externalValidation = read(`${referencesRoot}external_ids/validation.rs`);
req(providerValidation.includes("数据源代码、名称和类型不能为空"), "provider validation error changed");
req(externalValidation.includes("外部 ID 实体类型无效") && externalValidation.includes("外部 ID 不能为空"), "external id validation errors changed");
const port = read("crates/application/src/ports/player/mod.rs");
for (const method of ["list_references", "resolve_reference", "check_deletion", "bulk_archive", "create_data_provider", "add_external_id"]) req(port.includes(`async fn ${method}`), `EntityReferencePort changed: ${method}`);
console.log("R6-08 Entity Matching / References persistence ownership verified.");
''')

# R6-07 verifier must follow the new owner without weakening its R6-09 boundary.
replace_exact(
    "scripts/verify-coach-formation-persistence.mjs",
    'req(entity.includes("pub async fn list_entity_references") && entity.includes("pub async fn resolve_entity_reference") && entity.includes("pub async fn bulk_archive_entities"), "R6-08/R6-09 ownership moved early");',
    'req(!entity.includes("pub async fn list_entity_references") && !entity.includes("pub async fn resolve_entity_reference") && entity.includes("pub async fn bulk_archive_entities"), "R6-08 ownership switch incomplete or R6-09 ownership moved early");',
)

# Historical relationship verifier aggregates the authoritative R6-08 owners plus retained R6-09 owner.
replace_exact(
    "scripts/verify-entity-relationships.mjs",
    '  text("crates/persistence-postgres/src/adapters/catalog/coaches/team_periods/add.rs"),\n].join("\\n");',
    '  text("crates/persistence-postgres/src/adapters/catalog/coaches/team_periods/add.rs"),\n  text("crates/persistence-postgres/src/adapters/catalog/entity_matching/resolve.rs"),\n  text("crates/persistence-postgres/src/adapters/catalog/entity_matching/outcome.rs"),\n  text("crates/persistence-postgres/src/adapters/catalog/references/directory/list.rs"),\n].join("\\n");',
)

package_path = "package.json"
replace_exact(
    package_path,
    "&& node scripts/verify-coach-formation-persistence.mjs\",",
    "&& node scripts/verify-coach-formation-persistence.mjs && node scripts/verify-entity-matching-references-persistence.mjs\",",
)
replace_exact(
    package_path,
    '    "verify:coach-formation-persistence": "node scripts/verify-coach-formation-persistence.mjs"\n',
    '    "verify:coach-formation-persistence": "node scripts/verify-coach-formation-persistence.mjs",\n    "verify:entity-matching-references": "node scripts/verify-entity-matching-references-persistence.mjs"\n',
)

# ---------------------------------------------------------------------------
# Node record: VERIFYING after the implementation/minimum gate succeeds.
# This content is committed only by the workflow after all minimum checks pass.
# ---------------------------------------------------------------------------
write("docs/modular-rewrite/R06-entity-catalog-persistence/R06-08-entity-matching-and-references.md", r'''
# R6-08 Entity Matching 与 References

## 状态

`VERIFYING`

## 基线与范围

- 唯一阶段分支：`rewrite/r6-entity-catalog-persistence`。
- 节点起点：`a44e11aceccdb1c582636ff8d4c49f9d24529baa`（R6-07 已关闭，R6-08 为唯一 READY 节点）。
- 本节点只重写 Entity Matching 与 References persistence；R6-09 Archive/Delete/Force Delete 与 R6-10 Global Name Search 未提前迁移。

## 实现结果

- Entity Matching 已收敛到 `adapters/catalog/entity_matching/`：resolve orchestration、stable-ID existence、external-ID lookup、name candidates、normalization 与 outcome 分责。
- References 已收敛到 `adapters/catalog/references/`：reference directory read/mapper/list、provider validation/read/write/mapper、external-ID validation/write/mapper 分责。
- `resolve.rs` 与 reference `directory/list.rs` 仅编排，不直接执行 SQL；SQL 副作用集中在明确 read/write owner。
- `entity_catalog.rs` 已移除 R6-08 matching/reference-list owner，但继续保留 R6-09 deletion/archive/reference-count owner；`player_catalog.rs` 已移除 provider/external-ID owner。
- `EntityReferencePort`、Domain DTO、Schema/0001–0046 migrations、配置、错误语义与用户可观察行为保持不变；无新增生产依赖。

## 契约保持

- Stable entity ID 优先于 trusted provider external ID，external ID 优先于 normalized name matching。
- Team / Player / Coach canonical name 与 alias matching、country/nationality/date-of-birth disambiguation、exact/no_match/ambiguous 状态和既有 candidate score/reason 文案保持不变。
- Reference Directory 继续复用统一 `NameSearch`，保持中文、英文、别名、重音与多关键词检索；active-only、500 上限和稳定排序不变。
- Data Provider upsert、metadata merge、active re-enable 与 External ID upsert/metadata merge 语义不变。
- R6-09 删除预检、归档及 P4/历史引用保护未提前迁移或改写。

## 当前验证

- implementation/minimum gate：R6-08 ownership verifier、R6-07 retained verifier、历史 Entity Relationships verifier、完整 architecture、rustfmt、Persistence compile/unit tests、R6-08 PostgreSQL contract 编译与 Application compile 通过后才提交本记录。
- Stage Regression、真实 PostgreSQL 16 R6-08 contract 与 clean canonical 尚未执行，因此当前只标记 `VERIFYING`，不得提前标记 `DONE`。

## 未执行与剩余风险

- 用户现有 PostgreSQL 数据库真实 sample/write 与 Windows Full 人工交互验收继续保留到最终统一验收。
- R6-07 已确认的 4 个历史 full PostgreSQL baseline 既有失败不属于本节点；R6-08 不修改对应 Lineup/P4/Review owner。
''')

stage_path = "docs/modular-rewrite/R06-entity-catalog-persistence/README.md"
replace_exact(stage_path, "| R6-08 | Entity Matching 与 References | READY |", "| R6-08 | Entity Matching 与 References | VERIFYING |")
stage = (ROOT / stage_path).read_text(encoding="utf-8")
if "## R6-08 当前事实" not in stage:
    stage += r'''

## R6-08 当前事实

- 详细记录：[`R06-08-entity-matching-and-references.md`](R06-08-entity-matching-and-references.md)。
- Entity Matching / References persistence 已完成唯一 owner 切换并进入 `VERIFYING`：matching resolve 与 reference list coordinator SQL-free，SQL I/O 分别收敛到具名 read/write 模块；R6-09 deletion/archive/reference-count 继续由原 owner 持有。
- implementation/minimum gate 通过后才形成生产提交；Stage Regression、PostgreSQL 16 真实 contract 与 clean canonical 尚未完成，因此 R6-09～R6-10 继续 `BLOCKED`。
'''
    (ROOT / stage_path).write_text(stage, encoding="utf-8", newline="\n")

root_path = ROOT / "README.md"
root = root_path.read_text(encoding="utf-8")
section = r'''
### R6-08 Entity Matching 与 References

- Entity Matching persistence 已收敛到 `adapters/catalog/entity_matching/`，References persistence 已收敛到 `adapters/catalog/references/`；stable-ID/external-ID/name candidate read、reference directory、provider 与 external-ID writes 均按职责拆分，协调器保持 SQL-free。`entity_catalog.rs` 继续只持有后续 R6-09 deletion/archive/reference-count，`player_catalog.rs` 不再持有 provider/external-ID owner。
- `EntityReferencePort`、Domain DTO、Schema/0001–0046 migrations、配置、错误语义、统一 NameSearch、matching priority/candidate score/reason、历史 P4/运行引用和用户可观察行为保持不变；无新增生产依赖。
- R6-08 implementation/minimum gate 通过后进入 `VERIFYING`；Stage Regression、PostgreSQL 16 真实 contract 与 clean canonical 完成前不标记 `DONE`。

'''
anchor = "### R6-07 Coaches 与 Formation Usage\n"
if "### R6-08 Entity Matching 与 References" not in root:
    if root.count(anchor) != 1:
        raise SystemExit("README R6-07 anchor mismatch")
    root = root.replace(anchor, section + anchor, 1)
    root_path.write_text(root, encoding="utf-8", newline="\n")

print("R6-08 implementation payload applied")
