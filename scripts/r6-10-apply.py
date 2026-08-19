from pathlib import Path
import os

root = Path('.')
src = root / 'crates/persistence-postgres/src'
old_path = src / 'name_search.rs'
target = src / 'adapters/catalog/global_search'
if not old_path.exists() or target.exists():
    raise SystemExit('R6-10 apply helper requires the untouched name_search.rs baseline')
old = old_path.read_text(encoding='utf-8')

# Split the existing implementation by actual responsibility without rewriting its logic.
constants = old[old.index('const COMPACT_SQL_PATTERN'):old.index('#[derive(Debug, Clone, PartialEq, Eq)]')]
query = old[old.index('#[derive(Debug, Clone, PartialEq, Eq)]'):old.index('#[derive(Debug, Clone, Copy)]')]
predicate = old[old.index('#[derive(Debug, Clone, Copy)]'):old.index('fn normalize_query')]
normalization = old[old.index('fn normalize_query'):old.index('#[cfg(test)]')]

constants = constants.replace('const COMPACT_SQL_PATTERN', 'pub(super) const COMPACT_SQL_PATTERN', 1)
constants = constants.replace('const LATIN_FOLD_SOURCE', 'pub(super) const LATIN_FOLD_SOURCE', 1)
constants = constants.replace('const LATIN_FOLD_TARGET', 'pub(super) const LATIN_FOLD_TARGET', 1)
normalization = normalization.replace('fn normalize_query', 'pub(super) fn normalize_query', 1)
normalization = normalization.replace('fn compact_query', 'pub(super) fn compact_query', 1)

target.mkdir(parents=True)
(target / 'mod.rs').write_text(
    'mod normalization;\nmod predicate;\nmod query;\n\n'
    'pub(crate) use predicate::{push_name_search, NameSearchColumns};\n'
    'pub(crate) use query::NameSearch;\n',
    encoding='utf-8',
)
(target / 'query.rs').write_text(
    'use super::normalization::normalize_query;\n\n' + query + r'''#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_chinese_and_english_partial_terms() {
        let search = NameSearch::parse(Some("  Marlon · 索萨  ")).unwrap();
        assert_eq!(search.tokens(), &["marlon".to_string(), "索萨".to_string()]);
    }

    #[test]
    fn empty_query_is_ignored() {
        assert!(NameSearch::parse(Some("  --  ")).is_none());
        assert!(NameSearch::parse(None).is_none());
    }
}
''',
    encoding='utf-8',
)
(target / 'normalization.rs').write_text(
    constants + normalization + r'''#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn punctuation_free_query_can_match_compact_name() {
        assert_eq!(normalize_query("马龙·索萨"), "马龙 索萨");
        assert_eq!(compact_query("马龙·索萨"), "马龙索萨");
        assert_eq!(compact_query("marlon-sousa"), "marlonsousa");
    }

    #[test]
    fn latin_diacritics_are_folded_for_search() {
        assert_eq!(normalize_query("São Tomé"), "sao tome");
        assert_eq!(normalize_query("Kovačić"), "kovacic");
    }
}
''',
    encoding='utf-8',
)
(target / 'predicate.rs').write_text(
    'use super::{\n'
    '    normalization::{compact_query, COMPACT_SQL_PATTERN, LATIN_FOLD_SOURCE, LATIN_FOLD_TARGET},\n'
    '    query::NameSearch,\n};\n'
    'use sqlx::{Postgres, QueryBuilder};\n\n' + predicate + r'''#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn generated_sql_checks_primary_and_alias_names() {
        let mut builder = QueryBuilder::<Postgres>::new("SELECT 1 WHERE 1=1");
        let search = NameSearch::parse(Some("索萨")).unwrap();
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
        let sql = builder.sql();
        assert!(sql.contains("translate(player.normalized_name"));
        assert!(sql.contains("translate(lower(player.canonical_name)"));
        assert!(sql.contains("football.player_names alias"));
        assert!(sql.contains("translate(alias.normalized_name"));
        assert!(sql.contains("translate(lower(alias.name)"));
        assert!(sql.contains("regexp_replace"));
    }
}
''',
    encoding='utf-8',
)

