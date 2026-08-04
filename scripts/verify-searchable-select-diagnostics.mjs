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
const read = (relative) => fs.readFileSync(path.join(root, relative), "utf8");
const contract = JSON.parse(read("contracts/searchable-select-diagnostics-contract.json"));
const component = read("src/components/searchableSelect.ts");
const binding = read("src/diagnostics/searchableSelectDiagnostics.ts");
const client = read("src/api/client.ts");
const rust = read("src-tauri/src/commands/logging.rs");

for (const event of contract.required_events) {
  if (!component.includes(`"${event}"`)) {
    throw new Error(`可搜索选择器诊断缺少事件：${event}`);
  }
}
for (const key of contract.required_context) {
  if (!component.includes(`${key}:`)) {
    throw new Error(`可搜索选择器诊断缺少上下文字段：${key}`);
  }
}
if (!binding.includes("recordFrontendDiagnostic") || !binding.includes(contract.runtime_event)) {
  throw new Error("诊断事件没有接入前端运行日志桥接");
}
if (!client.includes('| "diagnostic"') || !client.includes("recordFrontendDiagnostic")) {
  throw new Error("API 客户端没有声明 diagnostic 运行日志阶段");
}
if (!rust.includes('"diagnostic" => Ok("diagnostic")')) {
  throw new Error("Tauri 运行日志命令不接受 diagnostic 阶段");
}
if (/addEventListener\("input"[\s\S]{0,500}emitDiagnostic\([^\n]+query_changed/.test(component)) {
  throw new Error("禁止为每次普通按键写入 query_changed 运行日志");
}

const browser = resolveChromiumExecutable();
if (!browser) throw new Error("未找到 Chromium、Chrome 或 Edge，无法验证诊断事件");
const javascript = ts.transpileModule(
  component.replaceAll("export function", "function"),
  {
    compilerOptions: {
      target: ts.ScriptTarget.ES2022,
      module: ts.ModuleKind.None,
      strict: true,
    },
  },
).outputText;
const html = `<!doctype html><html><head><meta charset="utf-8"><style>
body{font:14px system-ui;margin:20px}.test-marker{position:fixed;left:0;top:0;width:20px;height:20px;background:#f00}.searchable-select-native{display:none}.searchable-select-listbox[hidden]{display:none}
</style></head><body><div class="test-marker"></div><label id="field"><span>阵容球员</span><select id="player"><option value="">选择球员</option><option value="p1" data-search="乌戈 Hugo Souza">乌戈·索萨（Hugo Souza）</option></select></label><script>
${javascript}
const events=[];document.addEventListener('football:searchable-select-diagnostic',(event)=>events.push(event.detail.event));
enhanceSearchableSelects(document);
let input=document.querySelector('.searchable-select-input');
input.focus();input.value='Hug';input.dispatchEvent(new Event('input',{bubbles:true}));
const field=document.querySelector('#field');field.innerHTML='<span>阵容球员</span><select id="player"><option value="">选择球员</option><option value="p1" data-search="乌戈 Hugo Souza">乌戈·索萨（Hugo Souza）</option></select>';
enhanceSearchableSelects(field);
queueMicrotask(()=>{
 input=document.querySelector('.searchable-select-input');
 const required=['query_session_started','dom_detached_during_query','query_restored_after_dom_rebuild'];
 const pass=required.every(name=>events.includes(name))&&input.value==='Hug';
 document.querySelector('.test-marker').style.background=pass?'#0f0':'#f00';
 document.body.dataset.events=events.join('|');document.body.dataset.result=pass?'pass':'fail';
});
</script></body></html>`;

const temporaryDirectory = fs.mkdtempSync(path.join(os.tmpdir(), "football-select-diagnostics-"));
const htmlPath = path.join(temporaryDirectory, "fixture.html");
const screenshotPath = path.join(temporaryDirectory, "fixture.png");
fs.writeFileSync(htmlPath, html);
try {
  await captureHtmlScreenshot({ browser, htmlPath, outputPath: screenshotPath, width: 800, height: 300 });
  const image = decodePng(screenshotPath);
  const offset = (5 * image.width + 5) * image.channels;
  const [red, green, blue] = image.pixels.slice(offset, offset + 3);
  if (!(green > 220 && red < 40 && blue < 40)) {
    throw new Error(`选择器诊断浏览器验证失败，结果标记 RGB=${red},${green},${blue}`);
  }
} finally {
  fs.rmSync(temporaryDirectory, { recursive: true, force: true });
}
console.log("可搜索选择器异常诊断验证通过：会话、DOM 脱离、查询恢复和运行日志桥接均已锁定，且不记录每次普通按键。");
