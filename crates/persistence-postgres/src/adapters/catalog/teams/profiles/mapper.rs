use super::row::TeamProfileRow;
use football_domain::TeamProfileRecord;

pub(super) fn map_team_profile(row: TeamProfileRow) -> TeamProfileRecord {
    TeamProfileRecord {
        team_id: row.team_id,
        short_name: row.short_name,
        team_type: row.team_type,
        founded_year: row.founded_year,
        city: row.city,
        stadium: row.stadium,
        head_coach: row.head_coach,
        default_formation: row.default_formation,
        tactical_style: row.tactical_style,
        attack_rating: row.attack_rating,
        midfield_rating: row.midfield_rating,
        defence_rating: row.defence_rating,
        goalkeeper_rating: row.goalkeeper_rating,
        reputation: row.reputation,
        data_confidence: row.data_confidence,
        notes: row.notes,
        metadata: row.metadata,
        updated_at: row.updated_at,
    }
}
