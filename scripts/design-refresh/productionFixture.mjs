import {readProjectSource} from "../styles/projectSource.mjs";
import {fileURLToPath as stylesheetFilePath} from "node:url";
const stylesheetRoot=stylesheetFilePath(new URL("../../",import.meta.url));
import { existsSync } from "node:fs";
import { resolve, dirname, relative, join } from "node:path";
import { createRequire } from "node:module";
import { readStylesheet } from "../tokens/tokenStyles.mjs";

/** Bundle actual view owners and the exact production CSS import graph for isolated browser QA. */
export function productionFixture(root) {
  const ts=createRequire(import.meta.url)("typescript"), modules={};
  const read=file=>readProjectSource(stylesheetRoot,resolve(root,file),"utf8");
  const bundle=file=>{
    if(modules[file]) return;
    const output=ts.transpileModule(read(file),{compilerOptions:{target:ts.ScriptTarget.ES2022,module:ts.ModuleKind.CommonJS}}).outputText;
    modules[file]={output,imports:{}};
    for(const [,specifier] of output.matchAll(/require\("([^"]+)"\)/g)) {
      if(!specifier.startsWith(".")) throw Error(`Unexpected fixture dependency: ${specifier}`);
      const base=resolve(root,dirname(file),specifier), target=relative(root,specifier.endsWith(".css")||specifier.endsWith(".svg?raw")?base:existsSync(`${base}.ts`)?`${base}.ts`:join(base,"index.ts")).replaceAll("\\","/");
      modules[file].imports[specifier]=target;
      if(specifier.endsWith(".css")) modules[target]={output:"",imports:{}};
      else if(specifier.endsWith(".svg?raw")) modules[target]={output:`exports.default=${JSON.stringify(readProjectSource(stylesheetRoot,base.replace(/\?raw$/,""),"utf8"))}`,imports:{}};
      else bundle(target);
    }
  };
  const entry="scripts/design-refresh/browserFixture.ts";bundle(entry);bundle("src/features/players/playerDirectoryDom.ts");
  const css=readStylesheet(root,"src/styles/index.css");
  const runtime=`const modules=${JSON.stringify(modules)};const cache={};function load(id){if(cache[id])return cache[id];const r=modules[id],exports={};cache[id]=exports;new Function('exports','require',r.output)(exports,s=>load(r.imports[s]));return exports;}`;
  const main=read("src/main.ts");
  const filter = `const filterRulesCompetitionCatalogue=load("scripts/design-refresh/browserFixture.ts").filterRulesCompetitionCatalogue;`;
  return {css,script:`${runtime}\n${filter}\nwindow.setFixturePlayerLoading=active=>load("src/features/players/playerDirectoryDom.ts").setPlayerPageLoading({page:"players",list:load(${JSON.stringify(entry)}).fixturePlayerPage,pageNumber:1},active);window.renderFixture=load(${JSON.stringify(entry)}).renderFixture;window.clearRulesCatalogueFilters=load(${JSON.stringify(entry)}).clearRulesCatalogueFilters;`};
}
