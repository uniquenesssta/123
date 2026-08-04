import fs from "node:fs";
import os from "node:os";
import path from "node:path";
import ts from "typescript";
import {
  captureHtmlScreenshot,
  decodePng,
  resolveChromiumExecutable,
} from "./task-ui-screenshot-tools.mjs";

const root = path.resolve(import.meta.dirname, "..");
const read = (file) => fs.readFileSync(path.join(root, file), "utf8");
const check = (condition, message) => {
  if (!condition) throw new Error(message);
};

const contract = JSON.parse(read("contracts/stage-e1-ui-contract.json"));
const component = read("src/components/searchableSelect.ts");
const lineups = read("src/pages/lineups.ts");
const prediction = read("src/pages/prediction.ts");
const css = read("src/styles/taskWorkspace.css");

check(contract.format_version === "football.stage-e1-ui-contract.v1", "阶段 E1 UI 契约版本错误");
for (const token of [
  "eligibleSelects(root)",
  'select:not([multiple])',
  "data-native-select",
  "MutationObserver",
  "ensureSelectId",
  "applyControllerVariant",
]) {
  check(component.includes(token), `全局可搜索选择器缺少：${token}`);
}
check(!component.includes('querySelectorAll<HTMLSelectElement>("select[data-searchable-select]")'), "可搜索选择器仍仅处理手工标记控件");
check(lineups.includes('matches.length <= 4 ? " compact-directory"'), "比赛较少时未启用紧凑目录");
check(css.includes(".match-browser-sidebar.compact-directory") && css.includes("grid-auto-rows: max-content"), "比赛目录仍可能把单场卡片拉伸到页面底部");
check(css.includes("height: auto !important") && css.includes("align-content: start") && css.includes("grid-auto-rows: max-content"), "阵容列表未取消固定大高度或未顶端排列");
for (const token of [
  "modelSelectionGuide",
  "推荐默认",
  "当前比赛类型",
  "P4 是当前默认正式推演系列",
  "需要复核",
]) {
  check(prediction.includes(token), `模型选择说明缺少：${token}`);
}
check(prediction.includes('data-search-placeholder="输入 P4、P7、联赛、淘汰赛或友谊赛"'), "模型选择器未提供可搜索提示");

