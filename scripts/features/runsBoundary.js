// Explicit QA transport only. Production renders and Session/feature owners remain unchanged.
const previousRunsInvoke=window.__TAURI_INTERNALS__.invoke;
const runHash='a'.repeat(64);
const runRows=['a','b','c','d','e','f'].map((id,i)=>({id:'run-'+id,match_key:'QA-'+id,home_team_name:i===5?null:'QA主队 '+id,away_team_name:i===5?null:'QA客队 '+id,competition_name:'明确QA赛事',model_key:i===4?null:'p4',model_version:'qa-1',parameter_version:'qa-param',rule_package_name:i===4?null:'QA规则',snapshot_type:i===5?null:'T-1h',input_readiness_level:['formal_ready','ready_with_warnings','shadow_only','blocked',null,null][i],input_readiness_score:i<4?100-i*20:null,input_manifest_sha256:i<4?runHash:null,top_scoreline:i<2?'1-0':null,top_scoreline_probability:i<2?.2:null,kickoff_time:i===5?null:'2030-01-02T12:00:00Z',created_at:'2030-01-01T00:00:00Z',summary:{home_win:i===3?null:i===2?0:.5,draw:i===3?null:.3,away_win:i===3?null:.2}}));
window.__runs={rows:runRows,docs:Object.fromEntries(runRows.map(r=>[r.id,{...r,rule_package_id:r.rule_package_name?'qa-rule':null,input_sha256:r.input_manifest_sha256?'b'.repeat(64):null,input_audit:{readiness_level:r.input_readiness_level,readiness_score:r.input_readiness_score,manifest_sha256:r.input_manifest_sha256,feature_snapshot_id:'snapshot-'+r.id},input:{source:'explicit-QA-boundary'},output:{team_a:r.home_team_name,team_b:r.away_team_name,scorelines:r.top_scoreline?[{goals_a:1,goals_b:0,probability:.2,cumulative_probability:.2,rank:1}]:[]},history_hidden_at:null,history_hidden_reason:null}])),listPending:false,listFail:false,detailPending:null,detailFail:null,hideFail:false,writes:[],reads:[]};
window.__summary.data.recent_runs=[];
window.__TAURI_INTERNALS__.invoke=async(command,args)=>{
 const q=window.__runs;
 if(command==='read_p4_match_workspace'){window.__calls.push({command,args});const m=window.__fixtures.match;return {match_id:args.matchId,match_key:args.matchId,home_team_name:m.home_team_name,away_team_name:m.away_team_name,kickoff_at:m.kickoff_time,competition_name:m.competition_name,tasks:[]};}
 if(command==='list_recent_runs'){
  window.__calls.push({command,args});
  if(q.listPending)await new Promise(resolve=>q.releaseList=()=>{q.listPending=false;resolve();});
  if(q.listFail)throw Error('QA list failure');
  return structuredClone(q.rows);
 }
 if(command==='read_run'){
  window.__calls.push({command,args});q.reads.push(args.runId);
  if(q.detailPending===args.runId)await new Promise(resolve=>q.releaseDetail=()=>{q.detailPending=null;resolve();});
  if(q.detailFail===args.runId)throw Error('QA detail failure');
  if(!q.docs[args.runId])throw Error('QA run missing');
  return structuredClone(q.docs[args.runId]);
 }
 if(command==='hide_model_run_history'){
  window.__calls.push({command,args});q.writes.push(args.runId);
  q.docs[args.runId].history_hidden_at='2030-02-01T00:00:00Z';q.docs[args.runId].history_hidden_reason=args.reason;
  q.rows=q.rows.filter(r=>r.id!==args.runId);
  if(q.hideFail)throw Error('QA response lost after write');
  return null;
 }
 return previousRunsInvoke(command,args);
};
