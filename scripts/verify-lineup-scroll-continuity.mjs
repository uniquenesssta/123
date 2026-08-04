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
const failures = [];
const check = (condition, message) => { if (!condition) failures.push(message); };

const lineups = read("src/pages/lineups.ts");
const viewState = read("src/app/viewState.ts");

for (const key of [
  "lineups-builder",
  "lineups-chain",
  "lineups-history",
  "lineups-workbook",
  "lineups-match-list",
  "lineups-match-detail",
]) {
  check(lineups.includes(`data-workspace-scroll-key=\"${key}\"`), `比赛中心缺少真实滚动容器键：${key}`);
}
check(!lineups.includes('data-workspace-scroll-key="lineups-main"'), "不得继续把不负责滚动的 balanced-workspace-main 当作滚动容器");
check(
  viewState.indexOf("window.requestAnimationFrame") < viewState.indexOf("snapshot.internal_scrolls", viewState.indexOf("window.requestAnimationFrame")),
  "内部滚动位置必须在布局完成后的 animation frame 恢复",
);

if (failures.length) {
  console.error("阵容滚动连续性静态验证失败：");
  failures.forEach((failure) => console.error(`- ${failure}`));
  process.exit(1);
}

const browser = resolveChromiumExecutable();
if (!browser) throw new Error("未找到 Chromium、Chrome 或 Edge，无法验证阵容滚动连续性");

const source = viewState.replaceAll("export ", "");
const javascript = ts.transpileModule(source, {
  compilerOptions: {
    target: ts.ScriptTarget.ES2022,
    module: ts.ModuleKind.None,
    strict: true,
  },
}).outputText;

const html = `<!doctype html><html><head><meta charset="utf-8"><style>
body{margin:0;font:14px system-ui}.test-marker{position:fixed;left:0;top:0;width:20px;height:20px;background:#f00;z-index:10}.viewport{height:160px;overflow:auto;border:1px solid #999}.content{height:1200px;padding:12px}
</style></head><body><div class="test-marker"></div><div id="host"></div><script>${javascript}
const adapter={
  document:{schema_version:1,global:{sidebar_collapsed:false,ui_revision:4},modules:{}},
  async read(){return this.document},
  async save(document){this.document=document},
  async clear(){this.document={schema_version:1,global:{sidebar_collapsed:false,ui_revision:4},modules:{}};return this.document},
};
const store=new WorkspaceStateStore(adapter);
const host=document.querySelector('#host');
const markup=()=>'<div id="root"><section class="viewport" data-workspace-scroll-key="lineups-builder"><div class="content">阵容编辑内容</div></section></div>';
(async()=>{
  try{
    await store.initialize();
    host.innerHTML=markup();
    const first=host.querySelector('[data-workspace-scroll-key="lineups-builder"]');
    first.scrollTop=437;
    store.capture('lineups',host.querySelector('#root'),false);
    host.innerHTML=markup();
    const second=host.querySelector('[data-workspace-scroll-key="lineups-builder"]');
    if(second.scrollTop!==0)throw new Error('new DOM did not start at top');
    store.restore('lineups',host.querySelector('#root'));
    requestAnimationFrame(()=>requestAnimationFrame(()=>{
      const pass=Math.abs(second.scrollTop-437)<=1;
      document.querySelector('.test-marker').style.background=pass?'#00ff00':'#ff0000';
      document.body.dataset.testResult=pass?'pass':'fail';
      document.body.dataset.scrollTop=String(second.scrollTop);
    }));
  }catch(error){document.body.dataset.testResult='fail';document.body.dataset.error=String(error)}
})();</script></body></html>`;

const temporaryDirectory = fs.mkdtempSync(path.join(os.tmpdir(), "football-lineup-scroll-"));
const htmlPath = path.join(temporaryDirectory, "fixture.html");
const screenshotPath = path.join(temporaryDirectory, "fixture.png");
fs.writeFileSync(htmlPath, html);
try {
  await captureHtmlScreenshot({ browser, htmlPath, outputPath: screenshotPath, width: 640, height: 300 });
  const image = decodePng(screenshotPath);
  const offset = (5 * image.width + 5) * image.channels;
  const red = image.pixels[offset];
  const green = image.pixels[offset + 1];
  const blue = image.pixels[offset + 2];
  if (!(green > 220 && red < 40 && blue < 40)) {
    throw new Error(`阵容滚动恢复浏览器验证未通过，结果标记 RGB=${red},${green},${blue}`);
  }
  console.log("阵容滚动连续性验证通过：加入球员导致 DOM 重绘后，双方阵容分区仍恢复到操作前滚动位置。 ");
} finally {
  fs.rmSync(temporaryDirectory, { recursive: true, force: true });
}
