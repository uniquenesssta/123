use crate::{PersistenceResult, PostgresStore};
use football_domain::PlayerCatalogReferenceData;

impl PostgresStore {
    pub async fn player_catalog_reference_data(
        &self,
    ) -> PersistenceResult<PlayerCatalogReferenceData> {
        let teams = self.list_team_options(None, 500).await?;
        let season_team_memberships = self.list_season_team_memberships().await?;
        let formations = self.list_formations(true).await?;
        let providers = self.list_data_providers().await?;
        let positions = self.list_positions().await?;
        let ability_dimensions = self.list_ability_dimensions().await?;
        let dynamic_tag_definitions = self.list_dynamic_tag_definitions().await?;
        let upcoming_matches = self.list_upcoming_matches(100).await?;
        let managed_matches = self.list_managed_matches(300).await?;
        Ok(PlayerCatalogReferenceData {
            teams,
            season_team_memberships,
            formations,
            providers,
            positions,
            ability_dimensions,
            dynamic_tag_definitions,
            upcoming_matches,
            managed_matches,
        })
    }
}

#[cfg(test)]
mod tests {
    use crate::adapters::catalog::players::value_mapping::availability_status;
    use crate::adapters::catalog::players::{
        normalization::normalize_name,
        value_mapping::{player_status, preferred_foot},
    };
    use football_domain::{AvailabilityStatus, PlayerStatus, PreferredFoot};

    #[test]
    fn normalize_name_collapses_case_and_spacing() {
        assert_eq!(normalize_name("  Son   Heung-Min  "), "son heung-min");
    }

    #[test]
    fn persisted_enums_round_trip() {
        assert_eq!(preferred_foot("left").unwrap(), PreferredFoot::Left);
        assert_eq!(player_status("active").unwrap(), PlayerStatus::Active);
        assert_eq!(
            availability_status("returning").unwrap(),
            AvailabilityStatus::Returning
        );
    }

    #[test]
    fn unknown_persisted_enum_is_rejected() {
        assert!(preferred_foot("ambidextrous").is_err());
        assert!(availability_status("missing").is_err());
    }
}
