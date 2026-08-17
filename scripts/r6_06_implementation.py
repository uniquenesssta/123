from pathlib import Path
import json

root = Path('.')


def write(path: str, content: str):
    target = root / path
    target.parent.mkdir(parents=True, exist_ok=True)
    target.write_text(content, encoding='utf-8')


def between(text: str, start: str, end: str) -> str:
    a = text.index(start)
    b = text.index(end, a)
    return text[a:b].rstrip()


def remove_between(text: str, start: str, end: str) -> str:
    a = text.index(start)
    b = text.index(end, a)
    return text[:a] + text[b:]


# --- Ability owners ---
player_catalog_path = root / 'crates/persistence-postgres/src/player_catalog.rs'
player_catalog = player_catalog_path.read_text(encoding='utf-8')
ability_add = between(
    player_catalog,
    '    pub async fn add_player_ability_observation(',
    '\n\n    pub async fn add_external_entity_id(',
)
ability_list = between(
    player_catalog,
    '    pub async fn list_ability_dimensions(',
    '\n}\n\nasync fn resolve_match_scope_draft(',
)
player_catalog = remove_between(
    player_catalog,
    '    pub async fn add_player_ability_observation(',
    '    pub async fn add_external_entity_id(',
)
player_catalog = remove_between(
    player_catalog,
    '    pub async fn list_ability_dimensions(',
    '}\n\nasync fn resolve_match_scope_draft(',
)
player_catalog = remove_between(
    player_catalog,
    'fn player_ability_observation_from_row(',
    'fn external_entity_id_from_row(',
)
player_catalog = remove_between(
    player_catalog,
    'fn ability_dimension_from_row(',
    '#[cfg(test)]',
)
player_catalog = player_catalog.replace(
    '    AbilityDimensionRecord, AvailabilityStatus, DataProviderDraft, DataProviderRecord,\n',
    '    AvailabilityStatus, DataProviderDraft, DataProviderRecord,\n',
)
player_catalog = player_catalog.replace(
    '    MatchRecord, MatchStatus, PlayerAbilityObservationDraft, PlayerAbilityObservationRecord,\n',
    '    MatchRecord, MatchStatus,\n',
)
player_catalog_path.write_text(player_catalog, encoding='utf-8')

ability_add = ability_add.replace(
    'let row = sqlx::query(',
    'let row = sqlx::query_as::<_, PlayerAbilityObservationRow>(',
    1,
)
ability_add = ability_add.replace(
    '        player_ability_observation_from_row(&row)',
    '        Ok(map_player_ability_observation(row))',
)
inline_ability_validation = '''        if !(0.0..=1.0).contains(&draft.confidence) || draft.sample_size < 0 {
            return Err(PersistenceError::InvalidState(
                "能力观察可信度或样本量无效".to_string(),
            ));
        }
        if draft
            .effective_to
            .as_ref()
            .is_some_and(|value| value < &draft.effective_from)
        {
            return Err(PersistenceError::InvalidState(
                "能力观察失效时间不能早于生效时间".to_string(),
            ));
        }
'''
if inline_ability_validation not in ability_add:
    raise SystemExit('ability inline validation block not found')
ability_add = ability_add.replace(
    inline_ability_validation,
    '        validate_player_ability_observation(draft)?;\n',
    1,
)
write(
    'crates/persistence-postgres/src/adapters/catalog/abilities/observations/add.rs',
    f'''use super::{{
    input_policy::validate_player_ability_observation,
    mapper::map_player_ability_observation,
    row::PlayerAbilityObservationRow,
}};
use crate::{{PersistenceError, PersistenceResult, PostgresStore}};
use football_domain::{{PlayerAbilityObservationDraft, PlayerAbilityObservationRecord}};
use uuid::Uuid;

impl PostgresStore {{
{ability_add}
}}
'''.replace('use crate::{PersistenceError, PersistenceResult, PostgresStore};', 'use crate::{PersistenceError, PersistenceResult, PostgresStore};'),
)
write(
    'crates/persistence-postgres/src/adapters/catalog/abilities/observations/input_policy.rs',
    '''use crate::{PersistenceError, PersistenceResult};
use football_domain::PlayerAbilityObservationDraft;

pub(super) fn validate_player_ability_observation(
    draft: &PlayerAbilityObservationDraft,
) -> PersistenceResult<()> {
    if !(0.0..=1.0).contains(&draft.confidence) || draft.sample_size < 0 {
        return Err(PersistenceError::InvalidState(
            "能力观察可信度或样本量无效".to_string(),
        ));
    }
    if draft
        .effective_to
        .as_ref()
        .is_some_and(|value| value < &draft.effective_from)
    {
        return Err(PersistenceError::InvalidState(
            "能力观察失效时间不能早于生效时间".to_string(),
        ));
    }
    Ok(())
}
''',
)
write(
    'crates/persistence-postgres/src/adapters/catalog/abilities/observations/row.rs',
    '''use chrono::{DateTime, Utc};
use sqlx::FromRow;
use uuid::Uuid;

#[derive(Debug, FromRow)]
pub(in crate::adapters::catalog) struct PlayerAbilityObservationRow {
    pub id: Uuid,
    pub player_id: Uuid,
    pub dimension_code: String,
    pub dimension_name: String,
    pub context_type: String,
    pub context_id: Option<Uuid>,
    pub value: f64,
    pub confidence: f64,
    pub sample_size: i32,
    pub observed_at: DateTime<Utc>,
    pub effective_from: DateTime<Utc>,
    pub effective_to: Option<DateTime<Utc>>,
    pub calculation_version: String,
}
''',
)
write(
    'crates/persistence-postgres/src/adapters/catalog/abilities/observations/mapper.rs',
    '''use super::row::PlayerAbilityObservationRow;
use football_domain::PlayerAbilityObservationRecord;

pub(in crate::adapters::catalog) fn map_player_ability_observation(
    row: PlayerAbilityObservationRow,
) -> PlayerAbilityObservationRecord {
    PlayerAbilityObservationRecord {
        id: row.id,
        player_id: row.player_id,
        dimension_code: row.dimension_code,
        dimension_name: row.dimension_name,
        context_type: row.context_type,
        context_id: row.context_id,
        value: row.value,
        confidence: row.confidence,
        sample_size: row.sample_size,
        observed_at: row.observed_at,
        effective_from: row.effective_from,
        effective_to: row.effective_to,
        calculation_version: row.calculation_version,
    }
}
''',
)
write(
    'crates/persistence-postgres/src/adapters/catalog/abilities/observations/mod.rs',
    '''mod add;
mod input_policy;
mod mapper;
mod row;

pub(in crate::adapters::catalog) use mapper::map_player_ability_observation;
pub(in crate::adapters::catalog) use row::PlayerAbilityObservationRow;
''',
)

