use super::{
    player_periods::read_player_periods, read_names::read_names, read_profile::read_profile,
    read_recent_matches::read_recent_matches, read_squad::read_squad,
    read_team_record::read_team_record,
};
use crate::{PersistenceResult, PostgresStore};
use football_domain::{FormationDistributionQuery, FormationUsageListQuery, TeamDetail};
use uuid::Uuid;

impl PostgresStore {
    pub async fn read_team(&self, team_id: Uuid) -> PersistenceResult<TeamDetail> {
        let team = read_team_record(&self.pool, team_id).await?;
        let names = read_names(&self.pool, team_id).await?;
        let profile = read_profile(&self.pool, team_id).await?;
        let squad = read_squad(&self.pool, team_id).await?;
        let player_periods = read_player_periods(&self.pool, team_id).await?;
        let coach_periods = self.list_team_coach_periods(team_id).await?;
        let recent_matches = read_recent_matches(&self.pool, team_id).await?;
        let formation_usage = self
            .list_formation_usage_distributions(&FormationUsageListQuery {
                team_id: Some(team_id),
                coach_id: None,
                competition_id: None,
                limit: 200,
            })
            .await?;
        let resolved_formation_distribution = self
            .resolve_formation_distribution(&FormationDistributionQuery {
                match_id: None,
                team_id,
                coach_id: None,
                competition_id: None,
                as_of: None,
            })
            .await?;
        Ok(TeamDetail {
            team,
            names,
            profile,
            squad,
            player_periods,
            coach_periods,
            recent_matches,
            formation_usage,
            resolved_formation_distribution,
        })
    }
}
