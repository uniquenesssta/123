use crate::{PersistenceResult, PostgresStore};
use chrono::Utc;
use football_domain::{
    AiMatchPackageContext, AiMatchPlayerContext, PlayerMatchContributionRequest,
};
use serde_json::json;
use uuid::Uuid;

impl PostgresStore {
    pub async fn ai_match_package_context(
        &self,
        match_id: Uuid,
    ) -> PersistenceResult<AiMatchPackageContext> {
        let match_record = self.read_match(match_id).await?;
        let competition = match match_record.competition_id {
            Some(id) => self
                .list_competitions()
                .await?
                .into_iter()
                .find(|item| item.id == id),
            None => None,
        };
        let lineups = self.hydrated_active_lineups(match_id).await?;
        let mut players = Vec::new();
        for lineup in &lineups {
            for lineup_player in &lineup.players {
                if players
                    .iter()
                    .any(|item: &AiMatchPlayerContext| item.player.id == lineup_player.player_id)
                {
                    continue;
                }
                let detail = self.read_player(lineup_player.player_id).await?;
                let opponent_team_id = if lineup.team_id == match_record.home_team_id {
                    Some(match_record.away_team_id)
                } else {
                    Some(match_record.home_team_id)
                };
                let contribution = self
                    .calculate_player_match_contribution(&PlayerMatchContributionRequest {
                        player_id: lineup_player.player_id,
                        match_id: Some(match_id),
                        competition_id: match_record.competition_id,
                        position_code: lineup_player.position_code.clone(),
                        role_code: lineup_player.role_code.clone(),
                        role_origin: Some(lineup_player.role_origin.clone()),
                        role_source_position_code: lineup_player.role_source_position_code.clone(),
                        opponent_team_id,
                        as_of: match_record.kickoff_time,
                        data_cutoff_time: Some(Utc::now()),
                        expected_minutes: lineup_player.expected_minutes,
                    })
                    .await
                    .ok();
                players.push(AiMatchPlayerContext {
                    player: detail.player,
                    team_id: Some(lineup.team_id),
                    team_name: Some(lineup.team_name.clone()),
                    lineup_status: if lineup_player.is_starter {
                        "starter"
                    } else {
                        "bench"
                    }
                    .to_string(),
                    tactical_role_code: lineup_player.role_code.clone(),
                    tactical_role_origin: lineup_player.role_origin.clone(),
                    tactical_role_source_position_code: lineup_player
                        .role_source_position_code
                        .clone(),
                    // 兼容旧 AI 包字段；语义已纠正为战术角色，不再承载首发/替补状态。
                    lineup_role: lineup_player.role_code.clone(),
                    expected_minutes: lineup_player.expected_minutes,
                    ability_profile: detail.ability_profile,
                    availability: detail.availability,
                    dynamic_tags: self
                        .list_player_dynamic_tags(
                            lineup_player.player_id,
                            match_record.kickoff_time,
                        )
                        .await?,
                    contribution,
                });
            }
        }
        let lineup_count = lineups.len();
        let player_context_count = players.len();
        let inherited_role_count = players
            .iter()
            .filter(|item| item.tactical_role_origin == "player_position_default")
            .count();
        let overridden_role_count = players
            .iter()
            .filter(|item| item.tactical_role_origin == "lineup_override")
            .count();
        let missing_role_count = players
            .iter()
            .filter(|item| item.tactical_role_origin == "missing")
            .count();
        Ok(AiMatchPackageContext {
            match_record,
            competition,
            lineups,
            players,
            generated_at: Utc::now(),
            data_quality: json!({
                "lineup_count": lineup_count,
                "player_context_count": player_context_count,
                "inherited_role_count": inherited_role_count,
                "overridden_role_count": overridden_role_count,
                "missing_role_count": missing_role_count,
            }),
        })
    }
}
