import assert from "node:assert/strict";
import { readFileSync, existsSync } from "node:fs";
import { resolve, dirname } from "node:path";
import { createRequire } from "node:module";

/** Local fixture module loader. Called only by installed-dependency frontend gates. */
export function loadTeamsProduction(root) {
  const ts = createRequire(import.meta.url)("typescript"), cache = new Map();
  return function load(file) {
    file = resolve(root, file);
    if (cache.has(file)) return cache.get(file);
    const text = readFileSync(file.replace(/\?raw$/, ""), "utf8");
    if (file.endsWith(".svg?raw")) return { default: text };
    const exports = {}; cache.set(file, exports);
    const code = ts.transpileModule(text, { compilerOptions: { module: ts.ModuleKind.CommonJS, target: ts.ScriptTarget.ES2022 } }).outputText;
    new Function("exports", "require", code)(exports, specifier => {
      assert.ok(specifier.startsWith("."), "Team fixture dependencies must be local");
      const target = resolve(dirname(file), specifier);
      return load(specifier.endsWith(".svg?raw") ? target : existsSync(target + ".ts") ? target + ".ts" : target + "/index.ts");
    });
    return exports;
  };
}
