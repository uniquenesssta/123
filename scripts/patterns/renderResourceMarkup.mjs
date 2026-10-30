import {runInNewContext} from "node:vm";
import {productionFixture} from "../design-refresh/productionFixture.mjs";
let cachedRoot=null,cachedContext=null;
export function renderResourceMarkup(root,page,section="directory",preview=null,selected=false) {
 if(root!==cachedRoot){const {script}=productionFixture(root);cachedContext={window:{},console,Set};runInNewContext(script,cachedContext,{timeout:10000});cachedRoot=root;}
 if(!["teams","players"].includes(page))throw Error("Unknown resource page "+page);
 const fixture=runInNewContext('load("scripts/design-refresh/fixtureData.ts")',cachedContext);
 const view=runInNewContext('load("src/pages/'+page+'.ts")',cachedContext);
 const html=page==="teams" ? view.teamsPage(fixture.state,fixture.catalogue,section==="directory"&&!selected?null:fixture.detail,null,{active_only:true},new Set(),[],[],preview,[],[],[],null,"detail",false,true,section) : view.playersPage(fixture.state,fixture.references,fixture.playerCatalogue,null,{},preview,new Set(),[],null,"detail",false,true,section);
 if(typeof html!=="string"||!html.includes('data-ui-pattern="workbench"'))throw Error("Production resource view did not render "+page);
 return html;
}