old_import = 'name_search::{push_name_search, NameSearch, NameSearchColumns}'
new_import = 'adapters::catalog::global_search::{push_name_search, NameSearch, NameSearchColumns}'
changed = []
for path in src.rglob('*.rs'):
    if path == old_path:
        continue
    text = path.read_text(encoding='utf-8')
    if old_import in text:
        path.write_text(text.replace(old_import, new_import), encoding='utf-8')
        changed.append(path.as_posix())
expected = {
    'crates/persistence-postgres/src/adapters/catalog/players/directory/list_players.rs',
    'crates/persistence-postgres/src/adapters/catalog/teams/directory/list_teams.rs',
    'crates/persistence-postgres/src/adapters/catalog/teams/directory/list_team_options.rs',
    'crates/persistence-postgres/src/adapters/catalog/coaches/directory/list.rs',
    'crates/persistence-postgres/src/adapters/catalog/references/directory/read.rs',
}
if set(changed) != expected:
    raise SystemExit(f'unexpected NameSearch consumers: {changed}')

lib = src / 'lib.rs'
text = lib.read_text(encoding='utf-8')
if 'mod name_search;\n' not in text:
    raise SystemExit('lib.rs expected root name_search module missing')
lib.write_text(text.replace('mod name_search;\n', ''), encoding='utf-8')

catalog = src / 'adapters/catalog/mod.rs'
text = catalog.read_text(encoding='utf-8')
if 'pub(crate) mod global_search;' not in text:
    if 'mod formations;\n' not in text:
        raise SystemExit('catalog insertion anchor missing')
    text = text.replace('mod formations;\n', 'mod formations;\npub(crate) mod global_search;\n', 1)
catalog.write_text(text, encoding='utf-8')
old_path.unlink()

# Retarget the existing verifier while preserving its contract fixtures and UI assertions.
verify = root / 'scripts/verify-global-name-search.mjs'
v = verify.read_text(encoding='utf-8')
v = v.replace(
    'const helper = read("crates/persistence-postgres/src/name_search.rs");\n',
    'const catalogMod = read("crates/persistence-postgres/src/adapters/catalog/mod.rs");\n'
    'const globalSearchMod = read("crates/persistence-postgres/src/adapters/catalog/global_search/mod.rs");\n'
    'const queryOwner = read("crates/persistence-postgres/src/adapters/catalog/global_search/query.rs");\n'
    'const normalizationOwner = read("crates/persistence-postgres/src/adapters/catalog/global_search/normalization.rs");\n'
    'const predicateOwner = read("crates/persistence-postgres/src/adapters/catalog/global_search/predicate.rs");\n',
)
old_checks = '''requireTrue(persistenceLib.includes("mod name_search;"), "全局名称搜索模块未注册");
requireTrue(helper.includes("pub(crate) struct NameSearch"), "缺少统一名称搜索查询对象");
requireTrue(helper.includes('format!("%{token}%")'), "名称搜索仍未使用包含匹配");
requireTrue(helper.includes("regexp_replace"), "名称搜索未处理空格与标点差异");
requireTrue(helper.includes("LATIN_FOLD_SOURCE"), "名称搜索未处理拉丁重音字符");
requireTrue(helper.includes("character.is_alphanumeric()"), "名称搜索未统一中英文字符归一化");
requireTrue(helper.includes("alias.normalized_name"), "名称搜索未覆盖别名归一化字段");
requireTrue(helper.includes("alias.name"), "名称搜索未覆盖别名原始显示字段");
'''
new_checks = '''requireTrue(!fs.existsSync(path.join(root, "crates/persistence-postgres/src/name_search.rs")), "旧 name_search.rs 仍存在");
requireTrue(!persistenceLib.includes("mod name_search;"), "旧根模块入口仍存在");
requireTrue(catalogMod.includes("pub(crate) mod global_search;"), "Catalog 未注册 Global Search 模块");
requireTrue(globalSearchMod.includes("mod normalization;") && globalSearchMod.includes("mod predicate;") && globalSearchMod.includes("mod query;"), "Global Search 职责目录不完整");
requireTrue(queryOwner.includes("pub(crate) struct NameSearch"), "缺少统一名称搜索查询对象");
requireTrue(predicateOwner.includes('format!("%{token}%")'), "名称搜索仍未使用包含匹配");
requireTrue(predicateOwner.includes("regexp_replace"), "名称搜索未处理空格与标点差异");
requireTrue(normalizationOwner.includes("LATIN_FOLD_SOURCE"), "名称搜索未处理拉丁重音字符");
requireTrue(normalizationOwner.includes("character.is_alphanumeric()"), "名称搜索未统一中英文字符归一化");
requireTrue(predicateOwner.includes("columns.alias_normalized"), "名称搜索未覆盖别名归一化字段");
requireTrue(predicateOwner.includes("columns.alias_display"), "名称搜索未覆盖别名原始显示字段");
'''
if old_checks not in v:
    raise SystemExit('global search verifier ownership block changed unexpectedly')