ability_list = ability_list.replace(
    'let rows = sqlx::query(',
    'let rows = sqlx::query_as::<_, AbilityDimensionRow>(',
    1,
)
ability_list = ability_list.replace(
    '        rows.iter().map(ability_dimension_from_row).collect()',
    '        Ok(rows.into_iter().map(map_ability_dimension).collect())',
)
write(
    'crates/persistence-postgres/src/adapters/catalog/abilities/dimensions/list.rs',
    f'''use super::{{mapper::map_ability_dimension, row::AbilityDimensionRow}};
use crate::{{PersistenceResult, PostgresStore}};
use football_domain::AbilityDimensionRecord;

impl PostgresStore {{
{ability_list}
}}
''',
)
write(
    'crates/persistence-postgres/src/adapters/catalog/abilities/dimensions/row.rs',
    '''use sqlx::FromRow;

#[derive(Debug, FromRow)]
pub(super) struct AbilityDimensionRow {
    pub code: String,
    pub name: String,
    pub category: String,
    pub minimum_value: f64,
    pub maximum_value: f64,
    pub description: Option<String>,
}
''',
)
write(
    'crates/persistence-postgres/src/adapters/catalog/abilities/dimensions/mapper.rs',
    '''use super::row::AbilityDimensionRow;
use football_domain::AbilityDimensionRecord;

pub(super) fn map_ability_dimension(row: AbilityDimensionRow) -> AbilityDimensionRecord {
    AbilityDimensionRecord {
        code: row.code,
        name: row.name,
        category: row.category,
        minimum_value: row.minimum_value,
        maximum_value: row.maximum_value,
        description: row.description,
    }
}
''',
)
write(
    'crates/persistence-postgres/src/adapters/catalog/abilities/dimensions/mod.rs',
    'mod list;\nmod mapper;\nmod row;\n',
)
write(
    'crates/persistence-postgres/src/adapters/catalog/abilities/mod.rs',
    'mod dimensions;\npub(crate) mod observations;\n',
)

write(
    'crates/persistence-postgres/src/adapters/catalog/players/detail/abilities/observations/read.rs',
    '''use crate::{
    adapters::catalog::abilities::observations::{
        map_player_ability_observation, PlayerAbilityObservationRow,
    },
    PersistenceResult,
};
use football_domain::PlayerAbilityObservationRecord;
use sqlx::PgPool;
use uuid::Uuid;

pub(in crate::adapters::catalog::players::detail) async fn read_ability_observations(
    pool: &PgPool,
    player_id: Uuid,
) -> PersistenceResult<Vec<PlayerAbilityObservationRecord>> {
    let rows = sqlx::query_as::<_, PlayerAbilityObservationRow>(
        r#"
        SELECT
            observation.id, observation.player_id, observation.dimension_code,
            dimension.name AS dimension_name, observation.context_type,
            observation.context_id, observation.value, observation.confidence,
            observation.sample_size, observation.observed_at,
            observation.effective_from, observation.effective_to,
            observation.calculation_version
        FROM feature.player_ability_observations observation
        JOIN feature.player_ability_dimensions dimension
          ON dimension.code = observation.dimension_code
        WHERE observation.player_id = $1
        ORDER BY observation.observed_at DESC, observation.id DESC
        LIMIT 250
        "#,
    )
    .bind(player_id)
    .fetch_all(pool)
    .await?;
    Ok(rows
        .into_iter()
        .map(map_player_ability_observation)
        .collect())
}
''',
)
write(
    'crates/persistence-postgres/src/adapters/catalog/players/detail/abilities/observations/mod.rs',
    'mod read;\n\npub(in crate::adapters::catalog::players::detail) use read::read_ability_observations;\n',
)
(root / 'crates/persistence-postgres/src/adapters/catalog/players/detail/abilities/observations/mapper.rs').unlink()
(root / 'crates/persistence-postgres/src/adapters/catalog/players/detail/abilities/observations/row.rs').unlink()

