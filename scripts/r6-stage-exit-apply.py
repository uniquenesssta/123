from pathlib import Path

root = Path('.')
src = root / 'crates/persistence-postgres/src'

entity = src / 'entity_catalog.rs'
player_catalog = src / 'player_catalog.rs'
lib = src / 'lib.rs'
team_detail = src / 'adapters/catalog/teams/detail'
positions = src / 'adapters/catalog/players/positions'
team_periods = src / 'adapters/catalog/players/team_periods'

if not entity.exists():
    raise SystemExit('expected entity_catalog.rs residual owner')
if not player_catalog.exists():
    raise SystemExit('expected player_catalog.rs R7 source')

# 1. Team-facing player-period projection: move the final entity_catalog read owner
# into Team Detail, because it returns TeamPlayerPeriodRecord (including player_name).
projection = team_detail / 'player_periods'
if projection.exists():
    raise SystemExit('team detail player_periods projection already exists')
projection.mkdir()
(projection / 'mod.rs').write_text('mod mapper;\nmod read;\nmod row;\n\npub(super) use read::read_player_periods;\n', encoding='utf-8')
(projection / 'row.rs').write_text('''use chrono::NaiveDate;\nuse sqlx::FromRow;\nuse uuid::Uuid;\n\n#[derive(Debug, FromRow)]\npub(super) struct TeamPlayerPeriodRow {\n    pub id: Uuid,\n    pub team_id: Uuid,\n    pub team_name: String,\n    pub player_id: Uuid,\n    pub player_name: String,\n    pub season_id: Option<Uuid>,\n    pub season_name: Option<String>,\n    pub squad_number: Option<i16>,\n    pub valid_from: NaiveDate,\n    pub valid_to: Option<NaiveDate>,\n    pub registration_status: String,\n}\n''', encoding='utf-8')
(projection / 'mapper.rs').write_text('''use super::row::TeamPlayerPeriodRow;\nuse football_domain::TeamPlayerPeriodRecord;\n\npub(super) fn map_team_player_period(row: TeamPlayerPeriodRow) -> TeamPlayerPeriodRecord {\n    TeamPlayerPeriodRecord {\n        id: row.id,\n        team_id: row.team_id,\n        team_name: row.team_name,\n        player_id: row.player_id,\n        player_name: row.player_name,\n        season_id: row.season_id,\n        season_name: row.season_name,\n        squad_number: row.squad_number,\n        valid_from: row.valid_from,\n        valid_to: row.valid_to,\n        registration_status: row.registration_status,\n    }\n}\n''', encoding='utf-8')
(projection / 'read.rs').write_text('''use super::{mapper::map_team_player_period, row::TeamPlayerPeriodRow};\nuse crate::PersistenceResult;\nuse football_domain::TeamPlayerPeriodRecord;\nuse uuid::Uuid;\n\npub(super) async fn read_player_periods(\n    pool: &sqlx::PgPool,\n    team_id: Uuid,\n) -> PersistenceResult<Vec<TeamPlayerPeriodRecord>> {\n    sqlx::query_as::<_, TeamPlayerPeriodRow>(\n        r#"\n        SELECT period.id, period.team_id, team.canonical_name AS team_name,\n               period.player_id, player.canonical_name AS player_name,\n               period.season_id, season.name AS season_name, period.squad_number,\n               period.valid_from, period.valid_to, period.registration_status\n        FROM football.player_team_periods period\n        JOIN football.teams team ON team.id=period.team_id\n        JOIN football.players player ON player.id=period.player_id\n        LEFT JOIN football.seasons season ON season.id=period.season_id\n        WHERE period.team_id=$1\n        ORDER BY period.valid_from DESC, period.valid_to DESC NULLS FIRST,\n                 player.normalized_name, period.id\n        "#,\n    )\n    .bind(team_id)\n    .fetch_all(pool)\n    .await?\n    .into_iter()\n    .map(map_team_player_period)\n    .collect::<Vec<_>>()\n    .pipe(Ok)\n}\n\ntrait Pipe: Sized {\n    fn pipe<T>(self, f: impl FnOnce(Self) -> T) -> T {\n        f(self)\n    }\n}\nimpl<T> Pipe for T {}\n''', encoding='utf-8')
# Replace the temporary pipe helper with straightforward result construction to keep this module idiomatic.
read_path = projection / 'read.rs'
read_text = read_path.read_text(encoding='utf-8')
read_text = read_text.replace('''    .await?\n    .into_iter()\n    .map(map_team_player_period)\n    .collect::<Vec<_>>()\n    .pipe(Ok)\n}\n\ntrait Pipe: Sized {\n    fn pipe<T>(self, f: impl FnOnce(Self) -> T) -> T {\n        f(self)\n    }\n}\nimpl<T> Pipe for T {}\n''', '''    .await?\n    .into_iter()\n    .map(map_team_player_period)\n    .collect::<Vec<_>>()\n    .into();\n    unreachable!()\n}\n''')
# Write final implementation explicitly; the two-step construction above guards against accidental template drift.
read_path.write_text('''use super::{mapper::map_team_player_period, row::TeamPlayerPeriodRow};\nuse crate::PersistenceResult;\nuse football_domain::TeamPlayerPeriodRecord;\nuse uuid::Uuid;\n\npub(super) async fn read_player_periods(\n    pool: &sqlx::PgPool,\n    team_id: Uuid,\n) -> PersistenceResult<Vec<TeamPlayerPeriodRecord>> {\n    let rows = sqlx::query_as::<_, TeamPlayerPeriodRow>(\n        r#"\n        SELECT period.id, period.team_id, team.canonical_name AS team_name,\n               period.player_id, player.canonical_name AS player_name,\n               period.season_id, season.name AS season_name, period.squad_number,\n               period.valid_from, period.valid_to, period.registration_status\n        FROM football.player_team_periods period\n        JOIN football.teams team ON team.id=period.team_id\n        JOIN football.players player ON player.id=period.player_id\n        LEFT JOIN football.seasons season ON season.id=period.season_id\n        WHERE period.team_id=$1\n        ORDER BY period.valid_from DESC, period.valid_to DESC NULLS FIRST,\n                 player.normalized_name, period.id\n        "#,\n    )\n    .bind(team_id)\n    .fetch_all(pool)\n    .await?;\n    Ok(rows.into_iter().map(map_team_player_period).collect())\n}\n''', encoding='utf-8')

