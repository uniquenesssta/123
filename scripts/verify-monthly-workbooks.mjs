import fs from "node:fs";
import crypto from "node:crypto";
import { isVersionAtLeast } from "./version.mjs";

const read = (path) => fs.readFileSync(new URL(`../${path}`, import.meta.url), "utf8");
const requireTrue = (condition, message) => { if (!condition) throw new Error(message); };

const contractText = read("contracts/monthly-workbooks-contract.json");
const contract = JSON.parse(contractText);
const migration = read("crates/persistence-postgres/migrations/0023_monthly_workbooks.sql");
const domain = read("crates/domain/src/monthly_workbook.rs") + read("crates/domain/src/spreadsheet.rs");
const spreadsheetIo = read("crates/spreadsheet-io/src/monthly_workbook.rs");
const persistence = read("crates/persistence-postgres/src/monthly_workbooks.rs");
const legacyPersistence = read("crates/persistence-postgres/src/spreadsheet_exchange.rs");
const application = read("crates/application/src/spreadsheet.rs");
const commands = read("src-tauri/src/commands/exchange.rs");
const registry = read("src-tauri/src/lib.rs");
const client = read("src/api/client.ts");
const teams = read("src/pages/teams.ts");
const players = read("src/pages/players.ts");
const main = read("src/main.ts");
const pkg = JSON.parse(read("package.json"));
const hash = crypto.createHash("sha256").update(contractText).digest("hex");

requireTrue(isVersionAtLeast(pkg.version, contract.release_version), "当前项目版本早于阶段4版本0.17.0");
requireTrue(contract.release_version === "0.17.0" && contract.integration_point_h_started === false, "阶段4契约边界错误");
requireTrue(migration.includes(hash), "数据库迁移中的月度工作簿契约哈希不匹配");
requireTrue(migration.includes("team_tactical_observations") && migration.includes("team_ability_observations"), "球队历史观察表缺失");
requireTrue(domain.includes("PLAYER_MONTHLY_FORMAT") && domain.includes("TEAM_MONTHLY_FORMAT"), "月度工作簿格式常量缺失");
requireTrue(domain.includes("ReadyEndPrevious") && domain.includes("Clear"), "导入动作或状态契约缺失");

for (const sheet of [...contract.team_sheets, ...contract.player_sheets]) {
  requireTrue(spreadsheetIo.includes(`\"${sheet}\"`), `工作簿缺少工作表：${sheet}`);
}
for (const command of contract.commands) {
  requireTrue(commands.includes(`fn ${command}`), `Rust命令缺失：${command}`);
  requireTrue(registry.includes(command), `Tauri注册缺失：${command}`);
  requireTrue(client.includes(command), `前端API缺失：${command}`);
}
requireTrue(spreadsheetIo.includes("action=clear") && spreadsheetIo.includes("clear_fields"), "显式清空说明缺失");
requireTrue(persistence.includes("source_sha256") && persistence.includes("team_monthly_xlsx"), "球队工作簿幂等链缺失");
requireTrue(persistence.includes("begin().await") && persistence.includes("tx.commit().await"), "球队月度导入未使用事务");
requireTrue(persistence.includes("_resolved_{prefix}_id") && persistence.includes("resolve_entity_id_tx"), "实体匹配结果未进入提交链");
const previewImportSource = persistence.slice(
  persistence.indexOf("pub async fn read_team_monthly_import_preview"),
  persistence.indexOf("pub async fn resolve_team_monthly_import_conflict"),
);
const commitImportSource = persistence.slice(
  persistence.indexOf("pub async fn commit_team_monthly_import"),
  persistence.indexOf("pub async fn abort_team_monthly_import"),
);
requireTrue(
  previewImportSource.includes("let rows = sqlx::query(")
    && !previewImportSource.includes("let mut rows = sqlx::query(")
    && commitImportSource.includes("let mut rows = sqlx::query(")
    && commitImportSource.includes(".iter_mut()"),
  "球队月度导入查询游标可变性错误：预览 rows 不应为 mut，提交 rows 必须支持 iter_mut",
);
requireTrue(
  persistence.includes("fn normalize_team_type")
    && persistence.includes("fn normalize_team_type_payload")
    && persistence.includes('"nationalteam"')
    && persistence.includes('"国家队"')
    && persistence.includes("normalize_team_type_payload(&mut payload)?")
    && persistence.includes("UPDATE catalog.import_rows SET payload=$2 WHERE id=$1")
    && persistence.includes('values.insert("team_type".into(), Value::String(canonical))')
    && persistence.includes(".bind(team_type)"),
  "球队月度导入缺少预检、既有批次重写或最终写库标准化",
);
requireTrue(
  spreadsheetIo.includes("DataValidation")
    && spreadsheetIo.includes('"team_type" => Some(&["club", "national", "reserve", "youth", "women", "other"])')
    && spreadsheetIo.includes('"team_type" => "club"'),
  "球队月度模板缺少 team_type 标准枚举提示",
);
requireTrue(
  spreadsheetIo.includes("fn cell_text_for_header")
    && spreadsheetIo.includes('matches!(header, "verified_at" | "observed_at")')
    && spreadsheetIo.includes(".as_datetime()")
    && spreadsheetIo.includes("from_naive_utc_and_offset")
    && spreadsheetIo.includes("excel_datetime_cells_are_serialized_by_column_semantics"),
  "月度工作簿读取链未按列语义转换 Excel 日期单元格",
);
requireTrue(
  persistence.includes("fn parse_datetime")
    && persistence.includes("fn normalize_monthly_datetime_payload")
    && persistence.includes('["verified_at", "observed_at"]')
    && persistence.includes('"%Y-%m-%d %H:%M:%S%.f"')
    && persistence.includes("1899, 12, 30")
    && persistence.includes("SecondsFormat::AutoSi")
    && persistence.includes("normalize_monthly_datetime_payload(&mut payload)?")
    && commitImportSource.includes("normalize_monthly_datetime_payload(&mut payload)?")
    && persistence.includes("monthly_datetime_accepts_excel_serial_cells")
    && persistence.includes("monthly_datetime_rejects_unrecognized_text_before_database_write"),
  "球队月度导入缺少 verified_at/observed_at 预检、旧批次重写或 Excel 日期兼容",
);
requireTrue(!persistence.includes("DELETE FROM feature.team_tactical_observations"), "战术观察不得无痕覆盖");
requireTrue(!persistence.includes("DELETE FROM feature.team_ability_observations"), "球队能力观察不得无痕覆盖");
requireTrue(legacyPersistence.includes("PLAYER_MONTHLY_FORMAT") && application.includes("read_player_monthly_workbook"), "球员月度与旧模板兼容链缺失");
requireTrue(teams.includes("球队完整资料包") && main.includes("previewTeamPackageImport"), "球队完整资料包统一导入界面缺失");
requireTrue(main.includes("previewTeamImport") && client.includes("preview_team_monthly_import"), "旧球队月度工作簿兼容链缺失");
requireTrue(players.includes("球员月度工作包") && main.includes("球员月度更新.xlsx"), "球员月度工作包界面缺失");
requireTrue(main.includes("ended_previous_count"), "导入完成结果未显示结束旧记录数量");
console.log("阶段4球队与球员月度工作簿契约验证通过。");