# --- Dynamic Tag owners ---
legacy_dynamic_path = root / 'crates/persistence-postgres/src/dynamic_tags.rs'
dynamic = legacy_dynamic_path.read_text(encoding='utf-8')
list_defs = between(
    dynamic,
    '    pub async fn list_dynamic_tag_definitions(',
    '\n\n    pub async fn add_player_dynamic_tag(',
)
add_tag = between(
    dynamic,
    '    pub async fn add_player_dynamic_tag(',
    '\n\n    pub async fn read_player_dynamic_tag(',
)
read_tag = between(
    dynamic,
    '    pub async fn read_player_dynamic_tag(',
    '\n\n    pub async fn list_player_dynamic_tags(',
)
list_tags = between(
    dynamic,
    '    pub async fn list_player_dynamic_tags(',
    '\n\n    pub async fn calculate_player_match_contribution(',
)
contribution = between(
    dynamic,
    '    pub async fn calculate_player_match_contribution(',
    '\n    }\n}\n\nasync fn validate_dynamic_tag_draft(',
) + '\n    }'
validate_tag = between(
    dynamic,
    'async fn validate_dynamic_tag_draft(',
    '\n\nfn dynamic_tag_definition_from_row(',
)
availability_helper = between(dynamic, 'fn availability_multiplier(', '\n\nfn tag_value(')
tag_value_helper = between(dynamic, 'fn tag_value(', '\n\nfn component(')
component_helper = between(dynamic, 'fn component(', '\n\nfn component_from_tag(')
component_from_tag_helper = dynamic[dynamic.index('fn component_from_tag('):].rstrip()

list_defs = list_defs.replace(
    'let rows = sqlx::query(',
    'let rows = sqlx::query_as::<_, DynamicTagDefinitionRow>(',
    1,
)
list_defs = list_defs.replace(
    '        rows.iter().map(dynamic_tag_definition_from_row).collect()',
    '        Ok(rows.into_iter().map(map_dynamic_tag_definition).collect())',
)
write(
    'crates/persistence-postgres/src/adapters/catalog/dynamic_tags/definitions/list.rs',
    f'''use super::{{mapper::map_dynamic_tag_definition, row::DynamicTagDefinitionRow}};
use crate::{{PersistenceResult, PostgresStore}};
use football_domain::PlayerDynamicTagDefinitionRecord;

impl PostgresStore {{
{list_defs}
}}
''',
)
write(
    'crates/persistence-postgres/src/adapters/catalog/dynamic_tags/definitions/row.rs',
    '''use sqlx::FromRow;

#[derive(Debug, FromRow)]
pub(super) struct DynamicTagDefinitionRow {
    pub code: String,
    pub name: String,
    pub category: String,
    pub minimum_value: f64,
    pub maximum_value: f64,
    pub default_value: f64,
    pub default_ttl_hours: i32,
    pub is_multiplier: bool,
    pub description: Option<String>,
}
''',
)
write(
    'crates/persistence-postgres/src/adapters/catalog/dynamic_tags/definitions/mapper.rs',
    '''use super::row::DynamicTagDefinitionRow;
use football_domain::PlayerDynamicTagDefinitionRecord;

pub(super) fn map_dynamic_tag_definition(
    row: DynamicTagDefinitionRow,
) -> PlayerDynamicTagDefinitionRecord {
    PlayerDynamicTagDefinitionRecord {
        code: row.code,
        name: row.name,
        category: row.category,
        minimum_value: row.minimum_value,
        maximum_value: row.maximum_value,
        default_value: row.default_value,
        default_ttl_hours: row.default_ttl_hours,
        is_multiplier: row.is_multiplier,
        description: row.description,
    }
}
''',
)
write(
    'crates/persistence-postgres/src/adapters/catalog/dynamic_tags/definitions/mod.rs',
    'mod list;\nmod mapper;\nmod row;\n',
)

