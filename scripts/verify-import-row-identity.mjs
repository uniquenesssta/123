import { readFileSync, readdirSync } from "node:fs";
import { dirname, join } from "node:path";
import { fileURLToPath } from "node:url";

const root = join(dirname(fileURLToPath(import.meta.url)), "..");
const read = (path) => readFileSync(join(root, path), "utf8");
const requireTrue = (condition, message) => {
  if (!condition) throw new Error(message);
};

const entityMigration = read("crates/persistence-postgres/migrations/0029_import_row_entity_identity.sql");
const subrecordMigration = read("crates/persistence-postgres/migrations/0030_import_row_subrecord_identity.sql");
const teamPeriodMigration = read("crates/persistence-postgres/migrations/0043_import_row_team_period_identity.sql");
const parser = read("crates/spreadsheet-io/src/team_package.rs");
const persistence = ["mod", "preview", "conflict", "commit", "write", "names", "identity", "formation", "validation", "values"].map((name) => read(`crates/persistence-postgres/src/adapters/workbooks/team_package/${name}.rs`)).join("\n") + read("crates/persistence-postgres/src/adapters/workbooks/identity/teams.rs");
const packageJson = JSON.parse(read("package.json"));
const readme = read("README.md");
const databaseDoc = read("docs/DATABASE.md");

requireTrue(
  entityMigration.includes("DROP CONSTRAINT IF EXISTS import_rows_batch_id_sheet_name_row_number_key"),
  "0029未移除旧的工作表行号唯一键",
);
requireTrue(
  /UNIQUE\s*\(\s*batch_id\s*,\s*sheet_name\s*,\s*row_number\s*,\s*entity_type\s*\)/s.test(entityMigration),
  "0029历史迁移未按批次、工作表、行号和实体类型建立过渡唯一键",
);

requireTrue(
  subrecordMigration.includes("ADD COLUMN IF NOT EXISTS subrecord_key text"),
  "0030未新增导入子记录身份列",
);
requireTrue(
  subrecordMigration.includes("WHEN 'player_ability'") &&
    subrecordMigration.includes("payload ->> 'dimension_code'"),
  "0030未用能力维度区分同一行的多条player_ability",
);
requireTrue(
  subrecordMigration.includes("WHEN 'player_dynamic_tag'") &&
    subrecordMigration.includes("payload ->> 'tag_code'"),
  "0030未用标签代码区分同一行的多条player_dynamic_tag",
);
requireTrue(
  subrecordMigration.includes("DROP CONSTRAINT IF EXISTS import_rows_batch_sheet_row_entity_key"),
  "0030未移除过粗的四字段唯一键",
);
requireTrue(
  /UNIQUE\s*\(\s*batch_id\s*,\s*sheet_name\s*,\s*row_number\s*,\s*entity_type\s*,\s*subrecord_key\s*\)/s.test(subrecordMigration),
  "0030未按子记录身份建立五字段唯一键",
);
requireTrue(
  !/ADD CONSTRAINT[\s\S]*UNIQUE\s*\(\s*batch_id\s*,\s*sheet_name\s*,\s*row_number\s*,\s*entity_type\s*\)\s*;/s.test(subrecordMigration),
  "0030仍重新建立了过粗的四字段唯一键",
);

requireTrue(
  teamPeriodMigration.includes("WHEN 'player_team_period'") &&
    teamPeriodMigration.includes("payload ->> 'team_id'") &&
    teamPeriodMigration.includes("payload ->> 'team_key'") &&
    teamPeriodMigration.includes("payload ->> 'team_name'"),
  "0043未按球队ID、临时键或名称区分同一物理行中的多条player_team_period",
);
requireTrue(
  teamPeriodMigration.includes("DROP COLUMN IF EXISTS subrecord_key") &&
    /UNIQUE\s*\(\s*batch_id\s*,\s*sheet_name\s*,\s*row_number\s*,\s*entity_type\s*,\s*subrecord_key\s*\)/s.test(teamPeriodMigration),
  "0043未重建包含球队效力子身份的五字段唯一键",
);