v = v.replace(old_checks, new_checks)
v = v.replace(
    'const helperUsages = (combined.match(/NameSearch::parse\\(/g) ?? []).length;\n',
    'const helperUsages = (combined.match(/NameSearch::parse\\(/g) ?? []).length;\n'
    'requireTrue(!combined.includes("name_search::{"), "仍有调用点依赖旧 name_search owner");\n',
)
v = v.replace(
    'requireTrue(playerDirectoryList.includes("NameSearch::parse"), "R6-03 Player Directory list 未接入统一 NameSearch");',
    'requireTrue(playerDirectoryList.includes("adapters::catalog::global_search") && playerDirectoryList.includes("NameSearch::parse"), "R6-03 Player Directory list 未接入新 Global Search owner");',
)
v = v.replace(
    'requireTrue(teamDirectoryList.includes("NameSearch::parse"), "R6-01 Team Directory list 未接入统一 NameSearch");',
    'requireTrue(teamDirectoryList.includes("adapters::catalog::global_search") && teamDirectoryList.includes("NameSearch::parse"), "R6-01 Team Directory list 未接入新 Global Search owner");',
)
v = v.replace(
    'requireTrue(teamOptionList.includes("NameSearch::parse"), "R6-01 Team options 未接入统一 NameSearch");',
    'requireTrue(teamOptionList.includes("adapters::catalog::global_search") && teamOptionList.includes("NameSearch::parse"), "R6-01 Team options 未接入新 Global Search owner");',
)
v = v.replace(
    'requireTrue(coachDirectoryList.includes("NameSearch::parse"), "R6-07 Coach Directory list 未接入统一 NameSearch");',
    'requireTrue(coachDirectoryList.includes("adapters::catalog::global_search") && coachDirectoryList.includes("NameSearch::parse"), "R6-07 Coach Directory list 未接入新 Global Search owner");',
)
v = v.replace(
    'requireTrue(referenceDirectoryRead.includes("NameSearch::parse") && referenceDirectoryRead.includes("push_name_search"), "R6-08 Reference Directory 未接入统一 NameSearch");',
    'requireTrue(referenceDirectoryRead.includes("adapters::catalog::global_search") && referenceDirectoryRead.includes("NameSearch::parse") && referenceDirectoryRead.includes("push_name_search"), "R6-08 Reference Directory 未接入新 Global Search owner");',
)
v = v.replace('个后端入口已统一，', '个后端入口已统一到 catalog/global_search，')
verify.write_text(v, encoding='utf-8')

