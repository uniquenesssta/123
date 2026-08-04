use super::{parse_uuid, AppState};
use football_domain::{
    BulkArchiveResult, BulkDeleteResult, CoachDetail, CoachDraft, CoachListItem, CoachListQuery,
    CoachNameDraft, CoachNameRecord, CoachRecord, DataProviderDraft, DataProviderRecord,
    EntityDeletionCheck, EntityMatchRequest, EntityMatchResult, EntityReferenceQuery,
    EntityReferenceRecord, ExternalEntityIdDraft, ExternalEntityIdRecord,
    FormationDistributionQuery, FormationRecord, FormationUsageDistributionDraft,
    FormationUsageDistributionRecord, FormationUsageListQuery, LineupDraft,
    LineupHistoryRemovalResult, LineupPairDraft, LineupPairRecord, LineupRecord, MatchDraft,
    MatchLineupChain, MatchRecord, PlayerAbilityObservationDraft, PlayerAbilityObservationRecord,
    PlayerAvailabilityDraft, PlayerAvailabilityRecord, PlayerCatalogReferenceData, PlayerDetail,
    PlayerDraft, PlayerDynamicTagDraft, PlayerDynamicTagRecord, PlayerListPage, PlayerListQuery,
    PlayerMatchContribution, PlayerMatchContributionRequest, PlayerNameDraft, PlayerNameRecord,
    PlayerPositionDraft, PlayerPositionRecord, PlayerRecord, PlayerTeamPeriodDraft,
    PlayerTeamPeriodRecord, TeamCoachPeriodDraft, TeamCoachPeriodRecord, TeamDetail, TeamDraft,
    TeamForceDeletePreview, TeamForceDeleteRequest, TeamForceDeleteResult,
    TeamLineupPresetApplicationPreview, TeamLineupPresetDraft, TeamLineupPresetRecord,
    TeamListPage, TeamListQuery, TeamMatchLineupHistoryItem, TeamNameDraft, TeamNameRecord,
    TeamOption, TeamProfileDraft, TeamProfileRecord, TeamRecord,
};
use tauri::State;

#[tauri::command]
pub async fn list_formations(
    state: State<'_, AppState>,
    active_only: bool,
) -> Result<Vec<FormationRecord>, String> {
    state
        .service
        .list_formations(active_only)
        .await
        .map_err(|error| error.to_string())
}

#[tauri::command]
pub async fn save_formation_usage_distribution(
    state: State<'_, AppState>,
    draft: FormationUsageDistributionDraft,
) -> Result<FormationUsageDistributionRecord, String> {
    state
        .service
        .save_formation_usage_distribution(draft)
        .await
        .map_err(|error| error.to_string())
}

#[tauri::command]
pub async fn list_formation_usage_distributions(
    state: State<'_, AppState>,
    query: FormationUsageListQuery,
) -> Result<Vec<FormationUsageDistributionRecord>, String> {
    state
        .service
        .list_formation_usage_distributions(query)
        .await
        .map_err(|error| error.to_string())
}

#[tauri::command]
pub async fn resolve_formation_distribution(
    state: State<'_, AppState>,
    query: FormationDistributionQuery,
) -> Result<football_domain::ResolvedFormationDistribution, String> {
    state
        .service
        .resolve_formation_distribution(query)
        .await
        .map_err(|error| error.to_string())
}

