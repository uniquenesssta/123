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
const browser = resolveChromiumExecutable();
if (!browser) throw new Error("未找到 Chromium、Chrome 或 Edge，无法验证可搜索下拉框交互");

const source = fs
  .readFileSync(path.join(root, "src/components/searchableSelect.ts"), "utf8")
  .replaceAll("export function", "function");
const javascript = ts.transpileModule(source, {
  compilerOptions: {
    target: ts.ScriptTarget.ES2022,
    module: ts.ModuleKind.None,
    strict: true,
  },
}).outputText;
const componentCss = fs.readFileSync(path.join(root, "src/styles/components.css"), "utf8");
const searchableCss = componentCss.slice(componentCss.indexOf("/* Searchable hierarchy select"));
const html = `<!doctype html><html><head><meta charset="utf-8"><style>
:root{--control-height:38px;--line:#cbd5e1;--input:#fff;--panel:#fff;--text:#0f172a;--muted:#64748b;--accent:#2563eb;--soft:#f1f5f9;--soft-hover:#eff6ff}
body{font:14px system-ui;margin:30px;background:#f8fafc}.grid{display:grid;grid-template-columns:repeat(4,1fr);gap:14px}.field{display:grid;gap:7px;padding:14px;background:#fff;border:1px solid #dbe3ee;border-radius:12px}.field>span{font-weight:700}.test-marker{position:fixed;left:0;top:0;width:20px;height:20px;z-index:9999;background:#ff0000}${searchableCss}</style></head><body>
<div class="test-marker"></div><div class="grid">
<label class="field"><span>1级 · 参赛体系</span><select id="scope" data-searchable-select><option value="">选择参赛体系</option><option value="national">国家队赛事</option><option value="club">俱乐部赛事</option></select></label>
<label class="field"><span>2级 · 地区/足联</span><select id="region" data-searchable-select disabled><option value="">选择地区或足联</option><option value="欧洲 / UEFA" data-scope="club">欧洲 / UEFA</option><option value="亚洲 / AFC" data-scope="club">亚洲 / AFC</option></select></label>
<label class="field"><span>3级 · 具体赛事</span><select id="competition" data-searchable-select disabled><option value="">选择具体赛事</option><option value="ucl" data-region="欧洲 / UEFA" data-search="Champions UCL 欧冠">UEFA Champions League · 欧冠</option><option value="uel" data-region="欧洲 / UEFA" data-search="Europa UEL 欧联">UEFA Europa League · 欧联</option></select></label>
<label class="field" id="player-field"><span>阵容球员</span><select id="player" data-searchable-select data-search-placeholder="输入中文名或原文名"><option value="">选择球员</option><option value="p1" data-search="乌戈·索萨 Hugo Souza GK 门将 可用">乌戈·索萨（Hugo Souza） · 门将 · 可用</option><option value="p2" data-search="费利佩·隆戈 Felipe Longo GK 门将 可用">费利佩·隆戈（Felipe Longo） · 门将 · 可用</option></select></label>
</div><script>${javascript}
const scope=document.querySelector('#scope');const region=document.querySelector('#region');const competition=document.querySelector('#competition');
function sync(){region.disabled=!scope.value;competition.disabled=!region.value;for(const option of region.options){if(option.value){option.hidden=option.dataset.scope!==scope.value;option.disabled=option.hidden}}for(const option of competition.options){if(option.value){option.hidden=option.dataset.region!==region.value;option.disabled=option.hidden}}refreshSearchableSelects(document)}
scope.addEventListener('change',()=>{region.value='';competition.value='';sync()});region.addEventListener('change',()=>{competition.value='';sync()});
enhanceSearchableSelects(document);sync();
const controls=[...document.querySelectorAll('.searchable-select-input')];
function typeAndChoose(index,query,needle,useKeyboard=false){const input=controls[index];input.focus();input.value=query;input.dispatchEvent(new Event('input',{bubbles:true}));const options=[...input.closest('.searchable-select').querySelectorAll('.searchable-select-option')];const match=options.find(item=>item.textContent.includes(needle));if(!match)throw new Error('missing '+needle);if(useKeyboard){input.dispatchEvent(new KeyboardEvent('keydown',{key:'ArrowDown',bubbles:true}));input.dispatchEvent(new KeyboardEvent('keydown',{key:'Enter',bubbles:true}))}else match.click()}
try{
  typeAndChoose(0,'club','俱乐部赛事');
  typeAndChoose(1,'uef','欧洲 / UEFA');
  typeAndChoose(2,'ucl','UEFA Champions League',true);
  let playerInput=document.querySelector('#player-field .searchable-select-input');
  playerInput.focus();
  playerInput.dispatchEvent(new InputEvent('beforeinput',{bubbles:true,inputType:'insertText',data:'H'}));
  playerInput.setRangeText('Hug',playerInput.selectionStart??0,playerInput.selectionEnd??0,'end');
  playerInput.dispatchEvent(new InputEvent('input',{bubbles:true,inputType:'insertText',data:'Hug'}));
  refreshSearchableSelects(document);
  if(playerInput.value!=='Hug')throw new Error('refresh bounced active query');
  const field=document.querySelector('#player-field');
  field.innerHTML='<span>阵容球员</span><select id="player" data-searchable-select data-search-placeholder="输入中文名或原文名"><option value="">选择球员</option><option value="p1" data-search="乌戈·索萨 Hugo Souza GK 门将 可用">乌戈·索萨（Hugo Souza） · 门将 · 可用</option><option value="p2" data-search="费利佩·隆戈 Felipe Longo GK 门将 可用">费利佩·隆戈（Felipe Longo） · 门将 · 可用</option></select>';
  enhanceSearchableSelects(field);
  queueMicrotask(()=>{
    try{
      playerInput=document.querySelector('#player-field .searchable-select-input');
      if(playerInput.value!=='Hug')throw new Error('rerender bounced active query');
      if(playerInput.selectionStart!==3||playerInput.selectionEnd!==3)throw new Error('restored query selection is not collapsed at end');
      playerInput.setRangeText('o',playerInput.selectionStart,playerInput.selectionEnd,'end');
      playerInput.dispatchEvent(new Event('input',{bubbles:true}));
      if(playerInput.value!=='Hugo')throw new Error('next character replaced restored query');
      playerInput.dispatchEvent(new CompositionEvent('compositionstart',{data:'乌'}));
      playerInput.value='乌戈';
      playerInput.dispatchEvent(new Event('input',{bubbles:true}));
      refreshSearchableSelects(document);
      playerInput.dispatchEvent(new CompositionEvent('compositionend',{data:'乌戈'}));
      if(playerInput.value!=='乌戈')throw new Error('IME query bounced');
      const playerOptions=[...playerInput.closest('.searchable-select').querySelectorAll('.searchable-select-option')];
      const bilingual=playerOptions.some(item=>item.textContent.includes('乌戈·索萨')&&item.textContent.includes('Hugo Souza'));
      const pass=scope.value==='club'&&region.value==='欧洲 / UEFA'&&competition.value==='ucl'&&bilingual;
      document.querySelector('.test-marker').style.background=pass?'#00ff00':'#ff0000';
      document.body.dataset.testResult=pass?'pass':'fail';
      document.body.dataset.values=[scope.value,region.value,competition.value,playerInput.value].join('|');
    }catch(error){document.body.dataset.testResult='fail';document.body.dataset.error=String(error)}
  });
}catch(error){document.body.dataset.testResult='fail';document.body.dataset.error=String(error)}</script></body></html>`;

const temporaryDirectory = fs.mkdtempSync(path.join(os.tmpdir(), "football-searchable-select-"));
const htmlPath = path.join(temporaryDirectory, "fixture.html");
const screenshotPath = path.join(temporaryDirectory, "fixture.png");
fs.writeFileSync(htmlPath, html);
try {
  await captureHtmlScreenshot({ browser, htmlPath, outputPath: screenshotPath, width: 1200, height: 360 });
  const image = decodePng(screenshotPath);
  const offset = (5 * image.width + 5) * image.channels;
  const red = image.pixels[offset];
  const green = image.pixels[offset + 1];
  const blue = image.pixels[offset + 2];
  if (!(green > 220 && red < 40 && blue < 40)) {
    throw new Error(`可搜索下拉框浏览器交互未通过，结果标记 RGB=${red},${green},${blue}`);
  }
  console.log("可搜索下拉框浏览器交互验证通过：输入模糊匹配、跨刷新防回弹、中文 IME、双语球员检索、三级联动及键盘选择均正常。");
} finally {
  fs.rmSync(temporaryDirectory, { recursive: true, force: true });
}
