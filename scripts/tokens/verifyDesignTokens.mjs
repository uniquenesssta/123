import {productionStyleFiles,inspectStyleGraph} from "../styles/styleGraph.mjs";
import {readProjectSource} from "../styles/projectSource.mjs";
import {fileURLToPath as stylesheetFilePath} from "node:url";
const stylesheetRoot=stylesheetFilePath(new URL("../../",import.meta.url));
import assert from "node:assert/strict";
import { mkdtempSync, writeFileSync } from "node:fs";
import { resolve, join } from "node:path";
import { tmpdir } from "node:os";
import { readStylesheet } from "./tokenStyles.mjs";
import { resolveChromiumExecutable, captureHtmlScreenshot } from "../task-ui-screenshot-tools.mjs";

/** Compare actual CSS delivery with the read-only Figma snapshot, including native computed values. */
export async function verifyDesignTokens(root) {
  const record = JSON.parse(readProjectSource(stylesheetRoot,resolve(root, "docs/modular-rewrite/R12-frontend-foundation-ui-system/approved-design-tokens-20261028.json"), "utf8"));
  assert.equal(record.fileKey, "PN0Whgu6HLWIHx4Mv6aHfu");
  assert.equal(record.designSnapshot, "approved-live-2026-10-28");
  assert.equal(record.variables.length, 237); assert.equal(record.collections.length, 6); assert.equal(record.texts.length, 25); assert.equal(record.effects.length, 6);
  assert.equal(new Set(record.variables.map(v => v.id)).size, 237); assert.equal(new Set(record.variables.map(v => v.css)).size, 237);
  const byId = new Map(record.variables.map(v => [v.id, v])), collections = new Map(record.collections.map(c => [c.id, c]));
  for (const v of record.variables) {
    assert.match(v.css, /^--(?:fip-|shell-)/); assert.ok(v.file.startsWith("src/ui/tokens/"));
    for (const mode of collections.get(v.collectionId).modes) {
      assert.ok(Object.hasOwn(v.values, mode.modeId), `${v.id} missing mode ${mode.name}`);
      const seen = new Set(); let value = v.values[mode.modeId];
      while (value?.type === "VARIABLE_ALIAS") { assert.ok(!seen.has(value.id), "Cyclic Figma alias"); seen.add(value.id); const target = byId.get(value.id); assert.ok(target, "Missing Figma alias target"); const collection = collections.get(target.collectionId); value = target.values[collection.defaultModeId]; }
    }
  }
  const entry = "src/ui/tokens/index.css", css = readStylesheet(root, entry), styleGraph = inspectStyleGraph(root);
  const declarations = [...css.matchAll(/(--[a-z0-9-]+)\s*:\s*([^;{}]+);/g)];
  const names = new Set(declarations.map(m => m[1]));
  for (const v of record.variables) assert.ok(names.has(v.css), `Missing production token ${v.css}`);
  for (const [, , value] of declarations) for (const [, reference] of value.matchAll(/var\((--[a-z0-9-]+)\)/g)) assert.ok(names.has(reference), `Unresolved CSS alias ${reference}`);
  for (const category of ["color", "typography", "spacing", "size", "radius", "border", "shadow", "motion", "z-index", "breakpoint"]) assert.ok(styleGraph.sheets.get(entry).nodes.some(node=>node.type==="atrule"&&node.name==="import"&&node.params.includes(`./${category}/`)), `Missing token category ${category}`);
  assert.ok(!/--ui-(?:canvas|surface|text|accent)\s*:/.test(css), "R12 tokens must not override legacy Feature palette");
  const main = readProjectSource(stylesheetRoot,resolve(root, "src/main.ts"), "utf8"); assert.equal(productionStyleFiles(root).filter(file=>file==="src/ui/tokens/index.css").length,1);
  const browser = resolveChromiumExecutable(); assert.ok(browser, "Windows Chromium is required for token mode verification");
  const directory = mkdtempSync(join(tmpdir(), "football-r12-tokens-"));
  const htmlPath = join(directory, "tokens.html");
  writeFileSync(htmlPath, `<!doctype html><html data-theme="light"><head><meta charset="utf-8"><style>${css}</style></head><body><div id="probe"></div><main id="tokens-preview"></main></body></html>`);
  let assertions = 0;
  for (const theme of ["Light", "Dark"]) {
    const result = await captureHtmlScreenshot({ browser, htmlPath, outputPath: join(directory, `tokens-${theme.toLowerCase()}.png`), width: 1440, height: 1200, async exercise(cdp) {
      const expression = `(${browserChecks.toString()})(${JSON.stringify(record)}, ${JSON.stringify(theme)})`;
      const response = await cdp.send("Runtime.evaluate", { expression, returnByValue: true });
      if (response.exceptionDetails) throw new Error(response.exceptionDetails.exception?.description ?? response.exceptionDetails.text);
      return response.result.value;
    } });
    assert.equal(result.failures.length, 0, result.failures.join("\n")); assertions += result.assertions;
  }
  console.log(`Approved design refresh: 237 variables / 25 text styles / 6 effects; ${assertions} native computed-value checks PASS; evidence retained: ${directory}`);
}