# Focused PostgreSQL contract: canonical/localized/alias, diacritics, punctuation, mixed tokens and stable pagination.
(root / 'crates/persistence-postgres/tests/global_name_search_repository_contract.rs').write_text(r'''use football_domain::{
    CoachDraft, CoachListQuery, CoachNameDraft, EntityReferenceQuery, PlayerDraft, PlayerListQuery,
    PlayerNameDraft, PlayerStatus, PreferredFoot, TeamDraft, TeamListQuery, TeamNameDraft,
};
use football_persistence_postgres::{DatabaseOptions, PostgresStore};
use serde_json::json;
use std::collections::HashSet;
use uuid::Uuid;

#[tokio::test]
#[ignore = "需要专用且可写的 PostgreSQL 测试数据库；设置 FOOTBALL_TEST_DATABASE_URL 后显式运行"]
async fn global_name_search_contract_is_preserved() {
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

    let team = store.create_team(&TeamDraft {
        canonical_name: format!("São-Paulo Search {token}"),
        country_code: Some("BR".into()),
        metadata: json!({"contract":"r6-10"}),
    }).await.expect("create primary team");
    store.add_team_name(&TeamNameDraft {
        team_id: team.id,
        name: format!("圣保罗·验收 {token}"),
        language_code: Some("zh-CN".into()),
        valid_from: None,
        valid_to: None,
    }).await.expect("add team alias");
    let team_two = store.create_team(&TeamDraft {
        canonical_name: format!("Sao Paulo Reserve Search {token}"),
        country_code: Some("BR".into()),
        metadata: json!({"contract":"r6-10"}),
    }).await.expect("create pagination team");

    let options = store.list_team_options(Some(&format!("sao paulo {token}")), 20).await.expect("team selector search");
    assert!(options.iter().any(|item| item.id == team.id));
    let chinese = store.list_teams(&TeamListQuery {
        search: Some(format!("圣保罗验收 {token}")), country_code: None, team_type: None,
        active_only: true, limit: 20, cursor_name: None, cursor_id: None,
    }).await.expect("team compact alias search");
    assert!(chinese.items.iter().any(|item| item.id == team.id));
    let mixed = store.list_teams(&TeamListQuery {
        search: Some(format!("sao 验收 {token}")), country_code: None, team_type: None,
        active_only: true, limit: 20, cursor_name: None, cursor_id: None,
    }).await.expect("team mixed search");
    assert!(mixed.items.iter().any(|item| item.id == team.id));

    let first = store.list_teams(&TeamListQuery {
        search: Some(token.clone()), country_code: None, team_type: None,
        active_only: true, limit: 1, cursor_name: None, cursor_id: None,
    }).await.expect("first page");
    assert_eq!(first.items.len(), 1);
    assert!(first.has_more);
    let second = store.list_teams(&TeamListQuery {
        search: Some(token.clone()), country_code: None, team_type: None,
        active_only: true, limit: 1, cursor_name: first.next_cursor_name.clone(), cursor_id: first.next_cursor_id,
    }).await.expect("second page");
    assert_eq!(second.items.len(), 1);
    let ids = first.items.iter().chain(second.items.iter()).map(|item| item.id).collect::<HashSet<_>>();
    assert_eq!(ids, HashSet::from([team.id, team_two.id]));

    let player = store.create_player(&PlayerDraft {
        canonical_name: format!("Marlón Sousa {token}"),
        date_of_birth: None,
        nationality_code: Some("BR".into()),
        preferred_foot: PreferredFoot::Right,
        height_cm: Some(181),
        status: PlayerStatus::Active,
        metadata: json!({"contract":"r6-10"}),
    }).await.expect("create player");
    store.add_player_name(&PlayerNameDraft {
        player_id: player.id,
        name: format!("马龙·索萨 {token}"),
        language_code: Some("zh-CN".into()),
        is_primary: false,
        valid_from: None,
        valid_to: None,
    }).await.expect("player alias");
    let players = store.list_players(&PlayerListQuery {
        search: Some(format!("marlon 索萨 {token}")), team_id: None, position_code: None,
        availability_status: None, player_status: Some(PlayerStatus::Active), limit: 20,
        cursor_name: None, cursor_id: None,
    }).await.expect("player mixed search");
    assert!(players.items.iter().any(|item| item.id == player.id));

    let coach = store.create_coach(&CoachDraft {
        canonical_name: format!("José Search {token}"),
        nationality_code: Some("PT".into()), status: "active".into(),
        metadata: json!({"contract":"r6-10"}),
    }).await.expect("create coach");
    store.add_coach_name(&CoachNameDraft {
        coach_id: coach.id, name: format!("何塞·教练 {token}"), language_code: Some("zh-CN".into()),
        is_primary: false, valid_from: None, valid_to: None,
    }).await.expect("coach alias");
    let coaches = store.list_coaches(&CoachListQuery {
        search: Some(format!("jose 教练 {token}")), active_only: true, limit: 20,
    }).await.expect("coach mixed search");
    assert!(coaches.iter().any(|item| item.id == coach.id));

    for (entity_type, search, id) in [
        ("team", format!("sao 验收 {token}"), team.id),
        ("player", format!("marlon 索萨 {token}"), player.id),
        ("coach", format!("jose 教练 {token}"), coach.id),
    ] {
        let records = store.list_entity_references(&EntityReferenceQuery {
            entity_type: entity_type.into(), search: Some(search), active_only: true, limit: 20,
        }).await.expect("entity reference search");
        assert!(records.iter().any(|item| item.id == id));
    }
    store.close().await;
}
''', encoding='utf-8')