team_mod = team_detail / 'mod.rs'
text = team_mod.read_text(encoding='utf-8')
anchor = 'mod name_mapper;\n'
if 'mod player_periods;' not in text:
    if anchor not in text:
        raise SystemExit('team detail mod anchor missing')
    text = text.replace(anchor, anchor + 'mod player_periods;\n', 1)
team_mod.write_text(text, encoding='utf-8')

read_team = team_detail / 'read_team.rs'
text = read_team.read_text(encoding='utf-8')
old_use = 'use super::{\n    read_names::read_names, read_profile::read_profile, read_recent_matches::read_recent_matches,\n    read_squad::read_squad, read_team_record::read_team_record,\n};'
new_use = 'use super::{\n    player_periods::read_player_periods, read_names::read_names, read_profile::read_profile,\n    read_recent_matches::read_recent_matches, read_squad::read_squad,\n    read_team_record::read_team_record,\n};'
if old_use not in text:
    raise SystemExit('read_team import block changed unexpectedly')
text = text.replace(old_use, new_use, 1)
if 'let player_periods = self.list_team_player_periods(team_id).await?;' not in text:
    raise SystemExit('old team period call missing')
text = text.replace('let player_periods = self.list_team_player_periods(team_id).await?;', 'let player_periods = read_player_periods(&self.pool, team_id).await?;', 1)
read_team.write_text(text, encoding='utf-8')

