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
  const entry="scripts/design-refresh/browserFixture.ts";bundle(entry);
  const css=readStylesheet(root,"src/styles/index.css");
  const runtime=`const modules=${JSON.stringify(modules)};const cache={};function load(id){if(cache[id])return cache[id];const r=modules[id],exports={};cache[id]=exports;new Function('exports','require',r.output)(exports,s=>load(r.imports[s]));return exports;}`;
  const main=read("src/main.ts");
  const filterSource=main.slice(main.indexOf("function filterRulesCompetitionCatalogue()"),main.indexOf("function selectRulesDirectoryLevel("));
  if(!filterSource.startsWith("function filterRulesCompetitionCatalogue")) throw Error("Production competition filter missing");
  const filter=ts.transpileModule(filterSource,{compilerOptions:{target:ts.ScriptTarget.ES2022}}).outputText;
  const playerLoadingSource=main.slice(main.indexOf("function setPlayerPageLoading("),main.indexOf("async function loadPlayerPage("));
  if(!playerLoadingSource.startsWith("function setPlayerPageLoading"))throw Error("Production local player loading helper missing");
  const playerLoading=ts.transpileModule(playerLoadingSource,{compilerOptions:{target:ts.ScriptTarget.ES2022}}).outputText;
  return {css,script:`${runtime}\n${filter}\nconst updatePaginationView=load("src/ui/navigation/index.ts").updatePaginationView;let playerPageLoading=false;const playerCursorHistory=[];let playerListPage=load(${JSON.stringify(entry)}).fixturePlayerPage;${playerLoading}\nwindow.setFixturePlayerLoading=setPlayerPageLoading;window.renderFixture=load(${JSON.stringify(entry)}).renderFixture;window.clearRulesCatalogueFilters=load(${JSON.stringify(entry)}).clearRulesCatalogueFilters;`};
}