for (const entity of [
  "SpreadsheetEntityType::Player",
  "SpreadsheetEntityType::PlayerTeamPeriod",
  "SpreadsheetEntityType::PlayerPosition",
  "SpreadsheetEntityType::PlayerAvailability",
  "SpreadsheetEntityType::PlayerAbility",
  "SpreadsheetEntityType::PlayerDynamicTag",
]) {
  requireTrue(parser.includes(entity), `完整资料包解析器缺少同一球员行拆分实体：${entity}`);
}
requireTrue(
  parser.includes('ability.insert("dimension_code"') &&
    parser.includes('tag.insert("tag_code"') &&
    parser.includes('("club_team_key", "team_key")') &&
    parser.includes('("team_key", "team_key")'),
  "完整资料包解析器没有为能力、动态标签和球队效力记录保留稳定子身份字段",
);
const preservesPhysicalWorksheetRow =
  /for\s*\(\s*row_number\s*,\s*values\s*\)\s*in\s*read_business_rows\(\s*workbook\s*,\s*"球员与评分"\s*,\s*PLAYER_KEYS\s*\)\?/s.test(parser) &&
  /rows\.push\(\s*\(\s*\(\s*index\s*\+\s*FIRST_DATA_ROW\s*\+\s*1\s*\)\s*as\s*u32\s*,\s*values\s*\)\s*\)/s.test(parser) &&
  /raw\(\s*"球员与评分"\s*,\s*row_number\s*,\s*SpreadsheetEntityType::Player\s*,/s.test(parser);
requireTrue(
  preservesPhysicalWorksheetRow,
  "完整资料包解析器未从工作表读取、计算并传递物理行号",
);
requireTrue(
  parser.includes("physical_worksheet_row_number_survives_blank_rows"),
  "缺少跨空白行保留Excel物理行号的Rust回归测试",
);
requireTrue(
  parser.includes("push_unique_team_period") &&
    parser.includes("team_period_identity_aliases") &&
    parser.includes("emitted_team_period_identities"),
  "完整资料包解析器未在同一物理行内合并重复的主球队/俱乐部效力记录",
);
requireTrue(
  parser.includes("duplicate_main_and_club_team_period_is_emitted_once") &&
    parser.includes("distinct_main_and_club_team_periods_are_both_emitted"),
  "缺少同球队去重与不同球队双履历保留的 Rust 回归测试",
);
requireTrue(
  parser.includes("push_unique_team_entity") &&
    parser.includes("team_entity_identity_aliases") &&
    parser.includes("emitted_club_teams.extend"),
  "完整资料包解析器未让球队总览实体抑制球员表推导出的重复俱乐部实体",
);
requireTrue(
  parser.includes("explicit_team_overview_suppresses_implicit_club_team") &&
    parser.includes("distinct_implicit_club_team_is_preserved"),
  "缺少显式球队优先和不同隐式俱乐部保留的 Rust 回归测试",
);
requireTrue(
  persistence.includes("consolidate_duplicate_ready_add_team_rows") &&
    persistence.includes("同一资料包重复球队行已合并并跳过") &&
    persistence.includes("duplicate_package_team_identity_matches_explicit_and_implicit_rows"),
  "旧预检批次提交边界未合并显式球队行与球员表推导的重复球队行",
);

requireTrue(
  packageJson.scripts.build?.includes("verify-frontend.mjs") &&
    read("scripts/verify-frontend.mjs").includes("verify-import-row-identity.mjs"),
  "build未接入导入行身份专项门禁",
);
requireTrue(
  packageJson.scripts["verify:frontend"]?.includes("verify-frontend.mjs") &&
    read("scripts/verify-frontend.mjs").includes("verify-import-row-identity.mjs"),
  "verify:frontend未接入导入行身份专项门禁",
);
requireTrue(
  packageJson.scripts["verify:import-row-identity"] === "node scripts/verify-import-row-identity.mjs",
  "缺少verify:import-row-identity脚本",
);
requireTrue(readme.includes("导入行子记录身份修复"), "README未记录导入行子记录身份修复");
requireTrue(
  databaseDoc.includes("batch_id, sheet_name, row_number, entity_type, subrecord_key") &&
    databaseDoc.includes("dimension_code") && databaseDoc.includes("tag_code") &&
    databaseDoc.includes("player_team_period") && databaseDoc.includes("team_id") &&
    databaseDoc.includes("team_key") && databaseDoc.includes("team_name"),
  "数据库文档未同步导入子记录唯一身份",
);


// R7-15：定位、子记录约束和重复球队合并职责分别唯一持有。
const identityRoot="crates/persistence-postgres/src/adapters/workbooks/identity";
const rowIdentity=read(`${identityRoot}/row.rs`);
const teamsIdentity=read(`${identityRoot}/teams.rs`);
const ledgerRows=read("crates/persistence-postgres/src/adapters/workbooks/batch_ledger/rows.rs");
const oldTeamIdentity=read("crates/persistence-postgres/src/adapters/workbooks/team_package/identity.rs");
const teamCommit=read("crates/persistence-postgres/src/adapters/workbooks/team_package/commit.rs");
const paths=[];
const scan=(directory)=>{for(const entry of readdirSync(join(root,directory),{withFileTypes:true})){const child=`${directory}/${entry.name}`;if(entry.isDirectory())scan(child);else if(entry.name.endsWith(".rs"))paths.push(child);}};
scan("crates/persistence-postgres/src");
for(const [file,names] of [["teams",["team_ready_add_identity","payload_value_is_present","team_row_preference","merge_missing_team_payload_fields","consolidate_duplicate_ready_add_team_rows","normalized_source_urls","team_ready_add_source_identity","consolidate_duplicate_ready_add_team_rows_by_source"]],["row",["persisted_worksheet_row_number","validate_workbook_row_locations"]]]) {
  requireTrue(read(`${identityRoot}/mod.rs`).includes(`mod ${file};`),`${file} 身份职责未注册`);
  for(const name of names){
    const owners=paths.filter((file)=>new RegExp(`fn\\s+${name}\\s*\\(`).test(read(file)));
    requireTrue(owners.length===1 && owners[0]===`${identityRoot}/${file}.rs`,`${name} 必须只有一个实际身份 owner`);
  }
}
requireTrue(read("crates/persistence-postgres/src/adapters/workbooks/mod.rs").includes("mod identity;"),"共享行身份职责未登记");
requireTrue(!oldTeamIdentity.includes("fn team_ready_add_identity") && !oldTeamIdentity.includes("fn normalized_source_urls") && !oldTeamIdentity.includes("fn consolidate_duplicate_ready_add_team_rows"),"旧球队身份模块不得保留重复行合并实现");
requireTrue(teamCommit.includes("super::super::identity::teams") && oldTeamIdentity.includes("super::super::identity::teams::normalized_source_urls"),"原提交合并和批内来源关联必须切换共享唯一职责");
const rowProduction=rowIdentity.slice(0,rowIdentity.indexOf("#[cfg(test)]"));
requireTrue(rowProduction.includes("i32::try_from(row_number)") && rowProduction.includes("*value >= 2") && !rowProduction.includes("row_number as i32"),"物理行号必须遵守既有 >=2 CHECK 且拒绝整数溢出");
for(const token of ["row_id: row.id","batch_id,","sheet_name: &row.sheet_name","persisted_worksheet_row_number(row.row_number)?","entity_type: row.entity_type.as_str()"]){requireTrue(rowProduction.includes(token),`已暂存行定位不得重建或丢字段：${token}`);}
requireTrue(!/sqlx::|HashSet|HashMap|new_v4|dimension_code|tag_code|team_key|team_name/.test(rowProduction),"行定位不得生成第二份子记录规则、随机 ID 或物理行去重集合");
const insert=ledgerRows.slice(ledgerRows.indexOf("async fn insert_import_row("),ledgerRows.indexOf("fn count_preview_rows("));
requireTrue(insert.indexOf("ImportRowLocation::from_row(batch_id, row)?")<insert.indexOf("sqlx::query("),"账本插入必须先检查稳定定位");
for(const token of ["location.row_id","location.batch_id","location.sheet_name","location.row_number","location.entity_type","&row.payload","row.matched_entity_id","&row.conflict_candidates"]){requireTrue(insert.includes(`.bind(${token})`) || (token==="&row.conflict_candidates" && insert.includes("serde_json::to_value(&row.conflict_candidates)?")),`原账本插入丢失字段：${token}`);}
requireTrue(!insert.includes("subrecord_key") && !insert.includes("ON CONFLICT") && !insert.includes(".begin()") && !insert.includes(".commit()") && insert.includes("execute(&mut **tx)"),"原生成列/重复子记录拒绝与同事务 INSERT 必须保持");
for(const [family,next] of [["player_catalog","ledger::find_player_batch("],["team_package","ledger::find_team_batch("],["match_lineup","Uuid::new_v4()"]]) {
  const source=read(`crates/persistence-postgres/src/adapters/workbooks/${family}/preview.rs`);
  requireTrue(source.indexOf("validate_workbook_row_locations(parsed)?")>=0 && source.indexOf("validate_workbook_row_locations(parsed)?")<source.indexOf(next),`${family} 非法行号必须在数据库操作或随机 ID 创建前拒绝`);
}
requireTrue(!/sqlx::query|self\.pool|\.begin\(|\.commit\(|write_audit_event/.test(teamsIdentity) && teamsIdentity.includes("ledger_rows::set_import_payload_in_tx(") && teamsIdentity.includes("ledger_rows::skip_duplicate_import_row_in_tx("),"球队合并仅复用原账本调用方事务，不另建 SQL/事务/审计");
requireTrue(!teamsIdentity.includes("super::identity") && !teamsIdentity.includes("team_package::commit") && !teamsIdentity.includes("team_package::write"),"行合并职责不得反向依赖业务编排/写入");
for(const token of ["worksheet_row_number_rejects_headers_and_integer_wraparound","row_uuid_keeps_two_subrecords_on_one_physical_row_distinct","preflight_preserves_same_row_entities_and_subrecords_for_database_rules"]){requireTrue(rowIdentity.includes(token),`行定位边界 inline 回归缺少：${token}`);}
const integration=read("crates/persistence-postgres/tests/postgres_integration.rs");
for(const token of ["同物理行的多实体、多能力、多标签和双球队效力子记录全部暂存","暂存行 UUID 不由物理行号复用","重复子记录必须回滚新批次及此前合法行","非法物理行号不得取消同源 pending 批次","子记录读回保持原 UUID、工作表、物理行和载荷"]){requireTrue(integration.includes(token),`原子记录 PG 回归缺少：${token}`);}

console.log("导入行子记录身份专项验证通过：显式球队优先于球员表推导球队，同球队主/俱乐部关系会去重，不同球队双履历、多个能力维度和动态标签可共存，旧预检批次提交时也会合并重复球队实体。");