write(
    'crates/persistence-postgres/src/adapters/catalog/dynamic_tags/tags/add.rs',
    f'''use super::input_policy::validate_dynamic_tag_draft;
use crate::{{PersistenceResult, PostgresStore}};
use football_domain::{{PlayerDynamicTagDraft, PlayerDynamicTagRecord}};
use sqlx::Row;
use uuid::Uuid;

impl PostgresStore {{
{add_tag}
}}
''',
)
write(
    'crates/persistence-postgres/src/adapters/catalog/dynamic_tags/tags/input_policy.rs',
    f'''use crate::{{PersistenceError, PersistenceResult, PostgresStore}};
use football_domain::PlayerDynamicTagDraft;
use sqlx::Row;

pub(super) {validate_tag}
''',
)
read_tag = read_tag.replace(
    'let row = sqlx::query(',
    'let row = sqlx::query_as::<_, PlayerDynamicTagRow>(',
    1,
)
read_tag = read_tag.replace(
    '        player_dynamic_tag_from_row(&row)',
    '        Ok(map_player_dynamic_tag(row))',
)
write(
    'crates/persistence-postgres/src/adapters/catalog/dynamic_tags/tags/read.rs',
    f'''use super::{{mapper::map_player_dynamic_tag, row::PlayerDynamicTagRow}};
use crate::{{PersistenceResult, PostgresStore}};
use football_domain::PlayerDynamicTagRecord;
use uuid::Uuid;

impl PostgresStore {{
{read_tag}
}}
''',
)
list_tags = list_tags.replace(
    'let rows = sqlx::query(',
    'let rows = sqlx::query_as::<_, PlayerDynamicTagRow>(',
    1,
)
list_tags = list_tags.replace(
    '        rows.iter().map(player_dynamic_tag_from_row).collect()',
    '        Ok(rows.into_iter().map(map_player_dynamic_tag).collect())',
)
write(
    'crates/persistence-postgres/src/adapters/catalog/dynamic_tags/tags/list.rs',
    f'''use super::{{mapper::map_player_dynamic_tag, row::PlayerDynamicTagRow}};
use crate::{{PersistenceResult, PostgresStore}};
use chrono::{{DateTime, Utc}};
use football_domain::PlayerDynamicTagRecord;
use uuid::Uuid;

impl PostgresStore {{
{list_tags}
}}
''',
)
write(
    'crates/persistence-postgres/src/adapters/catalog/dynamic_tags/tags/row.rs',
    '''use chrono::{DateTime, Utc};
use serde_json::Value;
use sqlx::FromRow;
use uuid::Uuid;

#[derive(Debug, FromRow)]
pub(super) struct PlayerDynamicTagRow {
    pub id: Uuid,
    pub player_id: Uuid,
    pub tag_code: String,
    pub tag_name: String,
    pub category: String,
    pub value: f64,
    pub label: Option<String>,
    pub confidence: f64,
    pub observed_at: DateTime<Utc>,
    pub valid_from: DateTime<Utc>,
    pub valid_to: DateTime<Utc>,
    pub competition_id: Option<Uuid>,
    pub competition_name: Option<String>,
    pub position_code: Option<String>,
    pub opponent_team_id: Option<Uuid>,
    pub opponent_team_name: Option<String>,
    pub sample_size: i32,
    pub source_type: String,
    pub calculation_version: String,
    pub metadata: Value,
}
''',
)
write(
    'crates/persistence-postgres/src/adapters/catalog/dynamic_tags/tags/mapper.rs',
    '''use super::row::PlayerDynamicTagRow;
use football_domain::PlayerDynamicTagRecord;

pub(super) fn map_player_dynamic_tag(row: PlayerDynamicTagRow) -> PlayerDynamicTagRecord {
    PlayerDynamicTagRecord {
        id: row.id,
        player_id: row.player_id,
        tag_code: row.tag_code,
        tag_name: row.tag_name,
        category: row.category,
        value: row.value,
        label: row.label,
        confidence: row.confidence,
        observed_at: row.observed_at,
        valid_from: row.valid_from,
        valid_to: row.valid_to,
        competition_id: row.competition_id,
        competition_name: row.competition_name,
        position_code: row.position_code,
        opponent_team_id: row.opponent_team_id,
        opponent_team_name: row.opponent_team_name,
        sample_size: row.sample_size,
        source_type: row.source_type,
        calculation_version: row.calculation_version,
        metadata: row.metadata,
    }
}
''',
)
write(
    'crates/persistence-postgres/src/adapters/catalog/dynamic_tags/tags/mod.rs',
    '''mod add;
mod input_policy;
mod list;
mod mapper;
mod read;
mod row;

pub(super) use mapper::map_player_dynamic_tag;
pub(super) use row::PlayerDynamicTagRow;
''',
)

contribution = contribution.replace(
    'let tag_rows = sqlx::query(',
    'let tag_rows = sqlx::query_as::<_, PlayerDynamicTagRow>(',
    1,
)
contribution = contribution.replace(
    '''        let applied_tags = tag_rows
            .iter()
            .map(player_dynamic_tag_from_row)
            .collect::<PersistenceResult<Vec<_>>>()?;''',
    '''        let applied_tags = tag_rows
            .into_iter()
            .map(map_player_dynamic_tag)
            .collect::<Vec<_>>();''',
)
write(
    'crates/persistence-postgres/src/adapters/catalog/dynamic_tags/contribution/calculate.rs',
    f'''use super::scoring::{{
    availability_multiplier, component, component_from_tag, tag_value, CONTRIBUTION_VERSION,
}};
use crate::{{
    adapters::catalog::dynamic_tags::tags::{{map_player_dynamic_tag, PlayerDynamicTagRow}},
    role_resolution::{{
        normalize_role_origin, resolve_tactical_role, tactical_role_confidence,
        DefaultTacticalRole, ResolvedTacticalRole, ROLE_ORIGIN_PLAYER_POSITION_DEFAULT,
    }},
    PersistenceResult, PostgresStore,
}};
use football_domain::{{PlayerMatchContribution, PlayerMatchContributionRequest}};
use sqlx::Row;
use std::collections::HashMap;

impl PostgresStore {{
{contribution}
}}
''',
)
write(
    'crates/persistence-postgres/src/adapters/catalog/dynamic_tags/contribution/scoring.rs',
    f'''use football_domain::{{ContributionComponent, PlayerDynamicTagRecord}};
use std::collections::HashMap;

pub(super) const CONTRIBUTION_VERSION: &str = "match-contribution-v2-role-context";

pub(super) {availability_helper}

pub(super) {tag_value_helper}

pub(super) {component_helper}

pub(super) {component_from_tag_helper}
''',
)
write(
    'crates/persistence-postgres/src/adapters/catalog/dynamic_tags/contribution/mod.rs',
    'mod calculate;\nmod scoring;\n',
)
write(
    'crates/persistence-postgres/src/adapters/catalog/dynamic_tags/mod.rs',
    'mod contribution;\nmod definitions;\nmod tags;\n',
)
legacy_dynamic_path.unlink()