# 2. Position-reference list: it is catalog/players/positions persistence, not R7 Match/Lineup.
(positions / 'reference_row.rs').write_text('''use sqlx::FromRow;\n\n#[derive(Debug, FromRow)]\npub(super) struct PositionReferenceRow {\n    pub code: String,\n    pub name: String,\n    pub position_group: String,\n    pub sort_order: i16,\n}\n''', encoding='utf-8')
(positions / 'reference_mapper.rs').write_text('''use super::reference_row::PositionReferenceRow;\nuse football_domain::PositionReference;\n\npub(super) fn map_position_reference(row: PositionReferenceRow) -> PositionReference {\n    PositionReference {\n        code: row.code,\n        name: row.name,\n        position_group: row.position_group,\n        sort_order: row.sort_order,\n    }\n}\n''', encoding='utf-8')
(positions / 'list_positions.rs').write_text('''use super::{reference_mapper::map_position_reference, reference_row::PositionReferenceRow};\nuse crate::{PersistenceResult, PostgresStore};\nuse football_domain::PositionReference;\n\nimpl PostgresStore {\n    pub async fn list_positions(&self) -> PersistenceResult<Vec<PositionReference>> {\n        let rows = sqlx::query_as::<_, PositionReferenceRow>(\n            r#"\n            SELECT code, name, position_group, sort_order\n            FROM football.positions\n            ORDER BY sort_order, code\n            "#,\n        )\n        .fetch_all(&self.pool)\n        .await?;\n        Ok(rows.into_iter().map(map_position_reference).collect())\n    }\n}\n''', encoding='utf-8')
pos_mod = positions / 'mod.rs'
text = pos_mod.read_text(encoding='utf-8')
for declaration in ['mod list_positions;\n', 'mod reference_mapper;\n', 'mod reference_row;\n']:
    if declaration not in text:
        text = text.replace('mod mapper;\n', 'mod mapper;\n' + declaration, 1)
pos_mod.write_text(text, encoding='utf-8')

# 3. Season-team membership options support the R6 player-period catalog editor.
season = team_periods / 'season_memberships'
if season.exists():
    raise SystemExit('season_memberships owner already exists')
season.mkdir()
(season / 'mod.rs').write_text('mod list;\nmod mapper;\nmod row;\n', encoding='utf-8')
(season / 'row.rs').write_text('''use sqlx::FromRow;\nuse uuid::Uuid;\n\n#[derive(Debug, FromRow)]\npub(super) struct SeasonTeamMembershipRow {\n    pub season_id: Uuid,\n    pub team_id: Uuid,\n    pub registration_status: String,\n}\n''', encoding='utf-8')
(season / 'mapper.rs').write_text('''use super::row::SeasonTeamMembershipRow;\nuse football_domain::SeasonTeamMembershipOption;\n\npub(super) fn map_membership_option(row: SeasonTeamMembershipRow) -> SeasonTeamMembershipOption {\n    SeasonTeamMembershipOption {\n        season_id: row.season_id,\n        team_id: row.team_id,\n        registration_status: row.registration_status,\n    }\n}\n''', encoding='utf-8')
(season / 'list.rs').write_text('''use super::{mapper::map_membership_option, row::SeasonTeamMembershipRow};\nuse crate::{PersistenceResult, PostgresStore};\nuse football_domain::SeasonTeamMembershipOption;\n\nimpl PostgresStore {\n    pub(crate) async fn list_season_team_memberships(\n        &self,\n    ) -> PersistenceResult<Vec<SeasonTeamMembershipOption>> {\n        let rows = sqlx::query_as::<_, SeasonTeamMembershipRow>(\n            r#"\n            SELECT season_id, team_id, registration_status\n            FROM football.team_season_memberships\n            WHERE registration_status IN ('registered', 'guest')\n            ORDER BY season_id, team_id\n            "#,\n        )\n        .fetch_all(&self.pool)\n        .await?;\n        Ok(rows.into_iter().map(map_membership_option).collect())\n    }\n}\n''', encoding='utf-8')
period_mod = team_periods / 'mod.rs'
text = period_mod.read_text(encoding='utf-8')
if 'mod season_memberships;' not in text:
    text = text.replace('mod row;\n', 'mod row;\nmod season_memberships;\n', 1)
