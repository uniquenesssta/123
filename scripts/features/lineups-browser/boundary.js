window.__calls=[];window.__errors=[];window.__rejectUpdate=true;window.__file='a.xlsx';window.__slowPlayer=null;window.__slowFile=null;window.__rejectCommit=true;window.__mode='add_and_update';window.__deleted=[];
const clone=x=>structuredClone(x),f=window.__fixtures;
window.__summary={...clone(f.state),config_path:'boundary.json',runtime_log_path:'boundary.log',connection_error:null};window.__summary.data.default_match={kickoff_time:'2030-01-01T12:00:00Z',team_a:{name:'Home'},team_b:{name:'Away'}};window.__summary.data.default_rule_package={};
const player=id=>({player:{...clone(f.playerCatalogue.items.find(p=>p.id===id)),height_cm:null},names:[],positions:[],team_periods:[],availability:[],ability_profile:null,ability_observations:[],dynamic_tags:[],external_ids:[]});
const team=id=>({...clone(f.detail),team:{...f.teams.find(t=>t.id===id)},squad:clone(f.detail.squad).map((p,i)=>({...p,player_id:'fixture-player-'+i}))});
const counts={ready_add:1,ready_update:0,ready_end_previous:0,conflict:0,error:0,skip:0,imported:0};
const batch=id=>({batch_id:id,source_file_name:id+'.xlsx',source_sha256:id[0].repeat(64),import_mode:window.__mode,counts:{...counts},rows:[],created_at:'2030-01-01'});

const slots=['GK','RB','RCB','LCB','LB','DM','RCM','LCM','RW','ST','LW'];
const formation={id:'formation-test-433',code:'4-3-3',name:'验证阵型',line_structure:'4-3-3',slot_definition:slots,is_builtin:true,is_active:true,sort_order:1,metadata:{}};
const squad=id=>Array.from({length:13},(_,i)=>({...clone(f.detail.squad[i%f.detail.squad.length]),player_id:'fixture-player-'+i,player_name:'测试球员 '+(i+1),position_code:slots[i%11],role_code:null,squad_number:i+1}));
const preset=(teamId,id='preset-1')=>({id,team_id:teamId,team_name:f.teams.find(t=>t.id===teamId)?.canonical_name,name:id==='preset-1'?'联赛主力':'轮换方案',formation_id:formation.id,formation_code:formation.code,coach_id:null,coach_name:null,status:'active',version:1,is_default:id==='preset-1',usage_context:'general',usage_probability:1,member_count:13,starter_count:11,members:squad(teamId).map((p,i)=>({...p,is_starter:i<11,sequence_no:i+1,expected_minutes:i<11?90:20,shirt_number:i+1,bench_order:i<11?null:i-10,metadata:{}}))});
window.__rejectPresetSave=true;window.__copies=[];window.__archived=[];

window.__TAURI_INTERNALS__={invoke:async(command,args)=>{
 window.__calls.push({command,args:clone(args)});
 if(command==='bootstrap')return clone(window.__summary);
 if(command==='read_workspace_state')return {schema_version:1,global:{sidebar_collapsed:false,ui_revision:5},modules:{}};
 if(['record_runtime_operation','record_issue','save_workspace_state'].includes(command))return null;
 if(command==='list_lineups')return [];
 if(command==='list_teams')return {items:clone(f.teams),has_more:false,next_cursor_name:null,next_cursor_id:null};
 if(command==='read_team')return {...team(args.teamId),squad:squad(args.teamId)};
 if(command==='list_formations')return [formation];
 if(command==='list_team_lineup_presets')return [preset(args.teamId),...window.__copies.filter(p=>p.team_id===args.teamId)].filter(p=>args.includeArchived||!window.__archived.includes(p.id)).map(p=>({...p,status:window.__archived.includes(p.id)?'archived':'active'}));
 if(command==='duplicate_team_lineup_preset'){const copy={...preset(f.teams[0].id),'id':'copy-'+window.__copies.length,name:args.name,is_default:false};window.__copies.push(copy);return copy;}
 if(command==='preview_team_lineup_preset_application')return {preset:preset(f.teams[0].id),can_apply:!window.__applyBlocked,blockers:window.__applyBlocked?['球员状态待核对']:[],warnings:[]};
 if(command==='save_team_lineup_preset'){if(window.__rejectPresetSave)throw Error('预设保存验证失败');return {...preset(args.draft.team_id),...args.draft};}
 if(command==='archive_team_lineup_preset'){window.__archived.push(args.presetId);return null;}
 if(['list_coaches','list_formations','list_team_match_lineups','list_team_lineup_presets'].includes(command))return [];
 if(command==='player_catalog_reference_data')return {...clone(f.references),formations:[formation],data_providers:[],season_team_memberships:[]};
 if(command==='list_players'){const offset=args.query.cursor_id?2:0;return {items:clone(f.playerCatalogue.items.filter(p=>!window.__deleted.includes(p.id)).slice(offset,offset+2)),has_more:!offset,next_cursor_name:offset?null:'player1',next_cursor_id:offset?null:'fixture-player-1'};}
 if(command==='read_player'){const value=player(args.playerId);if(window.__slowPlayer===args.playerId)await new Promise(r=>window.__releasePlayer=r);return value;}
 if(command==='update_player'){if(window.__rejectUpdate)throw Error('boundary player edit rejected');return {...player(args.playerId).player,...args.draft};}
 if(['add_player_availability','add_player_dynamic_tag','add_player_ability_observation'].includes(command))return {};
 if(command==='plugin:dialog|open')return window.__file;
 if(command==='preview_player_catalog_import'){window.__mode=args.mode;const value=batch(args.inputPath[0]);if(window.__slowFile===args.inputPath)await new Promise(r=>window.__releaseFile=r);return value;}
 if(command==='commit_player_catalog_import'){if(window.__rejectCommit)throw Error('boundary player import rejected');return {batch_id:args.batchId,inserted_count:1,updated_count:0,ended_previous_count:0,skipped_count:0,error_count:1};}
 if(command==='read_player_catalog_import_preview')return {...batch(args.batchId),counts:{...counts,ready_add:0,error:1,imported:1}};
 if(command==='bulk_delete_players'){window.__deleted.push(args.playerIds[0]);return {requested_count:args.playerIds.length,deleted_ids:[args.playerIds[0]],blocked:[{entity_id:args.playerIds[1],label:'Blocked B',reason:'boundary protected history'}]};}
 if(command==='check_entity_deletion')return {entity_id:args.entityId,label:'Protected player',exists:true,can_permanently_delete:false,references:[{relation:'lineups',count:1}],reason:'boundary protected history'};
 throw Error('Unexpected boundary command '+command);
}};
window.addEventListener('error',e=>window.__errors.push(e.message));window.addEventListener('unhandledrejection',e=>window.__errors.push(String(e.reason)));