#[tauri::command]
pub async fn player_catalog_reference_data(
    state: State<'_, AppState>,
) -> Result<PlayerCatalogReferenceData, String> {
    state
        .service
        .player_catalog_reference_data()
        .await
        .map_err(|error| error.to_string())
}
#[tauri::command]
pub async fn create_coach(
    state: State<'_, AppState>,
    draft: CoachDraft,
) -> Result<CoachRecord, String> {
    state
        .service
        .create_coach(draft)
        .await
        .map_err(|error| error.to_string())
}
#[tauri::command]
pub async fn list_coaches(
    state: State<'_, AppState>,
    query: CoachListQuery,
) -> Result<Vec<CoachListItem>, String> {
    state
        .service
        .list_coaches(query)
        .await
        .map_err(|error| error.to_string())
}
#[tauri::command]
pub async fn read_coach(
    state: State<'_, AppState>,
    coach_id: String,
) -> Result<CoachDetail, String> {
    let coach_id = parse_uuid(&coach_id, "教练 ID")?;
    state
        .service
        .read_coach(coach_id)
        .await
        .map_err(|error| error.to_string())
}
#[tauri::command]
pub async fn add_coach_name(
    state: State<'_, AppState>,
    draft: CoachNameDraft,
) -> Result<CoachNameRecord, String> {
    state
        .service
        .add_coach_name(draft)
        .await
        .map_err(|error| error.to_string())
}
#[tauri::command]
pub async fn add_team_coach_period(
    state: State<'_, AppState>,
    draft: TeamCoachPeriodDraft,
) -> Result<TeamCoachPeriodRecord, String> {
    state
        .service
        .add_team_coach_period(draft)
        .await
        .map_err(|error| error.to_string())
}
#[tauri::command]
pub async fn list_entity_references(
    state: State<'_, AppState>,
    query: EntityReferenceQuery,
) -> Result<Vec<EntityReferenceRecord>, String> {
    state
        .service
        .list_entity_references(query)
        .await
        .map_err(|error| error.to_string())
}
#[tauri::command]
pub async fn resolve_entity_reference(
    state: State<'_, AppState>,
    request: EntityMatchRequest,
) -> Result<EntityMatchResult, String> {
    state
        .service
        .resolve_entity_reference(request)
        .await
        .map_err(|error| error.to_string())
}
#[tauri::command]
pub async fn check_entity_deletion(
    state: State<'_, AppState>,
    entity_type: String,
    entity_id: String,
) -> Result<EntityDeletionCheck, String> {
    let entity_id = parse_uuid(&entity_id, "实体 ID")?;
    state
        .service
        .check_entity_deletion(entity_type, entity_id)
        .await
        .map_err(|error| error.to_string())
}
#[tauri::command]
pub async fn bulk_archive_entities(
    state: State<'_, AppState>,
    entity_type: String,
    entity_ids: Vec<String>,
) -> Result<BulkArchiveResult, String> {
    let ids = entity_ids
        .iter()
        .map(|value| parse_uuid(value, "实体 ID"))
        .collect::<Result<Vec<_>, _>>()?;
    state
        .service
        .bulk_archive_entities(entity_type, ids)
        .await
        .map_err(|error| error.to_string())
}

#[tauri::command]
pub async fn create_team(
    state: State<'_, AppState>,
    draft: TeamDraft,
) -> Result<TeamRecord, String> {
    state
        .service
        .create_team(draft)
        .await
        .map_err(|error| error.to_string())
}
#[tauri::command]
pub async fn list_team_options(
    state: State<'_, AppState>,
    search: Option<String>,
    limit: u32,
) -> Result<Vec<TeamOption>, String> {
    state
        .service
        .list_team_options(search, limit)
        .await
        .map_err(|error| error.to_string())
}
#[tauri::command]
pub async fn list_teams(
    state: State<'_, AppState>,
    query: TeamListQuery,
) -> Result<TeamListPage, String> {
    state
        .service
        .list_teams(query)
        .await
        .map_err(|error| error.to_string())
}
#[tauri::command]
pub async fn read_team(state: State<'_, AppState>, team_id: String) -> Result<TeamDetail, String> {
    let team_id = parse_uuid(&team_id, "球队 ID")?;
    state
        .service
        .read_team(team_id)
        .await
        .map_err(|error| error.to_string())
}
#[tauri::command]
pub async fn update_team(
    state: State<'_, AppState>,
    team_id: String,
    draft: TeamDraft,
) -> Result<TeamRecord, String> {
    let team_id = parse_uuid(&team_id, "球队 ID")?;
    state
        .service
        .update_team(team_id, draft)
        .await
        .map_err(|error| error.to_string())
}
#[tauri::command]
pub async fn add_team_name(
    state: State<'_, AppState>,
    draft: TeamNameDraft,
) -> Result<TeamNameRecord, String> {
    state
        .service
        .add_team_name(draft)
        .await
        .map_err(|error| error.to_string())
}
#[tauri::command]
pub async fn upsert_team_profile(
    state: State<'_, AppState>,
    team_id: String,
    draft: TeamProfileDraft,
) -> Result<TeamProfileRecord, String> {
    let team_id = parse_uuid(&team_id, "球队 ID")?;
    state
        .service
        .upsert_team_profile(team_id, draft)
        .await
        .map_err(|error| error.to_string())
}
#[tauri::command]
pub async fn bulk_delete_players(
    state: State<'_, AppState>,
    player_ids: Vec<String>,
) -> Result<BulkDeleteResult, String> {
    let ids = player_ids
        .iter()
        .map(|value| parse_uuid(value, "球员 ID"))
        .collect::<Result<Vec<_>, _>>()?;
    state
        .service
        .bulk_delete_players(ids)
        .await
        .map_err(|error| error.to_string())
}
#[tauri::command]
pub async fn bulk_delete_teams(
    state: State<'_, AppState>,
    team_ids: Vec<String>,
) -> Result<BulkDeleteResult, String> {
    let ids = team_ids
        .iter()
        .map(|value| parse_uuid(value, "球队 ID"))
        .collect::<Result<Vec<_>, _>>()?;
    state
        .service
        .bulk_delete_teams(ids)
        .await
        .map_err(|error| error.to_string())
}

