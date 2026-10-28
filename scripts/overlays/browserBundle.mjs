import assert from "node:assert/strict";
import {readFileSync,existsSync} from "node:fs";
import {resolve,relative,dirname,join} from "node:path";
import {createRequire} from "node:module";
export function browserBundle(root,entry) {
  const ts=createRequire(import.meta.url)("typescript"),modules={};
  function bundle(file){
    if(modules[file])return;
    const output=ts.transpileModule(readFileSync(resolve(root,file),"utf8"),{compilerOptions:{target:ts.ScriptTarget.ES2022,module:ts.ModuleKind.CommonJS}}).outputText;
    modules[file]={output,imports:{}};
    for(const [,specifier]of output.matchAll(/require\("([^"]+)"\)/g)){
      assert.ok(specifier.startsWith("."),"no external runtime dependency: "+specifier);
      const base=resolve(root,dirname(file),specifier);
      if(specifier.endsWith(".svg?raw")){const target=relative(root,base).replaceAll("\\","/");modules[file].imports[specifier]=target;modules[target]={output:"exports.default="+JSON.stringify(readFileSync(base.replace(/\?raw$/,""),"utf8")),imports:{}};continue;}
      const target=relative(root,existsSync(base+".ts")?base+".ts":join(base,"index.ts")).replaceAll("\\","/");assert.ok(target.startsWith("src/")&&!target.includes("../"));modules[file].imports[specifier]=target;bundle(target);
    }
  }
  bundle(entry);
  return 'const modules='+JSON.stringify(modules)+';const cache={};function load(id){if(cache[id])return cache[id];const r=modules[id],exports={};cache[id]=exports;new Function("exports","require",r.output)(exports,s=>load(r.imports[s]));return exports;}';
}
