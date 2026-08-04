import fs from "node:fs";
import path from "node:path";
import process from "node:process";

const root = process.cwd();
const read = (relative) => fs.readFileSync(path.join(root, relative), "utf8");
const requireTrue = (condition, message) => {
  if (!condition) throw new Error(message);
};

const contract = JSON.parse(read("contracts/global-name-search-contract.json"));
const helper = read("crates/persistence-postgres/src/name_search.rs");
const playerCatalog = read("crates/persistence-postgres/src/player_catalog.rs");
const teamCatalog = read("crates/persistence-postgres/src/team_catalog.rs");
const entityCatalog = read("crates/persistence-postgres/src/entity_catalog.rs");
const persistenceLib = read("crates/persistence-postgres/src/lib.rs");
const playerPage = read("src/pages/players.ts");
const teamPage = read("src/pages/teams.ts");
const presetPage = read("src/pages/lineupPresets.ts");

requireTrue(contract.contract_id === "football.global-name-search.v1", "全局名称搜索契约编号错误");
requireTrue(contract.requirements.chinese_partial_matching, "契约未要求中文部分匹配");
requireTrue(contract.requirements.english_partial_matching, "契约未要求英文部分匹配");
requireTrue(contract.requirements.localized_names, "契约未覆盖本地化名称");
requireTrue(contract.requirements.alternate_names, "契约未覆盖别名");
requireTrue(contract.requirements.punctuation_insensitive, "契约未要求标点无关匹配");
requireTrue(contract.requirements.latin_diacritic_insensitive, "契约未要求拉丁重音无关匹配");

requireTrue(persistenceLib.includes("mod name_search;"), "全局名称搜索模块未注册");
requireTrue(helper.includes("pub(crate) struct NameSearch"), "缺少统一名称搜索查询对象");
requireTrue(helper.includes('format!("%{token}%")'), "名称搜索仍未使用包含匹配");
requireTrue(helper.includes("regexp_replace"), "名称搜索未处理空格与标点差异");
requireTrue(helper.includes("LATIN_FOLD_SOURCE"), "名称搜索未处理拉丁重音字符");
requireTrue(helper.includes("character.is_alphanumeric()"), "名称搜索未统一中英文字符归一化");
requireTrue(helper.includes("alias.normalized_name"), "名称搜索未覆盖别名归一化字段");
requireTrue(helper.includes("alias.name"), "名称搜索未覆盖别名原始显示字段");

const combined = `${playerCatalog}\n${teamCatalog}\n${entityCatalog}`;
const helperUsages = (combined.match(/NameSearch::parse\(/g) ?? []).length;
requireTrue(helperUsages >= 7, `全局名称搜索接入点不足：${helperUsages}/7`);
for (const source of [playerCatalog, teamCatalog, entityCatalog]) {
  requireTrue(!source.includes('format!("{search}%")'), "仍残留仅前缀匹配逻辑");
  requireTrue(!/normalized_name LIKE \$1 \|\| '%'/u.test(source), "实体引用仍残留仅前缀匹配SQL");
}

requireTrue(playerPage.includes("支持中文名、原名或别名的部分匹配"), "球员搜索提示未说明中英文部分匹配");
requireTrue(teamPage.includes("支持中英文球队名称或别名的部分匹配"), "球队搜索提示未说明中英文部分匹配");
requireTrue(presetPage.includes("支持中英文球队名称或别名的部分匹配"), "阵容预设球队搜索提示未同步");

const foldMap = new Map([
  ...["á", "à", "â", "ä", "ã", "å", "ā", "ă", "ą"].map((value) => [value, "a"]),
  ...["ç", "ć", "č"].map((value) => [value, "c"]),
  ...["ď", "đ"].map((value) => [value, "d"]),
  ...["é", "è", "ê", "ë", "ē", "ė", "ę", "ě"].map((value) => [value, "e"]),
  ...["í", "ì", "î", "ï", "ī", "į"].map((value) => [value, "i"]),
  ["ł", "l"],
  ...["ñ", "ń"].map((value) => [value, "n"]),
  ...["ó", "ò", "ô", "ö", "õ", "ø", "ō", "ő"].map((value) => [value, "o"]),
  ["ř", "r"],
  ...["ś", "š"].map((value) => [value, "s"]),
  ...["ú", "ù", "û", "ü", "ū", "ů", "ű"].map((value) => [value, "u"]),
  ...["ý", "ÿ"].map((value) => [value, "y"]),
  ...["ž", "ź", "ż"].map((value) => [value, "z"]),
]);
const normalize = (value) => [...value.trim().toLocaleLowerCase("zh-CN")]
  .map((character) => foldMap.get(character) ?? character)
  .map((character) => /[\p{L}\p{N}]/u.test(character) ? character : " ")
  .join("")
  .replace(/\s+/gu, " ")
  .trim();
const compact = (value) => normalize(value).replace(/\s+/gu, "");
const matches = (query, names) => {
  const tokens = normalize(query).split(" ").filter(Boolean);
  const normalizedNames = names.map(normalize);
  const compactNames = names.map(compact);
  return tokens.every((token) => {
    const compactToken = compact(token);
    return normalizedNames.some((name) => name.includes(token))
      || compactNames.some((name) => name.includes(compactToken));
  });
};

for (const example of contract.examples) {
  requireTrue(matches(example.query, example.names) === example.expected_match,
    `搜索示例失败：${example.query}`);
}

console.log(`全局中英文名称搜索验证通过：${helperUsages} 个后端入口已统一，${contract.examples.length} 个中英文示例匹配正确。`);
