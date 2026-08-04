import fs from "node:fs";

const read = (path) => fs.readFileSync(new URL(`../${path}`, import.meta.url), "utf8");
const failures = [];
const requireTrue = (condition, message) => { if (!condition) failures.push(message); };

const io = read("crates/spreadsheet-io/src/team_package.rs");
const app = read("crates/application/src/spreadsheet.rs");
const persistence = read("crates/persistence-postgres/src/monthly_workbooks.rs");

for (const sheet of ["球队名称", "球员名称"]) {
  requireTrue(io.includes(`set_name("${sheet}")`), `完整资料包未生成工作表：${sheet}`);
  requireTrue(io.includes(`read_business_rows(workbook, "${sheet}"`), `完整资料包未读取工作表：${sheet}`);
}
for (const field of ["name_value", "language_code", "is_primary", "valid_from", "valid_to"]) {
  requireTrue(io.includes(`"${field}"`), `多语言名称工作表缺少字段：${field}`);
}
requireTrue(io.includes('name.insert("language_code".into(), Value::String("en".into()))'), "兼容英文名列未写入明确 en 语言代码");
requireTrue(io.includes('workbook.sheet_names().iter().any(|name| name == "球队名称")'), "旧版资料包缺少球队名称工作表时未保持兼容");
requireTrue(io.includes('workbook.sheet_names().iter().any(|name| name == "球员名称")'), "旧版资料包缺少球员名称工作表时未保持兼容");
requireTrue(app.includes("visible_sheet_count: 7"), "完整资料包可见工作表数量未更新为 7");
requireTrue(persistence.includes('text(values, "is_primary")') && persistence.includes("UPDATE football.teams SET canonical_name"), "球队主显示名未同步到 canonical_name");
requireTrue(persistence.includes("INSERT INTO football.team_names"), "球队多语言名称未写入 team_names");

if (failures.length) {
  console.error("球队与球员多语言资料包验证失败：");
  failures.forEach((failure) => console.error(`- ${failure}`));
  process.exit(1);
}
console.log("球队与球员多语言资料包验证通过：中文名、英文名、历史名、语言代码和主显示名链路完整。 ");
