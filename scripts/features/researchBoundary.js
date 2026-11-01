// Explicit QA transport boundary; never imported by production modules.
const beforeResearchInvoke=window.__TAURI_INTERNALS__.invoke;
window.__fixtures.match.kickoff_time="2099-01-01T00:00:00Z";
window.__research={writes:0,plans:0,failAfterWrite:false,rows:{},delayTask:null};
const researchTask=(matchId,index)=>({id:matchId+"-research-"+index,match_id:matchId,match_key:matchId,horizon:["T-24h","T-6h","T-1h"][index],kickoff_at:"2099-01-01T00:00:00Z",data_cutoff_at:"2098-12-31T00:00:00Z",research_due_at:"2098-12-31T00:00:00Z",freeze_deadline_at:"2098-12-31T00:00:00Z",rule_package_id:"qa-rule-p4",model_version_id:"qa-model",parameter_set_id:"qa-parameters",competition_profile_id:"qa-league",research_schema_version_id:"qa-research",snapshot_schema_version_id:"qa-snapshot",trace_id:"qa-trace-"+index,requested_fact_keys:["lineup"],state:["RESEARCH_PARTIAL","FROZEN","FAILED"][index],research_run_id:"qa-research-run",snapshot_id:index===1?"qa-snapshot-"+matchId:null,task_fingerprint:"qa-task-fingerprint-"+index,idempotency_key:"qa-task-key-"+index,metadata:{},created_at:"2098-12-30T00:00:00Z",updated_at:"2098-12-30T00:00:00Z"});
const researchWorkspace=task=>({task,readiness:{task_id:task.id,ready:task.state==="FROZEN",research_status:task.state,requested_fact_count:2,routed_fact_count:1,missing_fact_count:0,ignored_fact_count:0,blocked_fact_count:task.state==="FROZEN"?0:1,blockers:task.state==="FROZEN"?[]:["首发来源存在冲突，尚未允许冻结。"]},research_run:{id:"qa-research-run"},evidence:[0,1].map((value,i)=>({id:"qa-evidence-"+i,field_key:"lineup",entity_type:"team",entity_id:"qa-home",value,verification_state:i?"UNVERIFIED":"PROBABLE",source_tier:"A",source_url:"https://example.test/source-"+i,source_title:"明确 QA 来源 "+(i+1),source_domain:"example.test",published_at:"2098-12-30T10:00:00Z",observed_at:null,effective_at:null,retrieved_at:"2098-12-30T11:00:00Z",timezone:"UTC",conflict_group_id:"qa-conflict",created_at:"2098-12-30T11:00:00Z"})),conflicts:[{id:"qa-conflict-"+task.id,field_key:"lineup",entity_type:"team",entity_id:"qa-home",conflict_key:"qa-conflict-key",status:"opened",evaluation_status:"manual_required",evidence_ids:["qa-evidence-0","qa-evidence-1"],selected_evidence_ids:[],manual_decision_kind:null,manual_decision_note:null,manual_decision_at:null,created_at:"2098-12-30T11:00:00Z"}],routes:[{field_key:"lineup",target_module:"lineup",target_slot:"starting",route_status:"blocked_conflict",verification_state:"CONFLICT",selected_value:null,reason:"来源不同，等待人工处理。"}],events:[{id:"qa-event",occurred_at:"2098-12-30T11:00:00Z",from_state:"RESEARCH_RUNNING",to_state:task.state,reason:task.state==="FAILED"?"QA external boundary: ModelProvider unavailable":"研究证据已保留，等待核验。"}],snapshot:task.snapshot_id?{snapshot:{id:task.snapshot_id,match_id:task.match_id,horizon:task.horizon},features:[],probabilities:[],input_payload:{}}:null});
window.__TAURI_INTERNALS__.invoke=async(command,args)=>{
 const q=window.__research;
 if(command==="read_p4_match_workspace"){
  window.__calls.push({command,args:structuredClone(args)});const id=args.matchId;
  return {match_id:id,match_key:id,home_team_name:id.endsWith("-b")?"B主队":"首尔 FC",away_team_name:"全北现代",competition_name:"QA 研究边界",kickoff_at:"2099-01-01T00:00:00Z",tasks:[0,1,2].map(i=>q.rows[researchTask(id,i).id]?.task??researchTask(id,i))};
 }
 if(command==="read_p4_task_workspace"){
  window.__calls.push({command,args:structuredClone(args)});
  const id=args.taskId;if(q.delayTask===id)await new Promise(resolve=>q.releaseTask=()=>{q.delayTask=null;resolve();});
  const matchId=id.replace(/-research-\d+$/,""),index=Number(id.at(-1));
  return structuredClone(q.rows[id]??researchWorkspace(researchTask(matchId,index)));
 }
 if(command==="resolve_p4_conflict"){
  window.__calls.push({command,args:structuredClone(args)});q.writes++;const c=args.command;
  const task=researchTask(c.task_id.replace(/-research-\d+$/,""),Number(c.task_id.at(-1))),workspace=structuredClone(q.rows[task.id]??researchWorkspace(task));
  Object.assign(workspace.conflicts[0],{manual_decision_kind:c.decision_kind,selected_evidence_ids:c.selected_evidence_ids,manual_decision_note:c.note,manual_decision_at:"2098-12-30T12:00:00Z"});
  workspace.readiness.blockers=[];workspace.readiness.blocked_fact_count=0;workspace.readiness.ready=true;q.rows[task.id]=workspace;
  if(q.failAfterWrite){q.failAfterWrite=false;throw Error("QA response lost after actual boundary write");}
  return structuredClone(workspace);
 }
 if(command==="plan_p4_horizons"){window.__calls.push({command,args:structuredClone(args)});q.plans++;return[0,1,2].map(i=>researchTask(args.command.match_id,i));}
 return beforeResearchInvoke(command,args);
};