catalog_mod = root / 'crates/persistence-postgres/src/adapters/catalog/mod.rs'
text = catalog_mod.read_text(encoding='utf-8')
if 'mod abilities;' not in text:
    text = 'mod abilities;\n' + text
if 'mod dynamic_tags;' not in text:
    text = text.replace('mod availability;\n', 'mod availability;\nmod dynamic_tags;\n', 1)
catalog_mod.write_text(text, encoding='utf-8')

lib_path = root / 'crates/persistence-postgres/src/lib.rs'
lib = lib_path.read_text(encoding='utf-8')
lib = lib.replace('mod dynamic_tags;\n', '', 1)
lib_path.write_text(lib, encoding='utf-8')

retained_path = root / 'scripts/verify-player-team-periods-availability.mjs'
retained = retained_path.read_text(encoding='utf-8')
retained_guard = '''check(legacy.includes("pub async fn add_player_ability_observation"), "R6-06 Ability owner 被提前迁移或删除");
const dynamicTags = read("crates/persistence-postgres/src/dynamic_tags.rs");
check(dynamicTags.includes("pub async fn add_player_dynamic_tag"), "R6-06 Dynamic Tag owner 被提前迁移或删除");
'''
if retained_guard not in retained:
    raise SystemExit('R6-05 temporal R6-06 guard not found')
retained = retained.replace(retained_guard, '', 1)
retained_path.write_text(retained, encoding='utf-8')

verifier = '''import fs from "node:fs";
import path from "node:path";

const root = path.resolve(import.meta.dirname, "..");
const read = (file) => fs.readFileSync(path.join(root, file), "utf8");
const exists = (file) => fs.existsSync(path.join(root, file));
const failures = [];
const check = (condition, message) => { if (!condition) failures.push(message); };
const required = [
  "crates/persistence-postgres/src/adapters/catalog/abilities/dimensions/list.rs",
  "crates/persistence-postgres/src/adapters/catalog/abilities/dimensions/mapper.rs",
  "crates/persistence-postgres/src/adapters/catalog/abilities/dimensions/row.rs",
  "crates/persistence-postgres/src/adapters/catalog/abilities/observations/add.rs",
  "crates/persistence-postgres/src/adapters/catalog/abilities/observations/input_policy.rs",
  "crates/persistence-postgres/src/adapters/catalog/abilities/observations/mapper.rs",
  "crates/persistence-postgres/src/adapters/catalog/abilities/observations/row.rs",
  "crates/persistence-postgres/src/adapters/catalog/dynamic_tags/definitions/list.rs",
  "crates/persistence-postgres/src/adapters/catalog/dynamic_tags/definitions/mapper.rs",
  "crates/persistence-postgres/src/adapters/catalog/dynamic_tags/definitions/row.rs",
  "crates/persistence-postgres/src/adapters/catalog/dynamic_tags/tags/add.rs",
  "crates/persistence-postgres/src/adapters/catalog/dynamic_tags/tags/input_policy.rs",
  "crates/persistence-postgres/src/adapters/catalog/dynamic_tags/tags/list.rs",
  "crates/persistence-postgres/src/adapters/catalog/dynamic_tags/tags/read.rs",
  "crates/persistence-postgres/src/adapters/catalog/dynamic_tags/tags/mapper.rs",
  "crates/persistence-postgres/src/adapters/catalog/dynamic_tags/tags/row.rs",
  "crates/persistence-postgres/src/adapters/catalog/dynamic_tags/contribution/calculate.rs",
  "crates/persistence-postgres/src/adapters/catalog/dynamic_tags/contribution/scoring.rs",
  "crates/persistence-postgres/tests/player_abilities_dynamic_tags_repository_contract.rs",
];
for (const file of required) check(exists(file), `R6-06 required file missing: ${file}`);
const legacy = read("crates/persistence-postgres/src/player_catalog.rs");
const lib = read("crates/persistence-postgres/src/lib.rs");
const catalog = read("crates/persistence-postgres/src/adapters/catalog/mod.rs");
const detailRead = read("crates/persistence-postgres/src/adapters/catalog/players/detail/abilities/observations/read.rs");
const port = read("crates/application/src/ports/player/mod.rs");
const adapter = read("crates/application/src/composition/adapters/players.rs");
check(!legacy.includes("pub async fn add_player_ability_observation"), "legacy player_catalog.rs 仍拥有 ability write");
check(!legacy.includes("pub async fn list_ability_dimensions"), "legacy player_catalog.rs 仍拥有 ability dimension query");
check(!legacy.includes("fn player_ability_observation_from_row"), "legacy player_catalog.rs 仍拥有 ability observation mapper");
check(!legacy.includes("fn ability_dimension_from_row"), "legacy player_catalog.rs 仍拥有 ability dimension mapper");
check(!exists("crates/persistence-postgres/src/dynamic_tags.rs"), "旧 dynamic_tags.rs 单文件 owner 尚未删除");
check(!lib.includes("mod dynamic_tags;"), "lib.rs 仍注册旧 dynamic_tags root owner");
check(catalog.includes("mod abilities;") && catalog.includes("mod dynamic_tags;"), "catalog 未注册 R6-06 owners");
check(detailRead.includes("catalog::abilities::observations"), "Player Detail ability read 未复用新 Row/mapper owner");
check(!exists("crates/persistence-postgres/src/adapters/catalog/players/detail/abilities/observations/mapper.rs"), "Detail ability duplicate mapper 未删除");
check(!exists("crates/persistence-postgres/src/adapters/catalog/players/detail/abilities/observations/row.rs"), "Detail ability duplicate row 未删除");
check(port.includes("async fn add_ability_observation(") && port.includes("async fn add_dynamic_tag(") && port.includes("async fn calculate_match_contribution("), "PlayerSignalPort 公共契约变化");
check(adapter.includes("self.add_player_ability_observation(draft)") && adapter.includes("self.add_player_dynamic_tag(draft)") && adapter.includes("self.calculate_player_match_contribution(request)"), "Application adapter 调用语义变化");
for (const modFile of [
  "crates/persistence-postgres/src/adapters/catalog/abilities/mod.rs",
  "crates/persistence-postgres/src/adapters/catalog/abilities/observations/mod.rs",
  "crates/persistence-postgres/src/adapters/catalog/dynamic_tags/mod.rs",
  "crates/persistence-postgres/src/adapters/catalog/dynamic_tags/tags/mod.rs",
  "crates/persistence-postgres/src/adapters/catalog/dynamic_tags/contribution/mod.rs",
]) {
  const text = read(modFile);
  check(!text.includes("sqlx::") && !text.includes("impl PostgresStore"), `${modFile} 不得承载 SQL/业务实现`);
}
if (failures.length) {
  console.error("R6-06 Abilities / Dynamic Tags ownership verification failed:");
  failures.forEach((failure) => console.error(`- ${failure}`));
  process.exit(1);
}
console.log("R6-06 Abilities / Dynamic Tags ownership verified: dimensions, observations, tag definitions, tag writes/reads and contribution scoring have unique modular owners.");
'''
write('scripts/verify-player-abilities-dynamic-tags.mjs', verifier)

