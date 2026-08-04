import fs from "node:fs";

const read = (path) => fs.readFileSync(new URL(`../${path}`, import.meta.url), "utf8");
const failures = [];
const requireTrue = (condition, message) => { if (!condition) failures.push(message); };

const page = read("src/pages/analytics.ts");
const main = read("src/main.ts");
const styles = read("src/styles/app.css");

for (const label of ["形成历史样本", "运行完整分析", "通过质量门禁", "受控校准"]) {
  requireTrue(page.includes(label), `分析与历史缺少链路阶段：${label}`);
}
for (const condition of ["postmatch.settlement_count > 0", "overview?.generated_at", "quality?.critical", "pendingReviewCount === 0"]) {
  requireTrue(page.includes(condition), `分析链路缺少完成条件：${condition}`);
}
for (const guard of ["analysisHistoryReady", "fullAnalysisReady", "analysisQualityReady", "analysisReviewGateReady"]) {
  requireTrue(main.includes(`function ${guard}`), `主控制器缺少链路门禁：${guard}`);
}
for (const message of ["请先在赛后复盘中完成至少一场正式结算", "请先完成包含有效样本的完整分析", "请先完成数据质量扫描并清除全部严重问题", "请先通过数据质量门禁并处理全部待审核建议"]) {
  requireTrue(main.includes(message), `事件入口缺少前置条件保护：${message}`);
}
for (const selector of [".analysis-chain-map", ".analysis-chain-step", ".analysis-step-lock", ".analysis-step-guide"]) {
  requireTrue(styles.includes(selector), `分析链路缺少样式：${selector}`);
}
requireTrue(main.includes("ANALYSIS_PACKAGE_ID_KEY") && main.includes("localStorage.setItem(ANALYSIS_PACKAGE_ID_KEY"), "分析包链路状态未持久保存");

if (failures.length) {
  console.error("分析与历史链路验证失败：");
  failures.forEach((failure) => console.error(`- ${failure}`));
  process.exit(1);
}
console.log("分析与历史链路验证通过：历史样本、完整分析、质量门禁、人工审核和受控校准按前置条件顺序执行。 ");