period_mod.write_text(text, encoding='utf-8')

# Remove only R6 SQL/reference mapping from player_catalog.rs. Match/Lineup remains untouched for R7.
p = player_catalog.read_text(encoding='utf-8')
p = p.replace('    PlayerCatalogReferenceData, PositionReference, SeasonTeamMembershipOption,\n', '    PlayerCatalogReferenceData,\n', 1)
start = p.index('    async fn list_season_team_memberships(')
positions_start = p.index('    pub async fn list_positions', start)
p = p[:start] + p[positions_start:]
positions_start = p.index('    pub async fn list_positions')
impl_end = p.index('\n}\n\nasync fn resolve_match_scope_draft', positions_start)
p = p[:positions_start] + p[impl_end:]
mapper_start = p.index('fn position_reference_from_row(')
tests_start = p.index('#[cfg(test)]', mapper_start)
p = p[:mapper_start] + p[tests_start:]
player_catalog.write_text(p, encoding='utf-8')

# Remove old entity catalog module and file.
lib_text = lib.read_text(encoding='utf-8')
if 'mod entity_catalog;\n' not in lib_text:
    raise SystemExit('lib.rs entity_catalog module missing')
lib.write_text(lib_text.replace('mod entity_catalog;\n', '', 1), encoding='utf-8')
entity.unlink()

# Strengthen existing R6 ownership verifiers instead of creating a parallel long-term gate.
v4 = root / 'scripts/verify-player-names-positions.mjs'
v = v4.read_text(encoding='utf-8')
v = v.replace('  "crates/persistence-postgres/tests/player_names_positions_repository_contract.rs", "R6-04 PostgreSQL contract test 缺失");', '  "crates/persistence-postgres/tests/player_names_positions_repository_contract.rs", "R6-04 PostgreSQL contract test 缺失");')
# Insert focused owner reads after positionsWrite declaration block.
anchor = 'const legacy = read("crates/persistence-postgres/src/player_catalog.rs");\n'
insert = '''const positionsList = requireTokens("crates/persistence-postgres/src/adapters/catalog/players/positions/list_positions.rs", [\n  "pub async fn list_positions",\n  "query_as::<_, PositionReferenceRow>",\n  "FROM football.positions",\n  "map_position_reference",\n], "Position reference read owner");\nconst positionReferenceRow = read("crates/persistence-postgres/src/adapters/catalog/players/positions/reference_row.rs");\nconst positionReferenceMapper = read("crates/persistence-postgres/src/adapters/catalog/players/positions/reference_mapper.rs");\n'''
if insert not in v:
    if anchor not in v:
        raise SystemExit('R6-04 verifier anchor missing')
    v = v.replace(anchor, insert + anchor, 1)
v = v.replace('check(positionsMod.includes("mod assign_player_position;") && positionsMod.includes("mod input_policy;") && positionsMod.includes("mod mapper;") && positionsMod.includes("mod row;"), "Positions 目录职责拆分不完整");', 'check(positionsMod.includes("mod assign_player_position;") && positionsMod.includes("mod input_policy;") && positionsMod.includes("mod mapper;") && positionsMod.includes("mod row;") && positionsMod.includes("mod list_positions;") && positionsMod.includes("mod reference_mapper;") && positionsMod.includes("mod reference_row;"), "Positions 目录职责拆分不完整");')
v = v.replace('check(!legacy.includes("fn player_position_from_row"), "legacy player_catalog.rs 仍拥有 PlayerPosition Row mapper");', 'check(!legacy.includes("fn player_position_from_row"), "legacy player_catalog.rs 仍拥有 PlayerPosition Row mapper");\ncheck(!legacy.includes("pub async fn list_positions") && !legacy.includes("fn position_reference_from_row"), "legacy player_catalog.rs 仍拥有 Position reference read/mapping");\ncheck(positionReferenceRow.includes("struct PositionReferenceRow") && positionReferenceMapper.includes("fn map_position_reference"), "Position reference Row/mapper owner 不完整");')
v4.write_text(v, encoding='utf-8')

