use super::row::TeamNameRow;
use football_domain::TeamNameRecord;

pub(super) fn map_team_name(row: TeamNameRow) -> TeamNameRecord {
    TeamNameRecord {
        id: row.id,
        team_id: row.team_id,
        name: row.name,
        normalized_name: row.normalized_name,
        language_code: row.language_code,
        valid_from: row.valid_from,
        valid_to: row.valid_to,
    }
}

#[cfg(test)]
mod tests {
    use super::map_team_name;
    use crate::adapters::catalog::teams::names::row::TeamNameRow;
    use chrono::NaiveDate;
    use uuid::Uuid;

    #[test]
    fn maps_all_team_name_fields_without_loss() {
        let id = Uuid::new_v4();
        let team_id = Uuid::new_v4();
        let valid_from = NaiveDate::from_ymd_opt(2020, 1, 1);
        let valid_to = NaiveDate::from_ymd_opt(2021, 1, 1);
        let record = map_team_name(TeamNameRow {
            id,
            team_id,
            name: "Alias".to_string(),
            normalized_name: "alias".to_string(),
            language_code: Some("en".to_string()),
            valid_from,
            valid_to,
        });
        assert_eq!(record.id, id);
        assert_eq!(record.team_id, team_id);
        assert_eq!(record.name, "Alias");
        assert_eq!(record.normalized_name, "alias");
        assert_eq!(record.language_code.as_deref(), Some("en"));
        assert_eq!(record.valid_from, valid_from);
        assert_eq!(record.valid_to, valid_to);
    }
}
