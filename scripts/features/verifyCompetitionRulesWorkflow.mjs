import assert from "node:assert/strict";
import { readFileSync, existsSync } from "node:fs";
import { resolve, dirname } from "node:path";
import { createRequire } from "node:module";

/** Executes actual Feature owners with boundary-only transports, not a second implementation. */
export async function verifyCompetitionRulesWorkflow(root, group) {
  const ts = createRequire(import.meta.url)("typescript"), cache = new Map();
  function load(file) {
    file = resolve(root, file);
    if (cache.has(file)) return cache.get(file);
    const text = readFileSync(file.replace(/\?raw$/, ""), "utf8");
    if (file.endsWith(".svg?raw")) return { default: text };
    const exports = {}; cache.set(file, exports);
    const code = ts.transpileModule(text, { compilerOptions: { module: ts.ModuleKind.CommonJS, target: ts.ScriptTarget.ES2022 } }).outputText;
    new Function("exports", "require", code)(exports, specifier => {
      assert.ok(specifier.startsWith("."));
      const target = resolve(dirname(file), specifier);
      return load(specifier.endsWith(".svg?raw") ? target : existsSync(target + ".ts") ? target + ".ts" : target + "/index.ts");
    });
    return exports;
  }
  const { CompetitionFeature, reconcileHierarchy, syncHierarchyControls, competitionCatalogue } = load("src/features/competitions/index.ts");
  const { RulesWorkflow } = load("src/features/rules/rulesWorkflow.ts");
  const { RulePackageImport, parseRulePackage } = load("src/features/rules/rulePackageImport.ts");
  const state = () => ({ data: { database_configured: true,
    competitions: ["a", "b"].map(id => ({ id, name: id, is_active: true, competition_kind: "league" })),
    seasons: ["a", "b"].map(id => ({ id: id + "s", competition_id: id })),
    stages: ["a", "b"].map(id => ({ id: id + "t", season_id: id + "s", competition_id: id })),
    rule_packages: [{ id: "rp", status: "active", model_id: "p4" }], competition_bindings: [],
  } });
  const hierarchy = (id = "a") => ({ competition_id: id, season_id: id + "s", stage_id: id + "t" });
  const binding = () => ({ ...hierarchy(), rule_package_id: "rp", binding_name: "draft", priority: 123, competition_kind: null, valid_from: null, valid_to: null });
  const command = () => ({ ...hierarchy(), kickoff_time: "2030-01-01T12:00:00Z", competition_kind: "league", explicit_rule_package_id: null, model_family: "p4" });
  const packageDraft = name => ({ format_version: "football.rule-package.v1", package_key: name, version: "1.0.0", display_name: name,
    competition_profile: { profile_id: "profile", name: "Profile", competition_kind: "league", normal_time_minutes: 90, extra_time_possible: false, penalties_possible: false, two_legged: false, neutral_venue: false, metadata: {} },
    routing: { model_id: "p4", model_version: "1", parameter_version: "1", priority: 100, activate_as_type_default: false, supported_snapshot_types: ["T-1h"] },
    parameters: {}, feature_requirements: {}, output_contract: {}, metadata: {}, source_document: null,
  });
  const file = name => ({ text: async () => JSON.stringify(packageDraft(name)) });
  const deferred = () => { let resolve, reject; const promise = new Promise((a,b) => { resolve=a; reject=b; }); return { promise, resolve, reject }; };
  function fixture(overrides = {}) {
    let current = state(); const calls = [];
    const ports = { snapshot: () => current,
      registerRulePackage: async draft => { calls.push(["register", draft]); },
      createBinding: async draft => { calls.push(["binding", draft]); },
      previewRoute: async draft => { calls.push(["preview", draft]); return { binding_id: "backend-selected" }; },
      refresh: async () => { calls.push(["refresh"]); return true; }, ...overrides };
    return { flow: new RulesWorkflow(ports), calls, ports, replace: () => { current = state(); } };
  }
  let cases = 0;
  const test = async (name, operation) => { try { await operation(); cases++; } catch (cause) { throw new Error(`R13-03 ${group}: ${name}`, { cause }); } };
  if (group === "hierarchy") {
    await test("directory cannot project a season/stage candidate as a whole-competition binding", () => {
      const s=state();s.data.competition_bindings=[{is_active:true,model_id:"p4",competition_id:"a",season_id:"as",stage_id:"at",rule_package_name:"Stage-only-secret",priority:999},{is_active:true,model_id:"p4",competition_id:"a",season_id:null,stage_id:null,rule_package_name:"Competition-scope",priority:10}];
      const html=competitionCatalogue(s);assert.ok(html.includes("Competition-scope"));assert.ok(!html.includes("Stage-only-secret"));
    });
    await test("clearing season also removes its stage-specific kind", () => {
      const s=state();s.data.stages[0].stage_kind="knockout_single_leg";
      const controls={"competition-id":{id:"competition-id",value:"a"},"season-id":{id:"season-id",value:"as",options:[]},"stage-id":{id:"stage-id",value:"at",options:[]},"competition-kind":{value:"league"}},root={querySelector:selector=>controls[selector.slice(1)]};
      syncHierarchyControls(root,s,"");assert.equal(controls["competition-kind"].value,"knockout_single_leg");controls["season-id"].value="";syncHierarchyControls(root,s,"","season-id");assert.equal(controls["competition-kind"].value,"league");
    });
    await test("changed competition clears both old descendants", () => assert.deepEqual(reconcileHierarchy(state(), { ...hierarchy(), competition_id: "b" }), { competition_id: "b", season_id: null, stage_id: null }));
    await test("a stage cannot survive a missing or different season", () => {
      assert.equal(reconcileHierarchy(state(), { ...hierarchy(), season_id: "bs" }).stage_id, null);
      assert.equal(reconcileHierarchy(state(), { ...hierarchy(), season_id: null }).stage_id, null);
    });
    await test("inactive/removed parent invalidates the whole chain", () => { const s=state(); s.data.competitions[0].is_active=false; assert.deepEqual(reconcileHierarchy(s,hierarchy()),{competition_id:null,season_id:null,stage_id:null}); });
    await test("actual DOM synchronizer resets and disables descendants, including restored stale values", () => {
      const option = (value, dataset={}) => ({value,dataset,hidden:false,disabled:false});
      const controls = { "binding-competition-id":{id:"binding-competition-id",value:"b"}, "binding-season-id":{id:"binding-season-id",value:"as",options:[option(""),option("as",{competitionId:"a"}),option("bs",{competitionId:"b"})]}, "binding-stage-id":{id:"binding-stage-id",value:"at",options:[option(""),option("at",{competitionId:"a",seasonId:"as"}),option("bt",{competitionId:"b",seasonId:"bs"})]} };
      const root={querySelector:selector=>controls[selector.slice(1)]}; syncHierarchyControls(root,state(),"binding-","binding-competition-id");
      assert.equal(controls["binding-season-id"].value,""); assert.equal(controls["binding-stage-id"].value,""); assert.equal(controls["binding-stage-id"].disabled,true); assert.equal(controls["binding-season-id"].options[1].disabled,true);
      controls["binding-season-id"].value="bs"; controls["binding-stage-id"].value="at"; syncHierarchyControls(root,state(),"binding-"); assert.equal(controls["binding-stage-id"].value,""); assert.equal(controls["binding-stage-id"].options[1].hidden,true);
    });
    function competitionFixture(overrides={}) {
      const calls=[], current=state(); let confirm;
      const ports={snapshot:()=>current,active:()=>true,busy:op=>op(),refresh:async()=>{calls.push("refresh");return true;},render:()=>calls.push("render"),notify:()=>calls.push("notify"),invalidateRoutes:()=>calls.push("invalidate"),showPath:()=>{},confirm:(...args)=>{confirm=args;},...Object.fromEntries(["createCompetition","createSeason","createStage","createRound","deleteCompetition"].map(key=>[key,async draft=>calls.push([key,draft])])),...overrides};
      return {feature:new CompetitionFeature(ports),calls,ports,confirm:()=>confirm};
    }
    await test("all four original creates carry original DTOs to the real port", async () => {
      const values={"competition-name":" Example ","competition-country":"KR","competition-timezone":"Asia/Seoul","new-competition-kind":"league","season-competition-id":"a","season-name":"2030","season-starts-on":"","season-ends-on":"","season-status":"active","stage-season-id":"as","stage-name":"League","stage-kind":"league","stage-sequence":"2","round-stage-id":"at","round-name":"R1","round-sequence":"3"};
      const root={querySelector:selector=>({value:values[selector.slice(1)]})}, f=competitionFixture();
      for(const method of ["createCompetition","createSeason","createStage","createRound"]) await f.feature[method](root);
      const writes=f.calls.filter(Array.isArray); assert.deepEqual(writes.map(x=>x[0]),["createCompetition","createSeason","createStage","createRound"]); assert.equal(writes[0][1].name,"Example"); assert.equal(writes[1][1].competition_id,"a"); assert.equal(writes[2][1].season_id,"as"); assert.equal(writes[3][1].stage_id,"at");
    });
    await test("cancelled deletion never writes; referenced deletion failure never refreshes or announces success", async () => {
      const f=competitionFixture({deleteCompetition:async()=>{throw Error("referenced");}}); f.feature.requestDelete("a"); assert.equal(f.calls.length,0); await assert.rejects(f.confirm()[4](),/referenced/); assert.deepEqual(f.calls,["invalidate"]);
    });
    await test("delete confirmation cannot cross Session identity", async () => {
      let current=state(); const f=competitionFixture({snapshot:()=>current}); f.feature.requestDelete("a"); current=state(); await assert.rejects(f.confirm()[4](),/上下文已变化/); assert.equal(f.calls.length,0);
    });
    await test("late creation after connection/context replacement cannot publish old summary", async () => {
      let current=state();const gate=deferred(),f=competitionFixture({snapshot:()=>current,deleteCompetition:()=>gate.promise});f.feature.requestDelete("a");const pending=f.confirm()[4]();current=state();gate.resolve();await assert.rejects(pending,/上下文已变化/);assert.deepEqual(f.calls,["invalidate"]);
    });
  }
  if (group === "bindings") {
    await test("serde-default omissions pass through without invented client values", () => {
      const candidate=packageDraft("optional");delete candidate.format_version;delete candidate.source_document;delete candidate.metadata;delete candidate.competition_profile.normal_time_minutes;delete candidate.competition_profile.metadata;delete candidate.routing.priority;delete candidate.routing.supported_snapshot_types;
      assert.deepEqual(parseRulePackage(JSON.stringify(candidate)),candidate);
    });
    await test("mismatched ancestors are rejected before any binding write", async () => { const f=fixture();await assert.rejects(f.flow.saveBinding({...binding(),stage_id:"bt"}),/不匹配/); assert.equal(f.calls.length,0); });
    await test("inactive or non-P4 package cannot be submitted through the P4 form", async () => { const f=fixture();await assert.rejects(f.flow.saveBinding({...binding(),rule_package_id:"missing"}),/有效赛事和 P4/);assert.equal(f.calls.length,0); });
    await test("binding DTO is detached before await and backend rejection keeps the caller's draft", async () => { const gate=deferred(),sent=[],f=fixture({createBinding:input=>{sent.push(input);return gate.promise;}}),draft=binding();const task=f.flow.saveBinding(draft);draft.binding_name="edit while saving";gate.reject(Error("backend invalid"));await assert.rejects(task,/backend invalid/);assert.equal(sent[0].binding_name,"draft");assert.equal(draft.binding_name,"edit while saving");assert.equal(f.calls.length,0); });
    await test("saved-but-refresh-failed is explicit and never auto retries", async () => { const f=fixture({refresh:async()=>false});await assert.rejects(f.flow.saveBinding(binding()),/已保存.*勿重复提交/);assert.equal(f.calls.length,1); });
    await test("nested malformed rule data fails shape precheck before rendering", () => { const candidate=packageDraft("bad");candidate.routing={};assert.throws(()=>parseRulePackage(JSON.stringify(candidate)),/模型路由/);assert.throws(()=>parseRulePackage("[]"),/名称/); });
    await test("late file A cannot replace B", async () => { const gate=deferred(),imports=new RulePackageImport();const a=imports.read({text:()=>gate.promise});await imports.read(file("b"));gate.resolve(JSON.stringify(packageDraft("a")));assert.equal(await a,false);assert.equal(imports.pending.package_key,"b"); });
    await test("clear cancels delayed file completion", async () => { const gate=deferred(),imports=new RulePackageImport(),task=imports.read({text:()=>gate.promise});imports.clear();gate.resolve(JSON.stringify(packageDraft("a")));assert.equal(await task,false);assert.equal(imports.pending,null); });
    await test("obsolete file error is ignored and malformed current file retains the previous draft", async () => { const gate=deferred(),imports=new RulePackageImport(),task=imports.read({text:()=>gate.promise});await imports.read(file("b"));gate.reject(Error("old read failed"));assert.equal(await task,false);await assert.rejects(imports.read({text:async()=>"{}"}),/规则包/);assert.equal(imports.pending.package_key,"b"); });
    await test("registration waits for current file read; getters and submitted DTOs are detached", async () => { const imports=new RulePackageImport();await imports.read(file("a"));imports.pending.package_key="tampered";assert.equal(imports.registration().draft.package_key,"a");const gate=deferred(),task=imports.read({text:()=>gate.promise});assert.throws(()=>imports.registration(),/等待/);gate.resolve(JSON.stringify(packageDraft("b")));await task; });
    await test("backend package validation remains authoritative and failed package is recoverable", async () => { const f=fixture({registerRulePackage:async()=>{throw Error("backend schema rejected");}});await f.flow.packages.read(file("a"));await assert.rejects(f.flow.register(),/backend schema/);assert.equal(f.flow.packages.pending.package_key,"a");assert.equal(f.calls.length,0); });
    await test("register A cannot clear a newly selected B; duplicate write is refused", async () => { const gate=deferred(),f=fixture({registerRulePackage:()=>gate.promise});await f.flow.packages.read(file("a"));const task=f.flow.register();await assert.rejects(f.flow.register(),/正在进行/);await f.flow.packages.read(file("b"));gate.resolve();await task;assert.equal(f.flow.packages.pending.package_key,"b"); });
    await test("successful registration clears exactly the committed candidate", async () => { const f=fixture();await f.flow.packages.read(file("a"));await f.flow.register();assert.equal(f.flow.packages.pending,null);assert.deepEqual(f.calls.map(x=>x[0]),["register","refresh"]); });
  }
  if (group === "route") {
    await test("public route views escape backend labels and never grant execution permission", () => {
      const {routeDetailBody,routeCard}=load("src/features/rules/index.ts"),route={competition_profile:{competition_kind:"league"},package_display_name:"<rule>",package_version:"1",model_version:"1",parameter_version:"1",rule_package_id:"rp",binding_id:null,model_version_id:"mv",parameter_set_id:"ps"},labels={source:"<scope>",model:"<model>"};
      const html=routeDetailBody(route,labels)+routeCard(route,labels);assert.ok(html.includes("&lt;rule&gt;")&&html.includes("&lt;scope&gt;"));assert.ok(html.includes("正式执行仍需后端重新校验"));assert.ok(!html.includes("可直接开始推演"));
    });
    await test("backend preview is returned without frontend selection or mutation", async () => {const result={binding_id:"backend"},f=fixture({previewRoute:async()=>result});assert.equal(await f.flow.resolve(command(),()=>true),result);assert.equal(f.flow.route,result);});
    await test("changed form identity discards late A", async () => {const gate=deferred(),f=fixture({previewRoute:()=>gate.promise});let current=true;const task=f.flow.resolve(command(),()=>current);current=false;gate.resolve({binding_id:"a"});assert.equal(await task,null);assert.equal(f.flow.route,null);});
    await test("Session replacement discards late summary even for the same selection", async () => {const gate=deferred(),f=fixture({previewRoute:()=>gate.promise}),task=f.flow.resolve(command(),()=>true);f.replace();gate.resolve({});assert.equal(await task,null);});
    await test("new preview wins over older response", async () => {const gate=deferred();let n=0;const f=fixture({previewRoute:()=>++n===1?gate.promise:Promise.resolve({binding_id:"b"})}),a=f.flow.resolve(command(),()=>true);assert.equal((await f.flow.resolve({...command(),...hierarchy("b")},()=>true)).binding_id,"b");gate.resolve({binding_id:"a"});assert.equal(await a,null);assert.equal(f.flow.route.binding_id,"b");});
    await test("binding write invalidates pending preview; next preview reaches updated backend", async () => {const gate=deferred();let changed=false;const f=fixture({previewRoute:()=>changed?Promise.resolve({binding_id:"new"}):gate.promise,createBinding:async()=>{changed=true;}}),task=f.flow.resolve(command(),()=>true);await f.flow.saveBinding(binding());gate.resolve({binding_id:"old"});assert.equal(await task,null);assert.equal((await f.flow.resolve(command(),()=>true)).binding_id,"new");});
    await test("current backend rejection propagates and clears old preview; stale error is suppressed", async () => {const gate=deferred(),f=fixture({previewRoute:()=>gate.promise});const task=f.flow.resolve(command(),()=>true);f.flow.invalidatePreview();gate.reject(Error("stale"));assert.equal(await task,null);const g=fixture({previewRoute:async()=>{throw Error("no valid rule");}});await assert.rejects(g.flow.resolve(command(),()=>true),/no valid rule/);assert.equal(g.flow.route,null);});
    await test("lifecycle disposal prevents stale route and imported file publication", async () => {const gate=deferred(),f=fixture({previewRoute:()=>gate.promise}),task=f.flow.resolve(command(),()=>true);f.flow.stop();gate.resolve({});assert.equal(await task,null);assert.equal(await f.flow.packages.read(file("closed")),false);});
  }
  console.log(`R13-03 production ${group} workflow: ${cases} behavior cases passed.`);
  return cases;
}