package_path = root / 'package.json'
package = json.loads(package_path.read_text(encoding='utf-8'))
architecture = package['scripts']['verify:architecture']
token = 'node scripts/verify-player-abilities-dynamic-tags.mjs'
if token not in architecture:
    package['scripts']['verify:architecture'] = architecture + ' && ' + token
package['scripts']['verify:player-abilities-dynamic-tags'] = token
package_path.write_text(json.dumps(package, ensure_ascii=False, indent=2) + '\n', encoding='utf-8')

contract = r'''use chrono::{Duration, Utc};
use football_domain::{
    PlayerAbilityObservationDraft, PlayerDraft, PlayerDynamicTagDraft,
    PlayerMatchContributionRequest, PlayerStatus, PreferredFoot,
};
use football_persistence_postgres::{DatabaseOptions, PersistenceError, PostgresStore};
use serde_json::json;
use sqlx::{postgres::PgPoolOptions, PgPool, Row};
use uuid::Uuid;

const DATABASE_ENV: &str = "FOOTBALL_TEST_DATABASE_URL";

struct TestDatabase {
    store: PostgresStore,
    pool: PgPool,
}

impl TestDatabase {
    async fn connect() -> Self {
        let connection_url = std::env::var(DATABASE_ENV).unwrap_or_else(|_| {
            panic!("运行 R6-06 契约测试前必须设置 {DATABASE_ENV}，并指向专用、可写的 PostgreSQL 测试数据库")
        });
        let options = DatabaseOptions {
            connection_url: connection_url.clone(),
            max_connections: 4,
            connect_timeout_seconds: 10,
        };
        let store = PostgresStore::connect(&options)
            .await
            .expect("连接 R6-06 PostgreSQL 测试数据库");
        store.migrate().await.expect("执行数据库迁移");
        let pool = PgPoolOptions::new()
            .max_connections(4)
            .connect(&connection_url)
            .await
            .expect("建立 R6-06 校验连接池");
        Self { store, pool }
    }

    async fn close(self) {
        self.pool.close().await;
        self.store.close().await;
    }
}

#[tokio::test]
#[ignore = "需要专用且可写的 PostgreSQL 测试数据库；设置 FOOTBALL_TEST_DATABASE_URL 后显式运行"]
async fn player_abilities_and_dynamic_tags_contract_is_preserved() {
    let database = TestDatabase::connect().await;
    let token = Uuid::new_v4().simple().to_string();
    let player = database
        .store
        .create_player(&PlayerDraft {
            canonical_name: format!("R6-06 Player {token}"),
            date_of_birth: None,
            nationality_code: Some("ZZ".to_string()),
            preferred_foot: PreferredFoot::Right,
            height_cm: Some(181),
            status: PlayerStatus::Active,
            metadata: json!({"contract": "r6-06"}),
        })
        .await
        .expect("创建 R6-06 contract player");

    let dimensions = database
        .store
        .list_ability_dimensions()
        .await
        .expect("读取能力维度");
    assert!(!dimensions.is_empty(), "迁移基线必须提供能力维度");
    let dimension = &dimensions[0];
    let ability_value = (dimension.minimum_value + dimension.maximum_value) / 2.0;
    let observed_at = Utc::now();
    let observation = database
        .store
        .add_player_ability_observation(&PlayerAbilityObservationDraft {
            player_id: player.id,
            dimension_code: format!(" {} ", dimension.code),
            context_type: " overall ".to_string(),
            context_id: None,
            value: ability_value,
            confidence: 0.8,
            sample_size: 5,
            observed_at,
            effective_from: observed_at,
            effective_to: Some(observed_at + Duration::days(1)),
            calculation_version: " r6-06-contract ".to_string(),
            source_document_id: None,
            metadata: json!({"contract": "r6-06"}),
        })
        .await
        .expect("写入能力观察");
    assert_eq!(observation.player_id, player.id);
    assert_eq!(observation.dimension_code, dimension.code);
    assert_eq!(observation.context_type, "overall");
    assert_eq!(observation.calculation_version, "r6-06-contract");

    let invalid_observation = database
        .store
        .add_player_ability_observation(&PlayerAbilityObservationDraft {
            player_id: player.id,
            dimension_code: dimension.code.clone(),
            context_type: "overall".to_string(),
            context_id: None,
            value: ability_value,
            confidence: 1.1,
            sample_size: 5,
            observed_at,
            effective_from: observed_at,
            effective_to: None,
            calculation_version: "r6-06-contract".to_string(),
            source_document_id: None,
            metadata: json!({}),
        })
        .await
        .expect_err("非法 ability confidence 必须失败");
    assert!(matches!(
        invalid_observation,
        PersistenceError::InvalidState(message) if message == "能力观察可信度或样本量无效"
    ));

    let definitions = database
        .store
        .list_dynamic_tag_definitions()
        .await
        .expect("读取动态标签定义");
    assert!(!definitions.is_empty(), "迁移基线必须提供动态标签定义");
    let definition = &definitions[0];
    let now = Utc::now();
    let tag = database
        .store
        .add_player_dynamic_tag(&PlayerDynamicTagDraft {
            player_id: player.id,
            tag_code: format!(" {} ", definition.code),
            value: definition.default_value,
            label: Some("contract".to_string()),
            confidence: 0.9,
            observed_at: now,
            valid_from: now - Duration::hours(1),
            valid_to: now + Duration::hours(2),
            competition_id: None,
            position_code: None,
            opponent_team_id: None,
            sample_size: 3,
            source_type: " manual ".to_string(),
            calculation_version: " r6-06-contract ".to_string(),
            source_document_id: None,
            metadata: json!({"contract": "r6-06"}),
        })
        .await
        .expect("写入动态标签");
    assert_eq!(tag.player_id, player.id);
    assert_eq!(tag.tag_code, definition.code);
    assert_eq!(tag.source_type, "manual");
    assert_eq!(tag.calculation_version, "r6-06-contract");

    let listed = database
        .store
        .list_player_dynamic_tags(player.id, now)
        .await
        .expect("读取动态标签");
    assert!(listed.iter().any(|item| item.id == tag.id));
    let reread = database
        .store
        .read_player_dynamic_tag(tag.id)
        .await
        .expect("按 ID 读取动态标签");
    assert_eq!(reread.id, tag.id);

    let invalid_tag = database
        .store
        .add_player_dynamic_tag(&PlayerDynamicTagDraft {
            player_id: player.id,
            tag_code: definition.code.clone(),
            value: definition.default_value,
            label: None,
            confidence: 1.1,
            observed_at: now,
            valid_from: now,
            valid_to: now + Duration::hours(1),
            competition_id: None,
            position_code: None,
            opponent_team_id: None,
            sample_size: 1,
            source_type: "manual".to_string(),
            calculation_version: "r6-06-contract".to_string(),
            source_document_id: None,
            metadata: json!({}),
        })
        .await
        .expect_err("非法 dynamic tag confidence 必须失败");
    assert!(matches!(
        invalid_tag,
        PersistenceError::InvalidState(message) if message == "动态标签 confidence 必须在 0–1 之间"
    ));

    let contribution = database
        .store
        .calculate_player_match_contribution(&PlayerMatchContributionRequest {
            player_id: player.id,
            match_id: None,
            competition_id: None,
            position_code: None,
            role_code: None,
            role_origin: None,
            role_source_position_code: None,
            opponent_team_id: None,
            as_of: now,
            data_cutoff_time: None,
            expected_minutes: Some(90),
        })
        .await
        .expect("计算球员比赛贡献");
    assert_eq!(contribution.player_id, player.id);
    assert_eq!(
        contribution.calculation_version,
        "match-contribution-v2-role-context"
    );
    assert!(contribution.effective_contribution >= 0.0);

    let row = sqlx::query(
        "SELECT count(*)::bigint AS count FROM feature.player_ability_observations WHERE player_id=$1",
    )
    .bind(player.id)
    .fetch_one(&database.pool)
    .await
    .expect("读取能力观察落库数量");
    let observation_count: i64 = row.try_get("count").expect("读取 count");
    assert!(observation_count >= 1);

    database.close().await;
}
'''
write(
    'crates/persistence-postgres/tests/player_abilities_dynamic_tags_repository_contract.rs',
    contract,
)

