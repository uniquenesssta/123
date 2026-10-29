import type { PlayerListPage } from "../../src/types";
import type { BootstrapResponse, TeamDetail, TeamListPage, PlayerCatalogReferenceData, PairedLineupBuilderState } from "../../src/types";

/** Synthetic boundary data for layout verification only; never imported by the application. */
export const teams = ["首尔 FC", "全北现代", "新建青年队", "资料待完善队"].map((name, i) => ({
  id: `layout-team-${i}`, canonical_name: name, country_code: i === 3 ? "GB" : "KR", team_type: i === 2 ? "youth" : "club",
  current_player_count: [6, 4, 0, 2][i], current_coach_name: i < 2 ? ["Kim", "Park"][i] : null, profile_confidence: .8,
}));
export const catalogue = { items: teams, has_more: true, next_cursor: "layout-next" } as unknown as TeamListPage;
export const match = { id: "layout-match", competition_id: "c1", competition_name: "示例职业联赛", home_team_id: teams[0].id, away_team_id: teams[1].id, home_team_name: teams[0].canonical_name, away_team_name: teams[1].canonical_name, kickoff_time: "2030-10-01T11:30:00Z", status: "scheduled", season_id: null, stage_id: null, round_id: null };
export const state = { data: {
  database_configured: true, app_version: "0.23.0", recent_runs: [], models: [], rule_packages: [], competition_bindings: [], seasons: [], stages: [], rounds: [], upcoming_matches: [match],
  competitions: Array.from({ length: 30 }, (_, i) => ({ id: `c${i+1}`, name: `示例赛事 ${i+1}`, code: `C${i+1}`, country_code: i % 2 ? "INT" : "KR", competition_kind: i % 2 ? "group_stage" : "league", metadata: {scope: i % 2 ? "national" : "club"} })),
  stats: { players: 712, pending_ability_updates: 3, matches: 86, active_lineups: 14, model_runs: 312, rule_packages: 8, large_counts_are_estimates: true },
  database_health: { latency_ms: 18, database_size_bytes: 2400000000 },
} } as unknown as BootstrapResponse;
const squad = Array.from({length:6},(_,i)=>({player_id:`layout-roster-${i}`,player_name:i===0?"Alexander-Arnold Test Player With A Long Name":`Roster player ${i+1}`,localized_name:i===0?"长名称球员布局边界示例":`示例球员 ${i+1}`,position_code:["GK","CB","LB","CM","RW","ST"][i],role_code:null,squad_number:i+1,registration_status:"active",availability_status:i===1?"injured":"available",ability_average:i===2?null:70+i}));
export const detail = { team: teams[0], profile: { team_type: "club", head_coach: "Kim", default_formation: "4-3-3", data_confidence: .8 }, names: [], squad, coach_periods: [], recent_matches: [], memberships: [], formation_history: [], formation_usage: [], resolved_formation_distribution: {source_label: "尚未观察",entries: []}, formation_distribution: null, player_periods: [], lineup_history: [] } as unknown as TeamDetail;
export const references = { teams, competitions: state.data.competitions, managed_matches: [match], upcoming_matches: [match], seasons: [], stages: [], rounds: [], formations: [], positions: ["GK","RB","CB","LB","CM","RW","ST","LW"].map(code => ({ code, name: `${code} 位置` })), lineup_types: [], snapshot_types: [] } as unknown as PlayerCatalogReferenceData;
const side = (index: number) => ({ team_id: teams[index].id, team_name: teams[index].canonical_name, formation: "4-3-3", formation_id: "", coach_id: "", quality_score: .95, candidates: [], players: Array.from({ length: 11 }, (_, i) => ({ player_id: `layout-player-${index}-${i}`, player_name: `${index ? "客队" : "主队"}球员 ${String(i+1).padStart(2,"0")}`, position_code: ["GK","RB","CB","CB","LB","CM","CM","CM","RW","ST","LW"][i], is_starter: true, availability_status: "available" })) });
export const builder = { match_id: match.id, lineup_type: "expected", snapshot_label: "T-1h", snapshot_type: "T-1h", source_url: "", recorded_at: "", observed_at: "2030-10-01T10:00", home: side(0), away: side(1) } as unknown as PairedLineupBuilderState;

/** Explicit synthetic boundary data for UI verification only; never a production data source. */
export const playerCatalogue:PlayerListPage={items:Array.from({length:18},(_,i)=>({id:"fixture-player-"+i,canonical_name:i===0?"Alexander-Arnold Very Long Boundary Player Name":"Fixture Player "+(i+1),localized_name:i===0?"长名称球员边界示例":"示例球员 "+(i+1),alternate_name:null,normalized_name:"fixture player "+i,date_of_birth:"2000-01-01",nationality_code:"KR",preferred_foot:"right",status:"active",current_team_id:teams[0].id,current_team_name:teams[0].canonical_name,primary_position_code:i%2?"CM":"GK",primary_role_code:null,position_role_map:{},availability_status:i===1?"injured":"available",availability_reason:i===1?"测试夹具伤病状态":null,availability_confidence:.8,availability_valid_to:null,availability_competition_name:null,ability_average:i===2?null:70+i,ability_confidence:i===2?null:.8,ability_dimension_count:i===2?0:6})),has_more:true,next_cursor_name:"fixture player17",next_cursor_id:"fixture-player-17"};