#[tauri::command]
pub async fn preview_force_delete_team(
    state: State<'_, AppState>,
    team_id: String,
) -> Result<TeamForceDeletePreview, String> {
    let team_id = parse_uuid(&team_id, "球队 ID")?;
    let service = state.service.clone();
    let runtime = tauri::async_runtime::handle();
    drop(state);

    tauri::async_runtime::spawn_blocking(move || {
        runtime
            .block_on(service.preview_force_delete_team(team_id))
            .map_err(|error| error.to_string())
    })
    .await
    .map_err(|error| format!("球队强制删除预检任务执行失败：{error}"))?
}

#[tauri::command]
pub async fn force_delete_team(
    state: State<'_, AppState>,
    request: TeamForceDeleteRequest,
) -> Result<TeamForceDeleteResult, String> {
    let service = state.service.clone();
    let runtime = tauri::async_runtime::handle();
    drop(state);

    tauri::async_runtime::spawn_blocking(move || {
        runtime
            .block_on(service.force_delete_team(request))
            .map_err(|error| error.to_string())
    })
    .await
    .map_err(|error| format!("球队强制删除任务执行失败：{error}"))?
}
#[tauri::command]
pub async fn create_data_provider(
    state: State<'_, AppState>,
    draft: DataProviderDraft,
) -> Result<DataProviderRecord, String> {
    state
        .service
        .create_data_provider(draft)
        .await
        .map_err(|error| error.to_string())
}
#[tauri::command]
pub async fn create_player(
    state: State<'_, AppState>,
    draft: PlayerDraft,
) -> Result<PlayerRecord, String> {
    state
        .service
        .create_player(draft)
        .await
        .map_err(|error| error.to_string())
}
#[tauri::command]
pub async fn delete_player(state: State<'_, AppState>, player_id: String) -> Result<(), String> {
    let player_id = parse_uuid(&player_id, "球员 ID")?;
    state
        .service
        .delete_player(player_id)
        .await
        .map_err(|error| error.to_string())
}
#[tauri::command]
pub async fn list_players(
    state: State<'_, AppState>,
    query: PlayerListQuery,
) -> Result<PlayerListPage, String> {
    state
        .service
        .list_players(query)
        .await
        .map_err(|error| error.to_string())
}
#[tauri::command]
pub async fn read_player(
    state: State<'_, AppState>,
    player_id: String,
) -> Result<PlayerDetail, String> {
    let player_id = parse_uuid(&player_id, "球员 ID")?;
    state
        .service
        .read_player(player_id)
        .await
        .map_err(|error| error.to_string())
}
#[tauri::command]
pub async fn add_player_name(
    state: State<'_, AppState>,
    draft: PlayerNameDraft,
) -> Result<PlayerNameRecord, String> {
    state
        .service
        .add_player_name(draft)
        .await
        .map_err(|error| error.to_string())
}
#[tauri::command]
pub async fn assign_player_position(
    state: State<'_, AppState>,
    draft: PlayerPositionDraft,
) -> Result<PlayerPositionRecord, String> {
    state
        .service
        .assign_player_position(draft)
        .await
        .map_err(|error| error.to_string())
}
#[tauri::command]
pub async fn add_player_team_period(
    state: State<'_, AppState>,
    draft: PlayerTeamPeriodDraft,
) -> Result<PlayerTeamPeriodRecord, String> {
    state
        .service
        .add_player_team_period(draft)
        .await
        .map_err(|error| error.to_string())
}
#[tauri::command]
pub async fn add_player_availability(
    state: State<'_, AppState>,
    draft: PlayerAvailabilityDraft,
) -> Result<PlayerAvailabilityRecord, String> {
    state
        .service
        .add_player_availability(draft)
        .await
        .map_err(|error| error.to_string())
}
#[tauri::command]
pub async fn add_player_ability_observation(
    state: State<'_, AppState>,
    draft: PlayerAbilityObservationDraft,
) -> Result<PlayerAbilityObservationRecord, String> {
    state
        .service
        .add_player_ability_observation(draft)
        .await
        .map_err(|error| error.to_string())
}
#[tauri::command]
pub async fn add_external_entity_id(
    state: State<'_, AppState>,
    draft: ExternalEntityIdDraft,
) -> Result<ExternalEntityIdRecord, String> {
    state
        .service
        .add_external_entity_id(draft)
        .await
        .map_err(|error| error.to_string())
}
#[tauri::command]
pub async fn create_match(
    state: State<'_, AppState>,
    draft: MatchDraft,
) -> Result<MatchRecord, String> {
    state
        .service
        .create_match(draft)
        .await
        .map_err(|error| error.to_string())
}
#[tauri::command]
pub async fn delete_match(state: State<'_, AppState>, match_id: String) -> Result<(), String> {
    let match_id = parse_uuid(&match_id, "比赛 ID")?;
    state
        .service
        .delete_match(match_id)
        .await
        .map_err(|error| error.to_string())
}