doc = '''# R06-06 — Abilities 与 Dynamic Tags

## 状态

`VERIFYING`

## 进入基线

- R6-05：`DONE`。
- 唯一阶段分支：`rewrite/r6-entity-catalog-persistence`。
- 节点起点：`f48841a8a50f51cd71e09969d858b95e7d9ba9fa`。

## 已实施职责边界

- `adapters/catalog/abilities/dimensions/`：能力维度 query、typed Row 与 Domain mapper。
- `adapters/catalog/abilities/observations/`：能力观察输入策略、写 SQL、typed Row 与 Domain mapper；Player Detail 只保留读取 SQL并复用该 Row/mapper owner。
- `adapters/catalog/dynamic_tags/definitions/`：动态标签定义读取。
- `adapters/catalog/dynamic_tags/tags/`：动态标签输入策略、写入、单条读取、时点列表、typed Row 与 mapper。
- `adapters/catalog/dynamic_tags/contribution/`：比赛贡献计算编排与纯评分组件分责。
- 旧 `dynamic_tags.rs` 单文件 owner 已删除；`player_catalog.rs` 已退出 R6-06 Abilities 职责。

## 保持不变的契约

- `PlayerSignalPort` 的 Ability、Dynamic Tag 与 Match Contribution 方法签名及 Application adapter 调用语义不变。
- Domain DTO、Schema、0001–0046 migration、配置、错误文案、默认战术角色/位置映射、历史 P4/运行引用和模型保护资产未主动修改。
- 未新增生产依赖。

## 验证

- R6-06 ownership verifier、R6-05 retained verifier、完整 architecture、模型保护、数据库基线、rustfmt、Persistence unit tests、Application check 与 PostgreSQL 16 专用 contract 均由本节点 implementation gate 执行；仅在全部成功后提交本源码树。
- 阶段 hard gate 与 clean canonical Public Platform CI 尚未执行，因此当前保持 `VERIFYING`。

## 尚未执行

- 用户现有 PostgreSQL 数据真实写入/sample 验收与 Windows Full 人工交互验收，继续留到最终统一验收。

## 回退点

- `f48841a8a50f51cd71e09969d858b95e7d9ba9fa`。
'''
write(
    'docs/modular-rewrite/R06-entity-catalog-persistence/R06-06-abilities-and-dynamic-tags.md',
    doc,
)

