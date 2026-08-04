import fs from "node:fs";

const read = (path) => fs.readFileSync(new URL(`../${path}`, import.meta.url), "utf8");
const check = (condition, message) => {
  if (!condition) throw new Error(message);
};

const exchange = read("crates/persistence-postgres/src/spreadsheet_exchange.rs");
const domain = read("crates/domain/src/lib.rs");
const migration = read("crates/persistence-postgres/migrations/0044_team_package_preview_recovery.sql");
const types = read("src/types.ts");
const players = read("src/pages/players.ts");
const packageWriter = read("crates/spreadsheet-io/src/team_package.rs");

check(exchange.includes('"public_roster_initialization"') && exchange.includes('"calculation".to_string()'), "公开名单初始化来源未归一为 calculation");
check(exchange.includes('"questionable" => "doubtful"'), "questionable 未归一为 doubtful");
check(exchange.includes("normalize_reference_name(package_name) == normalized_name"), "完整资料包球队名称未接入延迟关联");
check(exchange.includes('ReferenceResolution::DeferredExternal'), "球队名称延迟关联未保留包内球队身份");
check(domain.includes("Unavailable") && domain.includes('Self::Unavailable => "unavailable"'), "领域层缺少 unavailable 状态");
check(migration.includes("player_availability_status_check") && migration.includes("lineup_players_availability_status_check") && migration.includes("'unavailable'"), "0044 未升级两处可用状态约束");
check(types.includes('| "unavailable"'), "TypeScript 可用状态联合类型缺少 unavailable");
check(players.includes('unavailable: "不可出场"'), "球员页缺少 unavailable 中文显示");
check(packageWriter.includes('"unavailable"'), "球队完整资料包下拉校验缺少 unavailable");

console.log("球队完整资料包预检恢复门禁通过：动态来源、批次球队延迟关联与可用状态别名已统一。");