v5 = root / 'scripts/verify-player-team-periods-availability.mjs'
v = v5.read_text(encoding='utf-8')
v = v.replace('  "crates/persistence-postgres/src/adapters/catalog/players/team_periods/mod.rs",\n', '  "crates/persistence-postgres/src/adapters/catalog/players/team_periods/mod.rs",\n  "crates/persistence-postgres/src/adapters/catalog/players/team_periods/season_memberships/mod.rs",\n  "crates/persistence-postgres/src/adapters/catalog/players/team_periods/season_memberships/list.rs",\n  "crates/persistence-postgres/src/adapters/catalog/players/team_periods/season_memberships/row.rs",\n  "crates/persistence-postgres/src/adapters/catalog/players/team_periods/season_memberships/mapper.rs",\n  "crates/persistence-postgres/src/adapters/catalog/teams/detail/player_periods/mod.rs",\n  "crates/persistence-postgres/src/adapters/catalog/teams/detail/player_periods/read.rs",\n  "crates/persistence-postgres/src/adapters/catalog/teams/detail/player_periods/row.rs",\n  "crates/persistence-postgres/src/adapters/catalog/teams/detail/player_periods/mapper.rs",\n', 1)
v = v.replace('const legacy = read("crates/persistence-postgres/src/player_catalog.rs");', 'const legacy = read("crates/persistence-postgres/src/player_catalog.rs");\nconst persistenceLib = read("crates/persistence-postgres/src/lib.rs");\nconst teamDetailRead = read("crates/persistence-postgres/src/adapters/catalog/teams/detail/read_team.rs");\nconst teamPeriodProjection = read("crates/persistence-postgres/src/adapters/catalog/teams/detail/player_periods/read.rs");\nconst seasonMembershipList = read("crates/persistence-postgres/src/adapters/catalog/players/team_periods/season_memberships/list.rs");')
v = v.replace('check(teamPeriodsMod.includes("mod add_player_team_period;") && teamPeriodsMod.includes("mod input_policy;") && teamPeriodsMod.includes("mod mapper;") && teamPeriodsMod.includes("mod row;"), "Team Periods 目录职责拆分不完整");', 'check(teamPeriodsMod.includes("mod add_player_team_period;") && teamPeriodsMod.includes("mod input_policy;") && teamPeriodsMod.includes("mod mapper;") && teamPeriodsMod.includes("mod row;") && teamPeriodsMod.includes("mod season_memberships;"), "Team Periods 目录职责拆分不完整");')
v = v.replace('check(!legacy.includes("fn player_availability_from_row"), "legacy player_catalog.rs 仍拥有 Availability Row mapper");', 'check(!legacy.includes("fn player_availability_from_row"), "legacy player_catalog.rs 仍拥有 Availability Row mapper");\ncheck(!legacy.includes("async fn list_season_team_memberships"), "legacy player_catalog.rs 仍拥有 season-team membership SQL read");\ncheck(!exists("crates/persistence-postgres/src/entity_catalog.rs") && !persistenceLib.includes("mod entity_catalog;"), "legacy entity_catalog.rs 仍存在");\ncheck(teamDetailRead.includes("player_periods::read_player_periods") && teamPeriodProjection.includes("FROM football.player_team_periods") && teamPeriodProjection.includes("TeamPlayerPeriodRow"), "Team Detail player-period projection 未切换到新 owner");\ncheck(seasonMembershipList.includes("FROM football.team_season_memberships") && seasonMembershipList.includes("SeasonTeamMembershipRow"), "Season-team membership option read owner 不完整");')
v5.write_text(v, encoding='utf-8')