#[tauri::command]
pub async fn save_team_lineup_preset(
    state: State<'_, AppState>,
    draft: TeamLineupPresetDraft,
) -> Result<TeamLineupPresetRecord, String> {
    state
        .service
        .save_team_lineup_preset(draft)
        .await
        .map_err(|error| error.to_string())
}

#[tauri::command]
pub async fn list_team_lineup_presets(
    state: State<'_, AppState>,
    team_id: String,
    include_archived: bool,
) -> Result<Vec<TeamLineupPresetRecord>, String> {
    let team_id = parse_uuid(&team_id, "球队 ID")?;
    state
        .service
        .list_team_lineup_presets(team_id, include_archived)
        .await
        .map_err(|error| error.to_string())
}

#[tauri::command]
pub async fn preview_team_lineup_preset_application(
    state: State<'_, AppState>,
    preset_id: String,
) -> Result<TeamLineupPresetApplicationPreview, String> {
    let preset_id = parse_uuid(&preset_id, "阵容预设 ID")?;
    state
        .service
        .preview_team_lineup_preset_application(preset_id)
        .await
        .map_err(|error| error.to_string())
}

#[tauri::command]
pub async fn duplicate_team_lineup_preset(
    state: State<'_, AppState>,
    preset_id: String,
    name: String,
) -> Result<TeamLineupPresetRecord, String> {
    let preset_id = parse_uuid(&preset_id, "阵容预设 ID")?;
    state
        .service
        .duplicate_team_lineup_preset(preset_id, name)
        .await
        .map_err(|error| error.to_string())
}

