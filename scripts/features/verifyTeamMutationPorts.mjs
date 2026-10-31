import assert from "node:assert/strict";
import { readFileSync } from "node:fs";
import { createRequire } from "node:module";
import { runInNewContext } from "node:vm";

/** Execute the real composition callbacks; delayed transport must not publish into a new context. */
export async function verifyTeamMutationPorts(root) {
  const ts = createRequire(import.meta.url)("typescript");
  const text = readFileSync(root + "/src/main.ts", "utf8");
  const source = ts.createSourceFile("main.ts", text, ts.ScriptTarget.Latest, true);
  let ports;
  function visit(node) {
    if (ts.isVariableDeclaration(node) && node.name.getText(source) === "teamsFeature") ports = node.initializer.arguments[0];
    ts.forEachChild(node, visit);
  }
  visit(source); assert.ok(ports, "real Teams composition ports must exist");
  let cases = 0;
  for (const method of ["afterDelete", "afterForceDelete", "afterArchive"]) {
    const property = ports.properties.find(p => p.name?.getText(source) === method);
    assert.ok(property, "real callback " + method);
    const code = ts.transpileModule("var adapter = {" + property.getText(source) + "};", { compilerOptions: { target: ts.ScriptTarget.ES2022 } }).outputText;
    for (const mode of ["page", "team", "session", "success"]) {
      let release, started, revision=1, snapshot={}, applied=0, rendered=0, modal=0;
      const gate = new Promise(resolve => { release=resolve; }), entered = new Promise(resolve => { started=resolve; });
      const read = () => { started(); return gate; };
      const catalog = { get revision(){return revision;}, load:async()=>true, capture(){const version=revision,state=snapshot;return()=>version===revision&&state===snapshot;} };
      const context = { navigation:{page:"teams"}, teamsFeature:{catalog}, session:{refresh:async()=>{snapshot={};}}, api:{playerCatalogReferenceData:read}, fetchPlayerCatalog:read, applyPlayerCatalog:()=>{applied++;}, playerQuery:{}, selectedPlayer:null, playerReferences:"before", spreadsheetPreview:"before", matchSpreadsheetPreview:"before", removeWorkspaceObjects:()=>null, persistWorkspaceSelection:()=>{}, render:()=>{rendered++;}, showHtmlModal:()=>{modal++;}, bulkArchiveSummary:()=>"summary" };
      runInNewContext(code, context);
      const result = { deleted_ids:["a"], deleted_player_ids:[], team_id:"a", archived_ids:["a"], already_archived_ids:[] };
      const task=context.adapter[method](result,[]); await entered;
      if(mode==="page")context.navigation.page="players";
      if(mode==="team")revision++;
      if(mode==="session")snapshot={};
      release({references:{teams:[]}}); const outcome=await task;
      if(mode!=="success") { assert.equal(rendered,0,method+" late render "+mode);assert.equal(applied,0,method+" late player publication "+mode);assert.equal(context.playerReferences,"before");assert.equal(modal,0);if(method!=="afterArchive")assert.equal(outcome,false); }
      else { assert.equal(rendered,1);assert.equal(applied,method==="afterForceDelete"?1:0);if(method!=="afterForceDelete")assert.notEqual(context.playerReferences,"before");if(method!=="afterArchive")assert.equal(outcome,true); }
    }
    cases++;
  }
  return cases;
}