const browser = resolveChromiumExecutable();
if (!browser) throw new Error("未找到 Chromium、Chrome 或 Edge，无法验证阶段 E1 浏览器交互");
const source = component.replaceAll("export function", "function");
const javascript = ts.transpileModule(source, {
  compilerOptions: { target: ts.ScriptTarget.ES2022, module: ts.ModuleKind.None, strict: true },
}).outputText;
const html = `<!doctype html><html><head><meta charset="utf-8"><style>
:root{--line:#cbd5e1;--input:#fff;--panel:#fff;--surface-soft:#f8fafc;--text:#0f172a;--text-strong:#0f172a;--muted:#64748b;--accent:#2563eb;--accent-soft:#dbeafe;--warning:#d97706;--soft-hover:#eff6ff}
*{box-sizing:border-box}body{margin:0;padding:20px;font:14px system-ui;background:#eef3f8}.test-marker{position:fixed;left:0;top:0;width:20px;height:20px;background:#f00;z-index:9999}.app-shell.workspace-page .balanced-workspace-main .match-browser-layout{display:grid;grid-template-columns:minmax(220px,280px) minmax(0,1fr);gap:14px;align-items:start}.panel{padding:14px;border:1px solid var(--line);border-radius:12px;background:#fff}.match-browser-sidebar{display:grid;gap:10px;align-content:start}.match-browser-sidebar.compact-directory{align-self:start;height:auto;grid-template-rows:auto auto auto auto}.match-list{display:grid;gap:8px;align-content:start;grid-auto-rows:max-content}.match-list-item{padding:12px;border:1px solid var(--line);border-radius:10px}.detail{height:640px}.paired-lineup-side{display:grid;align-content:start;gap:10px;margin-top:18px}.balanced-lineup-list{display:grid;gap:6px;height:auto;min-height:0;max-height:300px;align-content:start;grid-auto-rows:max-content}.balanced-lineup-row{display:grid;grid-template-columns:24px 1fr 90px;gap:8px;align-items:center;min-height:42px;padding:7px;border:1px solid var(--line);border-radius:9px}.model-selection-guide{display:grid;grid-template-columns:repeat(4,1fr);gap:8px;margin-top:18px;padding:10px;border:1px solid var(--line);border-radius:11px;background:var(--surface-soft)}.model-selection-guide>div{display:grid;gap:3px;padding:9px;border:1px solid var(--line);border-radius:9px;background:#fff}.field{display:grid;gap:6px;margin-top:16px}${read("src/styles/components.css").slice(read("src/styles/components.css").indexOf("/* Searchable hierarchy select"))}${css.slice(css.indexOf("/* 0.23.0 · Stage E1"))}</style></head><body><div class="test-marker"></div><div class="app-shell workspace-page"><main class="balanced-workspace-main"><div class="match-browser-layout"><aside class="panel match-browser-sidebar compact-directory"><strong>已创建比赛</strong><input placeholder="搜索比赛"><div class="match-list"><article class="match-list-item">科林蒂安 vs 里奥</article></div><small>点击打开</small></aside><section class="panel detail">比赛编辑主区</section></div><article class="panel paired-lineup-side"><strong>本次阵容</strong><div class="balanced-lineup-list"><article class="balanced-lineup-row"><span>1</span><strong>阿兰</strong><select><option value="starter">首发</option><option value="substitute">替补</option></select></article></div></article><label class="field"><span>模型</span><select><option value="p4">P4 通用函数曲线协同模型 · 推荐：按赛事自动匹配</option><option value="p4_league" data-search="P4 league 联赛">P4 联赛 90 分钟模型</option><option value="p7">P7 通用函数曲线协同模型</option></select></label><aside class="model-selection-guide recommended"><div><span>模型选择建议</span><strong>P4 通用模型</strong><b>推荐默认</b></div><div><span>当前比赛类型</span><strong>联赛</strong></div><div><span>P4 定位</span><strong>正式主推</strong></div><div><span>怎么选</span><strong>不确定时保持自动匹配</strong></div></aside></main></div><script>${javascript}
enhanceSearchableSelects(document);
try{
 const wrappers=[...document.querySelectorAll('.searchable-select')];
 const selects=[...document.querySelectorAll('select')];
 if(wrappers.length!==2)throw new Error('global-select-count:'+wrappers.length);
 if(!selects.every(select=>select.id&&select.dataset.searchableSelect==='enhanced'))throw new Error('select-id-or-enhancement');
 const modelInput=wrappers[1].querySelector('.searchable-select-input');modelInput.focus();modelInput.value='league';modelInput.dispatchEvent(new Event('input',{bubbles:true}));
 const league=[...wrappers[1].querySelectorAll('.searchable-select-option')].find(item=>item.textContent.includes('联赛 90'));if(!league)throw new Error('fuzzy-model');league.click();
 const sidebar=document.querySelector('.match-browser-sidebar').getBoundingClientRect();const detail=document.querySelector('.detail').getBoundingClientRect();if(sidebar.height>=detail.height*.55)throw new Error('sidebar-stretched:'+sidebar.height);
 const list=document.querySelector('.balanced-lineup-list').getBoundingClientRect();const row=document.querySelector('.balanced-lineup-row').getBoundingClientRect();if(row.top-list.top>12||list.height>90)throw new Error('lineup-not-top:'+String(row.top-list.top)+'/'+list.height);
 document.querySelector('.test-marker').style.background='#00ff00';
}catch(error){document.body.dataset.error=String(error)}
</script></body></html>`;

const temporaryDirectory = fs.mkdtempSync(path.join(os.tmpdir(), "football-stage-e1-ui-"));
const htmlPath = path.join(temporaryDirectory, "fixture.html");
const screenshotPath = path.join(temporaryDirectory, "fixture.png");
fs.writeFileSync(htmlPath, html);
try {
  await captureHtmlScreenshot({ browser, htmlPath, outputPath: screenshotPath, width: 1280, height: 900 });
  const image = decodePng(screenshotPath);
  const offset = (5 * image.width + 5) * image.channels;
  const red = image.pixels[offset];
  const green = image.pixels[offset + 1];
  const blue = image.pixels[offset + 2];
  if (!(green > 220 && red < 40 && blue < 40)) {
    throw new Error(`阶段 E1 浏览器布局/交互验证失败，标记 RGB=${red},${green},${blue}`);
  }
  console.log("阶段 E1 验证通过：全局模糊选择、紧凑比赛目录、阵容顶端排列与模型说明均正常。");
} finally {
  fs.rmSync(temporaryDirectory, { recursive: true, force: true });
}
