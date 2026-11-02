const caInvoke=window.__TAURI_INTERNALS__.invoke;
window.__ca={rows:[structuredClone(window.__candidateData.parameter)],abilities:[structuredClone(window.__candidateData.ability)],validations:[],decisions:[],binding:'baseline',writes:[],lost:false,failRollback:false};
window.__TAURI_INTERNALS__.invoke=async(command,args)=>{
 const c=window.__ca,d=window.__candidateData,record=()=>window.__calls.push({command,args:structuredClone(args)}),write=()=>{record();c.writes.push({command,args:structuredClone(args)});};
 if(command==='list_parameter_tuning_candidates'){record();return structuredClone(c.rows);}
 if(command==='list_ability_candidates'){record();return structuredClone(c.abilities.filter(r=>!args.status||r.status===args.status));}
 if(command==='parameter_lifecycle_readiness'){record();const gate=structuredClone(d.gate);Object.assign(gate,{competition_id:args.request.competition_id,snapshot_type:args.request.snapshot_type,minimum_sample_size:args.request.minimum_sample_size});if(c.binding!=='baseline'){gate.active_model_version_id=c.binding==='candidate'?'qa-candidate-model':'qa-other-model';gate.active_parameter_set_id=c.binding==='candidate'?'qa-candidate-parameters':'qa-other-parameters';}if(c.blocked){gate.ready_for_shadow_validation=false;gate.ready_for_promotion=false;gate.blocked_reasons=['QA 精确分区结算不足'];}return gate;}
 if(command==='list_parameter_shadow_validations'){record();return structuredClone(c.validations.filter(r=>r.candidate_id===args.candidateId));}
 if(command==='list_parameter_promotion_decisions'){record();return structuredClone(c.decisions.filter(r=>r.candidate_id===args.candidateId));}
 if(command==='decide_ability_candidate'){write();const row=c.abilities.find(r=>r.id===args.draft.candidate_id);row.status=args.draft.decision==='accept'?'accepted':'rejected';if(row.status==='accepted')row.accepted_observation_id='qa-observation';return structuredClone(row);}
 if(command==='decide_parameter_tuning_candidate'){write();const row=c.rows.find(r=>r.id===args.draft.candidate_id);if(row.status!=='pending')throw Error('QA 候选已被其他人处理');row.status=args.draft.decision==='reject'?'rejected':'accepted_for_backtest';return structuredClone(row);}
 if(command==='run_parameter_shadow_validation'){write();const report=structuredClone(d.validation);report.id='qa-new-validation';c.validations.unshift(report);c.rows[0].status='shadow_passed';return report;}
 if(command==='promote_parameter_candidate'||command==='rollback_parameter_candidate'){
  write();const promote=command==='promote_parameter_candidate';if(!promote&&c.failRollback)throw Error('QA 回滚绑定冲突');if(promote&&c.binding!=='baseline'||!promote&&c.binding!=='candidate')throw Error('QA 正式版本已变化');c.binding=promote?'candidate':'baseline';c.rows[0].status=promote?'promoted':'rolled_back';const decision={id:promote?'qa-promoted':'qa-rolledback',candidate_id:c.rows[0].id,decision:promote?'promote':'rollback',previous_binding_state:[],new_binding_state:[],decided_by:'local_user',decision_note:args.request.decision_note,created_at:d.parameter.created_at};c.decisions.unshift(decision);if(c.lost)throw Error('QA 原决定回执丢失');return structuredClone(decision);
 }
 return caInvoke(command,args);
};
