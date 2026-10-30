import {productionStyleFiles} from "../styles/styleGraph.mjs";
import {readProjectSource} from "../styles/projectSource.mjs";
import {fileURLToPath as stylesheetFilePath} from "node:url";
const stylesheetRoot=stylesheetFilePath(new URL("../../",import.meta.url));
import assert from "node:assert/strict";
import { writeFileSync, existsSync, mkdtempSync, mkdirSync, copyFileSync, readdirSync } from "node:fs";
import { resolve, relative, dirname, join } from "node:path";
import { tmpdir } from "node:os";
import { createRequire } from "node:module";
import { readStylesheet } from "../tokens/tokenStyles.mjs";
import { resolveChromiumExecutable, captureHtmlScreenshot } from "../task-ui-screenshot-tools.mjs";

export async function verifyPrimitives(root) {
  const read = file => readProjectSource(stylesheetRoot,resolve(root, file), "utf8");
  const ts = createRequire(import.meta.url)("typescript"), modules = {};
  const bundle = file => {
    if (modules[file]) return;
    const output = ts.transpileModule(read(file), { compilerOptions: { target: ts.ScriptTarget.ES2022, module: ts.ModuleKind.CommonJS } }).outputText;
    modules[file] = { output, imports: {} };
    for (const [, specifier] of output.matchAll(/require\("([^"]+)"\)/g)) {
      assert.ok(specifier.startsWith("."), `Primitives must not acquire external runtime dependencies: ${specifier}`);
      const base = resolve(root, dirname(file), specifier);
      if (specifier.endsWith(".svg?raw")) {
        const target = relative(root, base).replaceAll("\\", "/"), svg = readProjectSource(stylesheetRoot,base.replace(/\?raw$/, ""), "utf8");
        assert.match(svg, /<svg\b[^>]*\bwidth="[\d.]+"[^>]*\bheight="[\d.]+"/);
        assert.ok(!/<(?:script|foreignObject)\b|\bon[a-z]+\s*=|(?:href|src)=["']https?:/i.test(svg), `Nonlocal or active SVG metadata: ${target}`);
        modules[file].imports[specifier] = target; modules[target] = { output: `exports.default=${JSON.stringify(svg)}`, imports: {} }; continue;
      }
      const target = relative(root, existsSync(`${base}.ts`) ? `${base}.ts` : join(base, "index.ts")).replaceAll("\\", "/");
      assert.ok(target.startsWith("src/") && !target.includes("../")); modules[file].imports[specifier] = target; bundle(target);
    }
  };
  const entry = "src/ui/primitives/__tests__/primitiveBrowserFixture.ts"; bundle(entry);
  const runtime = `const modules=${JSON.stringify(modules)};const cache={};function load(id){if(cache[id])return cache[id];const record=modules[id],exports={};cache[id]=exports;new Function('exports','require',record.output)(exports,s=>load(record.imports[s]));return exports;}`;
  const main = read("src/main.ts");
  assert.equal(productionStyleFiles(root).filter(file=>file==="src/ui/primitives/index.css").length,1, "Sole production primitive CSS entry");
  const styles = [...main.matchAll(/^import "(\.\/[^"\n]+\.css)";/gm)].map(match => match[1].replace("./", "src/")).map(file => readStylesheet(root, file).replace(/url\("([^"\n]+\.svg)"\)/g, (_match, asset) => {
    const bytes = readProjectSource(stylesheetRoot,resolve(root, dirname(file), asset)); assert.ok(bytes.length); return `url("data:image/svg+xml;base64,${bytes.toString("base64")}")`;
  })).join("\n");
  const defined = new Set([...styles.matchAll(/(--[a-z0-9-]+)\s*:/g)].map(match => match[1]));
  const primitiveCss = readStylesheet(root, "src/ui/primitives/index.css");
  for (const [, name] of primitiveCss.matchAll(/var\((--[a-z0-9-]+)/g)) assert.ok(defined.has(name), `Unresolved primitive token: ${name}`);
  const browser = resolveChromiumExecutable(); assert.ok(browser, "Windows Chromium required for primitive acceptance");
  const directory = mkdtempSync(join(tmpdir(), "football-r12-primitives-")), results = [];
  for (const [width, height] of [[960, 1800], [1440, 1500]]) for (const theme of ["light", "dark"]) {
    const name = `primitives-${width}-${theme}`, htmlPath = join(directory, `${name}.html`);
    writeFileSync(htmlPath, `<!doctype html><html lang="zh-CN" data-theme="${theme}"><head><meta charset="utf-8"><style>${styles}\nbody{margin:0;padding:16px;background:var(--fip-canvas);color:var(--fip-text)}.primitive-gallery{display:grid;grid-template-columns:repeat(3,minmax(0,1fr));gap:16px}.primitive-gallery section{min-width:0;display:flex;flex-direction:column;gap:12px;padding:12px;background:var(--fip-surface);border:1px solid var(--fip-border);border-radius:7px}.primitive-gallery h2{margin:0;font:600 14px/20px var(--fip-type-body-family)}.primitive-row{display:flex;align-items:center;gap:8px;flex-wrap:wrap}.primitive-column{display:flex;flex-direction:column;gap:12px}.primitive-accent{background:var(--fip-accent);padding:8px}.skeleton-sample{width:100%;margin-bottom:8px}#after-controls{margin-top:16px}</style></head><body><div id="primitive-root"></div><script>${runtime.replaceAll("</script", "<\\/script")}try{window.primitiveFixture=load(${JSON.stringify(entry)}).installPrimitiveFixture()}catch(error){window.primitiveFixtureError=String(error.stack)}</script></body></html>`);
    const result = await captureHtmlScreenshot({ browser, htmlPath, outputPath: join(directory, `${name}.png`), width, height, async exercise(cdp) {
      const evaluate = async expression => { const result = await cdp.send("Runtime.evaluate", { expression, awaitPromise: true, returnByValue: true }); if (result.exceptionDetails) throw new Error(result.exceptionDetails.exception?.description ?? result.exceptionDetails.text); return result.result?.value; };
      const key = async (key, code, vk, modifiers = 0) => { await cdp.send("Input.dispatchKeyEvent", { type: "keyDown", key, code, windowsVirtualKeyCode: vk, modifiers }); await cdp.send("Input.dispatchKeyEvent", { type: "keyUp", key, code, windowsVirtualKeyCode: vk, modifiers }); };
      await cdp.send("Page.bringToFront");
      const error = await evaluate("window.primitiveFixtureError"); assert.ok(!error, error);
      await evaluate("window.f=window.primitiveFixture;f.geometry();f.localBehavior();f.q('#text').focus();f.q('#text').select()");
      await cdp.send("Input.insertText", { text: "实际输入 <&> 中文" });
      await evaluate("f.expect(f.q('#text').value==='实际输入 <&> 中文'&&f.inputs()>0,'native typing/input event');f.q('[data-clear-input]').click();f.expect(f.q('#text').value===''&&document.activeElement===f.q('#text'),'clear emits input and restores focus');f.q('#number').focus()");
      await key("ArrowUp", "ArrowUp", 38);
      await evaluate("f.expect(f.q('#number').value==='0.81','native decimal ArrowUp');f.q('[data-number-step=down]').click();f.expect(f.q('#number').value==='0.8','decimal stepDown uses native decimal arithmetic');f.q('#number').value='1';f.numberHandle.sync();f.expect(f.q('[data-number-step=up]').disabled,'max disables increase');f.q('[data-number-step=down]').click();f.expect(f.q('#number').value==='0.99'&&f.changes()>0,'step emits native change');f.q('#number').value='0';f.numberHandle.sync();f.expect(f.q('[data-number-step=down]').disabled,'min disables decrease');f.q('#check').focus()");
      await key(" ", "Space", 32);
      await evaluate("f.expect(f.q('#check').checked,'native checkbox Space');f.q('#mixed').focus()");
      await key(" ", "Space", 32);
      await evaluate("f.expect(!f.q('#mixed').indeterminate&&f.q('#mixed').checked,'mixed Space becomes checked');f.q('#radio-a').focus()");
      await key("ArrowRight", "ArrowRight", 39);
      await evaluate("f.expect(f.q('#radio-b').checked&&!f.q('#radio-a').checked,'native radio group ArrowRight');f.q('#switch').focus()");
      await key(" ", "Space", 32);
      await evaluate("f.expect(f.q('#switch').checked,'native Switch Space');f.q('#switch-disabled').closest('label').click();f.expect(f.q('#switch-disabled').checked,'disabled Switch parent click cannot change value');f.q('#choice').focus()");
      await key("Enter", "Enter", 13); await key("ArrowDown", "ArrowDown", 40);
      await evaluate("f.expect(f.q('#choice').getAttribute('aria-activedescendant').endsWith('option-2'),'select skips disabled option');f.expect(f.q('.ui-select-native').value==='a','keyboard active is not committed value')");
      await key("Escape", "Escape", 27);
      await evaluate("f.expect(f.q('#choice').getAttribute('aria-expanded')==='false'&&f.q('.ui-select-native').value==='a'&&document.activeElement===f.q('#choice'),'Escape preserves value and focus')");
      await key(" ", "Space", 32); await key("End", "End", 35); await key("Enter", "Enter", 13);
      await evaluate("f.expect(f.q('.ui-select-native').value==='d'&&new FormData(f.form).get('choice')==='d','select commits native form value');f.q('#choice').focus()");
      await key("g", "KeyG", 71); await key("Enter", "Enter", 13);
      await evaluate("f.expect(f.q('.ui-select-native').value==='g','select typeahead commit');f.q('#choice').click();f.q('[data-value=b]').click();f.expect(f.q('.ui-select-native').value==='g','disabled option click cannot commit')");
      await key("Tab", "Tab", 9);
      await evaluate("f.expect(f.q('#choice').getAttribute('aria-expanded')==='false'&&document.activeElement!==f.q('#choice'),'Tab exits selection without focus trap');f.q('#choice').click();document.body.dispatchEvent(new PointerEvent('pointerdown',{bubbles:true}));f.expect(f.q('#choice').getAttribute('aria-expanded')==='false','outside closes selection');f.expect(!f.q('#choice-required').parentElement.querySelector('select').checkValidity()&&document.activeElement===f.q('#choice-required'),'required native validity focuses trigger');f.q('#text').value='theme draft';f.q('#notes').focus();window.previousTheme=document.documentElement.dataset.theme;document.documentElement.dataset.theme=previousTheme==='light'?'dark':'light';f.expect(f.q('#text').value==='theme draft'&&document.activeElement===f.q('#notes'),'theme switch retains live draft and focus');document.documentElement.dataset.theme=previousTheme");
      await evaluate("(async()=>{f.form.reset();await new Promise(resolve=>setTimeout(resolve,0));f.expect(f.q('.ui-select-native').value==='a'&&f.q('#choice .ui-select-value').textContent==='Alpha','native form reset synchronizes custom selection');f.expect(!f.q('#choice-required').hasAttribute('aria-invalid'),'form reset clears native reported validation');f.expect(!f.q('[data-clear-input]').hidden&&!f.q('[data-number-step=down]').disabled&&f.q('#mixed').indeterminate,'form reset synchronizes clear, stepper and mixed projection');f.expect(f.q('#choice').getAttribute('aria-expanded')==='false','form reset closes popup');const target=f.q('#choice').parentElement;window.popupOriginalStyle=target.style.cssText;target.style.cssText='position:fixed;left:24px;width:280px;top:'+(innerHeight-54)+'px';f.q('#choice').click();const popup=f.q('#choice-options').getBoundingClientRect();f.expect(popup.top>=0&&popup.bottom<=innerHeight&&popup.bottom<=f.q('#choice').getBoundingClientRect().top,'select flips above near viewport bottom');f.expect(Math.abs(f.q('#choice').getBoundingClientRect().width-280)<0.1,'select host honors explicit width');f.expect(Math.abs(popup.width-f.q('#choice').getBoundingClientRect().width)<0.1&&popup.left>=0&&popup.right<=innerWidth,'select popup matches trigger width within viewport: '+JSON.stringify({popup:popup.toJSON(),trigger:f.q('#choice').getBoundingClientRect().toJSON(),expanded:f.q('#choice').getAttribute('aria-expanded'),window:innerWidth}));})()");
      const popupImage = await cdp.send("Page.captureScreenshot", { format: "png", fromSurface: true, captureBeyondViewport: false });
      writeFileSync(join(directory, `${name}-select-open.png`), Buffer.from(popupImage.data, "base64"));
      await evaluate("f.selectHandle.close();f.q('#choice').parentElement.style.cssText=popupOriginalStyle");
      for (const [value, expected] of [["no-preference", "fip-spinner"], ["reduce", "none"]]) {
        await cdp.send("Emulation.setEmulatedMedia", { features: [{ name: "prefers-reduced-motion", value }] });
        await evaluate(`f.expect(getComputedStyle(f.q('.ui-spinner')).animationName===${JSON.stringify(expected)},'spinner motion ${value}')`);
      }
      await cdp.send("Emulation.setEmulatedMedia", { features: [{ name: "forced-colors", value: "active" }] });
      await evaluate("f.expect(getComputedStyle(f.q('#load-button')).borderTopColor!=='rgba(0, 0, 0, 0)','forced colors has visible control border')");
      await cdp.send("Emulation.setEmulatedMedia", { features: [] });
      await evaluate("f.cleanup();f.expect(!document.querySelector('script[src]')&&!document.querySelector('img[src^=http]'),'no remote runtime asset dependency');f.q('#after-controls').focus();document.activeElement.blur();scrollTo(0,0)");
      return await evaluate("({checks:f.checks,devicePixelRatio,nativeWindows:navigator.userAgent.includes('Windows')})");
    } });
    assert.ok(result?.nativeWindows, "Native Windows browser required"); results.push({ name, ...result });
    writeFileSync(join(directory, "primitive-results.json"), JSON.stringify(results, null, 2));
    console.log(`${name}: ${result.checks.length} primitive geometry/keyboard/input/form/lifecycle checks PASS`);
  }
  const evidence = resolve(root, "..", "logs", `r12-11-primitives-${Date.now()}`); mkdirSync(evidence, { recursive: true });
  for (const file of readdirSync(directory).filter(file => file.endsWith(".png") || file === "primitive-results.json")) copyFileSync(join(directory, file), join(evidence, file));
  console.log(`R12-11 Primitives: 14 families / ${results.reduce((sum, result) => sum + result.checks.length, 0)} native Windows checks PASS; screenshots: ${directory}; existing CI evidence: ${evidence}`);
}