#[tauri::command]
pub async fn archive_team_lineup_preset(
    state: State<'_, AppState>,
    preset_id: String,
) -> Result<TeamLineupPresetRecord, String> {
    let preset_id = parse_uuid(&preset_id, "阵容预设 ID")?;
    state
        .service
        .archive_team_lineup_preset(preset_id)
        .await
        .map_err(|error| error.to_string())
}

#[tauri::command]
pub async fn delete_team_lineup_preset(
    state: State<'_, AppState>,
    preset_id: String,
) -> Result<(), String> {
    let preset_id = parse_uuid(&preset_id, "阵容预设 ID")?;
    state
        .service
        .delete_team_lineup_preset(preset_id)
        .await
        .map_err(|error| error.to_string())
}

#[tauri::command]
pub async fn create_lineup(
    state: State<'_, AppState>,
    draft: LineupDraft,
) -> Result<LineupRecord, String> {
    state
        .service
        .create_lineup(draft)
        .await
        .map_err(|error| error.to_string())
}

#[tauri::command]
pub async fn create_lineup_pair(
    state: State<'_, AppState>,
    draft: LineupPairDraft,
) -> Result<LineupPairRecord, String> {
    state
        .service
        .create_lineup_pair(draft)
        .await
        .map_err(|error| error.to_string())
}

#[tauri::command]
pub async fn list_lineups(
    state: State<'_, AppState>,
    match_id: Option<String>,
    limit: u32,
) -> Result<Vec<LineupRecord>, String> {
    let match_id = match_id
        .map(|value| parse_uuid(&value, "比赛 ID"))
        .transpose()?;
    state
        .service
        .list_lineups(match_id, limit)
        .await
        .map_err(|error| error.to_string())
}
#[tauri::command]
pub async fn read_lineup(
    state: State<'_, AppState>,
    lineup_id: String,
) -> Result<LineupRecord, String> {
    let lineup_id = parse_uuid(&lineup_id, "阵容 ID")?;
    state
        .service
        .read_lineup(lineup_id)
        .await
        .map_err(|error| error.to_string())
}

#[tauri::command]
pub async fn remove_lineup_history(
    state: State<'_, AppState>,
    lineup_id: String,
    reason: Option<String>,
) -> Result<LineupHistoryRemovalResult, String> {
    state
        .service
        .remove_lineup_history(parse_uuid(&lineup_id, "阵容 ID")?, reason)
        .await
        .map_err(|error| error.to_string())
}
#[tauri::command]
pub async fn read_match_lineup_chain(
    state: State<'_, AppState>,
    match_id: String,
    snapshot_type: String,
) -> Result<MatchLineupChain, String> {
    state
        .service
        .read_match_lineup_chain(parse_uuid(&match_id, "比赛 ID")?, snapshot_type)
        .await
        .map_err(|error| error.to_string())
}

#[tauri::command]
pub async fn list_team_match_lineups(
    state: State<'_, AppState>,
    team_id: String,
    limit: u32,
) -> Result<Vec<TeamMatchLineupHistoryItem>, String> {
    state
        .service
        .list_team_match_lineups(parse_uuid(&team_id, "球队 ID")?, limit)
        .await
        .map_err(|error| error.to_string())
}

#[tauri::command]
pub async fn update_player(
    state: State<'_, AppState>,
    player_id: String,
    draft: PlayerDraft,
) -> Result<PlayerRecord, String> {
    let parsed = parse_uuid(&player_id, "球员 ID")?;
    state
        .service
        .update_player(parsed, draft)
        .await
        .map_err(|error| error.to_string())
}
#[tauri::command]
pub async fn add_player_dynamic_tag(
    state: State<'_, AppState>,
    draft: PlayerDynamicTagDraft,
) -> Result<PlayerDynamicTagRecord, String> {
    state
        .service
        .add_player_dynamic_tag(draft)
        .await
        .map_err(|error| error.to_string())
}
#[tauri::command]
pub async fn calculate_player_match_contribution(
    state: State<'_, AppState>,
    request: PlayerMatchContributionRequest,
) -> Result<PlayerMatchContribution, String> {
    state
        .service
        .calculate_player_match_contribution(request)
        .await
        .map_err(|error| error.to_string())
}