v9 = root / 'scripts/verify-entity-deletion-persistence.mjs'
v = v9.read_text(encoding='utf-8')
v = v.replace('const entity=read("crates/persistence-postgres/src/entity_catalog.rs");\n', '')
v = v.replace('for(const n of ["check_entity_deletion","bulk_archive_entities","team_reference_counts","player_reference_counts","coach_reference_counts"])req(!entity.includes(n),`legacy entity owner remains: ${n}`);\n', 'req(!exists("crates/persistence-postgres/src/entity_catalog.rs")&&!lib.includes("mod entity_catalog;"),"legacy entity catalog owner remains");\n')
v9.write_text(v, encoding='utf-8')

# Extend R6-05 PostgreSQL contract to exercise the moved team-facing projection.
contract = root / 'crates/persistence-postgres/tests/player_team_periods_availability_repository_contract.rs'
c = contract.read_text(encoding='utf-8')
anchor = '''    let detail = database\n        .store\n        .read_player(player.id)\n        .await\n        .expect("读取 Player Detail");\n'''
insert = '''    let team_detail = database\n        .store\n        .read_team(team.id)\n        .await\n        .expect("读取 Team Detail player periods");\n    assert!(team_detail.player_periods.iter().any(|item| {\n        item.id == period.id && item.player_id == player.id && item.team_id == team.id\n    }));\n\n'''
if insert not in c:
    if anchor not in c:
        raise SystemExit('R6-05 contract insertion anchor missing')
    c = c.replace(anchor, insert + anchor, 1)
contract.write_text(c, encoding='utf-8')

# Record the actual source change as VERIFYING; stage completion is written only after the full exit gate passes.
readme = root / 'README.md'
r = readme.read_text(encoding='utf-8')
section = '''### R6 Stage Exit Cleanup\n\n- R6 阶段出口正在清理最后的旧 catalog persistence owner：Team Detail player-period projection、Position reference list 与 season-team membership option read 迁入 `adapters/catalog/` 对应职责目录；`entity_catalog.rs` 删除。\n- `player_catalog.rs` 仅移除上述 R6 SQL/mapper，Match/Lineup 与其 reference-data 聚合入口保持原行为并留给 R7；公共 Port/DTO、Schema、0001–0046 migrations、配置、UI、错误语义和生产依赖不变。节点保持 `VERIFYING`，等待 R6 最终 Windows + PostgreSQL 16 出口门禁。\n\n'''
if '### R6 Stage Exit Cleanup' not in r:
    marker = '### R6-10 Global Name Search\n'
    if marker not in r:
        raise SystemExit('root README R6-10 marker missing')
    r = r.replace(marker, section + marker, 1)
readme.write_text(r, encoding='utf-8')

stage = root / 'docs/modular-rewrite/R06-entity-catalog-persistence/README.md'
s = stage.read_text(encoding='utf-8')
facts = '''## R6 阶段出口收尾\n\n- 状态：`VERIFYING`。\n- 最终审计发现旧 `entity_catalog.rs` 仍持有 Team Detail player-period projection；`player_catalog.rs` 仍持有 Position reference 与 season-team membership option 两条 R6 直接 SQL read。\n- 本次只迁移上述 R6 persistence owner；`player_catalog.rs` 的 Match/Lineup 职责明确保留给 R7，不提前跨阶段重写。\n- 完整出口门禁通过前不创建 `R06-stage-completion.md`，R6 阶段继续保持 `IN_PROGRESS`。\n\n'''
if '## R6 阶段出口收尾' not in s:
    marker = '## R6-10 当前事实\n'
    if marker not in s:
        raise SystemExit('stage README R6-10 marker missing')
    s = s.replace(marker, facts + marker, 1)
stage.write_text(s, encoding='utf-8')

print('R6 stage-exit ownership cleanup prepared')
