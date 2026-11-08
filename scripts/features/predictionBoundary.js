// Explicit QA native boundary; no production module imports these values.
const previousPredictionInvoke=window.__TAURI_INTERNALS__.invoke;
const qaModels=["p4","p7"].map(model_id=>({model_id,display_name:model_id.toUpperCase()+" QA boundary",engine_version:"qa-1",supported_competitions:["league"]}));
window.__summary.data.models=qaModels;
window.__summary.data.rule_packages=["p4","p7"].map((model,i)=>({id:"qa-rule-"+model,package_key:"qa-"+model,version:"qa-1",display_name:model.toUpperCase()+" QA规则",competition_kind:"league",model_id:model+"_league",model_version:"qa-1",parameter_version:"qa-param",priority:1,content_sha256:"c".repeat(64),status:"active",created_at:"2030-01-01"}));
window.__prediction={level:"formal_ready",pending:false,fail:false,failRead:false,unavailable:false,formal:0,shadow:0,rows:[],docs:{}};
const qaRoute=c=>({source:"competition_kind_default",binding_id:null,rule_package_id:c?.explicit_rule_package_id??"qa-rule-p4",package_key:"qa-p4",package_version:"qa-1",package_display_name:"明确QA外部边界",model_id:c?.model_family??"p4",model_version:"qa-1",parameter_version:"qa-param",competition_profile:{competition_kind:"league"},reason:{}});
window.__TAURI_INTERNALS__.invoke=async(command,args)=>{
 const q=window.__prediction,f=window.__fixtures;
 const matches=[f.match,{...f.match,id:"layout-match-b",home_team_name:"B主队",away_team_name:"B客队"}];
 if(command==="player_catalog_reference_data"){window.__calls.push({command,args});return {...structuredClone(f.references),managed_matches:matches,upcoming_matches:matches};}
 if(command==="read_p4_match_workspace"){const m=matches.find(m=>m.id===args.matchId)??matches[0];return {match_id:m.id,match_key:m.id,home_team_name:m.home_team_name,away_team_name:m.away_team_name,kickoff_at:m.kickoff_time,competition_name:m.competition_name,tasks:[]};}
 if(command==="inspect_match_prediction_readiness"){
  window.__calls.push({command,args:structuredClone(args)});const c=args.command;
  const r={audit_version:"qa-a1",match_id:c.match_id,match_key:c.match_id,snapshot_type:c.snapshot_type,model_family:c.model_family,assessed_at:"2030-01-01T00:00:00Z",data_cutoff_at:"2030-01-01T00:00:00Z",level:q.level,score:q.level==="blocked"?20:100,can_run_formal:q.level!=="blocked"&&q.level!=="shadow_only",can_run_shadow:q.level!=="blocked",blockers:q.level==="blocked"?["QA：缺少门将"]:[],warnings:[],checks:[{code:"identity",label:"比赛身份",status:"passed",score:10,weight:10,summary:"明确QA传输边界",details:[],metadata:{}}],input_manifest:{qa:true},input_manifest_sha256:"a".repeat(64),route_identity:{}};
  if(q.pending)await new Promise(resolve=>q.release=()=>{q.pending=false;resolve();});
  return r;
 }
 if(command==="read_match_lineup_chain"){
  window.__calls.push({command,args:structuredClone(args)});const m=matches.find(m=>m.id===args.matchId)??matches[0];
  return {match_record:m,snapshot_type:args.snapshotType,ready_for_model:true,data_cutoff_time:"2030-01-01T00:00:00Z",home:{team_id:m.home_team_id,team_name:m.home_team_name,versions:[],selected_lineup_id:"home-qa",blocking_issues:[]},away:{team_id:m.away_team_id,team_name:m.away_team_name,versions:[],selected_lineup_id:"away-qa",blocking_issues:[]},blocking_issues:[]};
 }
 if(command==="list_recent_runs"){window.__calls.push({command,args});if(q.refreshPending)await new Promise(resolve=>q.releaseRefresh=()=>{q.refreshPending=false;resolve();});return structuredClone(q.rows);}
 if(command==="read_run"){window.__calls.push({command,args});if(q.failRead)throw Error("QA readback timeout");return structuredClone(q.docs[args.runId]);}
 if(command==="preview_route"){window.__calls.push({command,args});return qaRoute(args.command);}
 if(command==="execute_prediction_from_match"||command==="execute_shadow_prediction_from_match"){
  window.__calls.push({command,args:structuredClone(args)});const shadow=command==="execute_shadow_prediction_from_match",c=args.command;shadow?q.shadow++:q.formal++;
  if(q.executePending)await new Promise(resolve=>q.releaseExecution=()=>{q.executePending=false;resolve();});
  if(q.unavailable)throw Error("模型运行时不可用：QA boundary provider unavailable");
  if(q.inputRejection)throw Error("模型执行失败：模型输入无效：WAITING_ALPHA_CALIBRATION");
  const id=shadow?"00000000-0000-0000-0000-000000000000":"qa-run-"+q.formal;
  const result={run_id:id,duration_ms:2,route:qaRoute(c),input_audit:{audit_version:"qa-a1",readiness_level:shadow?"shadow_only":"formal_ready",readiness_score:100,input_manifest_sha256:"a".repeat(64),input_sha256:"b".repeat(64)},output:{identity:{model_id:c.model_family,model_version:"qa-1",parameter_version:"qa-param",rule_package_version:"qa-1"},summary:{home_win:.5,draw:.3,away_win:.2,btts:null,over_2_5:null},payload:{team_a:"QA主队",team_b:"QA客队",scorelines:[{goals_a:1,goals_b:0,probability:.2,cumulative_probability:.2,rank:1}]},explanation:{}}};
  if(!shadow){q.rows.unshift({id,match_key:c.match_id,snapshot_type:c.snapshot_type,model_key:c.model_family,model_version:"qa-1",parameter_version:"qa-param",summary:result.output.summary,created_at:"2030-01-01",input_manifest_sha256:"a".repeat(64)});q.docs[id]={id,match_key:c.match_id,snapshot_type:c.snapshot_type,model_key:c.model_family+"_league",rule_package_id:c.explicit_rule_package_id,output:result.output.payload,summary:result.output.summary,input:{},input_audit:{manifest_sha256:"a".repeat(64),feature_snapshot_id:"qa-snapshot-"+id},route:result.route};}
  if(q.delayRefresh&&!shadow){q.refreshPending=true;q.delayRefresh=false;}
  if(q.fail)throw Error("QA transport timeout");
  return result;
 }
 if(command==="dry_run_default_fixture")throw Error("模型运行时不可用：QA public provider unavailable");
 return previousPredictionInvoke(command,args);
};
