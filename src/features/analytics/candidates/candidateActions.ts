import type {AnalyticsActionPorts} from '../analyticsActions';import type {CandidateFeature} from './candidateFeature';import type {CandidateOperation} from './candidatePorts';
type CandidateActionPorts=Omit<AnalyticsActionPorts,"notify">&{notify(text:string,tone?:"success"|"normal"):void};
export class CandidateActions {
 private detailSequence=0;
 constructor(private readonly feature:CandidateFeature,private readonly ports:CandidateActionPorts){}
 ability(id:string,decision:'accept'|'reject'){
  const t=this.feature.ability.prepare(id,decision),r=t.row;
  this.ports.confirm(decision==='accept'?'写入能力历史':'拒绝能力候选',decision==='accept'?'只接受原球员、维度和候选值；写前重读，后端检查当前能力投影。':'只拒绝原候选，不修改能力历史。',[['候选',r.id],['球员 / 维度',r.player_name+' / '+r.dimension_name],['原值 → 候选值',String(r.current_value??'未设置')+' → '+r.proposed_value],['计算版本',r.calculation_version]],'确认决定',async()=>{if(await this.feature.ability.submit(t))this.ports.notify(this.feature.context.state.outcome!,this.feature.context.state.outcomeTone==="warning"?"normal":"success");});
 }
 parameter(id:string,operation:CandidateOperation){
  const t=this.feature.parameters.prepare(id,operation),r=t.row,label=({accept_for_backtest:'接受候选进入影子队列',reject:'拒绝参数候选',shadow:'运行影子验证',promote:'人工晋升参数候选',rollback:'按原绑定快照回滚'} as Record<string,string>)[operation];
  this.ports.confirm(label,operation==='promote'?'影子通过不等于晋升。提交前重新检查原候选、精确分区、最新验证和正式基线。':operation==='rollback'?'仅恢复原晋升决定保存的绑定；其他版本已接管时拒绝覆盖。':'保留原候选身份；接受只进入影子队列，验证失败或阻断均不改变正式绑定。',[['候选',r.id],['赛事 / Profile',String(r.competition_name??r.competition_id)+' / '+r.competition_profile_id],['分区 / 时点',r.partition_key+' / '+r.snapshot_type],['基线 → 候选',r.parameter_version+' → '+(r.candidate_parameter_version??'未生成')],...(t.note?[['人工说明',t.note] as [string,string]]:[])],'确认操作',async()=>{if(await this.feature.parameters.submit(t))this.ports.notify(this.feature.context.state.outcome!,this.feature.context.state.outcomeTone==="warning"?"normal":"success");});
 }
 async history(id:string){const sequence=++this.detailSequence,result=await this.feature.context.history(id);if(sequence===this.detailSequence&&result)this.ports.show('原候选生命周期 '+id,result);}
 abilityDiagnostic(id:string){this.detailSequence++;const row=this.feature.context.runtime.abilities().find(r=>r.id===id);if(!row)throw Error('能力候选不在当前目录');this.ports.show('原能力候选证据 '+id,row);}
 diagnostic(id:string){this.detailSequence++;const row=this.feature.context.runtime.candidates().find(r=>r.id===id);if(!row)throw Error('候选不在当前目录');this.ports.show('原候选诊断 '+id,row);}
 provider(){this.detailSequence++;this.ports.show('提供器边界',{parameter_generation:'当前公开服务 generate_parameter_tuning_candidate 返回模型提供器不可用；不生成或伪造私有参数。',history:'保留真实历史候选与报告供核对；能力候选接受和参数晋升是不同写操作。',maximum_relative_change:this.feature.context.state.draft.fields.max_relative_change});}
}
