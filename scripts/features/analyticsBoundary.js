const anInvoke=window.__TAURI_INTERNALS__.invoke;
const ad=analysisData();
window.__an={...ad,latest:ad.overview(),jobs:[ad.job('qa-job-a','running'),ad.job('qa-job-b','failed','full_analysis_refresh','qa-c2')],suggestions:[],writes:[],preview:ad.preview(),file:'O:/qa-response.zip',lostCancel:false,lostImport:false};
window.__pm.records.push(pmRecord('a'));
window.__TAURI_INTERNALS__.invoke=async(command,args)=>{
 const a=window.__an,record=()=>window.__calls.push({command,args:structuredClone(args)}),write=()=>{record();a.writes.push({command,args:structuredClone(args)});};
 if(command==='analytics_overview'){record();return structuredClone(args.snapshotId?ad.overview(args.snapshotId,args.snapshotId==='qa-snapshot-b'?'qa-c2':null):a.latest);}
 if(command==='list_background_jobs'){record();return structuredClone(a.jobs);}
 if(command==='list_ai_analysis_suggestions'){record();return structuredClone(a.suggestions.filter(r=>!args.status||r.status===args.status));}
 if(command==='list_ability_candidates'||command==='list_parameter_tuning_candidates'){record();return [];}
 if(command==='enqueue_analysis_job'){write();let row=a.jobs.find(j=>j.key===args.draft.idempotency_key);if(!row){row={...ad.job('qa-enqueued-'+a.jobs.length,'queued',args.draft.job_type),key:args.draft.idempotency_key,payload:structuredClone(args.draft.payload)};a.jobs.unshift(row);}return structuredClone(row);}
 if(command==='cancel_background_job'){write();const row=a.jobs.find(j=>j.id===args.jobId);row.cancellation_requested=true;if(row.status==='queued')row.status='cancelled';if(a.lostCancel)throw Error('QA 取消回执丢失');return structuredClone(row);}
 if(command==='retry_background_job'){write();const row=a.jobs.find(j=>j.id===args.jobId);row.status='queued';row.updated_at='2026-11-02T05:00:00Z';return structuredClone(row);}
 if(command==='plugin:dialog|open'||command==='plugin:dialog|save'){record();return a.file;}
 if(command==='export_ai_analysis_package'){write();if(args.expectedSnapshotId!==a.latest.source_snapshot.id)throw Error('QA 原快照已变化');return{package_id:'qa-package',output_path:args.outputPath,content_sha256:'a'.repeat(64),sample_size:24,created_at:ad.stamp};}
 if(command==='export_ai_analysis_response_template'){write();return args.outputPath;}
 if(command==='preview_ai_analysis_response'){record();if(a.badFile)throw Error('QA 无效ZIP');return structuredClone(a.preview);}
 if(command==='import_ai_analysis_response'){write();if(JSON.stringify(args.expectedPreview)!==JSON.stringify(a.preview))throw Error('QA 预检已变化');if(a.suggestions.some(r=>r.response_id===a.preview.manifest.response_id))throw Error('QA 重复回包');a.suggestions.push(...structuredClone(a.preview.suggestions));if(a.lostImport)throw Error('QA 导入回执丢失');return structuredClone(a.suggestions);}
 if(command==='decide_ai_analysis_suggestion'){write();const row=a.suggestions.find(r=>r.id===args.draft.suggestion_id);if(row.status!=='pending')throw Error('QA 已处理');row.status=args.draft.decision==='accept'?'accepted':'rejected';return structuredClone(row);}
 if(command==='decide_data_quality_finding'){write();const row=a.latest.data_quality.findings.find(r=>r.id===args.draft.finding_id);row.status=args.draft.decision==='resolve'?'resolved':'ignored';return structuredClone(row);}
 return anInvoke(command,args);
};
