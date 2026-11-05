// Explicit test transport; no real user logs, filesystem writes or business requests.
const logBaseInvoke=window.__TAURI_INTERNALS__.invoke;
window.__logs={rows:window.__logFixtureRows,calls:[],readFailure:false,clearFailure:false,arrivalOnClear:false,savePath:null};
window.__TAURI_INTERNALS__.invoke=async(command,args)=>{
 const s=window.__logs,copy=value=>structuredClone(value);
 if(command==='list_issue_logs'){s.calls.push({command});if(s.readFailure)throw Error('QA history read failed');return copy(s.rows);}
 if(command==='plugin:dialog|save'){s.calls.push({command});return s.savePath;}
 if(command==='export_issue_logs'){s.calls.push({command,args:copy(args)});if(JSON.stringify(args.expectedEntries)!==JSON.stringify(s.rows))throw Error('问题日志集合已变化，请刷新后重新核对并确认');return null;}
 if(command==='clear_issue_logs'){s.calls.push({command,args:copy(args)});if(s.arrivalOnClear){s.rows.push({...copy(s.rows[0]),id:'ISSUE-ARRIVAL'});s.arrivalOnClear=false;}if(JSON.stringify(args.expectedEntries)!==JSON.stringify(s.rows))throw Error('问题日志集合已变化，请刷新后重新核对并确认');if(s.clearFailure)throw Error('QA clear acknowledgement lost');s.rows=[];return null;}
 return logBaseInvoke(command,args);
};
