const releaseInvoke=window.__TAURI_INTERNALS__.invoke;
window.__release={reports:[],writes:[],nextStatus:'warning',lost:false,failed:false,delayed:null};
window.__TAURI_INTERNALS__.invoke=async(command,args)=>{
 const s=window.__release,record=()=>window.__calls.push({command,args:structuredClone(args)});
 if(command==='list_release_acceptance_runs'){record();if(s.listError)throw Error('QA 历史读取失败');return structuredClone(s.reports).reverse();}
 if(command==='read_release_acceptance_run'){record();if(s.readError===args.runId)throw Error('QA 报告读取失败');if(s.delayed===args.runId)await new Promise(r=>{s.resolveDetail=r;});const row=s.reports.find(r=>r.id===args.runId);if(!row)throw Error('QA report missing');return structuredClone(row);}
 if(command==='run_release_acceptance'){record();s.writes.push(structuredClone(args.request));if(s.failed)throw Error('QA 请求连接中断');if(s.running)await new Promise(r=>{s.resolveRun=r;});const row=window.releaseReport('qa-report-'+s.writes.length,s.nextStatus,args.request);s.reports.push(row);if(s.lost)throw Error('QA 回执丢失');return structuredClone(row);}
 return releaseInvoke(command,args);
};