run_id = os.environ.get('GITHUB_RUN_ID', 'pending')
record = root / 'docs/modular-rewrite/R06-entity-catalog-persistence/R06-10-global-name-search.md'
record.write_text(f'''# R6-10 Global Name Search

## 状态

`VERIFYING`

## 基线与范围

- 源码基线：`e79f1becb1595b03f5d1ba4686363dd62fbac9e2`；继续使用唯一分支 `rewrite/r6-entity-catalog-persistence`。
- 唯一目标 owner：`crates/persistence-postgres/src/adapters/catalog/global_search/`。
- 仅迁移查询解析、名称归一化和 PostgreSQL 名称谓词；球队、球员、教练、球队 selector 与 Entity Reference 调用点只切换内部 owner。
- `football.global-name-search.v1`、公共 Port/DTO、Schema、0001–0046 migrations、配置、错误语义、UI 与模型保护资产均不改变。

## 实施结果

- `query.rs`：查询解析与多关键词 token。
- `normalization.rs`：大小写、拉丁重音、标点/空白与 compact 规则。
- `predicate.rs`：SQL QueryBuilder 名称谓词。
- `mod.rs`：只做模块声明和显式导出。
- 旧 `crates/persistence-postgres/src/name_search.rs` 删除，无兼容转发壳。
- 现有 7 个 `NameSearch::parse` 接入点保持 contains、多关键词 AND、正式名/本地化名/别名、中文/英文部分匹配、标点/空白无关与拉丁重音折叠语义。
- 新增 PostgreSQL contract 覆盖 Team Directory + 分页、team selector、Player Directory、Coach Directory 和 team/player/coach Entity Reference；不新增拼音或编辑距离纠错。

## 验证状态

- implementation/minimum gate run `{run_id}` 执行中；硬门禁通过前保持 `VERIFYING`。
- production owner-switch 后继续运行完整 Windows stage regression 与 R6-01～R6-10 retained PostgreSQL 16 contracts。

## 回退点

- 回退到源码基线 `e79f1becb1595b03f5d1ba4686363dd62fbac9e2`；不保留双实现。
''', encoding='utf-8')

stage = root / 'docs/modular-rewrite/R06-entity-catalog-persistence/README.md'
s = stage.read_text(encoding='utf-8')
if '| R6-10 | Global Name Search | READY |' not in s:
    raise SystemExit('stage README does not mark R6-10 READY')
s = s.replace('| R6-10 | Global Name Search | READY |', '| R6-10 | Global Name Search | VERIFYING |', 1)
if '## R6-10 当前事实' not in s:
    s = s.replace('## R6-01 当前事实', f'''## R6-10 当前事实

- 详细记录：[`R06-10-global-name-search.md`](R06-10-global-name-search.md)。
- 旧根部 `name_search.rs` 已按 `query / normalization / predicate` 分责迁入 `adapters/catalog/global_search/`；搜索契约与 UI 不变。
- implementation/minimum gate run `{run_id}` 执行中；完成前保持 `VERIFYING`。

## R6-01 当前事实''', 1)
stage.write_text(s, encoding='utf-8')

readme = root / 'README.md'
r = readme.read_text(encoding='utf-8')
if '### R6-10 Global Name Search' not in r:
    r = r.replace('### R6-09 Archive / Delete / Force Delete', f'''### R6-10 Global Name Search

- 统一名称搜索正迁移到 `adapters/catalog/global_search/`，按查询解析、归一化与 PostgreSQL 谓词分责；现有调用点只切换内部 owner。
- `football.global-name-search.v1` 的中文/英文/别名/重音/标点与空白无关、多关键词 AND 与 contains 语义保持不变；公共 DTO、Schema、0001–0046 migrations、配置、UI 与生产依赖不变。implementation/minimum gate run `{run_id}` 执行中，节点暂为 `VERIFYING`。

### R6-09 Archive / Delete / Force Delete''', 1)
readme.write_text(r, encoding='utf-8')

print('R6-10 implementation tree prepared successfully')
