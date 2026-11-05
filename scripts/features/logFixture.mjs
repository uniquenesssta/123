export const logEntry=(id='ISSUE-A',overrides={})=>({id,severity:'error',source:'frontend',app_version:'0.23.0',user_message:'QA 日志问题',technical_message:'QA diagnostic',occurrence_count:2,first_seen_at:'2026-11-05T00:00:00Z',last_seen_at:'2026-11-05T01:00:00Z',operations:['list_players'],occurrence_keys:['ipc-QA-A'],...overrides});
/** Only external APIs/storage are faked; every workflow assertion calls the production Feature. */
export function logFixture(LogsFeature){
  let rows=[logEntry()],saved=null;const calls={reads:0,exports:[],clears:[],choosers:0},view={current:true};
  const api={listIssueLogs:async()=>{calls.reads++;return structuredClone(rows);},chooseIssueLogExportFile:async()=>{calls.choosers++;return 'QA-report.txt';},exportIssueLogs:async(path,expected)=>{calls.exports.push({path,expected});},clearIssueLogs:async expected=>{calls.clears.push(structuredClone(expected));rows=[];}};
  const storage={read:()=>saved??{section:'overview',expanded:[],unknownClear:false,operationId:null},save:value=>{saved=structuredClone(value);}};
  const ports={api,storage,view:()=>()=>view.current,changed:()=>{},busy:run=>run()};
  return {feature:new LogsFeature(ports),api,calls,view,storage,get saved(){return saved;},rows:()=>structuredClone(rows),setRows:value=>{rows=structuredClone(value);},fresh:()=>new LogsFeature(ports)};
}
