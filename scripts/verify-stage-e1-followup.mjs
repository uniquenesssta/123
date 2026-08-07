import fs from "node:fs";
import path from "node:path";

const root = path.resolve(import.meta.dirname, "..");
const read = (file) => fs.readFileSync(path.join(root, file), "utf8");
const failures = [];
const check = (condition, message) => { if (!condition) failures.push(message); };

const contract = JSON.parse(read("contracts/stage-e1-followup-contract.json"));
const searchable = read("src/components/searchableSelect.ts");
const lineups = read("src/pages/lineups.ts");
const main = read("src/main.ts");
const players = read("src/pages/players.ts");
const teams = read("src/pages/teams.ts");
const taskCss = read("src/styles/taskWorkspace.css");
const entityCss = read("src/styles/entityCenter.css");
const types = read("src/types.ts");

const playerPersistence = read("crates/persistence-postgres/src/player_catalog.rs");
const domain = (read("crates/domain/src/lib.rs") + read("crates/domain/src/lineup/kind.rs") + read("crates/domain/src/lineup/player.rs") + read("crates/domain/src/lineup/snapshot.rs") + read("crates/domain/src/lineup/preset.rs") + read("crates/domain/src/lineup/chain.rs") + read("crates/domain/src/match_record/status.rs") + read("crates/domain/src/match_record/catalog.rs"));
const footballText = read("src/components/footballText.ts");
check(playerPersistence.includes("alternate_name.name AS alternate_name") && playerPersistence.includes(") alternate_name ON true"), "球员目录后端没有返回原文/英文别名");
check(domain.includes("pub alternate_name: Option<String>") && types.includes("alternate_name: string | null"), "球员双语字段没有贯通 Rust/TypeScript 契约");
check(footballText.includes("hasChineseText(canonical)") && footballText.includes("player.alternate_name"), "球员显示名没有在中文正式名场景回退到英文别名");

check(contract.format_version === "football.stage-e1-followup-contract.v1", "阶段 E1 后续契约版本错误");
for (const token of ["querying: boolean", "composing: boolean", "compositionstart", "compositionend", "preserveDraft", "startFreshQuery", "activeQueryDrafts", "captureDetachedActiveController", "resumableSelectId"]) {
  check(searchable.includes(token), `可搜索选择器缺少防回弹能力：${token}`);
}
check(searchable.includes("if (!preserveDraft)") && searchable.includes("controller.input.value = currentSelectedLabel"), "活动搜索词仍可能被刷新覆盖");
check(!searchable.includes("input.select()"), "搜索词恢复后仍可能因全选而被下一次输入覆盖");
check(searchable.includes("if (startFreshQuery) {") && searchable.includes('input.value = "";'), "空值提示没有在开始搜索时清空");
check(lineups.includes("displayPlayerName(item)") && lineups.includes("item.localized_name") && lineups.includes("item.alternate_name") && lineups.includes("item.canonical_name"), "阵容候选没有中英文显示与检索");
check(lineups.includes('data-search="${escapeHtml(searchText)}"'), "阵容候选没有写入双语模糊检索索引");
check(lineups.includes("player_secondary_name") && main.includes("player_secondary_name: playerName.secondary"), "已选阵容没有保留双语姓名");
check(lineups.includes('data-action="create-lineup-pair" ${submitReady ? "" : "disabled"}'), "双方首发未完整时仍允许提交");
check(lineups.includes("还需补齐") && taskCss.includes(".lineup-submit-readiness"), "双方阵容缺少可见补齐提示");
check(lineups.includes('data-team-id="${escapeHtml(team.team_id)}"') && lineups.includes('data-return-section="chain"'), "阵容球员跳转没有携带球队与返回上下文");
check(main.includes("async function openPlayerFromLineup") && main.includes('source: "match_lineup"'), "阵容球员仍依赖球队中心当前选择");
check(main.includes("async function returnToLineupWorkspace") && players.includes("return-to-lineup-workspace"), "球员档案缺少返回阵容链路");
check(types.includes('source: "team_roster" | "match_lineup"') && types.includes('origin_page: "teams" | "lineups"'), "球员导航上下文未覆盖阵容入口");
check(teams.includes("team-directory-profile-action") && teams.includes("team-directory-coach"), "球队目录卡片仍使用易挤压的旧操作排布");
for (const token of [".team-directory-profile-action", "white-space: nowrap", "writing-mode: horizontal-tb", ".team-directory-coach", "text-overflow: ellipsis"]) {
  check(entityCss.includes(token), `球队目录布局缺少：${token}`);
}

if (failures.length) {
  console.error("阶段 E1 后续修复验证失败：");
  failures.forEach((failure) => console.error(`- ${failure}`));
  process.exit(1);
}
console.log("阶段 E1 后续修复验证通过：搜索词不回弹、阵容球员双语检索、完整度门禁、阵容档案跳转与球队目录布局均已锁定。");
