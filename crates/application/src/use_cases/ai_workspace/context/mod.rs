use crate::{
    ports::{
        exchange::MatchLineupExchangePort, player::PlayerCatalogPort, team::TeamCatalogPort,
    },
    ApplicationError, ApplicationResult,
};
use football_domain::PlayerListQuery;
use serde_json::{json, Value};
use std::future::Future;
use uuid::Uuid;

pub(crate) async fn execute<P, F>(
    session: F,
    match_id: Option<Uuid>,
    entity_type: Option<&str>,
    entity_id: Option<Uuid>,
) -> ApplicationResult<Value>
where
    P: MatchLineupExchangePort + PlayerCatalogPort + TeamCatalogPort,
    F: Future<Output = ApplicationResult<P>>,
{
    let port = session.await?;
    let match_context = if let Some(match_id) = match_id {
        Some(serde_json::to_value(
            MatchLineupExchangePort::ai_match_package_context(&port, match_id).await?,
        )?)
    } else {
        None
    };

    let normalized_entity_type = entity_type.map(str::trim).filter(|value| !value.is_empty());
    match (normalized_entity_type, entity_id) {
        (Some("team"), Some(team_id)) => {
            let references = PlayerCatalogPort::reference_data(&port).await?;
            let team = TeamCatalogPort::read_team(&port, team_id).await?;
            return Ok(json!({
                "selected_match": match_context.clone(),
                "selected_entity": {"type": "team", "id": team_id},
                "scope": "complete_team_database_context",
                "team": team,
                "positions": references.positions,
                "ability_dimensions": references.ability_dimensions,
                "dynamic_tag_definitions": references.dynamic_tag_definitions,
                "note": "The selected team detail is complete for the desktop team center at request time. Public current facts still require source verification."
            }));
        }
        (Some("player"), Some(player_id)) => {
            let references = PlayerCatalogPort::reference_data(&port).await?;
            let player = PlayerCatalogPort::read_player(&port, player_id).await?;
            return Ok(json!({
                "selected_match": match_context.clone(),
                "selected_entity": {"type": "player", "id": player_id},
                "scope": "complete_player_database_context",
                "player": player,
                "positions": references.positions,
                "ability_dimensions": references.ability_dimensions,
                "dynamic_tag_definitions": references.dynamic_tag_definitions,
                "note": "The selected player detail is complete for the desktop player center at request time. Public current facts still require source verification."
            }));
        }
        (None, None) => {
            if let Some(context) = match_context {
                return Ok(context);
            }
        }
        _ => {
            return Err(ApplicationError::Validation(
                "API协作实体上下文必须同时提供有效的类型和ID".to_string(),
            ));
        }
    }

    let references = PlayerCatalogPort::reference_data(&port).await?;
    let query = PlayerListQuery {
        limit: 200,
        ..PlayerListQuery::default()
    };
    let players = PlayerCatalogPort::list_players(&port, &query).await?;
    Ok(json!({
        "selected_match": null,
        "selected_entity": null,
        "scope": "bounded_general_database_context",
        "teams": references.teams,
        "positions": references.positions,
        "ability_dimensions": references.ability_dimensions,
        "dynamic_tag_definitions": references.dynamic_tag_definitions,
        "managed_matches": references.managed_matches,
        "players": players.items,
        "players_truncated": players.has_more,
        "note": "General context is bounded to 200 active players. Open API collaboration from a team or player center, bind a match, or attach an exported file for complete target context."
    }))
}
