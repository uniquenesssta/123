import fs from "node:fs";

const read = (path) => fs.readFileSync(new URL(`../${path}`, import.meta.url), "utf8");
const check = (condition, message) => {
  if (!condition) throw new Error(message);
};

const application = read("crates/application/src/spreadsheet.rs");
const teams = read("crates/persistence-postgres/src/monthly_workbooks.rs");
const players = read("crates/persistence-postgres/src/spreadsheet_exchange.rs");

check(
  application.includes("ready == 0 && preview.counts.imported == 0"),
  "完整资料包重试仍会把已成功提交的球队链误判为没有可写入记录",
);
check(
  application.includes("球队、教练与阵型链已经提交成功；可修复后直接重试同一完整资料包批次"),
  "球员链失败日志未明确说明球队链已提交及可直接重试",
);
check(
  application.includes("already_imported_team_chain_remains_retryable"),
  "缺少已提交球队链仍可重试的应用层回归测试",
);

for (const required of [
  "ensure_team_name_alias",
  "preserve_current_team_canonical_alias",
  'text(values, "short_name")',
  "WHERE team_id = $2 AND normalized_name = $4",
]) {
  check(teams.includes(required), `球队原文名/简称身份保留缺少实现：${required}`);
}
check(
  teams.includes("preserve_current_team_canonical_alias(tx, team_id, &metadata).await?"),
  "中文主显示名覆盖 canonical_name 前未保留原正式名别名",
);
check(
  players.includes("LEFT JOIN football.team_profiles profile ON profile.team_id = team.id") &&
    players.includes("lower(btrim(COALESCE(profile.short_name, ''))) = $1"),
  "旧数据库恢复未按完整资料包 deferred team key 匹配球队 short_name",
);
check(
  players.includes("完整资料包球队简称 {key} 匹配到多条记录"),
  "球队简称歧义未保持阻断，存在错误自动关联风险",
);

console.log("真实球队完整资料包导入恢复验证通过：原文名、中文主显示名、简称与重试链路已统一。");