function browserChecks(record, theme) {
  const failures = []; let assertions = 0;
  const check = (ok, message) => { assertions++; if (!ok) failures.push(message); };
  document.documentElement.dataset.theme = theme.toLowerCase();
  const byId = new Map(record.variables.map(v => [v.id, v])), collections = new Map(record.collections.map(c => [c.id, c]));
  const resolveValue = (v, modeId, seen = new Set()) => {
    if (seen.has(v.id)) throw Error("Alias cycle"); seen.add(v.id);
    const value = v.values[modeId];
    if (value?.type !== "VARIABLE_ALIAS") return value;
    const target = byId.get(value.id), collection = collections.get(target.collectionId);
    return resolveValue(target, collection.modes.find(m => m.name === theme)?.modeId ?? collection.defaultModeId, seen);
  };
  const probe = document.querySelector("#probe"), preview = document.querySelector("#tokens-preview");
  const rgba = v => `rgba(${Math.round(v.r * 255)}, ${Math.round(v.g * 255)}, ${Math.round(v.b * 255)}, ${v.a ?? 1})`;
  const normalizeColor = value => { const el = document.createElement("span"); el.style.color = value; document.body.append(el); const color = getComputedStyle(el).color; el.remove(); return color; };
  for (const v of record.variables) {
    const collection = collections.get(v.collectionId);
    const modes = collection.name === "FIP Color" ? collection.modes.filter(m => m.name === theme) : collection.modes;
    for (const mode of modes) {
      delete probe.dataset.fipControlScale; delete probe.dataset.fipIconScale; delete probe.dataset.fipIconTone;
      if (collection.name === "FIP Control Scale") probe.dataset.fipControlScale = mode.name;
      if (collection.name === "FIP Icon Scale") probe.dataset.fipIconScale = mode.name;
      if (collection.name === "FIP Icon Tone") probe.dataset.fipIconTone = mode.name;
      const expected = resolveValue(v, mode.modeId), raw = getComputedStyle(probe).getPropertyValue(v.css).trim();
      check(raw.length > 0, `${theme}/${mode.name}/${v.name} is unset`);
      if (v.type === "COLOR") {
        probe.style.color = `var(${v.css})`; const actual = getComputedStyle(probe).color;
        check(actual === normalizeColor(rgba(expected)), `${theme}/${mode.name}/${v.name}: ${actual} differs from Figma`);
      } else {
        check(Math.abs(parseFloat(raw) - expected) < 0.0001, `${theme}/${mode.name}/${v.name}: ${raw} differs from Figma ${expected}`);
        check(raw.endsWith("px") === (v.unit === "px"), `Incorrect dimension for ${v.name}`);
      }
    }
  }
  delete probe.dataset.fipControlScale; delete probe.dataset.fipIconScale; delete probe.dataset.fipIconTone;
  for (const style of record.texts) {
    const prefix = style.css;
    Object.assign(probe.style, { fontFamily: `var(${prefix}-family)`, fontSize: `var(${prefix}-size)`, fontWeight: `var(${prefix}-weight)`, lineHeight: `var(${prefix}-line-height)`, letterSpacing: `var(${prefix}-letter-spacing)` });
    const actual = getComputedStyle(probe);
    check(parseFloat(actual.fontSize) === style.fontSize, `${style.name} font size`);
    check(parseFloat(actual.lineHeight) === style.lineHeight.value, `${style.name} line height`);
    check(Number(actual.fontWeight) === style.fontName.variationSettings.wght, `${style.name} weight`);
    const expectedSpacing = style.letterSpacing.unit === "PERCENT" ? style.fontSize * style.letterSpacing.value / 100 : style.letterSpacing.value;
    check(Math.abs((parseFloat(actual.letterSpacing) || 0) - expectedSpacing) < 0.0001, `${style.name} tracking`);
    check(actual.fontFamily.includes(style.fontName.family), `${style.name} family`);
  }
  for (const style of record.effects.filter(s => s.name.endsWith(theme))) {
    const shadow = style.effects.filter(e => e.visible).map(e => `${e.offset.x}px ${e.offset.y}px ${e.radius}px ${e.spread}px ${rgba(e.color)}`).join(", ");
    probe.style.boxShadow = `var(${style.css})`; const actual = getComputedStyle(probe).boxShadow;
    probe.style.boxShadow = shadow; check(actual === getComputedStyle(probe).boxShadow, `${style.name} layers/color/geometry`);
  }
  preview.style.cssText = "font-family:var(--fip-type-body-family);background:var(--fip-canvas);color:var(--fip-text);padding:var(--fip-space-xl);display:grid;grid-template-columns:repeat(4,1fr);gap:var(--fip-space-lg)";
  preview.innerHTML = `<h1 style="grid-column:1/-1;font-size:var(--fip-type-page-title-size);line-height:var(--fip-type-page-title-line-height)">FIP Design Tokens · ${theme}</h1>`;
  for (const v of record.variables.filter(v => v.collectionId === "VariableCollectionId:17:3")) {
    const card = document.createElement("section"); card.style.cssText = "padding:var(--fip-space-md);border:var(--fip-border-width) solid var(--fip-border);border-radius:var(--fip-radius-md);background:var(--fip-surface)";
    const swatch = document.createElement("div"); swatch.style.cssText = `height:32px;background:var(${v.css});border-radius:var(--fip-radius-sm)`;
    const label = document.createElement("div"); label.textContent = v.name; label.style.cssText = "font-size:var(--fip-type-caption-size);line-height:var(--fip-type-caption-line-height)";
    card.append(swatch, label); preview.append(card);
  }
  return { assertions, failures };
}