stage_path = root / 'docs/modular-rewrite/R06-entity-catalog-persistence/README.md'
stage = stage_path.read_text(encoding='utf-8')
stage = stage.replace(
    '| R6-06 | Abilities 与 Dynamic Tags | READY |',
    '| R6-06 | Abilities 与 Dynamic Tags | VERIFYING |',
    1,
)
if '## R6-06 当前事实' not in stage:
    stage += '''

## R6-06 当前事实

- 详细记录：[`R06-06-abilities-and-dynamic-tags.md`](R06-06-abilities-and-dynamic-tags.md)。
- Abilities 与 Dynamic Tags persistence 已按 dimensions / observations / definitions / tags / contribution 职责拆分，旧单文件 owner 删除；当前为 `VERIFYING`，等待阶段 hard gate 与 clean canonical CI。
'''
stage_path.write_text(stage, encoding='utf-8')

root_readme_path = root / 'README.md'
readme = root_readme_path.read_text(encoding='utf-8')
anchor = '## 模块化重写执行记录\n'
entry = '''
### R6-06 Abilities 与 Dynamic Tags

- Abilities persistence 已拆入 `adapters/catalog/abilities/{dimensions,observations}/`；Dynamic Tags 已拆入 `adapters/catalog/dynamic_tags/{definitions,tags,contribution}/`，旧 `dynamic_tags.rs` 单文件 owner 删除，Player Detail ability read 复用统一 typed Row/mapper。
- `PlayerSignalPort`、Domain DTO、Schema、0001–0046 migration、配置、错误语义、战术角色/位置映射、模型保护资产和生产依赖保持不变。
- 当前节点为 `VERIFYING`；implementation gate 必须实际通过 ownership/architecture/模型保护/数据库基线/Rust 与 PostgreSQL 16 contract 后才提交，阶段 hard gate 与 clean canonical CI 完成前不标记 `DONE`。

'''
if '### R6-06 Abilities 与 Dynamic Tags' not in readme:
    readme = readme.replace(anchor, anchor + entry, 1)
root_readme_path.write_text(readme, encoding='utf-8')
