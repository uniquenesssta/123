import fs from "node:fs";
import path from "node:path";

const root = path.resolve(import.meta.dirname, "..");
const read = (file) => fs.readFileSync(path.join(root, file), "utf8");
const check = (condition, message) => {
  if (!condition) throw new Error(message);
};

const lineups = read("src/pages/lineups.ts");
const main = read("src/main.ts");
const component = read("src/components/searchableSelect.ts");
const css = read("src/styles/components.css");
const persistence = read("crates/persistence-postgres/src/monthly_workbooks.rs");

for (const id of [
  "new-match-competition-scope",
  "new-match-competition-region",
  "new-match-competition",
]) {
  check(
    new RegExp(`id=\\"${id}\\"[^>]*data-searchable-select`).test(lineups),
    `赛事层级选择器 ${id} 未启用可输入模糊匹配`,
  );
}
check(main.includes("enhanceSearchableSelects(currentPageRoot)"), "赛事页面渲染后未初始化可搜索选择器");
check(main.includes("refreshSearchableSelects(app)"), "赛事三级联动后未同步可搜索选择器状态");
check(
  lineups.includes('data-search="${escapeHtml(searchText)}"'),
  "具体赛事选项未提供名称、代码、别名和地区搜索文本",
);
for (const behavior of [
  "fuzzyScore",
  "isSubsequence",
  'event.key === "ArrowDown"',
  'event.key === "Enter"',
  'event.key === "Escape"',
  'dispatchEvent(new Event("change", { bubbles: true }))',
  'role", "combobox"',
  'role", "listbox"',
]) {
  check(component.includes(behavior), `可搜索选择器缺少行为：${behavior}`);
}
for (const selector of [
  ".searchable-select-listbox",
  ".searchable-select-option",
  ".searchable-select-input",
  ".searchable-select.disabled",
]) {
  check(css.includes(selector), `可搜索选择器缺少样式：${selector}`);
}
check(
  persistence.includes("consolidate_duplicate_ready_add_team_rows_by_source(&mut tx, &mut rows).await?"),
  "球队资料包提交未按来源合并翻译名/原文名重复球队",
);
check(
  persistence.includes("let duplicate_source_groups = groups") &&
    persistence.includes(".into_values()") &&
    persistence.includes("for indices in duplicate_source_groups"),
  "同来源球队分组必须先收集独立索引，再修改导入行，避免 Rust E0506 借用冲突",
);
check(
  !persistence.includes("for indices in groups.values().filter(|indices| {"),
  "禁止在借用 rows 的 groups.values().filter 迭代器生命周期内修改 rows",
);
check(
  persistence.includes('rows[*index].sheet_name == "球队总览"') &&
    persistence.includes('rows[*index].sheet_name == "球员与评分"'),
  "同来源球队合并必须限制为球队总览与球员表推导记录，避免共用来源误合并不同球队",
);
check(
  persistence.includes("bind_batch_team_references(&mut tx, &mut rows).await?"),
  "球队资料包提交未建立批次内稳定球队 UUID 绑定",
);
check(
  persistence.includes('values.insert("_resolved_team_id".into(), json!(team_id))'),
  "球队依赖行未写入批次内已解析球队 UUID",
);
check(
  persistence.includes("package_team_reference_uses_unique_source_when_name_changed"),
  "缺少球队正式名被本地化后仍可按来源绑定的回归测试",
);

console.log("赛事三级可搜索选择器与球队资料包批次绑定验证通过。");
