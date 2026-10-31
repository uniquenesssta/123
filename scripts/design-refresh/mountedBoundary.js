// Explicit QA data; never imported into the production application.
const originalInvoke=window.__TAURI_INTERNALS__.invoke;
Object.assign(window.__summary.data.database_health,{database_name:"ui_design_qa",server_version:"PostgreSQL QA fixture",migration_count:46});
window.__summary.data.database_url="postgres://fixture:fixture@localhost:5432/ui_design_qa";
window.__TAURI_INTERNALS__.invoke=async(command,args)=>{
  if(command==="configure_database"&&window.__rejectConnection){window.__calls.push({command,args:{redacted:true}});throw Error("连接被拒绝（QA transport boundary）");}
  if(command==="list_players") {window.__calls.push({command,args:structuredClone(args)});const page=structuredClone(window.__fixtures.playerCatalogue);if(args.query?.cursor_id){page.items=page.items.slice(0,2);page.has_more=false;page.next_cursor_id=null;page.next_cursor_name=null;}return page;}
  if(command==="read_team") {window.__calls.push({command,args:structuredClone(args)});return {...structuredClone(window.__fixtures.detail),team:structuredClone(window.__fixtures.teams.find(t=>t.id===args.teamId))};}
  if(command==="read_player") {
    const record=window.__fixtures.playerCatalogue.items.find(p=>p.id===args.playerId);
    const member=window.__fixtures.detail.squad.find(p=>p.player_id===args.playerId);
    if(record||member) { window.__calls.push({command,args:structuredClone(args)});return {player:{...structuredClone(record??window.__fixtures.playerCatalogue.items[0]),id:args.playerId,canonical_name:record?.canonical_name??member.player_name},names:[],positions:[],team_periods:[],availability:[],ability_profile:null,ability_observations:[],dynamic_tags:[],external_ids:[]}; }
  }
  return originalInvoke(command,args);
};
