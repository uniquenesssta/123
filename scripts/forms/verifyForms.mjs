import assert from "node:assert/strict";
import { readFileSync, writeFileSync, existsSync, mkdtempSync, mkdirSync, copyFileSync, readdirSync } from "node:fs";
import { resolve, relative, dirname, join } from "node:path";
import { tmpdir } from "node:os";
import { createRequire } from "node:module";
import { readStylesheet } from "../tokens/tokenStyles.mjs";
import { resolveChromiumExecutable, captureHtmlScreenshot } from "../task-ui-screenshot-tools.mjs";

export async function verifyForms(root) {
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

  const entry = "src/ui/forms/__tests__/formsBrowserFixture.ts"; bundle(entry);
  const runtime = "const modules="+JSON.stringify(modules)+";const cache={};function load(id){if(cache[id])return cache[id];const record=modules[id],exports={};cache[id]=exports;new Function('exports','require',record.output)(exports,s=>load(record.imports[s]));return exports;}";
  const main=read("src/main.ts");
  assert.equal((main.match(/import "\.\/ui\/forms\/index.css";/g)??[]).length,1,"Sole Forms CSS entry");
  const css=readStylesheet(root,"src/ui/forms/index.css");
  const styles=[...main.matchAll(/^import "(\.\/[^"\n]+\.css)";/gm)].map(m=>m[1].replace("./","src/")).map(file=>readStylesheet(root,file).replace(/url\("([^"\n]+\.svg)"\)/g,(_m,asset)=>'url("data:image/svg+xml;base64,'+readFileSync(resolve(root,dirname(file),asset)).toString("base64")+'")')).join("\n");
  const defined=new Set([...styles.matchAll(/(--[a-z0-9-]+)\s*:/g)].map(m=>m[1]));
  for(const [,name]of css.matchAll(/var\((--[a-z0-9-]+)/g))assert.ok(defined.has(name),"Unresolved Forms token: "+name);
  const families=["form-field","field-label","field-hint","field-error","validation-summary","confirmation-input","form-actions"];
  for(const dir of families){const files=readdirSync(resolve(root,"src/ui/forms",dir));const index=read('src/ui/forms/'+dir+'/index.ts');assert.ok(index.trim().split('\n').every(line=>/^export (?:type )?\{.*\} from /.test(line)),"re-export only: "+dir);for(const file of files.filter(f=>f.endsWith('.ts')))assert.ok(!/\bfetch\(|\binvoke\(|localStorage|sessionStorage|from ["'][^"']*(?:api\/|platform\/)/.test(read('src/ui/forms/'+dir+'/'+file)),"Forms must not own I/O: "+file);}
  assert.ok(!main.includes('button.disabled = target.value.trim()'),'old duplicate name gate removed');
  assert.ok(main.includes('confirmation !== expected')&&main.includes('api.resetDatabase(confirmation)'),'host/backend guard remains');
  assert.ok(read('src/app/modal/workspacePanelController.ts').includes('this.releaseConfirmationInput?.()'),'workspace disposes local control lifecycle');
  const browser=resolveChromiumExecutable();assert.ok(browser,"Native Windows Chromium required");
  const directory=mkdtempSync(join(tmpdir(),"football-r12-forms-")),results=[];
  for(const width of [960,1440])for(const theme of ["light","dark"]){
    const name='forms-'+width+'-'+theme,htmlPath=join(directory,name+'.html');
    writeFileSync(htmlPath,'<!doctype html><html lang="zh-CN" data-theme="'+theme+'"><head><meta charset="utf-8"><style>'+styles+'\nbody{margin:0;padding:16px;background:var(--fip-canvas);color:var(--fip-text)}.forms-gallery{display:grid;grid-template-columns:repeat(3,minmax(0,1fr));gap:16px}.forms-gallery > section{min-width:0;display:flex;flex-direction:column;gap:12px;padding:12px;background:var(--fip-surface);border:1px solid var(--fip-border);border-radius:7px}.forms-gallery h2{margin:0;font:600 14px/20px var(--fip-type-body-family)}#inline-setup{margin-top:16px}#after-forms{margin-top:16px}</style></head><body><div id="forms-root"></div><script>'+runtime.replaceAll('</script','<\\/script')+';try{window.formsFixture=load('+JSON.stringify(entry)+').installFormsFixture()}catch(error){window.formsFixtureError=String(error.stack)}</script></body></html>');
    const result=await captureHtmlScreenshot({browser,htmlPath,outputPath:join(directory,name+'.png'),width,height:1500,async exercise(cdp){
      const evaluate=async expression=>{const r=await cdp.send('Runtime.evaluate',{expression,awaitPromise:true,returnByValue:true});if(r.exceptionDetails)throw Error(r.exceptionDetails.exception?.description??r.exceptionDetails.text);return r.result?.value;};
      const key=async(key,code,vk,modifiers=0)=>{await cdp.send('Input.dispatchKeyEvent',{type:'keyDown',key,code,windowsVirtualKeyCode:vk,modifiers});await cdp.send('Input.dispatchKeyEvent',{type:'keyUp',key,code,windowsVirtualKeyCode:vk,modifiers});};
      await cdp.send('Page.bringToFront');const error=await evaluate('window.formsFixtureError');assert.ok(!error,error);
      await evaluate('window.f=window.formsFixture;f.structure();f.messages();f.gates();f.workspace();f.q("#name").focus();f.q("#name").select()');
      await cdp.send('Input.insertText',{text:'原生输入 中文 <&>'});
      await evaluate('f.expect(f.q("#name").value==="原生输入 中文 <&>","native text typing");f.q("#notes").focus()');
      await cdp.send('Input.insertText',{text:'多行\n原生说明'});
      await evaluate('f.expect(f.q("#notes").value.includes("\\n"),"native multiline input");f.q("#summary a").focus()');
      await key('Enter','Enter',13);
      await evaluate('f.expect(document.activeElement===f.q("#name")&&f.q("#name").value==="原生输入 中文 <&>","summary keyboard focuses field preserving draft");f.q("#summary a[data-validation-target=choice]").focus()');
      await key('Enter','Enter',13);
      await evaluate('f.expect(document.activeElement===f.q("#choice"),"summary focuses custom Select trigger");f.q("#summary a[data-validation-target=disabled]").focus()');
      await key('Enter','Enter',13);
      await evaluate('f.expect(document.activeElement!==f.q("#disabled"),"summary cannot focus disabled target");f.input.focus();f.input.select()');
      await cdp.send('Input.insertText',{text:'football_local'});
      await evaluate('f.expect(!f.button.disabled&&new FormData(f.form).get("confirmation")==="football_local","native confirmation input enables gate and form value");f.form.reset()');
      await evaluate('(async()=>{await new Promise(r=>setTimeout(r,0));f.expect(f.input.value===""&&f.button.disabled&&!f.input.hasAttribute("aria-invalid"),"native reset clears draft, mismatch and gate");f.summary.focus();f.expect(document.activeElement===f.q("#summary"),"host can focus validation summary after failed submit");f.cleanup();f.q("#after-forms").focus();scrollTo(0,0)})()');
      return await evaluate('({checks:f.checks,devicePixelRatio,nativeWindows:navigator.userAgent.includes("Windows")})');
    }});
    assert.ok(result.nativeWindows,'Native Windows browser required');results.push({name,...result});writeFileSync(join(directory,'forms-results.json'),JSON.stringify(results,null,2));console.log(name+': '+result.checks.length+' Forms native checks PASS');
  }
  const evidence=resolve(root,'..','logs','r12-12-forms-'+Date.now());mkdirSync(evidence,{recursive:true});for(const file of readdirSync(directory).filter(f=>f.endsWith('.png')||f==='forms-results.json'))copyFileSync(join(directory,file),join(evidence,file));
  console.log('R12-12 Forms: 7 families / '+results.reduce((sum,r)=>sum+r.checks.length,0)+' native Windows checks PASS; evidence: '+evidence+'; screenshots: '+directory);
}
