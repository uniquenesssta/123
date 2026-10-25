import assert from "node:assert/strict";
import { readFileSync, writeFileSync, existsSync, mkdtempSync, mkdirSync, copyFileSync, readdirSync } from "node:fs";
import { resolve, relative, dirname, join } from "node:path";
import { tmpdir } from "node:os";
import { createRequire } from "node:module";
import { readStylesheet } from "../tokens/tokenStyles.mjs";
import { resolveChromiumExecutable, captureHtmlScreenshot } from "../task-ui-screenshot-tools.mjs";

export async function verifyDataDisplay(root) {
  const read = file => readFileSync(resolve(root, file), "utf8");
  const ts = createRequire(import.meta.url)("typescript"), modules = {};
  const bundle = file => {
    if (modules[file]) return;
    const output = ts.transpileModule(read(file), { compilerOptions: { target: ts.ScriptTarget.ES2022, module: ts.ModuleKind.CommonJS } }).outputText;
    modules[file] = { output, imports: {} };
    for (const [, specifier] of output.matchAll(/require\("([^"]+)"\)/g)) {
      assert.ok(specifier.startsWith("."), `Primitives must not acquire external runtime dependencies: ${specifier}`);
      const base = resolve(root, dirname(file), specifier);
      if (specifier.endsWith(".svg?raw")) {
        const target = relative(root, base).replaceAll("\\", "/"), svg = readFileSync(base.replace(/\?raw$/, ""), "utf8");
        assert.match(svg, /<svg\b[^>]*\bwidth="[\d.]+"[^>]*\bheight="[\d.]+"/);
        assert.ok(!/<(?:script|foreignObject)\b|\bon[a-z]+\s*=|(?:href|src)=["']https?:/i.test(svg), `Nonlocal or active SVG metadata: ${target}`);
        modules[file].imports[specifier] = target; modules[target] = { output: `exports.default=${JSON.stringify(svg)}`, imports: {} }; continue;
      }
      const target = relative(root, existsSync(`${base}.ts`) ? `${base}.ts` : join(base, "index.ts")).replaceAll("\\", "/");
      assert.ok(target.startsWith("src/") && !target.includes("../")); modules[file].imports[specifier] = target; bundle(target);
    }
  };

  const entry="src/ui/data-display/__tests__/displayBrowserFixture.ts";bundle(entry);
  const runtime="const modules="+JSON.stringify(modules)+";const cache={};function load(id){if(cache[id])return cache[id];const record=modules[id],exports={};cache[id]=exports;new Function('exports','require',record.output)(exports,s=>load(record.imports[s]));return exports;}";
  const main=read("src/main.ts");assert.equal((main.match(/import "\.\/ui\/data-display\/index.css";/g)??[]).length,1,"Sole Data Display style entry");
  const styles=[...main.matchAll(/^import "(\.\/[^"\n]+\.css)";/gm)].map(m=>m[1].replace("./","src/")).map(file=>readStylesheet(root,file).replace(/url\("([^"\n]+\.svg)"\)/g,(_m,asset)=>'url("data:image/svg+xml;base64,'+readFileSync(resolve(root,dirname(file),asset)).toString("base64")+'")')).join("\n");
  const defined=new Set([...styles.matchAll(/(--[a-z0-9-]+)\s*:/g)].map(m=>m[1]));for(const [,name]of readStylesheet(root,"src/ui/data-display/index.css").matchAll(/var\((--[a-z0-9-]+)/g))assert.ok(defined.has(name),"Unresolved display token: "+name);
  const families=["data-table","key-value-list","stat-card","status-badge","timeline","progress","empty-state","code-block"];
  for(const dir of families){const folder="src/ui/data-display/"+dir,index=read(folder+"/index.ts");assert.ok(index.trim().split("\n").every(l=>/^export (?:type )?\{.*\} from /.test(l)),"Explicit re-export only "+dir);for(const file of readdirSync(resolve(root,folder)).filter(n=>n.endsWith(".ts")))assert.ok(!/\bfetch\(|\binvoke\(|localStorage|sessionStorage|from ["'][^"']*(?:api\/|platform\/|pages\/)/.test(read(folder+"/"+file)),"Display must not acquire I/O/domain dependencies: "+file);}
  const browser=resolveChromiumExecutable();assert.ok(browser,"Native Windows Chromium required");const directory=mkdtempSync(join(tmpdir(),"football-r12-display-")),results=[];
  for(const width of [960,1440])for(const theme of ["light","dark"]){
    const name="display-"+width+"-"+theme,htmlPath=join(directory,name+".html");
    writeFileSync(htmlPath,'<!doctype html><html lang="zh-CN" data-theme="'+theme+'"><head><meta charset="utf-8"><style>'+styles+'\nbody{margin:0;padding:16px;background:var(--fip-canvas);color:var(--fip-text)}.display-gallery{display:grid;grid-template-columns:repeat(2,minmax(0,1fr));gap:16px;align-items:start}.display-gallery > section{display:flex;flex-direction:column;gap:12px;min-width:0;padding:12px;border:1px solid var(--fip-border);border-radius:7px;background:var(--fip-surface)}.display-gallery h2{margin:0;font:600 14px/20px var(--fip-type-body-family)}.display-stats{display:grid;grid-template-columns:repeat(2,minmax(0,1fr));gap:8px}.ui-code-block{max-height:220px}#after-display{margin-top:16px}</style></head><body><div id="display-root"></div><script>'+runtime.replaceAll('</script','<\\/script')+';try{window.displayFixture=load('+JSON.stringify(entry)+').installDisplayFixture()}catch(error){window.displayFixtureError=String(error.stack)}</script></body></html>');
    const result=await captureHtmlScreenshot({browser,htmlPath,outputPath:join(directory,name+".png"),width,height:2100,async exercise(cdp){
      const evaluate=async expression=>{const r=await cdp.send("Runtime.evaluate",{expression,awaitPromise:true,returnByValue:true});if(r.exceptionDetails)throw Error(r.exceptionDetails.exception?.description??r.exceptionDetails.text);return r.result?.value;};
      const key=async(key,code,vk,modifiers=0)=>{await cdp.send("Input.dispatchKeyEvent",{type:"keyDown",key,code,windowsVirtualKeyCode:vk,modifiers,...(key==="Enter"?{text:"\r"}:{})});await cdp.send("Input.dispatchKeyEvent",{type:"keyUp",key,code,windowsVirtualKeyCode:vk,modifiers});};
      await cdp.send("Page.bringToFront");const error=await evaluate("window.displayFixtureError");assert.ok(!error,error);
      await evaluate('window.f=window.displayFixture;f.structure();f.production();f.contracts();f.q("#loaded .ui-table-sort").focus()');await key("Enter","Enter",13);
      await evaluate('f.expect(f.sorts.length===1&&f.sorts[0]==="score:descending","native Enter reports sort gesture exactly once");f.expect(f.q("#loaded tbody tr").dataset.tableRow==="a","sorting gesture does not fabricate host row order");f.q("#loaded [data-table-select=b]").focus()');await key(" ","Space",32);
      await evaluate('f.expect(f.selections.includes("b:true"),"native Space reports row selection");f.q("#loaded [data-table-select-all]").focus()');await key(" ","Space",32);
      await evaluate('f.expect(f.selections.includes("all:true"),"native Space reports select-all");f.q("#empty-action").focus()');await key("Tab","Tab",9);
      await evaluate('f.expect(document.activeElement===f.q(".ui-code-block"),"Tab reaches native technical text scroll region");f.q(".ui-code-block").scrollTop=0');await key("ArrowDown","ArrowDown",40);
      await evaluate('(async()=>{await new Promise(r=>setTimeout(r,160));f.expect(f.q(".ui-code-block").scrollTop>0,"native keyboard scroll preserves technical text");})()');
      await cdp.send("Emulation.setEmulatedMedia",{features:[{name:"prefers-reduced-motion",value:"reduce"}]});await evaluate('f.expect(getComputedStyle(f.q("#progress-unknown .ui-progress-fill")).animationName==="none","reduced motion retains a static unknown progress marker")');
      await cdp.send("Emulation.setEmulatedMedia",{features:[{name:"forced-colors",value:"active"}]});await evaluate('f.expect(getComputedStyle(f.q("#progress-value .ui-progress")).outlineStyle!=="none","forced colors gives visible progress boundary")');
      await cdp.send("Emulation.setEmulatedMedia",{features:[]});await evaluate('f.cleanup();f.q(".ui-code-block").scrollTop=0;f.q("#after-display").focus();document.activeElement.blur();scrollTo(0,0)');
      return await evaluate('({checks:f.checks,devicePixelRatio,nativeWindows:navigator.userAgent.includes("Windows")})');
    }});assert.ok(result.nativeWindows,"Native Windows browser required");results.push({name,...result});writeFileSync(join(directory,"display-results.json"),JSON.stringify(results,null,2));console.log(name+": "+result.checks.length+" Data Display native checks PASS");
  }
  const evidence=resolve(root,"..","logs","r12-13-display-"+Date.now());mkdirSync(evidence,{recursive:true});for(const file of readdirSync(directory).filter(f=>f.endsWith(".png")||f==="display-results.json"))copyFileSync(join(directory,file),join(evidence,file));console.log("R12-13 Data Display: 8 families / "+results.reduce((sum,r)=>sum+r.checks.length,0)+" native Windows checks PASS; evidence: "+evidence+"; screenshots: "+directory);
}
