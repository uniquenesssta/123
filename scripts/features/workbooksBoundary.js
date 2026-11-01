// Synthetic native transport boundary for the production S09 workflow, never a production import.
const beforeWorkbooks=window.__TAURI_INTERNALS__.invoke;
window.__workbooks={file:"a.xlsx",kind:"ready",pending:false,timeout:false,applied:false,writes:0,exports:0,batches:{}};
window.__TAURI_INTERNALS__.invoke=async(command,args)=>{
 const w=window.__workbooks;
 const types={preview_team_monthly_import:"team",preview_player_catalog_import:"player",preview_match_lineup_import:"match"};
 const reads={read_team_monthly_import_preview:"team",read_player_catalog_import_preview:"player",read_match_lineup_import_preview:"match"};
 const commits={commit_team_monthly_import:"team",commit_player_catalog_import:"player",commit_match_lineup_import:"match"};
 const resolves={resolve_team_monthly_import_conflict:"team",resolve_player_catalog_import_conflict:"player",resolve_match_lineup_import_conflict:"match"};
 if(command==="plugin:dialog|open"){window.__calls.push({command,args});return w.file;}
 if(command==="plugin:dialog|save"){window.__calls.push({command,args});return w.save===null?null:"qa-export.xlsx";}
 if(types[command]) {
  window.__calls.push({command,args:structuredClone(args)});
  if(w.kind==="wrong")throw Error("QA: 错误工作簿类型");
  const id=types[command]+"-"+w.file,kind=w.kind,hash=(types[command]==="team"?"a":types[command]==="player"?"b":"c").repeat(64);
  const value={batch_id:id,source_sha256:hash,source_file_name:w.file,import_mode:args.mode,created_at:"2030-01-01",counts:{total:12,ready_add:kind==="none"?0:2,ready_update:0,ready_end_previous:0,conflict:kind==="conflict"?1:0,error:kind==="error"?1:0,skipped:0,imported:kind==="imported"?2:0},rows:Array.from({length:12},(_,i)=>({id:id+"-"+i,sheet_name:types[command],row_number:i+2,entity_type:types[command],action:"add",status:kind==="conflict"&&i===0?"conflict":kind==="error"&&i===0?"error":"ready_add",message:i===0?"实际QA行；未知值不补零":"带有很长的说明以验证自动换行和固定列对齐",payload:{canonical_name:"长名称球队或球员 "+i},matched_entity_id:null,conflict_candidates:kind==="conflict"&&i===0?[{entity_id:"candidate",display_name:"匹配对象",detail:"KR"}]:[]}))};
  if(w.pending)await new Promise(r=>w.release=r);
  w.batches[id]=structuredClone(value);return value;
 }
 if(resolves[command]) {window.__calls.push({command,args:structuredClone(args)});const value=w.batches[args.batchId];value.counts.conflict=0;value.rows[0].status="ready_add";return structuredClone(value);}
 if(commits[command]) {
  window.__calls.push({command,args:structuredClone(args)});w.writes++;
  if(w.applied||!w.timeout){const value=w.batches[args.batchId];value.counts.ready_add=0;value.counts.imported=2;for(const row of value.rows)row.status="imported";}
  if(w.timeout)throw Error("QA transport timeout");
  return {batch_id:args.batchId,inserted_count:2,updated_count:0,ended_previous_count:1,skipped_count:0,error_count:0,finished_at:"2030-01-01"};
 }
 if(reads[command]) {window.__calls.push({command,args:structuredClone(args)});return structuredClone(w.batches[args.batchId]);}
 if(command.startsWith("export_")&&/template|monthly_data|catalog_data|lineup_data/.test(command)){window.__calls.push({command,args:structuredClone(args)});w.exports++;return {output_path:"qa-export.xlsx",player_count:1,team_count:1,lineup_count:1,related_row_count:1};}
 return beforeWorkbooks(command,args);
};
