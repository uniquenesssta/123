import type { P4TaskWorkspace, ResolveP4ConflictCommand } from "../../../types";

export interface ResearchDecisionIntent {
  matchId:string; taskId:string; version:string; command:ResolveP4ConflictCommand;
}
export function researchVersion(workspace:P4TaskWorkspace):string {
  return JSON.stringify([workspace.task,workspace.readiness,workspace.conflicts,workspace.evidence,workspace.routes]);
}
export function validateResearchTask(workspace:P4TaskWorkspace,matchId:string,taskId:string):void {
  if(workspace.task.id!==taskId||workspace.task.match_id!==matchId||workspace.readiness.task_id!==taskId)
    throw Error("研究任务返回身份不匹配，请重新载入原比赛");
}
export function validateDecision(workspace:P4TaskWorkspace,command:ResolveP4ConflictCommand,now=Date.now()):void {
  const conflict=workspace.conflicts.find(item=>item.id===command.conflict_id);
  if(command.task_id!==workspace.task.id||!conflict)throw Error("冲突不属于当前任务");
  if(conflict.manual_decision_kind)throw Error("该冲突已有不可变决策，请查回实际记录");
  if(!["RESEARCH_PARTIAL","BLOCKED"].includes(workspace.task.state)||!(now< Date.parse(workspace.task.data_cutoff_at)))
    throw Error("该任务已只读或超过数据截止时间，不能追加决策");
  if(conflict.evaluation_status!=="manual_required"||!workspace.routes.some(r=>r.field_key===conflict.field_key&&r.route_status==="blocked_conflict"))
    throw Error("当前冲突不再允许人工处理，请刷新任务");
  if(command.decision_kind==="accept_unknown"){
    if(command.selected_evidence_ids.length)throw Error("接受未知不能同时选择证据");
  }else if(command.selected_evidence_ids.length!==1||!conflict.evidence_ids.includes(command.selected_evidence_ids[0])||!workspace.evidence.some(e=>e.id===command.selected_evidence_ids[0]))
    throw Error("请选择当前冲突中实际存在的证据来源");
}
export function decisionRecorded(workspace:P4TaskWorkspace,command:ResolveP4ConflictCommand):boolean {
  const conflict=workspace.conflicts.find(item=>item.id===command.conflict_id);
  return !!conflict&&conflict.manual_decision_kind===command.decision_kind&&
    JSON.stringify([...conflict.selected_evidence_ids].sort())===JSON.stringify([...command.selected_evidence_ids].sort())&&
    (conflict.manual_decision_note?.trim()||null)===(command.note?.trim()||null);
}
export function verifiedResearchSnapshot(workspace:P4TaskWorkspace):boolean {
  const snapshot=workspace.snapshot?.snapshot as Record<string,unknown>|undefined;
  return workspace.task.state==="FROZEN"&&!!workspace.task.snapshot_id&&snapshot?.id===workspace.task.snapshot_id&&
    snapshot.match_id===workspace.task.match_id&&snapshot.horizon===workspace.task.horizon;
}
